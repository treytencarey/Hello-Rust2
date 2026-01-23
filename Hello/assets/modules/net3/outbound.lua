-- Net3 Outbound System
-- Detects changes to synced entities and creates NetSyncOutbound message entities

local Components = require("modules/net3/components.lua")
local State = require("modules/net3/state.lua")
local Messages = require("modules/net3/messages.lua")
local json = require("modules/dkjson.lua")

local Outbound = {}

local state = State.get()

-- Configuration
local DEFAULT_RATE_LIMIT = 0.05  -- 50ms between updates per component

--------------------------------------------------------------------------------
-- Rate limiting tracking
--------------------------------------------------------------------------------

local function check_rate_limit(comp_name, config, rate_limit)
    local now = os.clock()
    local last_time = config.last_sync_times[comp_name] or 0
    local limit = rate_limit or DEFAULT_RATE_LIMIT
    
    if (now - last_time) < limit then
        return false
    end
    
    config.last_sync_times[comp_name] = now
    return true
end

--------------------------------------------------------------------------------
-- Main Outbound System
--------------------------------------------------------------------------------

--- Outbound system - detects changes and spawns NetSyncOutbound entities
--- @param world userdata The world object
--- @param context table Optional context { is_server, get_clients, filter_clients }
function Outbound.system(world, context)
    context = context or {}
    local is_server = context.is_server or false
    local get_clients = context.get_clients
    local filter_clients = context.filter_clients
    
    -- 1. Gather all frame-local changes for all active sync types
    local sync_types = State.get_sync_types()
    local frame_changes = {} -- component_name -> { entity_id -> true }
    
    for comp_name, _ in pairs(sync_types) do
        frame_changes[comp_name] = {}
        -- Query entities where this component changed
        -- Note: We include marker to only detect changes on networked entities
        local changed_entities = world:query({ Components.MARKER, "ScriptOwned", comp_name }, { comp_name })
        for _, changed_entity in ipairs(changed_entities) do
            frame_changes[comp_name][changed_entity:id()] = true
        end
    end

    -- Query all entities with NetworkSync marker (not remote)
    local entities = world:query({ Components.MARKER, "ScriptOwned" })
    
    for _, entity in ipairs(entities) do
        local sync = entity:get(Components.MARKER)
        
        -- Skip remote entities (we don't send updates for entities we don't own)
        if sync.authority == "remote" then
            goto continue_entity
        end
        
        -- Skip entities from other instances
        local script_owned = entity:get("ScriptOwned")
        if script_owned.instance_id ~= __INSTANCE_ID__ then
            goto continue_entity
        end
        
        local entity_id = entity:id()
        local net_id = sync.net_id or State.get_net_id(entity_id)
        
        -- If no net_id yet, assign one and register
        if not net_id then
            net_id = State.next_net_id()
            entity:patch({ [Components.MARKER] = { net_id = net_id } })
            State.register_entity(net_id, entity_id)
            State.set_owner(net_id, sync.owner_client)
        end
        
        local config = State.get_sync_config(entity_id, sync.sync_components)
        State.add_sync_types(config.sync_components)
        
        -- Handle initial spawn
        if not config.spawned then
            local full_entity = world:get_entity(entity_id)
            local spawn_msg = Messages.build_spawn(world, full_entity, net_id)
            if spawn_msg then
                -- Add targeting for server
                if is_server and get_clients then
                    local clients = get_clients(world)
                    if filter_clients then
                        clients = filter_clients(clients, entity, net_id)
                    end
                    spawn_msg.target_clients = clients
                    
                    -- Update client scopes
                    for _, client_id in ipairs(clients) do
                        State.add_to_client_scope(client_id, net_id)
                    end
                end
                
                spawn({ [Components.OUTBOUND] = spawn_msg })
                State.mark_spawned(entity_id)
            end
            goto continue_entity
        end
        
        -- 2. Update persistent dirty flags for this entity
        for comp_name, _ in pairs(config.sync_components) do
            if frame_changes[comp_name] and frame_changes[comp_name][entity_id] then
                config.dirty[comp_name] = true
            end
        end

        -- Collect changed components
        local changed = {}
        local has_changes = false
        
        for comp_name, comp_config in pairs(config.sync_components) do
            local rate_limit = comp_config.rate_limit
            
            -- Send if dirty and rate limit allows
            if config.dirty[comp_name] and check_rate_limit(comp_name, config, rate_limit) then
                local comp_data = entity:get(comp_name)
                if comp_data then
                    changed[comp_name] = comp_data
                    has_changes = true
                    
                    -- Clear dirty flag since we're including it in an update
                    config.dirty[comp_name] = nil
                    
                    if is_server then
                        -- print(string.format("[NET3_OUTBOUND] [SERVER] Entity %s changed: %s", net_id, comp_name))
                    else
                        -- print(string.format("[NET3_OUTBOUND] [CLIENT] Entity %s changed: %s", net_id, comp_name))
                    end
                end
            end
        end
        
        -- Send update if changes detected
        if has_changes then
            local seq = nil
            local ack_seq = nil
            
            -- For client-owned entities, include input sequence for prediction
            if sync.owner_client == State.my_client_id() then
                local pred = entity:get(Components.PREDICTION)
                if pred then
                    seq = pred.current_sequence
                end
            end
            
            -- For server, include last acked sequence
            if is_server then
                ack_seq = state.client_input_seq[entity_id]
            end
            
            local update_msg = Messages.build_update(
                sync.owner_client,
                entity,
                net_id,
                changed,
                seq,
                ack_seq
            )
            
            -- Add targeting for server
            if is_server and get_clients then
                local clients = {}
                for _, client_id in ipairs(get_clients(world)) do
                    if State.client_knows_entity(client_id, net_id) then
                        table.insert(clients, client_id)
                    end
                end
                if filter_clients then
                    clients = filter_clients(clients, entity, net_id)
                end
                update_msg.target_clients = clients
            end
            
            spawn({ [Components.OUTBOUND] = update_msg })
        end
        
        ::continue_entity::
    end
    
    -- Handle despawns via query_removed
    local removed = world:query_removed({ Components.MARKER })
    for _, entity_bits in ipairs(removed) do
        local net_id = State.get_net_id(entity_bits)
        if net_id then
            local config = state.entity_sync_config[entity_bits]
            
            -- Only send despawn if we created this entity locally
            if config and config.created_locally then
                local owner = State.get_owner(net_id)
                local despawn_msg = Messages.build_despawn(owner or 0, net_id)
                
                -- Add targeting for server
                if is_server and get_clients then
                    local clients = {}
                    for _, client_id in ipairs(get_clients(world)) do
                        if State.client_knows_entity(client_id, net_id) then
                            table.insert(clients, client_id)
                            State.remove_from_client_scope(client_id, net_id)
                        end
                    end
                    despawn_msg.target_clients = clients
                end
                
                spawn({ [Components.OUTBOUND] = despawn_msg })
            end
            
            -- Cleanup state
            if config then
                State.remove_sync_types(config.sync_components)
            end
            State.unregister_entity(net_id)
            State.clear_owner(net_id)
        end
    end
end

return Outbound
