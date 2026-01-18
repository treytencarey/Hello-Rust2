-- Network Sync Module
-- Generic ECS-based entity replication using NetworkSync component
--
-- Usage:
--   entity:set({ NetworkSync = { net_id = NetSync.next_net_id(), authority = "owner" } })
--   -- Change detection automatically picks up new/changed entities
--
--   register_system("Update", function(world) NetSync.outbound_system(world) end)
--   register_system("Update", function(world) NetSync.inbound_system(world) end)

local NetSync = {}

--------------------------------------------------------------------------------
-- Marker Component
--------------------------------------------------------------------------------

NetSync.MARKER = "NetworkSync"  -- Component to mark spawned entities

--------------------------------------------------------------------------------
-- JSON (using dkjson library)
--------------------------------------------------------------------------------

local json = require("modules/dkjson.lua")
local NetRole = require("modules/net_role.lua")

local function json_encode(tbl)
    return json.encode(tbl)
end

local function json_decode(str)
    local result, pos, err = json.decode(str)
    if err then
        print("[NET_SYNC] JSON decode error: " .. tostring(err))
        return nil
    end
    return result
end

--- Create canonical representation of a value for deterministic hashing
--- Sorts table keys recursively so JSON encoding is consistent
local function canonical_value(v)
    if type(v) ~= "table" then return v end
    
    -- Collect and sort keys
    local sorted_keys = {}
    for k in pairs(v) do table.insert(sorted_keys, k) end
    table.sort(sorted_keys, function(a, b) return tostring(a) < tostring(b) end)
    
    -- Build ordered result
    local result = {}
    for _, k in ipairs(sorted_keys) do
        result[k] = canonical_value(v[k])
    end
    
    -- Use ordered pairs iteration via metatable
    setmetatable(result, {__pairs = function(t)
        local i = 0
        return function()
            i = i + 1
            local k = sorted_keys[i]
            if k then return k, t[k] end
        end
    end})
    return result
end

-- Export JSON functions for other modules
NetSync.json_encode = json_encode
NetSync.json_decode = json_decode

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

local CHANNEL_RELIABLE = 0
local CHANNEL_UNRELIABLE = 1

-- Client prefix for net_id generation (set by server on connect)
local client_prefix = 0
local local_counter = 0

-- Use a GLOBAL shared table so server and client scripts share the same data
-- This is necessary in "both" mode where server and client are separate script instances
-- but need to share entity registration data
if not __NET_SYNC_SHARED__ then
    __NET_SYNC_SHARED__ = {
        known_entities = {},   -- net_id -> entity_id
        my_net_id = nil,       -- The net_id assigned to THIS client's character
    }
    print("[NET_SYNC] Created shared global __NET_SYNC_SHARED__")
end

-- Alias for easier access
local shared = __NET_SYNC_SHARED__

-- These remain per-script-instance as they don't need to be shared
-- Pending children waiting for parent to spawn
local pending_children = {}  -- net_id -> spawn_data

-- Last sync time per entity per component (for rate limiting)
local last_sync_times = {}  -- entity_id -> { component_name -> timestamp }

-- Last synced values per entity per component (for change detection)
local last_synced_values = {}  -- entity_id -> { component_name -> value_hash }

-- Entities we've sent spawn messages for
local spawned_net_ids = {}  -- net_id -> true

-- Cache of known syncable component types with reference counts
local known_sync_types = {}  -- comp_name -> reference_count

-- Cache of each entity's sync_components config (to detect config changes)
local entity_sync_config = {}  -- entity_id -> { comp_name -> config }

-- Interpolation targets for smooth transform updates
local interpolation_targets = {}  -- entity_id -> { position, rotation, timestamp }

-- Server's authoritative position for our own entity (for reconciliation)
-- We don't apply it to Transform (prediction handles that), but we use it to reconcile
local server_authoritative_state = nil  -- { position, velocity }

-- Protocol-level sequence tracking (NOT in components)
local entity_sequences = {}       -- entity_id -> outbound sequence counter (client-side)
local last_acked_sequences = {}   -- entity_id -> last ack received from server (client-side)
local client_input_sequences = {} -- entity_id -> last seq received from client (server-side, for ack_seq)

--------------------------------------------------------------------------------
-- Server Scope Tracking (for server-side use only)
--------------------------------------------------------------------------------

-- Track which net_ids each client knows about (server only)
local client_scope = {}  -- client_id -> { net_id = true, ... }

-- Track which client owns which net_id (server only)
local net_id_owners = {}  -- net_id -> client_id

-- Queue for "your_character" messages to send during outbound (server only)
local pending_your_character = {}  -- { {client_id, net_id}, ... }

--- Get interpolation targets (for use by transform_interpolation module)
function NetSync.get_interpolation_targets()
    return interpolation_targets
end

--- Get server's authoritative state for our own entity (for reconciliation)
function NetSync.get_server_authoritative_state()
    return server_authoritative_state
end

--- Clear server's authoritative state after reconciliation processes it
function NetSync.clear_server_authoritative_state()
    server_authoritative_state = nil
end

--------------------------------------------------------------------------------
-- Sequence Tracking (protocol-level, not in components)
--------------------------------------------------------------------------------

--- Get next sequence number for outbound updates (increments counter)
--- @param entity_id number
--- @return number sequence number
function NetSync.get_next_sequence(entity_id)
    entity_sequences[entity_id] = (entity_sequences[entity_id] or 0) + 1
    return entity_sequences[entity_id]
