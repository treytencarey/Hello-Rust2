-- Network Sync Module v2
-- ECS-based entity replication using message entities instead of send_fn closures
--
-- Usage:
--   entity:set({ NetworkSync = { net_id = NetSync2.next_net_id(), authority = "server", sync_components = {...} } })
--
--   -- Systems are registered via NetGame which orchestrates them
--   NetSync2.outbound_system(world)  -- Spawns NetSyncOutbound entities
--   NetSync2.inbound_system(world)   -- Processes NetSyncInbound entities

local NetRole = require("modules/net_role.lua")

local NetSync2 = {}

--------------------------------------------------------------------------------
-- Component Names
--------------------------------------------------------------------------------

NetSync2.MARKER = "NetworkSync"           -- Marker component on synced entities
NetSync2.OUTBOUND = "NetSyncOutbound"     -- Outbound message entities
NetSync2.INBOUND = "NetSyncInbound"       -- Inbound message entities
NetSync2.PREDICTION = "PredictionState"   -- Client prediction state
NetSync2.INTERPOLATION = "InterpolationTarget"  -- Remote entity interpolation

--------------------------------------------------------------------------------
-- JSON (using dkjson library)
--------------------------------------------------------------------------------

local json = require("modules/dkjson.lua")

local function json_encode(tbl)
    return json.encode(tbl)
end

local function json_decode(str)
    local result, pos, err = json.decode(str)
    if err then
        print("[NET_SYNC2] JSON decode error: " .. tostring(err))
        return nil
    end
    return result
end

-- Export for external use
NetSync2.json_encode = json_encode
NetSync2.json_decode = json_decode

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

local CHANNEL_RELIABLE = 0
local CHANNEL_UNRELIABLE = 1

NetSync2.CHANNEL_RELIABLE = CHANNEL_RELIABLE
NetSync2.CHANNEL_UNRELIABLE = CHANNEL_UNRELIABLE

--------------------------------------------------------------------------------
-- Per-Instance State
--------------------------------------------------------------------------------

local known_entities = {}   -- net_id -> entity_id
local my_client_id = nil    -- This client's assigned client_id (set by server on connect)

-- Net ID generation
local client_prefix = 0
local local_counter = 0

-- Pending children waiting for parent to spawn (net_id -> spawn_data)
local pending_children = {}

-- Pending children timestamps for cleanup (net_id -> timestamp)
local pending_children_time = {}
local PENDING_CHILD_TIMEOUT = 10.0  -- seconds before orphan cleanup

-- Cache of known sync component types with reference counts (comp_name -> count)
local known_sync_types = {}

-- Cache of each entity's sync_components config (entity_id -> { comp_name -> config })
local entity_sync_config = {}

-- Server-side scope tracking
local client_scope = {}      -- client_id -> { net_id = true, ... }
local net_id_owners = {}     -- net_id -> client_id

-- Track which clients we know about (for disconnect detection)
local known_clients = {}  -- client_id -> true

-- Sequence tracking (for client-side prediction ack)
local client_input_sequences = {}  -- entity_id -> last seq from client (server-side)

--------------------------------------------------------------------------------
-- Net ID Management
--------------------------------------------------------------------------------

--- Set client prefix (called by server on connect)
function NetSync2.set_client_prefix(prefix)
    client_prefix = prefix
    print(string.format("[NET_SYNC2] Client prefix set to %d", prefix))
end

--- Generate next unique net_id
function NetSync2.next_net_id()
    local_counter = local_counter + 1
    return client_prefix * 10000 + local_counter
end

--- Register entity mapping
function NetSync2.register_entity(net_id, entity_id)
    known_entities[net_id] = entity_id
end

--- Unregister entity mapping
function NetSync2.unregister_entity(net_id)
    known_entities[net_id] = nil
end

--- Get entity_id from net_id
function NetSync2.get_entity(net_id)
    return known_entities[net_id]
end

--- Get net_id from entity_id
function NetSync2.get_net_id(entity_id)
    for net_id, eid in pairs(known_entities) do
        if eid == entity_id then
            return net_id
        end
    end
    return nil
end

--- Find entity in world by net_id (world-level query for shared ECS)
--- Used for deduplication in "both" mode where server and client share the ECS world
--- @param world userdata
--- @param net_id number
--- @return number|nil entity_id if found
local function find_entity_by_net_id(world, net_id)
    local entities = world:query({NetSync2.MARKER})
    for _, entity in ipairs(entities) do
        local sync = entity:get(NetSync2.MARKER)
        if sync and sync.net_id == net_id then
            return entity:id()
        end
    end
    return nil
end

--- Set this client's assigned client_id (called when server sends client_id message)
function NetSync2.set_my_client_id(client_id)
    my_client_id = client_id
    print(string.format("[NET_SYNC2] My client_id set to %s", tostring(client_id)))
end

--- Get this client's assigned client_id
function NetSync2.get_my_client_id()
    return my_client_id
