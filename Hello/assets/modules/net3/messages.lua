-- Net3 Message Building Module
-- Utilities for constructing network messages (spawn, update, despawn, etc.)

local Components = require("modules/net3/components.lua")
local State = require("modules/net3/state.lua")

local Messages = {}

-- Channel constants
Messages.CHANNEL_RELIABLE = 0
Messages.CHANNEL_UNRELIABLE = 1

--------------------------------------------------------------------------------
-- Spawn Message
--------------------------------------------------------------------------------

--- Build a spawn message for an entity
--- @param world userdata The world object
--- @param entity userdata The entity snapshot
--- @param net_id number The network ID
--- @return table|nil The outbound message, or nil if failed
function Messages.build_spawn(world, entity, net_id)
    local sync = entity:get(Components.MARKER)
    if not sync then
        return nil
    end
    
    local entity_id = entity:id()
    local config = State.get_sync_config(entity_id, sync.sync_components)
    
    -- Collect all synced component data
    local components = {}
    for comp_name, _ in pairs(config.sync_components) do
        local comp_data = entity:get(comp_name)
        if comp_data then
            components[comp_name] = comp_data
        end
    end
    
    -- Include the NetworkSync marker itself
    components[Components.MARKER] = sync
    
    -- Check for parent relationship
    local parent_net_id = nil
    if entity:has("ChildOf") then
        local child_of = entity:get("ChildOf")
        if child_of and child_of.parent then
            parent_net_id = State.get_net_id(child_of.parent)
        end
    end
    
    return {
        msg_type = "spawn",
        channel = Messages.CHANNEL_RELIABLE,
        net_id = net_id,
        owner_client = sync.owner_client,
        payload = {
            components = components,
            parent_net_id = parent_net_id,
        },
    }
end

--------------------------------------------------------------------------------
-- Update Message
--------------------------------------------------------------------------------

--- Build an update message for changed components
--- @param owner_client number The owning client ID
--- @param entity userdata The entity snapshot
--- @param net_id number The network ID
--- @param changed_components table Map of component_name -> component_data
--- @param seq number|nil Input sequence number (for client prediction)
--- @param ack_seq number|nil Last acknowledged sequence (server -> client)
--- @return table The outbound message
function Messages.build_update(owner_client, entity, net_id, changed_components, seq, ack_seq)
    return {
        msg_type = "update",
        channel = Messages.CHANNEL_UNRELIABLE,
        net_id = net_id,
        owner_client = owner_client,
        payload = {
            components = changed_components,
            seq = seq,
            ack_seq = ack_seq,
        },
    }
end

--------------------------------------------------------------------------------
-- Despawn Message
--------------------------------------------------------------------------------

--- Build a despawn message
--- @param owner_client number The owning client ID
--- @param net_id number The network ID
--- @return table The outbound message
function Messages.build_despawn(owner_client, net_id)
    return {
        msg_type = "despawn",
        channel = Messages.CHANNEL_RELIABLE,
        net_id = net_id,
        owner_client = owner_client,
        payload = {},
    }
end

--------------------------------------------------------------------------------
-- Owner Change Message
--------------------------------------------------------------------------------

--- Build an owner change message
--- @param net_id number The network ID
--- @param new_owner number The new owning client ID
--- @return table The outbound message
function Messages.build_owner_change(net_id, new_owner)
    return {
        msg_type = "owner_change",
        channel = Messages.CHANNEL_RELIABLE,
        net_id = net_id,
        payload = {
            new_owner = new_owner,
        },
    }
end

--------------------------------------------------------------------------------
-- Client ID Message (server -> client)
--------------------------------------------------------------------------------

--- Build a client ID assignment message
--- @param client_id number The assigned client ID
--- @return table The outbound message
function Messages.build_client_id(client_id)
    return {
        msg_type = "client_id",
        channel = Messages.CHANNEL_RELIABLE,
        target_clients = { client_id },
        payload = {
            client_id = client_id,
        },
    }
end

return Messages