end

--- Get current sequence number without incrementing
--- @param entity_id number
--- @return number sequence number
function NetSync.get_current_sequence(entity_id)
    return entity_sequences[entity_id] or 0
end

--- Get last acknowledged sequence from server (for reconciliation)
--- @param entity_id number
--- @return number|nil last acked sequence or nil if none
function NetSync.get_last_acked_sequence(entity_id)
    return last_acked_sequences[entity_id]
end

--- Set last acknowledged sequence (called when receiving server ack)
--- @param entity_id number
--- @param seq number
function NetSync.set_last_acked_sequence(entity_id, seq)
    last_acked_sequences[entity_id] = seq
end

--------------------------------------------------------------------------------
-- Net ID Management
--------------------------------------------------------------------------------

--- Set client prefix (called by server on connect)
function NetSync.set_client_prefix(prefix)
    client_prefix = prefix
    print(string.format("[NET_SYNC] Client prefix set to %d", prefix))
end

--- Generate next unique net_id
function NetSync.next_net_id()
    local_counter = local_counter + 1
    return client_prefix * 10000 + local_counter
end

--- Get entity by net_id
function NetSync.get_entity(net_id)
    return shared.known_entities[net_id]
end

--- Register known entity
function NetSync.register_entity(net_id, entity_id)
    shared.known_entities[net_id] = entity_id
    print(string.format("[NET_SYNC] Registered entity net_id=%s entity_id=%s (shared table=%s)", 
        tostring(net_id), tostring(entity_id), tostring(shared)))
end

--- Get all known net_ids (for late-join sync)
--- @return table Array of net_ids
function NetSync.get_all_net_ids()
    local result = {}
    for net_id, _ in pairs(shared.known_entities) do
        table.insert(result, net_id)
    end
    return result
end

--- Get the net_id assigned to this client's character (nil if not yet assigned)
--- @return number|nil
function NetSync.get_my_net_id()
    return shared.my_net_id
end

--- Set the net_id for this client's character (for "both" mode where server sets it directly)
--- @param net_id number
function NetSync.set_my_net_id(net_id)
    shared.my_net_id = net_id
    print(string.format("[NET_SYNC] Set my character assignment: net_id=%s (shared table=%s)", tostring(net_id), tostring(shared)))
end

--- Get the entity_id of this client's character (nil if not yet spawned)
--- @return number|nil
function NetSync.get_my_entity()
    if shared.my_net_id then
        local entity_id = shared.known_entities[shared.my_net_id]
        if not entity_id then
            -- Debug: dump all known entities
            local count = 0
            for k, v in pairs(shared.known_entities) do
                count = count + 1
                if count <= 5 then -- Only print first 5 to avoid spam
                    print(string.format("[NET_SYNC DEBUG] shared.known_entities[%s] = %s", tostring(k), tostring(v)))
                end
            end
            if count == 0 then
                print("[NET_SYNC DEBUG] shared.known_entities is EMPTY! shared=" .. tostring(shared))
            end
        end
        return entity_id
    end
    return nil
end

--- Get spawn message for a specific net_id (for late-join sync)
--- @param world userdata
--- @param net_id number
--- @return string|nil JSON spawn message or nil if not found
function NetSync.get_spawn_message_for(world, net_id)
    local entity_id = shared.known_entities[net_id]
    if not entity_id then 
        print(string.format("[NET_SYNC] get_spawn_message_for: net_id=%d not in known_entities", net_id))
        return nil 
    end
    
    local entity = world:get_entity(entity_id)
    if not entity then 
        print(string.format("[NET_SYNC] get_spawn_message_for: net_id=%d entity_id=%s world:get_entity returned nil", 
            net_id, tostring(entity_id)))
        return nil 
    end
    
    local spawn_msg = NetSync.build_spawn_msg(world, entity, net_id)
    return json_encode(spawn_msg)
end

--------------------------------------------------------------------------------
-- Server Client Management (for server-side use only)
--------------------------------------------------------------------------------

--- Called when a new client connects - sends late-join sync
--- @param client_id number The client ID
--- @param world userdata The ECS world
--- @param send_fn function(client_id, channel, msg_str) Send function
function NetSync.on_client_connected(client_id, world, send_fn)
    client_scope[client_id] = {}
    
    -- Late-join sync: Send all existing entities to new client
    local all_net_ids = NetSync.get_all_net_ids()
    for _, net_id in ipairs(all_net_ids) do
        local spawn_msg = NetSync.get_spawn_message_for(world, net_id)
        if spawn_msg then
            print(string.format("[NET_SYNC] Sending spawn to new client %s: net_id=%d", client_id, net_id))
            send_fn(client_id, CHANNEL_RELIABLE, spawn_msg)
            client_scope[client_id][net_id] = true
        end
    end
end