end

--- Check if entity is owned by us (by checking owner_client)
--- @param world userdata
--- @param entity userdata|number Entity or entity_id
--- @return boolean
function NetSync2.is_my_entity(world, entity)
    if NetSync2.get_my_client_id() == nil then
        return false
    end

    -- If passed entity_id, get the entity
    if type(entity) == "number" then
        entity = world:get_entity(entity)
        if not entity then return false end
    end

    -- Check PlayerState.owner_client or NetworkSync.owner_client
    local player_state = entity:get("PlayerState")
    if player_state and player_state.owner_client == NetSync2.get_my_client_id() then
        return true
    end

    local sync = entity:get(NetSync2.MARKER)
    if sync and sync.owner_client == NetSync2.get_my_client_id() then
        return true
    end

    return false
end

--------------------------------------------------------------------------------
-- Ownership Management (Server-side)
--------------------------------------------------------------------------------

--- Set owner of a net_id
function NetSync2.set_net_id_owner(net_id, client_id)
    net_id_owners[net_id] = client_id
end

--- Get owner of a net_id
function NetSync2.get_net_id_owner(net_id)
    return net_id_owners[net_id]
end

--- Get all net_ids owned by a specific client
--- @param client_id number
--- @return table Array of net_ids
function NetSync2.get_net_ids_for_client(client_id)
    local result = {}
    for net_id, owner in pairs(net_id_owners) do
        if owner == client_id then
            table.insert(result, net_id)
        end
    end
    return result
end

--------------------------------------------------------------------------------
-- Client Scope Management (Server-side)
--------------------------------------------------------------------------------

--- Initialize scope for a new client
function NetSync2.init_client_scope(client_id)
    client_scope[client_id] = {}
    known_clients[client_id] = true
end

--- Check if client knows about entity
function NetSync2.client_knows_entity(client_id, net_id)
    local scope = client_scope[client_id]
    return scope and scope[net_id] == true
end

--- Add entity to client's scope
function NetSync2.add_to_client_scope(client_id, net_id)
    if not client_scope[client_id] then
        client_scope[client_id] = {}
    end
    client_scope[client_id][net_id] = true
end

--- Remove entity from client's scope
function NetSync2.remove_from_client_scope(client_id, net_id)
    if client_scope[client_id] then
        client_scope[client_id][net_id] = nil
    end
end

--- Remove client's entire scope (on disconnect)
function NetSync2.remove_client_scope(client_id)
    client_scope[client_id] = nil
    known_clients[client_id] = nil
end

--- Get all known clients
function NetSync2.get_known_clients()
    local clients = {}
    for client_id, _ in pairs(known_clients) do
        table.insert(clients, client_id)
    end
    return clients
end

--- Get all net_ids in client's scope
function NetSync2.get_client_scope(client_id)
    return client_scope[client_id] or {}
end

--------------------------------------------------------------------------------
-- Sync Type Reference Counting
--------------------------------------------------------------------------------

local function add_sync_types(sync_components)
    for comp_name, _ in pairs(sync_components) do
        known_sync_types[comp_name] = (known_sync_types[comp_name] or 0) + 1
    end
end

local function remove_sync_types(sync_components)
    for comp_name, _ in pairs(sync_components) do
        local count = known_sync_types[comp_name] or 0
        if count > 1 then
            known_sync_types[comp_name] = count - 1
        else
            known_sync_types[comp_name] = nil
        end
    end
end

--- Get all sync component types currently tracked
function NetSync2.get_known_sync_types()
    return known_sync_types
end

--------------------------------------------------------------------------------
-- Message Building
--------------------------------------------------------------------------------

--- Build spawn message payload
--- @param world userdata
--- @param entity userdata
--- @param net_id number
--- @return table spawn message
function NetSync2.build_spawn_msg(world, entity, net_id)
    local sync = entity:get(NetSync2.MARKER)
    if not sync then return nil end

    local sync_components = sync.sync_components or { Transform = {} }
    local components = {}

    -- Always include NetworkSync
    -- Deep copy sync_components to strip functions (like validation callbacks)
    local function strip_functions(t)
        if type(t) ~= "table" then return t end
        local clean = {}
        for k, v in pairs(t) do
            if type(v) == "table" then
                clean[k] = strip_functions(v)
            elseif type(v) ~= "function" then
                clean[k] = v
            end
        end
        return clean
    end

    components[NetSync2.MARKER] = {
        net_id = net_id,
        authority = sync.authority or "server",
        sync_components = strip_functions(sync_components),
        owner_client = sync.owner_client,
    }

    -- Include all sync_components
    for comp_name, _ in pairs(sync_components) do
        local comp_data = entity:get(comp_name)
        if comp_data then
            components[comp_name] = comp_data
        end
    end

    -- Check for parent relationship (ChildOf component)
    local parent_net_id = nil
    local child_of = entity:get("ChildOf")
    if child_of and child_of.parent then
        parent_net_id = NetSync2.get_net_id(child_of.parent)
    end

    return {
        msg_type = "spawn",
        channel = CHANNEL_RELIABLE,
        net_id = net_id,
        owner_client = sync.owner_client,
        payload = {
            authority = sync.authority or "server",
            parent_net_id = parent_net_id,
            components = components,
        }
    }
