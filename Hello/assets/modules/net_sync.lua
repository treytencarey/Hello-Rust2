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

-- Simple value hash for change detection
local function hash_value(val)
    local t = type(val)
    if t == "table" then
        local parts = {}
        for k, v in pairs(val) do
            parts[#parts + 1] = tostring(k) .. "=" .. hash_value(v)
        end
        table.sort(parts)
        return "{" .. table.concat(parts, ",") .. "}"
    else
        return tostring(val)
    end
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

--- Query for changed NetworkSync entities and send updates
--- @param world userdata
--- @param send_fn function(channel, message_string) - How to send data
function NetSync.outbound_system(world, send_fn)
    if not send_fn then return end
    
    local now = os.clock()

    -- Query entities with NetworkSync AND Transform that have changed NetworkSync
    -- We must include Transform in the query because the snapshot only includes queried components.
    -- This ensures we only get fully-spawned entities (not still in spawn queue).
    local synced = world:query({"NetworkSync", "Transform"}, {"NetworkSync"})
    
    for _, entity in ipairs(synced) do
        local sync = entity:get("NetworkSync")
        if not sync or not sync.net_id then goto continue end
        
        -- Only sync entities we own/control
        -- "owner" = we created it locally
        -- "server" = server created it (server syncs these to clients)
        -- "any" = anyone can sync it
        if sync.authority ~= "owner" and sync.authority ~= "server" and sync.authority ~= "any" then
            goto continue
        end
        
        local net_id = sync.net_id
        local entity_id = entity:id()
        
        -- Check if this is a new entity (needs spawn message)
        if not spawned_net_ids[net_id] then
            -- Debug: show what components this entity actually has
            local components = entity:get_components()
            local comp_names = {}
            for name, _ in pairs(components) do
                table.insert(comp_names, name)
            end
            print(string.format("[NET_SYNC] OUTBOUND entity net_id=%d entity_id=%d authority=%s has components: %s",
                net_id, entity_id, tostring(sync.authority), table.concat(comp_names, ", ")))
                
            local spawn_msg = NetSync.build_spawn_msg(world, entity, net_id)
            print(json_encode(spawn_msg))
            send_fn(CHANNEL_RELIABLE, json_encode(spawn_msg))
            spawned_net_ids[net_id] = true
            shared.known_entities[net_id] = entity_id
            print(string.format("[NET_SYNC] Sent spawn for net_id=%d entity_id=%d", net_id, entity_id))
            goto continue
        end
        
        -- Rate limit updates per component
        local entity_times = last_sync_times[entity_id] or {}
        last_sync_times[entity_id] = entity_times
        
        local entity_values = last_synced_values[entity_id] or {}
        last_synced_values[entity_id] = entity_values
        
        local changed_components = {}
        local has_changes = false
        
        -- Determine which components to check
        local components_to_check = sync.sync_components or { Transform = { rate_hz = 30 } }
        
        for comp_name, config in pairs(components_to_check) do
            local rate_hz = config.rate_hz or 30
            local interval = 1.0 / rate_hz
            local last_time = entity_times[comp_name] or 0
            
            -- Rate limit check
            if (now - last_time) >= interval then
                if entity:has(comp_name) then
                    local current_value = entity:get(comp_name)
                    local current_hash = hash_value(current_value)
                    local last_hash = entity_values[comp_name]
                    
                    -- Only send if value actually changed
                    if current_hash ~= last_hash then
                        changed_components[comp_name] = current_value
                        entity_times[comp_name] = now
                        entity_values[comp_name] = current_hash
                        has_changes = true
                    end
                end
            end
        end
        
        if has_changes then
            local update_msg = NetSync.build_update_msg(entity, net_id, changed_components)
            local reliable = false
            -- Use reliable channel if any component requires it
            for comp_name, config in pairs(components_to_check) do
                if config.reliable and changed_components[comp_name] then
                    reliable = true
                    break
                end
            end
            send_fn(reliable and CHANNEL_RELIABLE or CHANNEL_UNRELIABLE, json_encode(update_msg))
        end
        
        ::continue::
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
    
    -- Apply component updates
    for comp_name, comp_data in pairs(msg.components or {}) do
        if comp_name ~= "NetworkSync" then  -- Don't overwrite our NetworkSync
            entity:set({ [comp_name] = comp_data })
        end
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
    end
end

print("[NET_SYNC] Module loaded")

return NetSync