--- Called when a client disconnects - cleans up scope and notifies other clients
--- @param client_id number The client ID
--- @param send_fn function(client_id, channel, msg_str) Send function
--- @param get_clients_fn function() Returns array of connected client IDs
function NetSync.on_client_disconnected(client_id, send_fn, get_clients_fn)
    -- Find entities owned by this client and send despawns to others
    local clients = get_clients_fn()
    for net_id, owner in pairs(net_id_owners) do
        if owner == client_id then
            local despawn_msg = json_encode({ type = "despawn", net_id = net_id })
            for _, other_id in ipairs(clients) do
                if other_id ~= client_id then
                    send_fn(other_id, CHANNEL_RELIABLE, despawn_msg)
                end
            end
            net_id_owners[net_id] = nil
            
            -- Remove from all client scopes
            for _, scope in pairs(client_scope) do
                scope[net_id] = nil
            end
        end
    end
    
    client_scope[client_id] = nil
    print(string.format("[NET_SYNC] Client %s disconnected, cleaned up scope", client_id))
end

--- Check if a client is known (has scope tracking initialized)
--- @param client_id number
--- @return boolean
function NetSync.is_client_known(client_id)
    return client_scope[client_id] ~= nil
end

--- Get all known client IDs (for detecting disconnections)
--- @return table Array of client_ids
function NetSync.get_known_clients()
    local result = {}
    for client_id, _ in pairs(client_scope) do
        table.insert(result, client_id)
    end
    return result
end

--- Get the owner of a net_id
--- @param net_id number
--- @return number|nil client_id or nil
function NetSync.get_net_id_owner(net_id)
    return net_id_owners[net_id]
end

--- Set the owner of a net_id (called when processing spawn from client)
--- @param net_id number
--- @param client_id number|nil
function NetSync.set_net_id_owner(net_id, client_id)
    net_id_owners[net_id] = client_id
    if client_id then
        -- Mark owner as knowing their own entity
        client_scope[client_id] = client_scope[client_id] or {}
        client_scope[client_id][net_id] = true
        
        -- Queue "your_character" message to be sent during outbound_system
        table.insert(pending_your_character, {client_id = client_id, net_id = net_id})
        print(string.format("[NET_SYNC] Queued your_character for client %s: net_id=%d", client_id, net_id))
    end
end

--- Check if a client knows about a specific entity
--- @param client_id number
--- @param net_id number
--- @return boolean
function NetSync.client_knows_entity(client_id, net_id)
    local scope = client_scope[client_id]
    return scope and scope[net_id] == true
end

--- Remove an entity from a client's scope (for filtered despawns)
--- @param client_id number
--- @param net_id number
function NetSync.remove_from_scope(client_id, net_id)
    local scope = client_scope[client_id]
    if scope then
        scope[net_id] = nil
    end
end

--- Add an entity to a client's scope
--- @param client_id number
--- @param net_id number
function NetSync.add_to_scope(client_id, net_id)
    client_scope[client_id] = client_scope[client_id] or {}
    client_scope[client_id][net_id] = true
end

--- Get all net_ids owned by a specific client
--- @param client_id number
--- @return table Array of net_ids
function NetSync.get_net_ids_for_client(client_id)
    local result = {}
    for net_id, owner in pairs(net_id_owners) do
        if owner == client_id then
            table.insert(result, net_id)
        end
    end
    return result
end

--------------------------------------------------------------------------------
-- Message Building
--------------------------------------------------------------------------------

--- Build spawn message with all components
function NetSync.build_spawn_msg(world, entity, net_id)
    -- IMPORTANT: The 'entity' parameter from query() only has queried components.
    -- We need a FULL snapshot with all components, so use world:get_entity().
    local entity_id = entity:id()
    local full_entity = world:get_entity(entity_id)
    if not full_entity then
        -- Fall back to using the query entity (limited but better than nothing)
        full_entity = entity
    end
    
    local sync = full_entity:get(NetSync.MARKER)
    
    local msg = {
        type = "spawn",
        net_id = net_id,
        authority = sync.authority or "owner",
        components = {}
    }
    
    -- Get parent net_id if entity has ChildOf/Parent
    if full_entity:has("ChildOf") then
        local child_of = full_entity:get("ChildOf")
        if child_of and child_of.parent then
            local parent_entity = world:get_entity(child_of.parent)
            if parent_entity and parent_entity:has(NetSync.MARKER) then
                local parent_sync = parent_entity:get(NetSync.MARKER)
                msg.parent_net_id = parent_sync.net_id
            end
        end
    end
    
    -- Serialize components (based on sync_components config or all)
    local components_to_sync = sync.sync_components
    if components_to_sync then
        for comp_name, _ in pairs(components_to_sync) do
            if full_entity:has(comp_name) then
                msg.components[comp_name] = full_entity:get(comp_name)
            else
                print(string.format("[NET_SYNC] build_spawn_msg: Entity missing %s", comp_name))
            end
        end
    else
        -- Sync Transform by default
        if full_entity:has("Transform") then
            msg.components["Transform"] = full_entity:get("Transform")
        end
    end
    
    -- Include NetworkSync itself
    msg.components[NetSync.MARKER] = sync
    
    return msg
end

--- Build update message with only changed components
--- @param entity userdata
--- @param net_id number
--- @param changed_components table
--- @param seq number|nil optional sequence number (for client->server)
--- @param ack_seq number|nil optional ack sequence (for server->client)
function NetSync.build_update_msg(entity, net_id, changed_components, seq, ack_seq)
    local msg = {
        type = "update",
        net_id = net_id,
        components = changed_components
    }
    if seq then msg.seq = seq end
    if ack_seq then msg.ack_seq = ack_seq end
    return msg
end

--------------------------------------------------------------------------------
-- Outbound System (Local → Network)
--------------------------------------------------------------------------------

