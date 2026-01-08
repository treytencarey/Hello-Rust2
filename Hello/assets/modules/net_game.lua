-- Network Game Module
-- High-level game hosting/joining abstraction
--
-- Usage (Server):
--   local NetGame = require("modules/net_game.lua")
--   NetGame.host({
--       port = 5000,
--       max_players = 10,
--       on_player_join = function(client_id) ... end,
--       on_player_leave = function(client_id) ... end
--   })
--
-- Usage (Client):
--   NetGame.join({
--       server_addr = "127.0.0.1",
--       port = 5000,
--       on_connected = function() ... end,
--       on_disconnected = function() ... end
--   })

local NetRole = require("modules/net_role.lua")
local NetSync = require("modules/net_sync.lua")
local NetServer = require("modules/net_server.lua")

local NetGame = {}

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

NetGame.config = {
    reconnection = {
        enabled = false,                   -- Placeholder for future implementation
        grace_period_seconds = 300,        -- How long to preserve state (future)
        identification = "client_uid",     -- "client_uid" | "session_token" | "account" (future)
    }
}

--------------------------------------------------------------------------------
-- State
--------------------------------------------------------------------------------

local hosted = false
local joined = false
local server_config = nil
local client_config = nil

--------------------------------------------------------------------------------
-- Server Hosting
--------------------------------------------------------------------------------

--- Host a game server
--- @param config table { port, max_players, on_player_join(client_id), on_player_leave(client_id) }
function NetGame.host(config)
    if hosted then
        print("[NET_GAME] Already hosting, ignoring")
        return
    end
    
    local port = config.port or 5000
    local max_players = config.max_players or 10
    
    server_config = config
    
    -- Start server
    NetServer.start(port, max_players)
    
    -- Set up callbacks
    if config.on_player_join then
        NetServer.on_client_connect(function(client_id)
            print(string.format("[NET_GAME] Player joined: %d", client_id))
            config.on_player_join(client_id)
        end)
    end
    
    if config.on_player_leave then
        NetServer.on_client_disconnect(function(client_id)
            print(string.format("[NET_GAME] Player left: %d", client_id))
            config.on_player_leave(client_id)
        end)
    end
    
    hosted = true
    print(string.format("[NET_GAME] Hosting on port %d (max %d players)", port, max_players))
end

--------------------------------------------------------------------------------
-- Client Joining
--------------------------------------------------------------------------------

--- Join a game as client
--- @param config table { server_addr, port, on_connected(), on_disconnected() }
function NetGame.join(config)
    if joined then
        print("[NET_GAME] Already joined, ignoring")
        return
    end
    
    local server_addr = config.server_addr or "127.0.0.1"
    local port = config.port or 5000
    
    client_config = config
    
    -- Create client resources
    insert_resource("RenetClient", {})
    insert_resource("NetcodeClientTransport", {
        server_addr = server_addr,
        port = port
    })
    
    joined = true
    print(string.format("[NET_GAME] Connecting to %s:%d", server_addr, port))
    
    -- TODO: Set up connection status monitoring for callbacks
    -- For now, on_connected would need to be called after verifying connection
end

--------------------------------------------------------------------------------
-- State Queries
--------------------------------------------------------------------------------

--- Check if hosting
function NetGame.is_hosting()
    return hosted
end

--- Check if joined as client
function NetGame.is_joined()
    return joined
end

--- Check if we're a server (hosting or role is server/both)
function NetGame.is_server()
    return hosted or NetRole.is_server()
end

--- Check if we're a client (joined or role is client/both)
function NetGame.is_client()
    return joined or NetRole.is_client()
end

--------------------------------------------------------------------------------
-- Update System (Register in game)
--------------------------------------------------------------------------------

--- Create update function for server (call in register_system)
--- @return function
function NetGame.create_server_update()
    return function(world)
        if NetGame.is_server() then
            NetServer.update(world)
        end
    end
end

--- Create outbound sync function for entity replication
--- @return function
function NetGame.create_sync_outbound()
    return function(world)
        NetSync.outbound_system(world, function(channel, message)
            if NetGame.is_server() then
                -- Server: broadcast to all clients
                pcall(function()
                    local clients = world:call_resource_method("RenetServer", "clients_id")
                    for _, client_id in ipairs(clients or {}) do
                        world:call_resource_method("RenetServer", "send_message", client_id, channel, message)
                    end
                end)
            else
                -- Client: send to server
                pcall(function()
                    world:call_resource_method("RenetClient", "send_message", channel, message)
                end)
            end
        end)
    end
end

--- Create inbound sync function for entity replication
--- @return function
function NetGame.create_sync_inbound()
    return function(world)
        NetSync.inbound_system(world, function(channel)
            -- Try to receive as client if RenetClient exists
            local ok, msg = pcall(function()
                return world:call_resource_method("RenetClient", "receive_message", channel)
            end)
            if ok and msg then
                return msg
            end
            return nil
        end)
    end
end

print("[NET_GAME] Module loaded")

return NetGame
