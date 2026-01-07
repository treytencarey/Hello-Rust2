-- Network Client Module
-- Handles client connection and uses NetSync for entity replication
--
-- Usage:
--   local NetClient = require("modules/net_client.lua")
--   NetClient.connect("127.0.0.1", 5000)
--   register_system("Update", function(world) NetClient.update(world) end)

local NetSync = require("modules/net_sync.lua")

local NetClient = {}

-- Channels
local CHANNEL_RELIABLE = 0
local CHANNEL_UNRELIABLE = 1

-- Connection state
local is_connected_flag = false

--------------------------------------------------------------------------------
-- Connection
--------------------------------------------------------------------------------

--- Connect to a server
function NetClient.connect(server_addr, port)
    insert_resource("RenetClient", {})
    insert_resource("NetcodeClientTransport", {
        server_addr = server_addr,
        port = port
    })
    print(string.format("[NET_CLIENT] Connecting to %s:%d", server_addr, port))
end

--- Check if connected
function NetClient.is_connected(world)
    local ok, connected = pcall(function()
        return world:call_resource_method("RenetClient", "is_connected")
    end)
    is_connected_flag = ok and connected
    return is_connected_flag
end

--------------------------------------------------------------------------------
-- Send/Receive Wrappers
--------------------------------------------------------------------------------

--- Send on channel
local function send(world, channel, message)
    pcall(function()
        world:call_resource_method("RenetClient", "send_message", channel, message)
    end)
end

--- Receive from channel
local function receive(world, channel)
    local ok, msg = pcall(function()
        return world:call_resource_method("RenetClient", "receive_message", channel)
    end)
    if ok and msg then
        return msg
    end
    return nil
end

--------------------------------------------------------------------------------
-- Update System
--------------------------------------------------------------------------------

--- Main update - call in your Update system
function NetClient.update(world)
    if not NetClient.is_connected(world) then
        return
    end
    
    -- Outbound: Send our changes
    NetSync.outbound_system(world, function(channel, message)
        send(world, channel, message)
    end)
    
    -- Inbound: Receive remote changes
    NetSync.inbound_system(world, function(channel)
        return receive(world, channel)
    end)
end

--------------------------------------------------------------------------------
-- Helpers
--------------------------------------------------------------------------------

--- Get next net_id (delegates to NetSync)
function NetClient.next_net_id()
    return NetSync.next_net_id()
end

--- Set client prefix (called when server assigns our ID)
function NetClient.set_prefix(prefix)
    NetSync.set_client_prefix(prefix)
end

print("[NET_CLIENT] Module loaded")

return NetClient