--- Increment reference count for a component type
local function add_sync_type_ref(comp_name)
    known_sync_types[comp_name] = (known_sync_types[comp_name] or 0) + 1
end

--- Decrement reference count for a component type (removes if reaches 0)
local function remove_sync_type_ref(comp_name)
    local count = known_sync_types[comp_name]
    if count then
        if count <= 1 then
            known_sync_types[comp_name] = nil
        else
            known_sync_types[comp_name] = count - 1
        end
    end
end

--- Check if entity can sync a component based on authority
local function can_sync_component(comp_authority)
    if comp_authority == "server" and NetRole.is_server() then
        return true
    elseif comp_authority == "client" and NetRole.is_client() then
        return true
    elseif comp_authority == "owner" or comp_authority == "any" then
        return true
    end
    return false
end

--- Query for changed NetworkSync entities and send updates
--- @param world userdata
--- @param send_fn function(client_id, channel, msg_str) - Send to specific client (for server)
---                OR function(channel, msg_str) - Simple broadcast (for client)
--- @param get_clients_fn function()|nil - Returns array of client_ids (server only, nil for clients)
function NetSync.outbound_system(world, send_fn, get_clients_fn)
    if not send_fn then return end
    
    -- Use get_clients_fn presence to determine server context (more reliable than NetRole in "both" mode)
    -- Server context: get_clients_fn provided, iterate clients with scope tracking
    -- Client context: get_clients_fn nil, simple broadcast to server
    
    local now = os.clock()
    local entities_to_spawn = {}  -- entity_id -> entity
    local changed_components = {}  -- entity_id -> { comp_name -> true }
    
    -- Step 1: Query for NetworkSync changes (new entities or config changes)
    local network_sync_changed = world:query({NetSync.MARKER}, {NetSync.MARKER})
    for _, entity in ipairs(network_sync_changed) do
        local sync = entity:get(NetSync.MARKER)
        if not sync then goto continue_ns end
        
        local entity_id = entity:id()
        local current_config = sync.sync_components or { Transform = { rate_hz = 30 } }
        local cached_config = entity_sync_config[entity_id]
        
        if not cached_config then
            -- New entity: cache config, increment reference counts
            entity_sync_config[entity_id] = current_config
            for comp_name, _ in pairs(current_config) do
                add_sync_type_ref(comp_name)
            end
            entities_to_spawn[entity_id] = entity
        else
            -- Existing entity: check if config changed
            local config_changed = false
            for comp_name, _ in pairs(current_config) do
                if not cached_config[comp_name] then config_changed = true break end
            end
            for comp_name, _ in pairs(cached_config) do
                if not current_config[comp_name] then config_changed = true break end
            end
            
            if config_changed then
                -- Remove references from old config
                for comp_name, _ in pairs(cached_config) do
                    if not current_config[comp_name] then
                        remove_sync_type_ref(comp_name)
                    end
                end
                -- Add references from new config
                for comp_name, _ in pairs(current_config) do
                    if not cached_config[comp_name] then
                        add_sync_type_ref(comp_name)
                    end
                end
                entity_sync_config[entity_id] = current_config
            end
        end
        
        ::continue_ns::
    end
    
    -- Step 2: Query for component data changes (only types we know about)
    for comp_name, _ in pairs(known_sync_types) do
        local comp_changed = world:query({NetSync.MARKER, comp_name}, {comp_name})
        for _, entity in ipairs(comp_changed) do
            local entity_id = entity:id()
            -- Skip entities marked for spawn (they send all components anyway)
            if not entities_to_spawn[entity_id] then
                changed_components[entity_id] = changed_components[entity_id] or {}
                changed_components[entity_id][comp_name] = true
            end
        end
    end
    
    -- Step 3a: Process spawn entities (send all sync_components)
    for entity_id, entity in pairs(entities_to_spawn) do
        local sync = entity:get(NetSync.MARKER)
        local net_id = sync.net_id

        -- Skip entities we received from network (authority == "remote")
        -- They shouldn't be sent back out
        if sync.authority == "remote" then
            goto continue_spawn
        end
        
        -- Skip client predictions: client + authority="server" + no net_id
        -- These are local predictions waiting for server confirmation
        local is_client_prediction = (not NetRole.is_server()) 
            and (sync.authority == "server") 
            and (net_id == nil)
        if is_client_prediction then
            print(string.format("[NET_SYNC] Skipping client prediction entity_id=%s (pending server confirmation)", tostring(entity_id)))
            goto continue_spawn
        end
        
        local full_entity = world:get_entity(entity_id)
        if not full_entity then goto continue_spawn end
        
        -- Server: auto-assign net_id if not set
        if NetRole.is_server() and net_id == nil then
            net_id = NetSync.next_net_id()
            sync.net_id = net_id
            entity:patch({ [NetSync.MARKER] = sync })
            print(string.format("[NET_SYNC] Auto-assigned net_id=%d to entity_id=%s", net_id, tostring(entity_id)))
            
            -- Auto-detect owner from PlayerState.owner_client if present
            if full_entity:has("PlayerState") then
                local player_state = full_entity:get("PlayerState")
                if player_state and player_state.owner_client then
                    NetSync.set_net_id_owner(net_id, player_state.owner_client)
                    print(string.format("[NET_SYNC] Auto-set owner for net_id=%d to client=%s", 
                        net_id, tostring(player_state.owner_client)))
                end
            end
        end
        
        if net_id and spawned_net_ids[net_id] then goto continue_spawn end
        
        -- Build spawn message
        local spawn_msg = NetSync.build_spawn_msg(world, full_entity, net_id)
        local spawn_msg_str = json_encode(spawn_msg)
        
        -- Mark as spawned locally
        spawned_net_ids[net_id] = true
        
        -- Register in known_entities (important for "both" mode - client side will see it's known)
        NetSync.register_entity(net_id, entity_id)
        
        -- Send to clients
        if get_clients_fn then
            -- Server context: send to clients with scope tracking
            for _, client_id in ipairs(get_clients_fn()) do
                send_fn(client_id, CHANNEL_RELIABLE, spawn_msg_str)
                client_scope[client_id] = client_scope[client_id] or {}
                client_scope[client_id][net_id] = true
            end
        else
            -- Client context: simple broadcast to server)
            send_fn(CHANNEL_RELIABLE, spawn_msg_str)
        end
        
        -- Initialize hash cache for future change detection
        local sync_comps = sync.sync_components or { Transform = { rate_hz = 30 } }
        last_synced_values[entity_id] = {}
        last_sync_times[entity_id] = {}
        for comp_name, _ in pairs(sync_comps) do
            if full_entity:has(comp_name) then
                last_synced_values[entity_id][comp_name] = json_encode(canonical_value(full_entity:get(comp_name)))
                last_sync_times[entity_id][comp_name] = now
            end
        end
        
        ::continue_spawn::
    end
    
    -- Step 3b: Process entities with changed components (send only changed)
    for entity_id, comps_changed in pairs(changed_components) do
        local full_entity = world:get_entity(entity_id)
        if not full_entity then goto continue_update end
        if not full_entity:has(NetSync.MARKER) then goto continue_update end
        local sync = full_entity:get(NetSync.MARKER)
        if not sync or not sync.net_id then goto continue_update end
        
        local net_id = sync.net_id
        
        -- Skip if not yet spawned (will be caught next frame)
        if not spawned_net_ids[net_id] then goto continue_update end
        
        local entity_times = last_sync_times[entity_id] or {}
        last_sync_times[entity_id] = entity_times
        
        local entity_values = last_synced_values[entity_id] or {}
        last_synced_values[entity_id] = entity_values
        
        local sync_comps = sync.sync_components or { Transform = { rate_hz = 30 } }
        local components_to_send = {}
        local has_changes = false
        local needs_reliable = false
        
        -- Only check components that ECS reported as changed
        for comp_name, _ in pairs(comps_changed) do
            local config = sync_comps[comp_name]
            -- Skip if not in this entity's sync_components
            if not config then goto continue_comp end

            -- Authority check
            local comp_authority = config.authority or sync.authority or "owner"
            if not can_sync_component(comp_authority) then goto continue_comp end
            
            -- Rate limit check
            local rate_hz = config.rate_hz or 30
            local interval = 1.0 / rate_hz
            local last_time = entity_times[comp_name] or 0
            if (now - last_time) < interval then goto continue_comp end
            
            -- Hash check (final verification)
            if full_entity:has(comp_name) then
                local current_value = full_entity:get(comp_name)
                local current_hash = json_encode(canonical_value(current_value))
                local last_hash = entity_values[comp_name]
                
                if current_hash ~= last_hash then
                    components_to_send[comp_name] = current_value
                    entity_times[comp_name] = now
                    entity_values[comp_name] = current_hash
                    has_changes = true
                    if config.reliable then needs_reliable = true end
                end
            end
            
            ::continue_comp::
        end
        
        if has_changes then
            local channel = needs_reliable and CHANNEL_RELIABLE or CHANNEL_UNRELIABLE
            
            if get_clients_fn then
                -- Server context: scope-aware sending with client list
                -- Spawn for clients who don't know, update for those who do
                -- Also handle filter_fn for area-of-interest
                local owner_client = net_id_owners[net_id]
                local spawn_msg_str = nil  -- Lazy build
                local update_msg_str = nil -- Lazy build
                local despawn_msg_str = nil -- Lazy build
                
                for _, client_id in ipairs(get_clients_fn()) do
                    local scope = client_scope[client_id] or {}
                    local knows_entity = scope[net_id]
                    
                    if knows_entity then
                        -- Send update
                        -- Include ack_seq for entity owner (for reconciliation)
                        local is_owner = (client_id == owner_client)
                        local ack_seq = nil
                        if is_owner then
                            -- Get the last sequence we received from this client's input
                            local entity_id = full_entity:id()
                            ack_seq = client_input_sequences[entity_id]  -- Last received from client
                        end
                        
                        if is_owner or not update_msg_str then
                            local update_msg = NetSync.build_update_msg(full_entity, net_id, components_to_send, nil, ack_seq)
                            if is_owner then
                                -- Send personalized message with ack_seq to owner
                                send_fn(client_id, channel, json_encode(update_msg))
                            else
                                -- Cache for non-owners
                                update_msg_str = json_encode(update_msg)
                            end
                        end
                        
                        if not is_owner then
                            send_fn(client_id, channel, update_msg_str)
                        end
                    else
                        -- Send spawn (full state) for clients who don't know this entity
                        if not spawn_msg_str then
                            local spawn_msg = NetSync.build_spawn_msg(world, full_entity, net_id)
                            spawn_msg_str = json_encode(spawn_msg)
                        end
                        send_fn(client_id, CHANNEL_RELIABLE, spawn_msg_str)
                        client_scope[client_id] = client_scope[client_id] or {}
                        client_scope[client_id][net_id] = true
                    end

                    ::continue_send_client::
                end
            else
                -- Client context: simple broadcast to server with sequence
                local entity_id = full_entity:id()
                entity_sequences[entity_id] = (entity_sequences[entity_id] or 0) + 1
                local seq = entity_sequences[entity_id]
                local update_msg = NetSync.build_update_msg(full_entity, net_id, components_to_send, seq, nil)
                print(string.format("[NET_SYNC] Sending update message to server (net_id=%d): %s", net_id, json_encode(update_msg)))
                send_fn(channel, json_encode(update_msg))
            end
        end
        
        ::continue_update::
    end
    
    -- Step 4: Detect removed NetworkSync entities and send despawn messages
    -- Uses world:query_removed() to get entities that had NetworkSync removed this frame
    local removed_entities = world:query_removed({NetSync.MARKER})
    for _, entity_bits in ipairs(removed_entities) do
        -- Find the net_id for this entity
        local removed_net_id = nil
        for net_id, known_entity_id in pairs(shared.known_entities) do
            if known_entity_id == entity_bits then
                removed_net_id = net_id
                break
            end
        end
        
        if removed_net_id then
            print(string.format("[NET_SYNC] Detected despawned entity: net_id=%d entity_bits=%s", 
                removed_net_id, tostring(entity_bits)))
            
            -- Send despawn message
            local despawn_msg_str = json_encode({ type = "despawn", net_id = removed_net_id })
            
            if get_clients_fn then
                -- Server context: send to all clients who know this entity
                for _, client_id in ipairs(get_clients_fn()) do
                    if client_scope[client_id] and client_scope[client_id][removed_net_id] then
                        send_fn(client_id, CHANNEL_RELIABLE, despawn_msg_str)
                        client_scope[client_id][removed_net_id] = nil
                    end
                end
            else
                -- Client context: broadcast to server
                send_fn(CHANNEL_RELIABLE, despawn_msg_str)
            end
            
            -- Clean up tracking state
            shared.known_entities[removed_net_id] = nil
            spawned_net_ids[removed_net_id] = nil
            net_id_owners[removed_net_id] = nil
            last_sync_times[entity_bits] = nil
            last_synced_values[entity_bits] = nil
            if entity_sync_config[entity_bits] then
                for comp_name, _ in pairs(entity_sync_config[entity_bits]) do
                    remove_sync_type_ref(comp_name)
                end
                entity_sync_config[entity_bits] = nil
            end
        end
    end
    
    -- Step 5: Send queued "your_character" messages (server only)
    if get_clients_fn and #pending_your_character > 0 then
        for _, msg_data in ipairs(pending_your_character) do
            local your_char_msg = json_encode({ type = "your_character", net_id = msg_data.net_id })
            send_fn(msg_data.client_id, CHANNEL_RELIABLE, your_char_msg)
            print(string.format("[NET_SYNC] Sent your_character to client %s: net_id=%d", 
                msg_data.client_id, msg_data.net_id))
        end
        pending_your_character = {}  -- Clear queue
    end