end

--- Build update message payload
--- @param owner_client number
--- @param entity userdata
--- @param changed_components table { comp_name = comp_data, ... }
--- @param seq number|nil client sequence for prediction
--- @param ack_seq number|nil server ack for reconciliation
--- @return table update message
function NetSync2.build_update_msg(owner_client, entity, changed_components, seq, ack_seq)
    local sync = entity:get(NetSync2.MARKER)
    if not sync then return nil end

    -- Determine channel based on component reliability
    local needs_reliable = false
    local sync_components = sync.sync_components or {}
    for comp_name, _ in pairs(changed_components) do
        local config = sync_components[comp_name] or {}
        if config.reliable then
            needs_reliable = true
            break
        end
    end

    return {
        msg_type = "update",
        channel = needs_reliable and CHANNEL_RELIABLE or CHANNEL_UNRELIABLE,
        net_id = sync.net_id,
        owner_client = owner_client,
        payload = {
            components = changed_components,
            seq = seq,
            ack_seq = ack_seq,
        }
    }
end

--- Build despawn message payload
--- @param owner_client number
--- @param net_id number
--- @param entity_id number|nil for source tracking
--- @return table despawn message
function NetSync2.build_despawn_msg(owner_client, net_id, entity_id)
    return {
        msg_type = "despawn",
        channel = CHANNEL_RELIABLE,
        net_id = net_id,
        owner_client = owner_client,
        payload = {}
    }
end

--- Build owner_change message payload
--- @param net_id number
--- @param new_owner number|nil client_id or nil for server
--- @return table owner_change message
function NetSync2.build_owner_change_msg(net_id, new_owner)
    return {
        msg_type = "owner_change",
        channel = CHANNEL_RELIABLE,
        net_id = net_id,
        payload = {
            new_owner = new_owner
        }
    }
end

--- Build client_id message payload (tells client their assigned client_id)
--- @param client_id number
--- @return table client_id message
function NetSync2.build_client_id_msg(client_id)
    return {
        msg_type = "client_id",
        channel = CHANNEL_RELIABLE,
        net_id = nil,
        payload = {
            client_id = client_id
        }
    }
end

--------------------------------------------------------------------------------
-- Outbound System
--------------------------------------------------------------------------------

local test_count = -1
register_system("Update", function(world)
    local entities = world:query({"PlayerState"})
    if #entities ~= test_count then
        test_count = #entities
        print(string.format("[NET_SYNC2] There are %d PlayerState entities", test_count))
    end
end)

