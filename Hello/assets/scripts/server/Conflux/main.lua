-- Conflux Server Entry Point
-- Server-side game logic for networked 3D game
--
-- Run with: cargo run --features networking -- --network server

local NetRole = require("modules/net_role.lua")
local NetGame = require("modules/net_game.lua")
local NetSync = require("modules/net_sync.lua")
local NetServer = require("modules/net_server.lua")
local WorldBuilder = require("modules/world_builder.lua")
local SpawnSystem = require("modules/spawn_system.lua")
local PlayerController = require("modules/player_controller.lua")
local CharacterSync = require("modules/character_sync.lua")

print("[CONFLUX SERVER] Initializing...")

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

local GAME_CONFIG = {
    port = 5000,
    max_players = 4,
    floor_size = {x = 50, y = 1, z = 50},
    floor_color = {r = 0.3, g = 0.7, b = 0.3, a = 1.0},
    model_path = "Conflux/Placeholder-Character.glb#Scene0",
}

--------------------------------------------------------------------------------
-- World Setup (Server has collision but no visuals)
--------------------------------------------------------------------------------

WorldBuilder.create_floor({
    size = GAME_CONFIG.floor_size,
    position = {x = 0, y = -0.5, z = 0},
    color = GAME_CONFIG.floor_color
})

print("[CONFLUX SERVER] World geometry created")

--------------------------------------------------------------------------------
-- Spawn System
--------------------------------------------------------------------------------

SpawnSystem.init({
    locations = {
        {x = -5, y = 0, z = -5},
        {x = 5, y = 0, z = -5},
        {x = -5, y = 0, z = 5},
        {x = 5, y = 0, z = 5},
    },
    show_debug = false
})

--------------------------------------------------------------------------------
-- Player Management
--------------------------------------------------------------------------------

local player_entities = {}  -- client_id -> entity_id
local next_net_id = 1000    -- Server-controlled net_ids start at 1000

local function spawn_player(client_id)
    local spawn_pos = SpawnSystem.claim_spawn(client_id)
    if not spawn_pos then
        print(string.format("[CONFLUX SERVER] No spawn available for client %d", client_id))
        return nil
    end
    
    local net_id = next_net_id
    next_net_id = next_net_id + 1
    
    local entity_id = CharacterSync.create_character({
        model_path = GAME_CONFIG.model_path,
        position = spawn_pos,
        owner_client = client_id,
        net_id = net_id
    })
    
    player_entities[client_id] = { entity_id = entity_id, net_id = net_id }
    NetSync.register_entity(net_id, entity_id)
    
    -- Send "your_character" message to THIS client so they know which entity is theirs
    register_system("PostUpdate", function(world)
        local json = require("modules/dkjson.lua")
        local msg = json.encode({ type = "your_character", net_id = net_id })
        local channel = 0  -- Reliable
        world:call_resource_method("RenetServer", "send_message", client_id, channel, msg)
        print(string.format("[CONFLUX SERVER] Sent your_character to client %d: net_id=%d", client_id, net_id))
        return true  -- Run once
    end)
    
    print(string.format("[CONFLUX SERVER] Spawned player for client %d at (%.1f, %.1f, %.1f) net_id=%d",
        client_id, spawn_pos.x, spawn_pos.y, spawn_pos.z, net_id))
    
    return entity_id
end

local function despawn_player(client_id)
    local player_data = player_entities[client_id]
    if player_data then
        local entity_id = player_data.entity_id
        local net_id = player_data.net_id
        
        -- Prepare despawn message
        local json = require("modules/dkjson.lua")
        local despawn_msg = json.encode({ type = "despawn", net_id = net_id })
        
        -- Use a one-shot system to broadcast (world not available in callback)
        register_system("PostUpdate", function(world)
            print(string.format("[CONFLUX SERVER] Broadcasting despawn message: %s", despawn_msg))
            local clients = world:call_resource_method("RenetServer", "clients_id")
            print(string.format("[CONFLUX SERVER] Broadcasting to %d clients", #(clients or {})))
            for _, cid in ipairs(clients or {}) do
                world:call_resource_method("RenetServer", "send_message", cid, 0, despawn_msg)
                print(string.format("[CONFLUX SERVER] Sent despawn to client %d", cid))
            end
            return true  -- Run once
        end)
        
        -- Clear local tracking
        NetSync.despawn_by_net_id(nil, net_id, nil)  -- No send_fn, we handle broadcast above
        
        despawn(entity_id)
        player_entities[client_id] = nil
    end
    SpawnSystem.release_spawn(client_id)
    print(string.format("[CONFLUX SERVER] Despawned player for client %d", client_id))
end

--------------------------------------------------------------------------------
-- Host Game
--------------------------------------------------------------------------------

NetGame.host({
    port = GAME_CONFIG.port,
    max_players = GAME_CONFIG.max_players,
    on_player_join = function(client_id)
        spawn_player(client_id)
    end,
    on_player_leave = function(client_id)
        despawn_player(client_id)
    end
})

--------------------------------------------------------------------------------
-- Server Update Systems
--------------------------------------------------------------------------------

-- Process player inputs and update authoritative state
register_system("Update", function(world)
    -- Update server networking
    NetServer.update(world)
    
    -- Process PlayerInput changes from clients
    local players = world:query({"NetworkSync", "PlayerInput", "Transform", "PlayerState"}, {"PlayerInput"})
    
    for _, entity in ipairs(players) do
        -- Skip local player in "both" mode - client handles prediction
        local sync = entity:get("NetworkSync")
        if sync and sync.net_id == NetSync.get_my_net_id() then
            goto continue
        end
        
        local result = PlayerController.process_server_input(entity)
        if result then
            -- Use patch() to preserve model_path and other existing fields
            entity:patch({
                Transform = result.transform,
                PlayerState = {
                    velocity = result.velocity,
                    last_acked_seq = result.last_acked_seq
                }
            })
        end
        
        ::continue::
    end
end)

-- Update animation states
register_system("Update", CharacterSync.create_animation_system())

-- Outbound sync (send entity updates to clients)
register_system("Update", NetGame.create_sync_outbound())

print("[CONFLUX SERVER] Ready on port " .. GAME_CONFIG.port)