end

--------------------------------------------------------------------------------
-- Inbound System (Network → Local)
--------------------------------------------------------------------------------

--- Process received messages and apply to entities
--- @param world userdata
--- @param receive_fn function(channel) -> message_string or nil
function NetSync.inbound_system(world, receive_fn)
    if not receive_fn then return end
    
    -- Process reliable channel first
    while true do
        local msg_str = receive_fn(CHANNEL_RELIABLE)
        if not msg_str then break end
        NetSync.process_message(world, json_decode(msg_str))
    end
    
    -- Then unreliable
    while true do
        local msg_str = receive_fn(CHANNEL_UNRELIABLE)
        if not msg_str then break end
        NetSync.process_message(world, json_decode(msg_str))
    end
    
    -- Process pending children
    NetSync.process_pending_children(world)
end

--- Process a single message
function NetSync.process_message(world, msg)
    if not msg or not msg.type then return end
    
    if msg.type == "spawn" then
        NetSync.handle_spawn(world, msg)
    elseif msg.type == "update" then
        NetSync.handle_update(world, msg)
    elseif msg.type == "despawn" then
        NetSync.handle_despawn(world, msg)
    elseif msg.type == "owner_change" then
        NetSync.handle_owner_change(world, msg)
    elseif msg.type == "your_character" then
        -- Server is telling us which character is ours
        NetSync.set_my_net_id(msg.net_id)
    end

    -- Debug: print message processing
    print(string.format("[NET_SYNC] process_message: type=%s net_id=%s", 
        tostring(msg.type), tostring(msg.net_id)))
    print(string.format("[NET_SYNC] %s", json_encode(msg)))