--- Main outbound system - detects changes and spawns NetSyncOutbound entities
--- @param world userdata
--- @param context table { is_server = bool, get_clients = fn|nil }
function NetSync2.outbound_system(world, context)
    context = context or {}
    local get_clients = context.get_clients  -- function returning client list (server only)

    local now = os.clock()

    -- Step 1: Detect new NetworkSync entities (no change detection - entity_sync_config check filters new vs existing)
    local sync_entities = world:query({NetSync2.MARKER, "ScriptOwned"}, {NetSync2.MARKER})

    for _, entity in ipairs(sync_entities) do
        print(string.format("[NET_SYNC2] Checking entity: entity_id=%d", entity:id()))
        local sync = entity:get(NetSync2.MARKER)
        local entity_id = entity:id()

        -- Skip messages from other instances (e.g. server/client instanced scripts)
        local script_owned = entity:get("ScriptOwned")
        if script_owned.instance_id ~= __INSTANCE_ID__ then
            print(string.format("[NET_SYNC2] Skipping entity: entity_id=%d", entity:id()))
            goto continue_sync
        end

        -- Check if this is a new entity we need to spawn
        local cached_config = entity_sync_config[entity_id]

        if not cached_config then
            -- New entity: cache config and prepare spawn
            entity_sync_config[entity_id] = {
                sync_components = sync.sync_components or { Transform = {} },
                last_sync_times = {},
                spawned = false,
                created_locally = true,  -- This instance created the entity (vs adopted from network)
            }
            cached_config = entity_sync_config[entity_id]
            add_sync_types(cached_config.sync_components)
        end

        -- Handle spawn if not yet spawned
        if not cached_config.spawned then
            local net_id = sync.net_id

            -- Server auto-assigns net_id if missing
            if NetRole.is_server() and not net_id then
                net_id = NetSync2.next_net_id()
                entity:patch({ [NetSync2.MARKER] = { net_id = net_id, owner_client = sync.owner_client } })
                sync = entity:get(NetSync2.MARKER)
            end

            if net_id then
                local full_entity = world:get_entity(entity_id)
                
                -- Register entity
                NetSync2.register_entity(net_id, entity_id)
                NetSync2.set_net_id_owner(net_id, sync.owner_client)

                -- Build and spawn outbound message
                local spawn_msg = NetSync2.build_spawn_msg(world, full_entity, net_id)
                if spawn_msg then
                    if NetRole.is_server() and get_clients then
                        -- Server: target all connected clients
                        spawn_msg.target_clients = get_clients()
                    end

                    spawn({ [NetSync2.OUTBOUND] = spawn_msg })
                end

                cached_config.spawned = true

                -- Initialize hash cache for all sync components
                for comp_name, _ in pairs(cached_config.sync_components) do
                    local comp_data = full_entity:get(comp_name)
                    if comp_data then
                        cached_config.last_sync_times[comp_name] = now
                    end
                end
            end
        end

        ::continue_sync::
    end

    -- Step 2: Detect component changes for known sync types
    for comp_name, _ in pairs(known_sync_types) do
        local changed = world:query({NetSync2.MARKER, comp_name}, {comp_name})

        for _, entity in ipairs(changed) do
            local sync = entity:get(NetSync2.MARKER)
            local entity_id = entity:id()
            local full_entity = world:get_entity(entity_id)
            local cached_config = entity_sync_config[entity_id]

            -- Skip if not properly initialized (not in this instance's sync cache)
            if not cached_config or not cached_config.spawned then
                goto continue_comp
            end

            -- Skip ALL components for adopted entities
            -- In "both" mode (shared ECS), the other instance already sees our changes directly.
            -- In separate mode, entities are spawned locally (created_locally = true), not adopted.
            if not cached_config.created_locally then
                goto continue_comp
            end

            local comp_config = cached_config.sync_components[comp_name]
            if not comp_config then
                goto continue_comp
            end

            -- Authority check for locally-created entities
            local comp_authority = comp_config.authority or sync.authority or "server"
            if NetRole.is_server() and comp_authority == "client" then
                goto continue_comp  -- Server shouldn't send client-authority components
            end
            if not NetRole.is_server() and comp_authority == "server" then
                goto continue_comp  -- Client shouldn't send server-authority components
            end

            -- Rate limiting
            local rate_hz = comp_config.rate_hz or 30
            local interval = 1.0 / rate_hz
            local last_time = cached_config.last_sync_times[comp_name] or 0

            if (now - last_time) < interval then
                goto continue_comp
            end

            -- Hash-based change detection
            local comp_data = entity:get(comp_name)

            -- Build update message
            local changed_components = { [comp_name] = comp_data }

            -- Determine seq/ack_seq
            local seq = nil
            local ack_seq = nil

            -- FIX: Correct condition - client sets seq, server sets ack_seq
            if not NetRole.is_server() then
                -- Client: include prediction sequence
                local pred = full_entity:get(NetSync2.PREDICTION)
                if pred then
                    seq = pred.current_sequence
                end
                print(string.format("[DEBUG:ACK_BRANCH] CLIENT branch: seq=%s (%s)", tostring(seq), json_encode(pred)))
            else
                -- Server: include ack_seq for entity owner
                ack_seq = client_input_sequences[entity_id]
                print(string.format("[DEBUG:ACK_BRANCH] SERVER branch: ack_seq=%s (entity=%d)",
                    tostring(ack_seq), entity_id))
            end

            -- DEBUG: Final seq/ack_seq values being sent
            print(string.format("[DEBUG:OUTBOUND] Entity %d comp=%s seq=%s ack_seq=%s is_server=%s",
                entity_id, comp_name, tostring(seq), tostring(ack_seq), tostring(NetRole.is_server())))

            local update_msg = NetSync2.build_update_msg(sync.owner_client, entity, changed_components, seq, ack_seq)
            if update_msg then
                if NetRole.is_server() and get_clients then
                    -- TODO: Review
                    -- -- Server: target clients in scope
                    -- local targets = {}
                    -- for _, client_id in ipairs(get_clients()) do
                    --     if NetSync2.client_knows_entity(client_id, sync.net_id) then
                    --         table.insert(targets, client_id)
                    --     end
                    -- end
                    -- update_msg.target_clients = targets

                    -- Include owner info for ack routing
                    update_msg.owner_client = NetSync2.get_net_id_owner(sync.net_id)
                end

                spawn({ [NetSync2.OUTBOUND] = update_msg })
            end

            -- Update cache
            cached_config.last_sync_times[comp_name] = now

            ::continue_comp::
        end
    end

    -- Step 3: Detect despawned entities
    local removed = world:query_removed({NetSync2.MARKER})

    for _, entity_id in ipairs(removed) do
        local sync = entity:get(NetSync2.MARKER)
        local net_id = NetSync2.get_net_id(entity_id)

        -- Only send despawns for entities this instance created (not adopted ones)
        -- We can't query ScriptOwned on removed entities, so we check the cached config
        local cached_config = entity_sync_config[entity_id]
        if not cached_config or not cached_config.created_locally then
            goto continue_removed
        end

        if net_id then
            -- Build despawn message
            local despawn_msg = NetSync2.build_despawn_msg(sync.owner_client, net_id, entity_id)

            if NetRole.is_server() and get_clients then
                -- Server: target all clients who knew this entity
                local targets = {}
                for _, client_id in ipairs(get_clients()) do
                    if NetSync2.client_knows_entity(client_id, net_id) then
                        table.insert(targets, client_id)
                        NetSync2.remove_from_client_scope(client_id, net_id)
                    end
                end
                despawn_msg.target_clients = targets
            end

            spawn({ [NetSync2.OUTBOUND] = despawn_msg })

            -- Cleanup
            NetSync2.unregister_entity(net_id)
            net_id_owners[net_id] = nil

            -- Remove cached config
            local cached_config = entity_sync_config[entity_id]
            if cached_config then
                remove_sync_types(cached_config.sync_components)
                entity_sync_config[entity_id] = nil
            end
        end

        ::continue_removed::
    end
end

--------------------------------------------------------------------------------
-- Inbound System
--------------------------------------------------------------------------------

--- Handle update message
local function handle_update(world, msg, owner_client)
    local net_id = msg.net_id
    local payload = msg.payload
    local entity_id = known_entities[net_id]

    if not entity_id then
        print(string.format("[NET_SYNC2] Update for unknown entity %d", net_id))
        return
    end

    local entity = world:get_entity(entity_id)
    if not entity then
        print(string.format("[NET_SYNC2] Entity %d not found in world", entity_id))
        return
    end

    local sync = entity:get(NetSync2.MARKER)
    local sync_components = (sync and sync.sync_components) or {}
    local is_own_entity = NetSync2.is_my_entity(world, entity)

    -- Skip updates for adopted entities (in "both" mode, we see changes directly on shared ECS)
    local cached_config = entity_sync_config[entity_id]
    if cached_config and cached_config.created_locally == false then
        -- Adopted entity: don't apply network updates, we already see changes on shared ECS
        return
    end

    -- Process each component update
    for comp_name, comp_data in pairs(payload.components or {}) do
        local comp_config = sync_components[comp_name] or {}

        -- Server-side authority validation
        if owner_client then
            -- local comp_authority = comp_config.authority or "server"
            -- if comp_authority ~= "client" and comp_authority ~= "any" then
            --     print(string.format("[NET_SYNC2] Authority rejected %s from client %s", comp_name, owner_client))
            --     goto continue_update
            -- end

            -- Run validation function if configured
            if comp_config.validation then
                local old_value = entity:get(comp_name)
                local is_valid, sanitized = comp_config.validation(world, entity, old_value, comp_data, owner_client)

                if not is_valid then
                    print(string.format("[NET_SYNC2] Validation rejected %s from client %s", comp_name, owner_client))
                    goto continue_update
                end

                comp_data = sanitized or comp_data
            end

            -- Record sequence for ack
            if payload.seq then
                client_input_sequences[entity_id] = payload.seq
                -- DEBUG: Server received seq from client
                print(string.format("[DEBUG:SERVER:RECV] Entity %d received seq=%d from client %s (stored for ack)",
                    entity_id, payload.seq, tostring(owner_client)))
            end
        end

        -- Handle own entity specially (for prediction)
        if is_own_entity then
            if comp_name == "Transform" then
                -- Don't apply Transform directly - store for reconciliation
                local pred = entity:get(NetSync2.PREDICTION)
                if pred then
                    -- DEBUG: Client receiving server state with ack_seq
                    print(string.format("[DEBUG:CLIENT:ACK] Entity %d received Transform with ack_seq=%s server_pos=(%.2f,%.2f,%.2f)",
                        entity_id, tostring(payload.ack_seq),
                        comp_data.translation.x, comp_data.translation.y, comp_data.translation.z))

                    entity:patch({
                        [NetSync2.PREDICTION] = {
                            server_state = {
                                position = comp_data.translation,
                                rotation = comp_data.rotation,
                            },
                            last_acked_sequence = payload.ack_seq,
                        }
                    })
                end
                goto continue_update
            end
        end

        -- Handle interpolation for remote entities
        if not is_own_entity and comp_config.interpolate and comp_name == "Transform" then
            -- Set interpolation target instead of applying directly
            entity:set({
                [NetSync2.INTERPOLATION] = {
                    position = comp_data.translation,
                    rotation = comp_data.rotation,
                    scale = comp_data.scale or { x = 1, y = 1, z = 1 },
                    timestamp = os.clock(),
                    lerp_speed = 15.0,
                    snap_threshold = 5.0,
                }
            })
            print(string.format("[NET_SYNC2] Applied remote %s from client %s", comp_name, owner_client))
            goto continue_update
        end

        -- Normal component update
        entity:set({ [comp_name] = comp_data })

        ::continue_update::
    end
end

--- Handle spawn message
local function handle_spawn(world, msg, owner_client)
    local net_id = msg.net_id
    local payload = msg.payload

    -- Skip if already known in this instance's registry (might be a reconnect/duplicate)
    if known_entities[net_id] then
        print(string.format("[NET_SYNC2] Spawn skipped - entity %d already known", net_id))
        -- Treat as update instead
        return handle_update(world, msg, owner_client)
    end

    -- Check if entity exists in world (for "both" mode where ECS is shared between instances)
    local existing_entity_id = find_entity_by_net_id(world, net_id)
    if existing_entity_id then
        print(string.format("[NET_SYNC2] Adopting existing entity %d for net_id %d", existing_entity_id, net_id))
        known_entities[net_id] = existing_entity_id

        -- Cache sync config for this instance so we can process updates
        local entity = world:get_entity(existing_entity_id)
        local sync = entity:get(NetSync2.MARKER)
        if sync and not entity_sync_config[existing_entity_id] then
            entity_sync_config[existing_entity_id] = {
                sync_components = sync.sync_components or { Transform = {} },
                last_sync_times = {},
                spawned = true,
                created_locally = false,  -- Adopted from another instance, don't send despawns
            }
            add_sync_types(entity_sync_config[existing_entity_id].sync_components)
        end

        -- Apply any component updates from the spawn message
        return handle_update(world, msg, owner_client)
    end

    -- Check for parent dependency
    local parent_net_id = payload.parent_net_id
    if parent_net_id then
        local parent_entity_id = known_entities[parent_net_id]
        if not parent_entity_id then
            -- Defer until parent arrives
            pending_children[net_id] = { msg = msg, owner_client = owner_client }
            pending_children_time[net_id] = os.clock()
            print(string.format("[NET_SYNC2] Deferred spawn of %d waiting for parent %d", net_id, parent_net_id))
            return
        end
    end

    -- Build spawn data
    local spawn_data = payload.components or {}

    -- Set authority to "remote" so we don't re-broadcast
    if spawn_data[NetSync2.MARKER] then
        spawn_data[NetSync2.MARKER].authority = "remote"
    else
        spawn_data[NetSync2.MARKER] = {
            net_id = net_id,
            owner_client = owner_client,
            authority = "remote",
        }
    end

    -- Handle parent relationship
    if parent_net_id then
        local parent_entity_id = known_entities[parent_net_id]
        spawn_data.ChildOf = { parent = parent_entity_id }
    end

    -- Auto-add PredictionState for own entity (client-side prediction support)
    if owner_client == NetSync2.get_my_client_id() then
        spawn_data[NetSync2.PREDICTION] = {
            server_state = nil,
            last_acked_sequence = nil,
            current_sequence = 0,
            predictions = {},
            snap_threshold = 2.0,
        }
        print(string.format("[NET_SYNC2] Added PredictionState for own entity"))
    else
        print(string.format("[NET_SYNC2] Could not add PredictionState for owner client %d, my id: %d", owner_client, NetSync2.get_my_client_id()))
    end

    -- Spawn the entity
    local entity_id = spawn(spawn_data):id()
    known_entities[net_id] = entity_id

    print(string.format("[NET_SYNC2] Spawned entity %d with net_id %d", entity_id, net_id))

    -- Process any pending children waiting for this entity
    for child_net_id, child_data in pairs(pending_children) do
        if child_data.msg.payload.parent_net_id == net_id then
            pending_children[child_net_id] = nil
            pending_children_time[child_net_id] = nil
            handle_spawn(world, child_data.msg, child_data.owner_client)
        end
    end
end

--- Handle despawn message
local function handle_despawn(world, msg, owner_client)
    local net_id = msg.net_id
    local entity_id = known_entities[net_id]

    if entity_id then
        print(string.format("[NET_SYNC2] Despawning entity %d (net_id %d)", entity_id, net_id))
        despawn(entity_id)
        known_entities[net_id] = nil

        -- Cleanup cached config
        local cached_config = entity_sync_config[entity_id]
        if cached_config then
            remove_sync_types(cached_config.sync_components)
            entity_sync_config[entity_id] = nil
        end
    end
end

--- Handle owner_change message
local function handle_owner_change(world, msg, owner_client)
    local net_id = msg.net_id
    local new_owner = msg.payload.new_owner
    local entity_id = known_entities[net_id]

    if entity_id then
        local entity = world:get_entity(entity_id)
        if entity then
            entity:patch({ [NetSync2.MARKER] = { owner_client = new_owner } })
            print(string.format("[NET_SYNC2] Owner changed for %d to %s", net_id, tostring(new_owner)))
        end
    end
end

--- Handle client_id message (server tells us our assigned client_id)
local function handle_client_id(world, msg)
    local client_id = msg.payload.client_id
    NetSync2.set_my_client_id(client_id)
    print(string.format("[NET_SYNC2] Received client_id: %s", tostring(client_id)))
end

--- Main inbound system - processes NetSyncInbound entities
--- @param world userdata
function NetSync2.inbound_system(world)
    -- Query all inbound message entities
    local inbound = world:query({NetSync2.INBOUND, "ScriptOwned"})

    for _, msg_entity in ipairs(inbound) do
        local msg = msg_entity:get(NetSync2.INBOUND)
        local owner_client = msg.owner_client

        -- Skip messages from other instances (e.g. server/client instanced scripts)
        local script_owned = msg_entity:get("ScriptOwned")
        if script_owned.instance_id ~= __INSTANCE_ID__ then
            goto continue_entities
        end

        -- Dispatch based on message type
        print(string.format("[NET_SYNC2] Testing %s", json_encode(msg)))
        if msg.msg_type == "spawn" then
            handle_spawn(world, msg, owner_client)
        elseif msg.msg_type == "update" then
            handle_update(world, msg, owner_client)
        elseif msg.msg_type == "despawn" then
            handle_despawn(world, msg, owner_client)
        elseif msg.msg_type == "owner_change" then
            handle_owner_change(world, msg, owner_client)
        elseif msg.msg_type == "client_id" then
            handle_client_id(world, msg)
        end

        -- Despawn the message entity
        despawn(msg_entity:id())

        ::continue_entities::
    end

    -- Cleanup timed-out pending children
    local now = os.clock()
    for net_id, timestamp in pairs(pending_children_time) do
        if (now - timestamp) > PENDING_CHILD_TIMEOUT then
            print(string.format("[NET_SYNC2] Orphaned pending child %d timed out", net_id))
            pending_children[net_id] = nil
            pending_children_time[net_id] = nil
        end
    end
end

--------------------------------------------------------------------------------
-- Interpolation System
--------------------------------------------------------------------------------

--- Interpolation system for remote entities
--- @param world userdata
function NetSync2.interpolation_system(world)
    local dt = world:delta_time()
    local entities = world:query({NetSync2.INTERPOLATION, "Transform", "ScriptOwned"})

    for _, entity in ipairs(entities) do
        local target = entity:get(NetSync2.INTERPOLATION)
        local transform = entity:get("Transform")

        -- Skip entities from other instances (e.g. server/client instanced scripts)
        local script_owned = entity:get("ScriptOwned")
        if script_owned.instance_id ~= __INSTANCE_ID__ then
            goto continue_entities
        end

        local t = math.min(1.0, target.lerp_speed * dt)

        -- Calculate distance
        local dx = target.position.x - transform.translation.x
        local dy = target.position.y - transform.translation.y
        local dz = target.position.z - transform.translation.z
        local dist = math.sqrt(dx*dx + dy*dy + dz*dz)

        if dist > target.snap_threshold then
            -- Snap position, slerp rotation
            entity:set({
                Transform = {
                    translation = target.position,
                    rotation = transform.rotation,  -- Keep current, will slerp next frame
                    scale = target.scale,
                }
            })
        elseif dist > 0.001 then
            -- Lerp position
            entity:set({
                Transform = {
                    translation = {
                        x = transform.translation.x + dx * t,
                        y = transform.translation.y + dy * t,
                        z = transform.translation.z + dz * t,
                    },
                    rotation = transform.rotation,  -- TODO: slerp rotation
                    scale = target.scale,
                }
            })
        end

        ::continue_entities::
    end
end

--------------------------------------------------------------------------------
-- Prediction System
--------------------------------------------------------------------------------

--- Prediction reconciliation system for own entity
--- @param world userdata
function NetSync2.prediction_system(world)
    local entities = world:query({NetSync2.PREDICTION, "Transform", "ScriptOwned"})

    for _, entity in ipairs(entities) do
        local pred = entity:get(NetSync2.PREDICTION)

        -- Skip messages from other instances (e.g. server/client instanced scripts)
        local script_owned = entity:get("ScriptOwned")
        if script_owned.instance_id ~= __INSTANCE_ID__ then
            goto continue_pred
        end

        -- Only process if we have server state to reconcile
        if not pred.server_state then
            goto continue_pred
        end

        local transform = entity:get("Transform")
        local entity_id = entity:id()

        -- DEBUG: Show NS2 prediction system processing
        print(string.format("[DEBUG:NS2] Entity %d processing - ack_seq=%s comp_preds=%d",
            entity_id, tostring(pred.last_acked_sequence),
            pred.predictions and #pred.predictions or 0))

        -- Calculate error
        local dx = pred.server_state.position.x - transform.translation.x
        local dy = pred.server_state.position.y - transform.translation.y
        local dz = pred.server_state.position.z - transform.translation.z
        local error_dist = math.sqrt(dx*dx + dy*dy + dz*dz)

        -- DEBUG: Show error calculation details
        print(string.format("[DEBUG:NS2] Entity %d error=%.2f threshold=%.2f server=(%.2f,%.2f,%.2f) current=(%.2f,%.2f,%.2f)",
            entity_id, error_dist, pred.snap_threshold,
            pred.server_state.position.x, pred.server_state.position.y, pred.server_state.position.z,
            transform.translation.x, transform.translation.y, transform.translation.z))

        if error_dist > pred.snap_threshold then
            -- Large error: snap to server position
            entity:set({
                Transform = {
                    translation = pred.server_state.position,
                    rotation = pred.server_state.rotation or transform.rotation,
                    scale = transform.scale,
                }
            })
            print(string.format("[DEBUG:NS2] Entity %d SNAP to server (error=%.2f > threshold=%.2f)",
                entity_id, error_dist, pred.snap_threshold))
        elseif error_dist > 0.01 then
            -- Small error: smooth correction
            local lerp_factor = math.min(0.3, error_dist * 0.8)
            entity:set({
                Transform = {
                    translation = {
                        x = transform.translation.x + dx * lerp_factor,
                        y = transform.translation.y + dy * lerp_factor,
                        z = transform.translation.z + dz * lerp_factor,
                    },
                    rotation = transform.rotation,
                    scale = transform.scale,
                }
            })
            print(string.format("[DEBUG:NS2] Entity %d LERP correction (error=%.2f, factor=%.2f)",
                entity_id, error_dist, lerp_factor))
        else
            print(string.format("[DEBUG:NS2] Entity %d error=%.4f (within tolerance, no correction)",
                entity_id, error_dist))
        end

        -- Clear server state after processing
        entity:patch({ [NetSync2.PREDICTION] = { server_state = nil } })

        -- Prune old predictions
        if pred.last_acked_sequence then
            local new_predictions = {}
            for seq, data in pairs(pred.predictions or {}) do
                if seq > pred.last_acked_sequence then
                    new_predictions[seq] = data
                end
            end
            entity:patch({ [NetSync2.PREDICTION] = { predictions = new_predictions } })
        end

        ::continue_pred::
    end
end

--------------------------------------------------------------------------------
-- Client Connection Handlers
--------------------------------------------------------------------------------

--- Called when a new client connects (server-side)
--- @param client_id number
--- @param world userdata
function NetSync2.on_client_connected(client_id, world)
    print(string.format("[NET_SYNC2] Client %s connected", client_id))

    -- Initialize scope
    NetSync2.init_client_scope(client_id)

    -- Spawn client_id message
    spawn({
        [NetSync2.OUTBOUND] = {
            msg_type = "client_id",
            channel = CHANNEL_RELIABLE,
            target_clients = { client_id },
            payload = { client_id = client_id }
        }
    })

    -- Send all existing entities to new client
    local count = 0
    for net_id, entity_id in pairs(known_entities) do
        local entity = world:get_entity(entity_id)
        if entity then
            local sync = entity:get(NetSync2.MARKER)
            if sync and sync.authority ~= "remote" then
                local spawn_msg = NetSync2.build_spawn_msg(world, entity, net_id)
                if spawn_msg then
                    spawn_msg.target_clients = { client_id }
                    spawn({ [NetSync2.OUTBOUND] = spawn_msg })
                    NetSync2.add_to_client_scope(client_id, net_id)
                    count = count + 1
                end
            end
        end
    end

    print(string.format("[NET_SYNC2] Spawned %d entity messages for client %s", count, client_id))
end

--- Called when a client disconnects (server-side)
--- @param client_id number
--- @param world userdata
--- @param get_clients function
function NetSync2.on_client_disconnected(client_id, world, get_clients)
    print(string.format("[NET_SYNC2] Client %s disconnected", client_id))

    -- Find and despawn entities owned by this client
    for net_id, owner in pairs(net_id_owners) do
        if owner == client_id then
            local despawn_msg = NetSync2.build_despawn_msg(owner, net_id)

            -- Send to remaining clients
            local remaining = {}
            for _, cid in ipairs(get_clients()) do
                if cid ~= client_id and NetSync2.client_knows_entity(cid, net_id) then
                    table.insert(remaining, cid)
                    NetSync2.remove_from_client_scope(cid, net_id)
                end
            end
            despawn_msg.target_clients = remaining

            spawn({ [NetSync2.OUTBOUND] = despawn_msg })

            -- Despawn locally
            local entity_id = known_entities[net_id]
            if entity_id then
                despawn(entity_id)
                known_entities[net_id] = nil
            end

            net_id_owners[net_id] = nil
        end
    end

    -- Remove client scope
    NetSync2.remove_client_scope(client_id)
end

--------------------------------------------------------------------------------
-- Utility
--------------------------------------------------------------------------------

--- Get the current handle_update function (for external override if needed)
NetSync2.handle_spawn = handle_spawn
NetSync2.handle_update = handle_update
NetSync2.handle_despawn = handle_despawn

return NetSync2
