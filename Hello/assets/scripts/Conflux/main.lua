-- Conflux Client Entry Point
-- Client-side game logic for networked 3D game
--
-- Run with: cargo run --features networking -- --network client

local NetRole = require("modules/net_role.lua")
local NetGame = require("modules/net_game.lua")
local WorldBuilder = require("modules/world_builder.lua")

print("[CONFLUX CLIENT] Initializing...")

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

local GAME_CONFIG = {
    server_addr = "127.0.0.1",
    port = 5000,
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

print("[CONFLUX CLIENT] World geometry created")

--------------------------------------------------------------------------------
-- Connect to Server (deferred to first Update system)
--------------------------------------------------------------------------------

local connection_initialized = false

register_system("Startup", function(world)
    if not connection_initialized then
        NetGame.join(world, {
            server_addr = GAME_CONFIG.server_addr,
            port = GAME_CONFIG.port,
            -- Game-specific modules (can be swapped for different games)
            camera_script = "scripts/Conflux/modules/camera.lua",
            controller_script = "scripts/Conflux/modules/controller.lua",
            on_connected = function()
                connected = true
                print("[CONFLUX CLIENT] Connected to server!")
            end,
            on_disconnected = function()
                connected = false
                my_entity_id = nil
                my_controller = nil
                print("[CONFLUX CLIENT] Disconnected from server")
            end
        })
        connection_initialized = true
    end
    return true  -- Run once
end)

--------------------------------------------------------------------------------
-- Client Update Systems
--------------------------------------------------------------------------------

-- Main update - NetGame handles networking, player input, camera, and interpolation
register_system("Update", function(world)
    NetGame.update(world)
end)