end

--- Handle spawn message
function NetSync.handle_spawn(world, msg)
    local net_id = msg.net_id
    if not net_id then return end

    print(string.format("[NET_SYNC] handle_spawn: net_id=%d shared=%s known_entities[%d]=%s",
        net_id, tostring(shared), net_id, tostring(shared.known_entities[net_id])))
    
    -- Check if this is our character and we have a pending prediction to patch
    if net_id == shared.my_net_id then
        local results = world:query({NetSync.MARKER})
        for _, entity in ipairs(results) do
            local sync = entity:get(NetSync.MARKER)
            -- Find pending prediction: authority="server" and no net_id
            if sync.authority == "server" and sync.net_id == nil then
                -- Patch the pending entity with server's net_id
                local server_sync = msg.components and msg.components[NetSync.MARKER] or {}
                server_sync.net_id = net_id
                entity:patch({ [NetSync.MARKER] = server_sync })
                shared.known_entities[net_id] = entity:id()
                print(string.format("[NET_SYNC] Patched pending prediction with net_id=%d entity_id=%s", 
                    net_id, tostring(entity:id())))
                return  -- Done, don't spawn new entity
            end
        end
    end
    
    -- Skip if already known
    if shared.known_entities[net_id] then
        print(string.format("[NET_SYNC] handle_spawn: SKIPPING net_id=%d - already known as entity_id=%s",
            net_id, tostring(shared.known_entities[net_id])))
        -- Just update components
        NetSync.handle_update(world, msg)
        return
    end
    
    -- Check if parent exists (if needed)
    if msg.parent_net_id and not shared.known_entities[msg.parent_net_id] then
        -- Queue for later
        pending_children[net_id] = msg
        print(string.format("[NET_SYNC] Queued spawn net_id=%d waiting for parent=%d", net_id, msg.parent_net_id))
        return
    end
    
    -- Spawn the entity
    local spawn_data = msg.components or {}
    spawned_net_ids[net_id] = true
    
    -- Ensure NetworkSync component has remote authority
    spawn_data.NetworkSync = spawn_data.NetworkSync or {}
    spawn_data.NetworkSync.net_id = net_id
    spawn_data.NetworkSync.authority = "remote"  -- Mark as not ours
    
    print(string.format("[NET_SYNC] handle_spawn: BEFORE SPAWN net_id=%d authority=%s", 
        net_id, tostring(spawn_data.NetworkSync.authority)))
    
    local entity_id = spawn(spawn_data):id()
    
    -- Parent if needed
    if msg.parent_net_id then
        local parent_entity_id = shared.known_entities[msg.parent_net_id]
        if parent_entity_id then
            with_parent(entity_id, parent_entity_id)
        end
    end
    
    shared.known_entities[net_id] = entity_id
    print(string.format("[NET_SYNC] Spawned remote entity net_id=%s entity_id=%s", tostring(net_id), tostring(entity_id)))
    
    -- Check if any pending children can now spawn
    NetSync.process_pending_children(world)
