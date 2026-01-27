-- Net3 Client Module
-- Client-specific Renet handling: send/receive

local Components = require("modules/net3/components.lua")
local Messages = require("modules/net3/messages.lua")
local json = require("modules/dkjson.lua")

local Client = {}

--------------------------------------------------------------------------------
-- Send System - Process NetSyncOutbound entities
--------------------------------------------------------------------------------

--- Client send system - sends outbound messages to server via Renet
--- @param world userdata The world object
function Client.send_system(world)
    -- Guard: ensure RenetClient resource exists
    if not world:query_resource("RenetClient") then return end
    
    local outbound = world:query({ Components.OUTBOUND, "ScriptOwned" })
    
    for _, msg_entity in ipairs(outbound) do
        -- Skip messages from other instances
        local script_owned = msg_entity:get("ScriptOwned")
        if script_owned.instance_id ~= __INSTANCE_ID__ then
            goto continue_send
        end
        
        local msg = msg_entity:get(Components.OUTBOUND)
        local channel = msg.channel or Messages.CHANNEL_RELIABLE
        
        -- Encode message
        local encoded = json.encode({
            msg_type = msg.msg_type,
            net_id = msg.net_id,
            owner_client = msg.owner_client,
            payload = msg.payload,
        })
        -- print(string.format("[NET3_CLIENT] Sending message (%s): %s", channel == Messages.CHANNEL_RELIABLE and "RELIABLE" or "UNRELIABLE", encoded))
        
        -- Send to server
        world:call_resource_method("RenetClient", "send_message", channel, encoded)
        
        -- Despawn the outbound entity
        despawn(msg_entity:id())
        
        ::continue_send::
    end
end

--------------------------------------------------------------------------------
-- Receive System - Process incoming server messages
--------------------------------------------------------------------------------

--- Client receive system - receives messages from server and creates NetSyncInbound entities
--- @param world userdata The world object
function Client.receive_system(world)
    -- Guard: ensure RenetClient resource exists
    if not world:query_resource("RenetClient") then return end
    
    -- Reliable channel
    while true do
        local msg_data = world:call_resource_method("RenetClient", "receive_message", Messages.CHANNEL_RELIABLE)
        if not msg_data or msg_data == "" then break end
        -- print(string.format("[NET3_CLIENT] Received message: %s", msg_data))
        
        local success, msg = pcall(json.decode, msg_data)
        if success and msg then
            spawn({ [Components.INBOUND] = msg })
        end
    end
    
    -- Unreliable channel
    while true do
        local msg_data = world:call_resource_method("RenetClient", "receive_message", Messages.CHANNEL_UNRELIABLE)
        if not msg_data or msg_data == "" then break end
        -- print(string.format("[NET3_CLIENT] Received message: %s", msg_data))
        
        local success, msg = pcall(json.decode, msg_data)
        if success and msg then
            spawn({ [Components.INBOUND] = msg })
        end
    end
end

return Client
