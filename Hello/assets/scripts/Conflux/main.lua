-- Conflux Client Entry Point
-- Client-side game logic for networked 3D game
--
-- Run with: cargo run --features networking -- --network client

local NetRole = require("modules/net_role.lua")
local NetGame = require("modules/net_game.lua")
local NetSync = require("modules/net_sync.lua")
local WorldBuilder = require("modules/world_builder.lua")
local InputManager = require("modules/input_manager.lua")
local PlayerController = require("modules/player_controller.lua")
local CameraController = require("modules/camera_controller.lua")

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
-- Client State
--------------------------------------------------------------------------------

local my_entity_id = nil
local my_controller = nil
local connected = false

--------------------------------------------------------------------------------
-- Connect to Server
--------------------------------------------------------------------------------

NetGame.join({
    server_addr = GAME_CONFIG.server_addr,
    port = GAME_CONFIG.port,
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

--------------------------------------------------------------------------------
-- Camera Setup
--------------------------------------------------------------------------------

CameraController.create_camera()

-- Default to third person, VR auto-switches to first person
if NetRole.is_offline() then
    CameraController.set_mode("third_person")
end

--------------------------------------------------------------------------------
-- Client Update Systems
--------------------------------------------------------------------------------

-- Find our character when it spawns
local debug_find_character = true  -- Enable to debug
register_system("Update", function(world)
    if my_entity_id then return end  -- Already found
    
    -- Wait for server to tell us which character is ours
    local my_net_id = NetSync.get_my_net_id()
    if not my_net_id then
        if debug_find_character then
            debug_find_character = false  -- Only print once
            print("[CONFLUX CLIENT] Waiting for my_net_id...")
        end
        return
    end
    
    -- Get our entity from the net_id
    local entity_id = NetSync.get_my_entity()
    if not entity_id then
        print(string.format("[CONFLUX CLIENT] Have my_net_id=%s but entity not in known_entities yet", tostring(my_net_id)))
        return
    end
    
    -- Get entity wrapper
    local entity = world:get_entity(entity_id)
    if not entity then
        print(string.format("[CONFLUX CLIENT] Have entity_id=%s but world:get_entity returned nil", tostring(entity_id)))
        return
    end
    
    local sync = entity:get("NetworkSync")
    if not sync then
        print("[CONFLUX CLIENT] Entity exists but no NetworkSync component")
        return
    end
    
    my_entity_id = entity_id
    
    -- Create client controller for prediction
    my_controller = PlayerController.create_client_controller(entity, function(input_msg)
        -- Send input to server via NetworkSync
        entity:set({
            PlayerInput = input_msg
        })
    end)
    
    -- Attach camera
    CameraController.attach(my_entity_id)
    
    print(string.format("[CONFLUX CLIENT] Found my character: entity=%d net_id=%d", 
        my_entity_id, my_net_id))
end)

-- Process input and apply prediction
register_system("Update", function(world)
    if not my_controller then return end
    
    local input = InputManager.get_movement_input(world)
    local dt = world:delta_time()
    
    -- Process input with prediction
    my_controller.process_input(world, input, dt)
end)

-- Update camera
register_system("Update", function(world)
    local dt = world:delta_time()
    CameraController.update(world, dt)
end)

-- Inbound sync (receive entity updates from server)
register_system("Update", NetGame.create_sync_inbound())

-- Handle server state updates for reconciliation
register_system("Update", function(world)
    if not my_controller or not my_entity_id then return end
    
    local entity = world:get_entity(my_entity_id)
    if not entity then return end
    
    local player_state = entity:get("PlayerState")
    if player_state then
        -- Reconcile with server state
        my_controller.on_server_state({
            sequence = player_state.last_acked_seq,
            position = entity:get("Transform").translation,
            velocity = player_state.velocity
        })
    end
end)

print("[CONFLUX CLIENT] Connecting to " .. GAME_CONFIG.server_addr .. ":" .. GAME_CONFIG.port)