end

--- Handle update message
function NetSync.handle_update(world, msg)
    local json = require("modules/dkjson.lua")
    local json_encode = json.encode

    local now = os.clock()
    
    local net_id = msg.net_id
    if not net_id then return end
    
    local entity_id = shared.known_entities[net_id]
    if not entity_id then 
        -- Unknown entity - could request spawn (NAK)
        print(string.format("[NET_SYNC] Update for unknown net_id=%d", net_id))
        return
    end
    
    local entity = world:get_entity(entity_id)
    if not entity then return end
    
    -- Track if this is our own entity (prediction handles Transform, but we need PlayerState for reconciliation)
    local is_own_entity = (net_id == shared.my_net_id)
    
    -- Protocol-level sequence handling (NOT in components)
    if NetRole.is_server() and msg.seq then
        -- Server: track last received sequence from client for ack_seq
        client_input_sequences[entity_id] = msg.seq
    end
    
    if NetRole.is_client() and is_own_entity and msg.ack_seq then
        -- Client: store ack for reconciliation
        last_acked_sequences[entity_id] = msg.ack_seq
    end
    
    -- Get entity's NetworkSync for authority checks
    local sync = entity:get(NetSync.MARKER)
    
    -- Apply component updates (with authority validation on server)
    for comp_name, comp_data in pairs(msg.components or {}) do
        if comp_name == NetSync.MARKER then
            goto continue_comp  -- Never overwrite NetworkSync
        end

        -- Skip Transform for own entity - client prediction handles it
        -- BUT store server's position for reconciliation
        if is_own_entity and comp_name == "Transform" then
            server_authoritative_state = {
                position = comp_data.translation,
                rotation = comp_data.rotation
            }
            print(string.format("[NET_SYNC] Stored server auth state: (%.2f,%.2f,%.2f)",
                comp_data.translation.x, comp_data.translation.y, comp_data.translation.z))
            goto continue_comp
        end

        -- Server-side authority validation - ALWAYS check for server authority components
        if NetRole.is_server() then
            local sync_comps = sync and sync.sync_components
            local config = sync_comps and sync_comps[comp_name]
            -- Default to "server" authority if not specified (safe default)
            local comp_authority = (config and config.authority) or (sync and sync.authority) or "server"

            -- Server only accepts updates for "client" authority components
            if comp_authority ~= "client" then
                print(string.format("[NET_SYNC] SERVER REJECTING %s update (net_id=%d) - authority=%s, sync_comps=%s",
                    comp_name, net_id, comp_authority, tostring(sync_comps ~= nil)))
                goto continue_comp
            else
                print(string.format("[NET_SYNC] SERVER ACCEPTING %s update (net_id=%d) - authority=%s",
                    comp_name, net_id, comp_authority))
            end
        end
        
        -- Check if this component should be interpolated
        local config = sync and sync.sync_components and sync.sync_components[comp_name]
        if config and config.interpolate and comp_name == "Transform" then
            -- Queue for interpolation instead of direct apply
            interpolation_targets[entity_id] = {
                position = comp_data.translation,
                rotation = comp_data.rotation,
                scale = comp_data.scale,
                timestamp = os.clock()
            }
            goto continue_comp
        end
        
        print(string.format("[NET_SYNC] Setting component %s for entity %s", comp_name, entity_id))
        entity:set({ [comp_name] = comp_data })

        -- Initialize hash cache for future change detection
        local sync_comps = sync and sync.sync_components or { Transform = { rate_hz = 30 } }
        last_synced_values[entity_id] = {}
        last_sync_times[entity_id] = {}
        for comp_name, _ in pairs(sync_comps) do
            if entity:has(comp_name) then
                last_synced_values[entity_id][comp_name] = json_encode(entity:get(comp_name))
                last_sync_times[entity_id][comp_name] = now
            end
        end
        
        ::continue_comp::
    end
