-- Net3 Inbound System
-- Processes NetSyncInbound entities and applies changes to ECS

local Components = require("modules/net3/components.lua")
local State = require("modules/net3/state.lua")
local json = require("modules/dkjson.lua")

local Inbound = {}

local state = State.get()

-- Configuration
local PENDING_CHILD_TIMEOUT = 10.0  -- Seconds before orphaned children are discarded

--------------------------------------------------------------------------------
-- Entity Lookup
--------------------------------------------------------------------------------

local function find_entity_by_net_id(world, net_id)
    -- First check our known entities cache
    local cached = State.get_entity_id(net_id)
    if cached then
        local entity = world:get_entity(cached)
        if entity then
            return cached
        end
    end
    
    -- Fallback: query world
    local entities = world:query({ Components.MARKER, "ScriptOwned" })
    for _, entity in ipairs(entities) do
        local sync = entity:get(Components.MARKER)
        if sync and sync.net_id == net_id then
            State.register_entity(net_id, entity:id())
            return entity:id()
        end
    end
    
    return nil
end

--------------------------------------------------------------------------------
-- Message Handlers
--------------------------------------------------------------------------------

--- Handle spawn message
local function handle_spawn(world, msg, owner_client)
    local net_id = msg.net_id
    local payload = msg.payload
    
    -- Skip if already known
    if State.is_known(net_id) then
        print(string.format("[NET3] Spawn skipped - entity %d already known", net_id))
        -- Treat as update instead
        return Inbound.handle_update(world, msg, owner_client)
    end
    
    -- Check if entity exists in world (for "both" mode)
    local existing_id = find_entity_by_net_id(world, net_id)
    if existing_id then
        print(string.format("[NET3] Adopting existing entity %d for net_id %d", existing_id, net_id))
        State.register_entity(net_id, existing_id)
        
        local entity = world:get_entity(existing_id)
        if entity then
            local sync = entity:get(Components.MARKER)
            local config = State.get_sync_config(existing_id, sync and sync.sync_components)
            config.created_locally = false
            State.add_sync_types(config.sync_components)
        end
        
        return Inbound.handle_update(world, msg, owner_client)
    end
    
    -- Check for parent dependency
    local parent_net_id = payload.parent_net_id
    if parent_net_id then
        local parent_entity_id = State.get_entity_id(parent_net_id)
        if not parent_entity_id then
            State.add_pending_child(net_id, msg, owner_client)
            print(string.format("[NET3] Deferred spawn of %d waiting for parent %d", net_id, parent_net_id))
            return
        end
    end
    
    -- Build spawn data
    local spawn_data = payload.components or {}
    
    -- Set authority to "remote"
    if spawn_data[Components.MARKER] then
        spawn_data[Components.MARKER].authority = "remote"
    else
        spawn_data[Components.MARKER] = {
            net_id = net_id,
            owner_client = owner_client,
            authority = "remote",
        }
    end
    
    -- Handle parent relationship
    if parent_net_id then
        local parent_entity_id = State.get_entity_id(parent_net_id)
        spawn_data.ChildOf = { parent = parent_entity_id }
    end
    
    -- Add PredictionState for own entity
    if owner_client == State.my_client_id() then
        spawn_data[Components.PREDICTION] = {
            server_state = nil,
            last_acked_sequence = nil,
            current_sequence = 0,
            predictions = {},
            snap_threshold = 2.0,
        }
        print("[NET3] Added PredictionState for own entity")
    end
    
    -- Spawn the entity
    local entity_id = spawn(spawn_data):id()
    State.register_entity(net_id, entity_id)
    
    -- Set up sync config
    local sync = spawn_data[Components.MARKER]
    local config = State.get_sync_config(entity_id, sync.sync_components)
    config.spawned = true
    config.created_locally = false
    State.add_sync_types(config.sync_components)
    
    print(string.format("[NET3] Spawned entity %d with net_id %d", entity_id, net_id))
    
    -- Process any pending children
    local pending = State.get_pending_children_for_parent(net_id)
    for child_net_id, child_data in pairs(pending) do
        State.take_pending_child(child_net_id)
        handle_spawn(world, child_data.msg, child_data.owner_client)
    end
end

