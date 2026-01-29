-- NET Message Building Module
-- Utilities for constructing network messages (spawn, update, despawn, etc.)

local Components = require("modules/net/components.lua")
local State = require("modules/net/state.lua")

local Messages = {}

-- Channel constants
Messages.CHANNEL_RELIABLE = 0
Messages.CHANNEL_UNRELIABLE = 1

--------------------------------------------------------------------------------
-- Spawn Message
--------------------------------------------------------------------------------

--- Build a spawn message for an entity
--- @param net_id number The network ID
--- @param owner_client number The owning client ID
--- @param components table { comp_name -> value } All component data to include
--- @param parent_net_id number|nil Parent's net_id if entity has ChildOf
--- @return table The outbound message
function Messages.build_spawn(net_id, owner_client, components, parent_net_id)
    return {
        msg_type = "spawn",
        channel = Messages.CHANNEL_RELIABLE,
        net_id = net_id,
        owner_client = owner_client,
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
--- @param channel number|nil Optional channel (defaults to RELIABLE)
--- @return table The outbound message
function Messages.build_update(owner_client, entity, net_id, changed_components, seq, ack_seq, channel)
    return {
        msg_type = "update",
        channel = channel or Messages.CHANNEL_RELIABLE,
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
