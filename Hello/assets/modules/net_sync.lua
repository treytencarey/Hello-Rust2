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

--- Get interpolation targets (for use by transform_interpolation module)
function NetSync.get_interpolation_targets()
    return interpolation_targets
end

--- Get server's authoritative state for our own entity (for reconciliation)
function NetSync.get_server_authoritative_state()
    return server_authoritative_state
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
-- Message Building
--------------------------------------------------------------------------------

--- Build spawn message with all components
function NetSync.build_spawn_msg(world, entity, net_id)
    -- IMPORTANT: The 'entity' parameter from query() only has queried components.
    -- We need a FULL snapshot with all components, so use world:get_entity().
    local entity_id = entity:id()
    local full_entity = world:get_entity(entity_id)
    if not full_entity then
        print(string.format("[NET_SYNC] build_spawn_msg: world:get_entity(%s) returned nil!", tostring(entity_id)))
        -- Fall back to using the query entity (limited but better than nothing)
        full_entity = entity
    end
    
    local sync = full_entity:get("NetworkSync")
    
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
            if parent_entity and parent_entity:has("NetworkSync") then
                local parent_sync = parent_entity:get("NetworkSync")
                msg.parent_net_id = parent_sync.net_id
            end
        end
    end
    
    -- Serialize components (based on sync_components config or all)
    local components_to_sync = sync.sync_components
    if components_to_sync then
        print(string.format("[NET_SYNC] build_spawn_msg net_id=%s: Using sync_components config", tostring(net_id)))
        for comp_name, _ in pairs(components_to_sync) do
            if full_entity:has(comp_name) then
                msg.components[comp_name] = full_entity:get(comp_name)
                print(string.format("[NET_SYNC] build_spawn_msg: Added %s", comp_name))
            else
                print(string.format("[NET_SYNC] build_spawn_msg: Entity missing %s", comp_name))
            end
        end
    else
        -- Sync Transform by default
        print(string.format("[NET_SYNC] build_spawn_msg net_id=%s: No sync_components, using default", tostring(net_id)))
        if full_entity:has("Transform") then
            msg.components["Transform"] = full_entity:get("Transform")
        end
    end
    
    -- Include NetworkSync itself
    msg.components["NetworkSync"] = sync
    
    -- Debug: check if PlayerState has model_path
    if msg.components.PlayerState then
        print(string.format("[NET_SYNC] build_spawn_msg: PlayerState.model_path = %s", 
            tostring(msg.components.PlayerState.model_path)))
    else
        print("[NET_SYNC] build_spawn_msg: No PlayerState in components!")
    end
    
    return msg
end