end

--- Handle despawn message
function NetSync.handle_despawn(world, msg)
    local net_id = msg.net_id
    if not net_id then return end
    
    print(string.format("[NET_SYNC] handle_despawn: net_id=%s", tostring(net_id)))
    
    local entity_id = shared.known_entities[net_id]
    print(string.format("[NET_SYNC] handle_despawn: entity_id from known_entities = %s", tostring(entity_id)))
    
    if entity_id then
        print(string.format("[NET_SYNC] Despawning entity %s for net_id=%s", tostring(entity_id), tostring(net_id)))
        despawn(entity_id)
        shared.known_entities[net_id] = nil
        print(string.format("[NET_SYNC] Despawned remote entity net_id=%s", tostring(net_id)))
    else
        print(string.format("[NET_SYNC] handle_despawn: No entity found for net_id=%s", tostring(net_id)))
    end
end

--- Handle ownership change
function NetSync.handle_owner_change(world, msg)
    local net_id = msg.net_id
    local new_owner = msg.new_owner
    if not net_id or not new_owner then return end
    
    local entity_id = shared.known_entities[net_id]
    if entity_id then
        local entity = world:get_entity(entity_id)
        if entity then
            local sync = entity:get(NetSync.MARKER) or {}
            sync.authority = new_owner
            entity:set({ NetworkSync = sync })
            print(string.format("[NET_SYNC] Ownership change net_id=%d -> %s", net_id, new_owner))
        end
    end
end

--- Handle despawn message from server
function NetSync.handle_despawn(world, msg)
    local net_id = msg.net_id
    if not net_id then return end
    
    local entity_id = shared.known_entities[net_id]
    if entity_id then
        print(string.format("[NET_SYNC] Despawning remote entity net_id=%d entity_id=%s", net_id, tostring(entity_id)))
        
        -- Despawn the entity
        pcall(function()
            despawn(entity_id)
        end)
        
        -- Clear tracking state
        shared.known_entities[net_id] = nil
        spawned_net_ids[net_id] = nil
        last_sync_times[entity_id] = nil
        last_synced_values[entity_id] = nil
        if entity_sync_config[entity_id] then
            entity_sync_config[entity_id] = nil
        end
    else
        print(string.format("[NET_SYNC] handle_despawn: net_id=%d not in known_entities", net_id))
    end
end

--- Process queued children whose parents may now exist
function NetSync.process_pending_children(world)
    local processed = {}
    
    for net_id, msg in pairs(pending_children) do
        if msg.parent_net_id and shared.known_entities[msg.parent_net_id] then
            NetSync.handle_spawn(world, msg)
            processed[net_id] = true
        end
    end
    
    for net_id, _ in pairs(processed) do
        pending_children[net_id] = nil
    end
end

--------------------------------------------------------------------------------
-- Cleanup
--------------------------------------------------------------------------------

--- Mark entity as despawned and send despawn message (by entity_id lookup)
function NetSync.despawn(world, entity_id, send_fn)
    -- Find net_id for this entity
    for net_id, eid in pairs(shared.known_entities) do
        if eid == entity_id then
            NetSync.despawn_by_net_id(world, net_id, send_fn)
            return
        end
    end
    print(string.format("[NET_SYNC] despawn: entity_id=%s not found in known_entities", tostring(entity_id)))
end

--- Mark entity as despawned by net_id (more reliable)
function NetSync.despawn_by_net_id(world, net_id, send_fn)
    if not net_id then return end
    
    local entity_id = shared.known_entities[net_id]
    print(string.format("[NET_SYNC] despawn_by_net_id: net_id=%d entity_id=%s", net_id, tostring(entity_id)))
    
    if send_fn then
        send_fn(CHANNEL_RELIABLE, json_encode({ type = "despawn", net_id = net_id }))
    end
    
    -- Clear tracking state
    shared.known_entities[net_id] = nil
    spawned_net_ids[net_id] = nil
    if entity_id then
        last_sync_times[entity_id] = nil
        last_synced_values[entity_id] = nil
        if entity_sync_config[entity_id] then
            -- Decrement reference counts for this entity's components
            for comp_name, _ in pairs(entity_sync_config[entity_id]) do
                remove_sync_type_ref(comp_name)
            end
            entity_sync_config[entity_id] = nil
        end
    end
end

print("[NET_SYNC] Module loaded")

return NetSync