--- Handle update message
function Inbound.handle_update(world, msg, owner_client)
    local net_id = msg.net_id
    local payload = msg.payload
    local entity_id = State.get_entity_id(net_id)
    
    if not entity_id then
        print(string.format("[NET3] Update for unknown entity %d", net_id))
        return
    end
    
    local entity = world:get_entity(entity_id)
    if not entity then
        print(string.format("[NET3] Entity %d not found in world (might be pending spawn)", entity_id))
        return
    end
    
    -- Check if this is for our own entity (prediction reconciliation)
    local is_own_entity = (owner_client == State.my_client_id())
    local is_server = (state.mode == "server")
    
    -- On server, track the last processed sequence number from the client
    if is_server and payload.seq then
        state.client_input_seq[entity_id] = payload.seq
    end
    
    -- Get sync config to check authority (initialize if missing)
    local sync = entity:get(Components.MARKER)
    local config = State.get_sync_config(entity_id, sync and sync.sync_components)
    
    for comp_name, comp_data in pairs(payload.components or {}) do
        -- 0. Shared Entity Suppression (for "both" mode)
        -- If this entity is managed by another instance in the same world, 
        -- we don't need to apply the update because the source instance
        -- already updated the shared component data.
        local script_owned = entity:get("ScriptOwned")
        if script_owned and script_owned.instance_id ~= __INSTANCE_ID__ then
            if config and config.created_locally == false then
                goto continue_update
            end
        end

        -- 1. Authority Validation
        local comp_config = config and config.sync_components and config.sync_components[comp_name] or {}
        local authority = comp_config.authority or "server"
        
        if is_server then
            -- Server: Only accept components with client authority
            if authority ~= "client" then
                print(string.format("[NET3] Server rejected update for %s (authority=%s)", comp_name, authority))
                goto continue_update
            end
        elseif is_own_entity then
            -- Client (Owner): Only accept components with server authority
            -- (Local components with client authority are handled locally)
            if authority ~= "server" then
                print(string.format("[NET3] Owner rejected update for %s (authority=%s)", comp_name, authority))
                goto continue_update
            end
        end

        -- 2. Special handling for Transform (interpolation/prediction)
        if comp_name == "Transform" then
            if is_own_entity then
                -- Update prediction state for reconciliation
                local pred = entity:get(Components.PREDICTION)
                if pred then
                    entity:patch({
                        [Components.PREDICTION] = {
                            server_state = {
                                position = comp_data.translation,
                                rotation = comp_data.rotation,
                            },
                            last_acked_sequence = payload.ack_seq,
                        }
                    })
                    goto continue_update
                end
            else
                -- Set interpolation target for remote entities
                if entity:has(Components.INTERPOLATION) then
                    entity:patch({
                        [Components.INTERPOLATION] = {
                            position = comp_data.translation,
                            rotation = comp_data.rotation,
                            scale = comp_data.scale,
                        }
                    })
                    goto continue_update
                else
                    -- Add interpolation component if not present
                    entity:set({
                        [Components.INTERPOLATION] = {
                            position = comp_data.translation,
                            rotation = comp_data.rotation or { x = 0, y = 0, z = 0, w = 1 },
                            scale = comp_data.scale or { x = 1, y = 1, z = 1 },
                            lerp_speed = 10.0,
                            snap_threshold = 5.0,
                        }
                    })
                    goto continue_update
                end
            end
        end
        
        -- 3. Apply normal component update
        entity:set({ [comp_name] = comp_data })
        
        ::continue_update::
    end
end

--- Handle despawn message
local function handle_despawn(world, msg, owner_client)
    local net_id = msg.net_id
    local entity_id = State.get_entity_id(net_id)
    
    if entity_id then
        print(string.format("[NET3] Despawning entity %d (net_id %d)", entity_id, net_id))
        despawn(entity_id)
        
        local config = state.entity_sync_config[entity_id]
        if config then
            State.remove_sync_types(config.sync_components)
        end
        State.unregister_entity(net_id)
    end
end

--- Handle owner change message
local function handle_owner_change(world, msg, owner_client)
    local net_id = msg.net_id
    local new_owner = msg.payload.new_owner
    local entity_id = State.get_entity_id(net_id)
    
    if entity_id then
        local entity = world:get_entity(entity_id)
        if entity then
            entity:patch({ [Components.MARKER] = { owner_client = new_owner } })
            State.set_owner(net_id, new_owner)
            print(string.format("[NET3] Owner changed for %d to %s", net_id, tostring(new_owner)))
        end
    end
end

--- Handle client_id message
local function handle_client_id(world, msg)
    local client_id = msg.payload.client_id
    State.my_client_id(client_id)
    State.set_client_prefix(client_id)
    print(string.format("[NET3] Received client_id: %s", tostring(client_id)))
end

--------------------------------------------------------------------------------
-- Main Inbound System
--------------------------------------------------------------------------------

--- Inbound system - processes NetSyncInbound entities
--- @param world userdata The world object
function Inbound.system(world)
    local inbound = world:query({ Components.INBOUND, "ScriptOwned" })
    
    for _, msg_entity in ipairs(inbound) do
        local msg = msg_entity:get(Components.INBOUND)
        local owner_client = msg.owner_client
        
        -- Skip messages from other instances
        local script_owned = msg_entity:get("ScriptOwned")
        if script_owned.instance_id ~= __INSTANCE_ID__ then
            goto continue_entities
        end
        
        -- Dispatch based on message type
        if msg.msg_type == "spawn" then
            handle_spawn(world, msg, owner_client)
        elseif msg.msg_type == "update" then
            Inbound.handle_update(world, msg, owner_client)
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
    local timed_out = State.cleanup_pending_children(PENDING_CHILD_TIMEOUT)
    for _, net_id in ipairs(timed_out) do
        print(string.format("[NET3] Orphaned pending child %d timed out", net_id))
    end
end

-- Export handlers for external use
Inbound.handle_spawn = handle_spawn
Inbound.handle_despawn = handle_despawn
Inbound.handle_owner_change = handle_owner_change
Inbound.handle_client_id = handle_client_id

return Inbound