--- Build update message with only changed components
function NetSync.build_update_msg(entity, net_id, changed_components)
    return {
        type = "update",
        net_id = net_id,
        components = changed_components
    }
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
--- @param send_fn function(channel, message_string) - How to send data
function NetSync.outbound_system(world, send_fn)
    if not send_fn then return end
    
    local now = os.clock()
    local entities_to_spawn = {}  -- entity_id -> entity
    local changed_components = {}  -- entity_id -> { comp_name -> true }
    
    -- Step 1: Query for NetworkSync changes (new entities or config changes)
    local network_sync_changed = world:query({"NetworkSync"}, {"NetworkSync"})
    for _, entity in ipairs(network_sync_changed) do
        local sync = entity:get("NetworkSync")
        if not sync or not sync.net_id then goto continue_ns end
        
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
        local comp_changed = world:query({"NetworkSync", comp_name}, {comp_name})
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
        local sync = entity:get("NetworkSync")
        local net_id = sync.net_id
        
        if spawned_net_ids[net_id] then goto continue_spawn end
        
        local full_entity = world:get_entity(entity_id)
        if not full_entity then goto continue_spawn end
        
        local spawn_msg = NetSync.build_spawn_msg(world, full_entity, net_id)
        send_fn(CHANNEL_RELIABLE, json_encode(spawn_msg))
        spawned_net_ids[net_id] = true
        shared.known_entities[net_id] = entity_id
        
        -- Initialize hash cache for future change detection
        local sync_comps = sync.sync_components or { Transform = { rate_hz = 30 } }
        last_synced_values[entity_id] = {}
        last_sync_times[entity_id] = {}
        for comp_name, _ in pairs(sync_comps) do
            if full_entity:has(comp_name) then
                last_synced_values[entity_id][comp_name] = json_encode(full_entity:get(comp_name))
                last_sync_times[entity_id][comp_name] = now
            end
        end
        
        ::continue_spawn::
    end
    
    -- Step 3b: Process entities with changed components (send only changed)
    for entity_id, comps_changed in pairs(changed_components) do
        local full_entity = world:get_entity(entity_id)
        if not full_entity then goto continue_update end
        if not full_entity:has("NetworkSync") then goto continue_update end
        
        local sync = full_entity:get("NetworkSync")
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
                local current_hash = json_encode(current_value)
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
            local update_msg = NetSync.build_update_msg(full_entity, net_id, components_to_send)
            send_fn(needs_reliable and CHANNEL_RELIABLE or CHANNEL_UNRELIABLE, json_encode(update_msg))
        end
        
        ::continue_update::
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
    print(string.format("[NET_SYNC] Received message: %s", json_encode(msg)))
    
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
        shared.my_net_id = msg.net_id
        print(string.format("[NET_SYNC] Received my character assignment: net_id=%s (shared table=%s)", 
            tostring(shared.my_net_id), tostring(shared)))
    end
end

--- Handle spawn message
function NetSync.handle_spawn(world, msg)
    local net_id = msg.net_id
    if not net_id then return end

    print(string.format("[NET_SYNC] handle_spawn: net_id=%d shared=%s known_entities[%d]=%s",
        net_id, tostring(shared), net_id, tostring(shared.known_entities[net_id])))
    
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
    
    -- Ensure NetworkSync component has remote authority
    spawn_data.NetworkSync = spawn_data.NetworkSync or {}
    spawn_data.NetworkSync.net_id = net_id
    spawn_data.NetworkSync.authority = "remote"  -- Mark as not ours
    
    -- Add visual components if PlayerState has model_path
    -- (Server doesn't send visuals, clients load them locally)
    local player_state = spawn_data.PlayerState
    if player_state and player_state.model_path then
        local model_path = player_state.model_path
        print(string.format("[NET_SYNC] Adding visuals for net_id=%s model=%s", tostring(net_id), model_path))
        local handle = load_asset(model_path)
        print(string.format("[NET_SYNC] handle = %s", tostring(handle)))
        if handle then
            spawn_data.SceneRoot = handle
            print(string.format("[NET_SYNC] Added SceneRoot to spawn_data for net_id=%s", tostring(net_id)))
        else
            print(string.format("[NET_SYNC] Warning: Could not load model '%s' for net_id=%s (will retry on hot reload)", model_path, tostring(net_id)))
        end
    end
    
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
    
    -- Get entity's NetworkSync for authority checks
    local sync = entity:get("NetworkSync")
    
    -- Apply component updates (with authority validation on server)
    for comp_name, comp_data in pairs(msg.components or {}) do
        if comp_name == "NetworkSync" then
            goto continue_comp  -- Never overwrite NetworkSync
        end
        
        -- Server-side authority validation
        if NetRole.is_server() and sync and sync.sync_components then
            local config = sync.sync_components[comp_name]
            local comp_authority = config and config.authority or sync.authority or "server"
            
            -- Server only accepts updates for "client" authority components
            if comp_authority ~= "client" then
                print(string.format("[NET_SYNC] Rejecting %s update from client - authority=%s", 
                    comp_name, comp_authority))
                goto continue_comp
            end
        end
        
        entity:set({ [comp_name] = comp_data })
        
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
            local sync = entity:get("NetworkSync") or {}
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
            rebuild_known_sync_types()
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
