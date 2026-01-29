-- NET Outbound System
-- Detects changes to synced entities and creates NetSyncOutbound message entities

local Components = require("modules/net/components.lua")
local State = require("modules/net/state.lua")
local Messages = require("modules/net/messages.lua")
local json = require("modules/dkjson.lua")

local Outbound = {}

local state = State.get()

-- Configuration
local DEFAULT_RATE_LIMIT = 0.05  -- 50ms between updates per component

--------------------------------------------------------------------------------
-- Rate limiting tracking
--------------------------------------------------------------------------------

local function check_rate_limit(comp_name, config, rate_limit)
    -- If no rate limit is specified, don't throttle
    if not rate_limit then
        return true
    end

    local now = os.clock()
    local last_time = config.last_sync_times[comp_name] or 0
    
    if (now - last_time) < rate_limit then
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
--- @param context table Optional context { is_server, get_clients, filter }
function Outbound.system(world, context)
    context = context or {}
    local is_server = context.is_server or false
    local get_clients = context.get_clients
    local filter = context.filter  -- signature: filter(world, client_id, net_id) -> boolean

    local now = os.clock()

    ---------------------------------------------------------------------------
    -- Phase 1: Registration (MARKER added or changed)
    ---------------------------------------------------------------------------
    local new_or_changed_markers = world:query({
        with = { Components.MARKER, "ScriptOwned" },
        ["or"] = {
            added = { Components.MARKER },
            changed = { Components.MARKER },
        }
    })

    for _, entity in ipairs(new_or_changed_markers) do
        local entity_id = entity:id()
        local script_owned = entity:get("ScriptOwned")

        -- Skip entities from other instances
        if script_owned.instance_id ~= __INSTANCE_ID__ then
            goto continue_registration
        end

        local sync = entity:get(Components.MARKER)

        -- Skip remote entities (we don't originate updates for them)
        if sync.authority == "remote" then
            goto continue_registration
        end

        local net_id = sync.net_id

        -- Assign net_id if missing
        if not net_id then
            net_id = State.next_net_id()
            entity:patch({ [Components.MARKER] = { net_id = net_id } })
            sync.net_id = net_id  -- Update local copy
        end

        -- Register entity mapping
        State.register_entity(net_id, entity_id)
        State.set_owner(net_id, sync.owner_client)

        -- Store normalized sync_components config
        local sync_components = sync.sync_components or { Transform = {} }
        State.set_registered_sync_components(entity_id, sync_components)

        -- Add to sync_types for query optimization
        State.add_sync_types(sync_components)

        -- Initialize sync config
        local config = State.get_sync_config(entity_id, sync_components)

        -- Queue ALL sync components as pending for initial spawn
        local pending_data = {
            net_id = net_id,
            owner_client = sync.owner_client,
            authority = sync.authority,
            components = {}
        }
        for comp_name, _ in pairs(sync_components) do
            local comp_value = entity:get(comp_name)
            if comp_value then
                pending_data.components[comp_name] = {
                    value = comp_value,
                    queued_at = now,
                }
            end
        end
        State.set_pending_entity(entity_id, pending_data)

        ::continue_registration::
    end

    ---------------------------------------------------------------------------
    -- Phase 2: Change Detection
    ---------------------------------------------------------------------------
    local sync_type_names = {}
    for comp_name, _ in pairs(State.get_sync_types()) do
        table.insert(sync_type_names, comp_name)
    end

    if #sync_type_names > 0 then
        local changed_entities = world:query({
            with = { Components.MARKER, "ScriptOwned" },
            ["or"] = {
                changed = sync_type_names,
                added = sync_type_names,
            }
        })

        for _, entity in ipairs(changed_entities) do
            local entity_id = entity:id()
            local script_owned = entity:get("ScriptOwned")

            -- Skip entities from other instances
            if script_owned.instance_id ~= __INSTANCE_ID__ then
                goto continue_change
            end

            local sync = entity:get(Components.MARKER)

            -- Skip remote entities
            if sync.authority == "remote" then
                goto continue_change
            end

            local registered = State.get_registered_sync_components(entity_id)
            if not registered then
                -- Entity not registered yet, will be caught next frame
                goto continue_change
            end

            -- Get precise list of changed/added components
            local changed_list = entity:changed_components()
            local added_list = entity:added_components()

            -- Merge into single set
            local changed_set = {}
            for _, name in ipairs(changed_list) do changed_set[name] = true end
            for _, name in ipairs(added_list) do changed_set[name] = true end

            -- Initialize pending if not exists
            local pending = State.get_pending_entity(entity_id)
            if not pending then
                pending = {
                    net_id = sync.net_id,
                    owner_client = sync.owner_client,
                    authority = sync.authority,
                    components = {}
                }
                State.set_pending_entity(entity_id, pending)
            end

            -- Queue changed components that we're tracking
            for comp_name, _ in pairs(changed_set) do
                if registered[comp_name] then
                    local value = entity:get(comp_name)
                    if value then
                        -- Check State for source (synchronous, set by inbound/movement systems)
                        local source_client = State.get_inbound_source(entity_id, comp_name)
                        pending.components[comp_name] = {
                            value = value,
                            queued_at = now,
                            source_client = source_client,
                        }
                    end
                end
            end
            
            ::continue_change::
        end
    end

    ---------------------------------------------------------------------------
    -- Phase 3: Send Phase (scoping, rate limiting, spawn/update/despawn)
    ---------------------------------------------------------------------------

    -- Build entity lookup from batch query for spawn data
    -- Include all sync_type_names so component values are available for spawn messages
    local entity_lookup = {}
    if #sync_type_names > 0 then
        -- Build query with all sync components
        local query_spec = {
            with = { Components.MARKER, "ScriptOwned" },
            any_of = sync_type_names  -- Use any_of so we get entities even if they don't have all components
        }
        local all_marker_entities = world:query(query_spec)
        for _, entity in ipairs(all_marker_entities) do
            entity_lookup[entity:id()] = entity
        end
    end

    -- Process all entities with pending values
    for entity_id, pending in pairs(State.get_pending_values()) do
        local entity = entity_lookup[entity_id]

        -- Entity no longer exists (despawned)
        if not entity then
            State.clear_pending_entity(entity_id)
            goto continue_send
        end

        local net_id = pending.net_id
        local owner_client = pending.owner_client
        local registered = State.get_registered_sync_components(entity_id)
        local config = State.get_sync_config(entity_id)

        if not net_id or not registered then
            goto continue_send
        end

        -- Compute scope (spawn_targets, update_targets, despawn_targets)
        local spawn_targets = {}
        local update_targets = {}
        local despawn_targets = {}

        if is_server and get_clients then
            local all_clients = get_clients(world)

            for _, client_id in ipairs(all_clients) do
                -- Call filter once per client per entity
                local in_scope = true
                if filter then
                    in_scope = filter(world, client_id, net_id)
                end

                local client_knew = State.client_knows_entity(client_id, net_id)

                if in_scope then
                    if client_knew then
                        table.insert(update_targets, client_id)
                    else
                        table.insert(spawn_targets, client_id)
                    end
                else
                    if client_knew then
                        table.insert(despawn_targets, client_id)
                    end
                end
            end
        else
            -- Client mode: server is target 0
            if State.client_knows_entity(0, net_id) then
                -- Echo suppression: don't send updates back to the server if it's the owner
                if owner_client ~= 0 then
                    table.insert(update_targets, 0)
                end
            else
                table.insert(spawn_targets, 0)
            end
        end

        -- Handle scope despawns (clients who left scope)
        if #despawn_targets > 0 then
            local despawn_msg = Messages.build_despawn(owner_client or 0, net_id)
            despawn_msg.target_clients = despawn_targets
            spawn({ [Components.OUTBOUND] = despawn_msg })

            for _, client_id in ipairs(despawn_targets) do
                State.remove_from_client_scope(client_id, net_id)
            end
        end

        -- Handle spawns (new clients in scope)
        if #spawn_targets > 0 then
            -- Collect all component values for spawn
            local spawn_components = {}
            for comp_name, _ in pairs(registered) do
                local value = entity:get(comp_name)
                if value then
                    spawn_components[comp_name] = value
                end
            end

            -- Include the marker
            local sync = entity:get(Components.MARKER)
            spawn_components[Components.MARKER] = sync

            -- Check for parent relationship
            local parent_net_id = nil
            if entity:has("ChildOf") then
                local child_of = entity:get("ChildOf")
                if child_of and child_of.parent then
                    parent_net_id = State.get_net_id(child_of.parent)
                end
            end

            local spawn_msg = Messages.build_spawn(net_id, owner_client, spawn_components, parent_net_id)
            spawn_msg.target_clients = spawn_targets
            spawn({ [Components.OUTBOUND] = spawn_msg })

            -- Mark clients as knowing this entity
            for _, client_id in ipairs(spawn_targets) do
                State.add_to_client_scope(client_id, net_id)
            end

            -- Mark as spawned
            State.mark_spawned(entity_id)

            -- Clear pending components since we sent them all in the spawn
            pending.components = {}
        end

        -- Process pending updates (use stored values, apply rate limiting)
        if #update_targets > 0 then
            local ready_components = {}
            local remaining = {}

            -- 1. Rate Limiting Pass
            for comp_name, pending_comp in pairs(pending.components) do
                local comp_config = registered[comp_name]
                if not comp_config then
                    goto next_pending_comp
                end

                -- Authority check
                local authority = comp_config.authority
                if not is_server and authority ~= "client" then
                    -- Client can only send client-authority components
                    goto next_pending_comp
                end

                -- Rate limit check
                local can_send = true
                local rate_limit = comp_config.rate_limit
                if rate_limit then
                    local last_time = config.last_sync_times[comp_name] or 0
                    if (now - last_time) < rate_limit then
                        can_send = false
                    end
                end

                if can_send then
                    ready_components[comp_name] = pending_comp
                    config.last_sync_times[comp_name] = now
                else
                    -- Keep in pending for next frame (guaranteed delivery)
                    remaining[comp_name] = pending_comp
                end

                ::next_pending_comp::
            end

            -- 2. Send Pass
            if next(ready_components) then
                -- Split targets
                local targets_others = {}
                local targets_owner = {}

                for _, client_id in ipairs(update_targets) do
                    if client_id == owner_client then
                        table.insert(targets_owner, client_id)
                    else
                        table.insert(targets_others, client_id)
                    end
                end

                -- Helper to build and send
                local function send_subset(targets, filter_func)
                    if #targets == 0 then return end

                    local payload = {}
                    local has_reliable = false

                    for name, p_comp in pairs(ready_components) do
                        if not filter_func or filter_func(p_comp) then
                            payload[name] = p_comp.value
                            if registered[name].reliable then 
                                has_reliable = true 
                            end
                        end
                    end

                    if next(payload) then
                        local channel = has_reliable and Messages.CHANNEL_RELIABLE or Messages.CHANNEL_UNRELIABLE
                        local update_msg = Messages.build_update(
                            owner_client,
                            entity,
                            net_id,
                            payload,
                            nil, -- seq
                            nil, -- ack_seq
                            channel
                        )
                        update_msg.target_clients = targets
                        spawn({ [Components.OUTBOUND] = update_msg })
                    end
                end
                
                -- Send to others (full updates)
                send_subset(targets_others, nil)

                -- Send to owner (filter out updates they originated)
                send_subset(targets_owner, function(p_comp)
                    -- send if we (the logic) are not the source (nil check for safe operator)
                    -- source_client is nil if server/local generated.
                    -- if owner_client matched source_client, it means they sent it.
                    return p_comp.source_client ~= owner_client
                end)
            end

            -- Update pending components (keep rate-limited ones)
            pending.components = remaining
        end

        -- Cleanup if no pending components left
        if not State.has_pending_components(entity_id) then
            State.clear_pending_entity(entity_id)
        end

        ::continue_send::
    end

    ---------------------------------------------------------------------------
    -- Phase 4: Despawn Handling (entity removal)
    ---------------------------------------------------------------------------
    local removed = world:query({
        removed = { Components.MARKER }
    })

    for _, removed_entity in ipairs(removed) do
        local entity_bits = removed_entity:id()
        local net_id = State.get_net_id(entity_bits)

        if not net_id then
            goto continue_removed
        end

        local config = state.entity_sync_config[entity_bits]

        -- Only send despawn if we created this entity locally
        if not config or not config.created_locally then
            goto continue_removed
        end

        local owner = State.get_owner(net_id)

        -- Collect clients who know this entity
        local despawn_clients = {}
        if is_server and get_clients then
            for _, client_id in ipairs(get_clients(world)) do
                if State.client_knows_entity(client_id, net_id) then
                    table.insert(despawn_clients, client_id)
                    State.remove_from_client_scope(client_id, net_id)
                end
            end
        else
            -- Client: notify server
            if State.client_knows_entity(0, net_id) then
                table.insert(despawn_clients, 0)
            end
        end

        if #despawn_clients > 0 then
            local despawn_msg = Messages.build_despawn(owner or 0, net_id)
            despawn_msg.target_clients = despawn_clients
            spawn({ [Components.OUTBOUND] = despawn_msg })
        end

        -- Cleanup state
        local registered = State.get_registered_sync_components(entity_bits)
        if registered then
            State.remove_sync_types(registered)
        end
        State.clear_registered_sync_components(entity_bits)
        State.clear_pending_entity(entity_bits)
        State.unregister_entity(net_id)
        State.clear_owner(net_id)

        ::continue_removed::
    end

    ---------------------------------------------------------------------------
    -- Phase 5: Clear inbound sources (for next frame)
    ---------------------------------------------------------------------------
    State.clear_inbound_sources()
end

--- Outbound system - detects changes and spawns NetSyncOutbound entities
--- @param world userdata The world object
--- @param context table Optional context { is_server, get_clients, filter_clients }
function Outbound.system_old(world, context)
    context = context or {}
    local is_server = context.is_server or false
    local get_clients = context.get_clients
    local filter_clients = context.filter_clients
    
    local sync_types = State.get_sync_types()
    local sync_type_names = {}
    for comp_name, _ in pairs(sync_types) do
        table.insert(sync_type_names, comp_name)
    end

    -- 1. Change Detection: Batch query for ANY changes or additions
    -- We include the MARKER and ScriptOwned in 'added' to catch new entities
    local added_filters = { Components.MARKER, "ScriptOwned" }
    for _, name in ipairs(sync_type_names) do table.insert(added_filters, name) end

    local changed_entities = world:query({
        with = { Components.MARKER, "ScriptOwned" },
        ["or"] = {
            changed = sync_type_names,
            added = added_filters,
        },
    })

    -- Mark each entity and its specific components as dirty
    for _, changed_entity in ipairs(changed_entities) do
        local entity_id = changed_entity:id()
        local sync = changed_entity:get(Components.MARKER)
        local config = State.get_sync_config(entity_id, sync.sync_components)
        
        -- Use precise detection to avoid over-sending
        local changed = changed_entity:changed_components()
        local added = changed_entity:added_components()
        
        local has_new_dirty = false
        for _, name in ipairs(changed) do
            if config.sync_components[name] then
                config.dirty[name] = true
                has_new_dirty = true
            end
        end
        for _, name in ipairs(added) do
            -- If the marker or scriptowned was added, or a sync component was added
            if name == Components.MARKER or name == "ScriptOwned" or config.sync_components[name] then
                config.dirty[name] = true
                has_new_dirty = true
            end
        end
        
        if has_new_dirty then
            State.mark_pending_update(entity_id)
        end
    end

    -- 2. Processing Loop: Only iterate over entities with changes OR pending spawns
    local pending = State.get_pending_updates()
    for entity_id, _ in pairs(pending) do
        -- Get the entity from the world (it might have been despawned this frame)
        local entity_entities = world:query({
            with = { Components.MARKER, "ScriptOwned" },
            ids = { entity_id }
        })
        
        if #entity_entities == 0 then
            State.clear_pending_update(entity_id)
            goto continue_pending
        end
        
        local entity = entity_entities[1]
        local sync = entity:get(Components.MARKER)
        
        -- Skip remote entities (we don't send updates for entities we don't own)
        if sync.authority == "remote" then
            State.clear_pending_update(entity_id)
            goto continue_pending
        end
        
        -- Skip entities from other instances
        local script_owned = entity:get("ScriptOwned")
        if script_owned.instance_id ~= __INSTANCE_ID__ then
            State.clear_pending_update(entity_id)
            goto continue_pending
        end
        
        local net_id = sync.net_id or State.get_net_id(entity_id)
        
        -- If no net_id yet, assign one and register
        if not net_id then
            net_id = State.next_net_id()
            entity:patch({ [Components.MARKER] = { net_id = net_id } })
            State.set_owner(net_id, sync.owner_client)
        end
        
        local config = State.get_sync_config(entity_id, sync.sync_components)
        
        -- Handle Scoping
        local spawn_targets = {}
        local update_targets = {}
        
        if is_server and get_clients then
            local all_clients = get_clients(world)
            print(string.format("Sending to %d clients", #all_clients))
            if filter_clients then all_clients = filter_clients(all_clients, entity, net_id) end
            
            for _, client_id in ipairs(all_clients) do
                if State.client_knows_entity(client_id, net_id) then
                    table.insert(update_targets, client_id)
                else
                    table.insert(spawn_targets, client_id)
                end
            end
        else
            -- Client: check if server knows
            if State.client_knows_entity(0, net_id) then
                table.insert(update_targets, 0)
            else
                table.insert(spawn_targets, 0)
            end
        end

        -- 3. Handle Spawns (Initial or New Clients)
        if not config.spawned then
            State.register_entity(net_id, entity_id)
            State.add_sync_types(config.sync_components)
            State.mark_spawned(entity_id)
        end

        if #spawn_targets > 0 then
            local spawn_msg = Messages.build_spawn(world, entity, net_id)
            if spawn_msg then
                spawn_msg.target_clients = spawn_targets
                spawn({ [Components.OUTBOUND] = spawn_msg })
                
                -- Mark clients as knowing this entity
                for _, client_id in ipairs(spawn_targets) do
                    State.add_to_client_scope(client_id, net_id)
                end
            end
        end

        -- 4. Handle Updates (Reliable/Unreliable partitioning)
        local reliable_changed = {}
        local unreliable_changed = {}
        local has_reliable = false
        local has_unreliable = false
        
        local still_dirty = false

        for comp_name, comp_config in pairs(config.sync_components) do
            local authority = comp_config.authority or "server"
            
            -- Skip if client doesn't have authority
            if not is_server and authority ~= "client" then goto next_comp end

            if config.dirty[comp_name] then
                if check_rate_limit(comp_name, config, comp_config.rate_limit) then
                    local comp_data = entity:get(comp_name)
                    if comp_data then
                        if comp_config.reliable == false then
                            unreliable_changed[comp_name] = comp_data
                            has_unreliable = true
                        else
                            reliable_changed[comp_name] = comp_data
                            has_reliable = true
                        end
                        config.dirty[comp_name] = nil
                    end
                else
                    still_dirty = true
                end
            end
            ::next_comp::
        end

        -- Helper for update messages
        local function send_update_to(changed_set, channel, targets)
            if #targets == 0 then return end
            
            local update_msg = Messages.build_update(
                sync.owner_client,
                entity,
                net_id,
                changed_set,
                nil, -- Prediction seq handled in build_update contextually
                nil, -- Ack seq handled in build_update contextually
                channel
            )
            update_msg.target_clients = targets
            spawn({ [Components.OUTBOUND] = update_msg })
        end

        if has_reliable then send_update_to(reliable_changed, Messages.CHANNEL_RELIABLE, update_targets) end
        if has_unreliable then send_update_to(unreliable_changed, Messages.CHANNEL_UNRELIABLE, update_targets) end

        -- 5. Cleanup: Only removal from pending set if NO components are still throttled
        if not still_dirty then
            State.clear_pending_update(entity_id)
        end
        
        ::continue_pending::
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
