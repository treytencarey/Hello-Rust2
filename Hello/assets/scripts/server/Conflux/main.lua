-- Conflux Server Main Entry Point
-- Entry point for server-side game using net modules
-- Just requiring modules registers their systems automatically

local NetGame = require("modules/net/net_game.lua", { instanced = true })
require("modules/net/server_movement.lua")

print("[CONFLUX_SERVER] Starting...")

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

local config = define_resource("ConfluxServerConfig", {
    port = 5000,
})

--------------------------------------------------------------------------------
-- Parse command line arguments
--------------------------------------------------------------------------------

local args = get_args()
for i, arg in ipairs(args) do
    if arg == "--port" and args[i+1] then
        config.port = tonumber(args[i+1])
    end
end

--------------------------------------------------------------------------------
-- Player Spawning
--------------------------------------------------------------------------------

local function spawn_player_for_client(client_id, world)
    print(string.format("[CONFLUX_SERVER] Spawning player for client %s", client_id))
    
    local player_id = spawn({
        Transform = {
            translation = { x = 0, y = 1, z = 0 },
            rotation = { x = 0, y = 0, z = 0, w = 1 },
            scale = { x = 1, y = 1, z = 1 },
        },
        SceneRoot = {
            id = load_asset("Conflux/Placeholder-Character.glb#Scene0")
        },
        PlayerState = {
            health = 100,
            name = "Player_" .. client_id,
        },
        [NetGame.MARKER] = {
            owner_client = client_id,
            authority = "server",
            sync_components = {
                SceneRoot = { rate_hz = 1.0 }, -- 1Hz
                Transform = { reliable = false, rate_limit = 0.033 },  -- 30Hz
                PlayerState = { rate_limit = 0.5 },   -- 2Hz
                PlayerInput = { authority = "client" }, -- No rate limit (unlimited)
            },
        },
    }):id()
    
    print(string.format("[CONFLUX_SERVER] Player spawned: entity=%d", player_id))
    
    return player_id
end

local function despawn_player_for_client(client_id, world)
    print(string.format("[CONFLUX_SERVER] Cleaning up player for client %d", client_id))
    
    -- Find and despawn player entity
    local state = NetGame.get_state()
    for net_id, entity_id in pairs(state.known_entities) do
        if state.net_id_owners[net_id] == client_id then
            local entity = world:get_entity(entity_id)
            if entity and entity:has("PlayerState") then
                despawn(entity_id)
                print(string.format("[CONFLUX_SERVER] Despawned player entity %d", entity_id))
            end
        end
    end
end

--------------------------------------------------------------------------------
-- Initialize
--------------------------------------------------------------------------------

local function on_game_ready()
    print("[CONFLUX_SERVER] Server ready and listening")
end

local function on_player_joined(client_id, world)
    print(string.format("[CONFLUX_SERVER] Player joined: %s", client_id))
    spawn_player_for_client(client_id, world)
end

local function on_player_left(client_id, world)
    print(string.format("[CONFLUX_SERVER] Player left: %s", client_id))
    despawn_player_for_client(client_id, world)
end

-- Host the server
NetGame.host(config.port, {
    on_game_ready = on_game_ready,
    on_player_joined = on_player_joined,
    on_player_left = on_player_left,
})

print(string.format("[CONFLUX_SERVER] Hosting on port %d, instance_id: %d", config.port, __INSTANCE_ID__))
