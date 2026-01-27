-- Net3 Server Module
-- Server-specific Renet handling: send/receive, client connection management

local Components = require("modules/net3/components.lua")
local State = require("modules/net3/state.lua")
local Messages = require("modules/net3/messages.lua")
local json = require("modules/dkjson.lua")

local Server = {}

local state = State.get()

--------------------------------------------------------------------------------
-- Send System - Process NetSyncOutbound entities
--------------------------------------------------------------------------------

--- Server send system - sends outbound messages to clients via Renet
--- @param world userdata The world object
function Server.send_system(world)
    -- Guard: ensure RenetServer resource exists
    if not world:query_resource("RenetServer") then return end
    
    local outbound = world:query({ Components.OUTBOUND, "ScriptOwned" })
    
    for _, msg_entity in ipairs(outbound) do
        -- Skip messages from other instances
        local script_owned = msg_entity:get("ScriptOwned")
        if script_owned.instance_id ~= __INSTANCE_ID__ then
            goto continue_send
        end
        
        local msg = msg_entity:get(Components.OUTBOUND)
        local target_clients = msg.target_clients or {}
        local channel = msg.channel or Messages.CHANNEL_RELIABLE
        
        -- Encode message
        local encoded = json.encode({
            msg_type = msg.msg_type,
            net_id = msg.net_id,
            owner_client = msg.owner_client,
            payload = msg.payload,
        })
        -- print(string.format("[NET3_SERVER] Sending message (%s): %s", channel == Messages.CHANNEL_RELIABLE and "RELIABLE" or "UNRELIABLE", encoded))
        
        -- Send to each target client
        for _, client_id in ipairs(target_clients) do
            world:call_resource_method("RenetServer", "send_message", client_id, channel, encoded)
        end
        
        -- Despawn the outbound entity
        despawn(msg_entity:id())
        
        ::continue_send::
    end
end

--------------------------------------------------------------------------------
-- Receive System - Process incoming client messages
--------------------------------------------------------------------------------

--- Server receive system - receives messages from clients and creates NetSyncInbound entities
--- @param world userdata The world object
--- @param get_clients function Returns list of connected client IDs
function Server.receive_system(world, get_clients)
    -- Guard: ensure RenetServer resource exists
    if not world:query_resource("RenetServer") then return end
    
    for _, client_id in ipairs(get_clients(world)) do
        -- Reliable channel
        while true do
            local msg_data = world:call_resource_method("RenetServer", "receive_message", client_id, Messages.CHANNEL_RELIABLE)
            if not msg_data or msg_data == "" then break end
            
            local success, msg = pcall(json.decode, msg_data)
            if success and msg then
                msg.owner_client = client_id
                spawn({ [Components.INBOUND] = msg })
            end
        end
        
        -- Unreliable channel
        while true do
            local msg_data = world:call_resource_method("RenetServer", "receive_message", client_id, Messages.CHANNEL_UNRELIABLE)
            if not msg_data or msg_data == "" then break end
            
            local success, msg = pcall(json.decode, msg_data)
            if success and msg then
                msg.owner_client = client_id
                spawn({ [Components.INBOUND] = msg })
            end
        end
    end
end

--------------------------------------------------------------------------------
-- Client Connection Handlers
--------------------------------------------------------------------------------

--- Called when a new client connects
--- @param client_id number
--- @param world userdata
function Server.on_client_connected(client_id, world)
    print(string.format("[NET3_SERVER] Client %s connected", client_id))

    -- Initialize scope
    State.init_client_scope(client_id)

    -- Send client_id assignment message
    local client_id_msg = Messages.build_client_id(client_id)
    spawn({ [Components.OUTBOUND] = client_id_msg })

    -- Build a lookup of all synced entities via query (avoids world:get_entity)
    -- Include all sync components so their values are available
    local sync_types = State.get_sync_types()
    local sync_type_names = {}
    for comp_name, _ in pairs(sync_types) do
        table.insert(sync_type_names, comp_name)
    end

    local query_spec = {
        with = { Components.MARKER, "ScriptOwned" }
    }
    if #sync_type_names > 0 then
        query_spec.any_of = sync_type_names
    end

    local all_synced = world:query(query_spec)
    local entity_by_id = {}
    for _, entity in ipairs(all_synced) do
        entity_by_id[entity:id()] = entity
    end

    -- Send all existing entities to new client
    local count = 0
    for net_id, entity_id in pairs(state.known_entities) do
        local entity = entity_by_id[entity_id]
        if entity then
            local sync = entity:get(Components.MARKER)
            if sync and sync.authority ~= "remote" then
                -- Collect all component values for spawn
                local sync_components = sync.sync_components or { Transform = {} }
                local components = {}
                for comp_name, _ in pairs(sync_components) do
                    local comp_data = entity:get(comp_name)
                    if comp_data then
                        components[comp_name] = comp_data
                    end
                end
                components[Components.MARKER] = sync

                -- Check for parent relationship
                local parent_net_id = nil
                if entity:has("ChildOf") then
                    local child_of = entity:get("ChildOf")
                    if child_of and child_of.parent then
                        parent_net_id = State.get_net_id(child_of.parent)
                    end
                end

                local spawn_msg = Messages.build_spawn(net_id, sync.owner_client, components, parent_net_id)
                spawn_msg.target_clients = { client_id }
                spawn({ [Components.OUTBOUND] = spawn_msg })
                State.add_to_client_scope(client_id, net_id)
                count = count + 1
            end
        end
    end

    print(string.format("[NET3_SERVER] Sent %d entities to client %s", count, client_id))
end

--- Called when a client disconnects
--- @param client_id number
--- @param world userdata
--- @param get_clients function
function Server.on_client_disconnected(client_id, world, get_clients)
    print(string.format("[NET3_SERVER] Client %s disconnected", client_id))
    
    -- Find and despawn entities owned by this client
    for net_id, owner in pairs(state.net_id_owners) do
        if owner == client_id then
            local despawn_msg = Messages.build_despawn(owner, net_id)
            
            -- Send to remaining clients
            local remaining = {}
            for _, cid in ipairs(get_clients(world)) do
                if cid ~= client_id and State.client_knows_entity(cid, net_id) then
                    table.insert(remaining, cid)
                    State.remove_from_client_scope(cid, net_id)
                end
            end
            despawn_msg.target_clients = remaining
            
            spawn({ [Components.OUTBOUND] = despawn_msg })
            
            -- Despawn locally
            local entity_id = state.known_entities[net_id]
            if entity_id then
                despawn(entity_id)
            end
            
            State.unregister_entity(net_id)
            State.clear_owner(net_id)
        end
    end
    
    -- Remove client scope
    State.remove_client_scope(client_id)
end

return Server
