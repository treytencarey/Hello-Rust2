-- Client-side module for Conflux2
-- Doesn't modify NetGame directly, just provides world setup

local NetGame = require("modules/net_game.lua")
local PlayerController = require("scripts/server/Conflux2/modules/shared/player_controller.lua")
local NetClient = require("modules/net_client.lua")
local WorldBuilder = require("modules/world_builder.lua")

print(string.format("[CONFLUX_NET_GAME] instance: %s", __LUA_STATE_ID__))

register_system("Update", function(world)
    if NetClient.is_connected(world) then
        print("Connected!")
        return true -- Stop processing system
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

-- Return NetGame directly - no separate module needed
-- Any overrides applied by server module will be used
return NetGame