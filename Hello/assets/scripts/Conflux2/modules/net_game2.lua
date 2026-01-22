-- Client-side NetGame2 module for Conflux2
-- Sets up world visuals and uses ECS-based message system

local NetGame2 = require("modules/net_game2.lua")
local NetClient2 = require("modules/net_client2.lua")
local WorldBuilder = require("modules/world_builder.lua")

-- Load player controller for camera attachment on client
local PlayerController = require("scripts/server/Conflux2/modules/shared/player_controller2.lua")

print(string.format("[CONFLUX_NET_GAME2] instance: %s", __LUA_STATE_ID__))

-- Track connection for one-time log
register_system("Update", function(world)
    if NetClient2.is_connected(world) then
        print("[CONFLUX_NET_GAME2] Connected!")
        return true  -- Stop system
    end
end)

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

local GAME_CONFIG = {
    floor_size = {x = 50, y = 1, z = 50},
    floor_color = {r = 0.3, g = 0.7, b = 0.3, a = 1.0},
}

--------------------------------------------------------------------------------
-- World Setup (Client has visuals)
--------------------------------------------------------------------------------

WorldBuilder.create_floor({
    size = GAME_CONFIG.floor_size,
    position = {x = 0, y = -0.5, z = 0},
    color = GAME_CONFIG.floor_color
})

WorldBuilder.create_light({
    position = {x = 10, y = 20, z = 10},
    intensity = 1000000.0
})

--------------------------------------------------------------------------------
-- Client Module
--------------------------------------------------------------------------------

local M = {}

--- Join a server
function M.join(world, config)
    config = config or {}

    NetGame2.join(world, {
        server_addr = config.server_addr or "127.0.0.1",
        port = config.port or 5001,

        on_connected = function()
            print("[CONFLUX_NET_GAME2] Connected to server")
        end,

        on_disconnected = function()
            print("[CONFLUX_NET_GAME2] Disconnected from server")
        end,
    })
end

--- Update wrapper
function M.update(world)
    NetGame2.update(world)
end

print("[CONFLUX_NET_GAME2] Module loaded")

return M
