-- local PlayerController = require("modules/player_controller.lua")
local CameraController = require("modules/modular_camera_controller.lua")
local Camera = require("modules/cameras/third_person.lua")
local Movement = require("modules/movement/face_movement.lua")
local NetSync = require("modules/net_sync.lua")
local NetRole = require("modules/net_role.lua")
local SpawnSystem = require("modules/spawn_system.lua")
local M = {}

-- Swap camera by changing: require("modules/cameras/first_person.lua")
-- Swap movement by changing: require("modules/movement/strafe_style.lua")
CameraController.init({
    camera = Camera,
    movement = Movement,
})

register_system("Startup", function(world)
    -- Initialize spawn system
    SpawnSystem.init(world, {
        locations = {
            {x = -5, y = 0, z = -5},
            {x = 5, y = 0, z = -5},
            {x = -5, y = 0, z = 5},
            {x = 5, y = 0, z = 5},
        },
        show_debug = false  -- Set to true to show spawn markers on clients
    })
    print("[CONFLUX_SERVER_PLAYER_CONTROLLER] SpawnSystem initialized")
    return true -- Single-run
end)

-- Wait for my character to spawn, attach camera (client only)
local function wait_my_character()
    register_system("Update", function(world)
        if NetRole.is_server() then
            return true -- Stop this system on server
        end

        local entities = world:query({NetSync.MARKER, "PlayerState"})
        local my_net_id = NetSync.get_my_net_id()
        
        for _, entity in ipairs(entities) do
            local sync = entity:get(NetSync.MARKER)
            local entity_id = entity:id()
            
            -- On client: attach camera when we find our entity
            print(string.format("[CONFLUX_SERVER_PLAYER_CONTROLLER] Found entity %s with net_id %s, my id: %s", entity_id, sync.net_id, my_net_id))
            if my_net_id == sync.net_id then
                CameraController.attach(world, entity_id)
                return true -- Successful completion, stop system
            end
        end
    end)
end
wait_my_character()

--------------------------------------------------------------------------------
-- spawn_player
--------------------------------------------------------------------------------

--- Spawn a player entity (works on both client and server)
--- net_id and ownership are auto-assigned by NetSync.outbound_system
--- @param client_id number|nil The client_id (used for owner_client in PlayerState)
function M.spawn_player(client_id)
    local spawn_data = SpawnSystem.claim_spawn(client_id)
    if not spawn_data then
        print(string.format("[PLAYER_CONTROLLER] No spawn available for client %s", client_id))
        return nil
    end
    
    local spawn_pos = spawn_data.position
    local spawn_marker = spawn_data.marker
    
    local entity_id = spawn({
        SceneRoot = {
            id = load_asset("Conflux/Placeholder-Character.glb#Scene0")
        },
        Transform = {
            translation = spawn_pos,
            rotation = {x = 0, y = 0, z = 0, w = 1},
            scale = {x = 1, y = 1, z = 1}
        },
        -- Spawn marker for hot-reload persistence
        [SpawnSystem.MARKER] = spawn_marker,
        -- Network sync: net_id auto-assigned by outbound_system on server
        -- On client: no net_id = pending prediction
        [NetSync.MARKER] = {
            authority = "server",
            sync_components = {
                SceneRoot = { rate_hz = 100, reliable = true, authority = "server" },
                Transform = { rate_hz = 20, authority = "server", interpolate = true },
                PlayerState = { rate_hz = 20, reliable = true, authority = "server" },
                MovementConfig = { rate_hz = 1, authority = "server" },
                AnimationState = { rate_hz = 10, authority = "server" },
                PlayerInput = { rate_hz = 60, authority = "client" }
            }
        },
        -- Movement configuration (per-player physics settings)
        MovementConfig = {
            module_path = "modules/movement/strafe_style.lua",
            walk_speed = 10.0,
            run_speed = 20.0,
            gravity = 15.0,
            jump_velocity = 5.0,
            rotation_speed = 20.0,
        },
        -- Player state (owner_client used for auto-ownership detection)
        PlayerState = {
            velocity = {x = 0, y = 0, z = 0},
            owner_client = client_id,
            spawn_pos = spawn_pos
        },
        AnimationState = { current = "idle" },
        -- Player input (world-space format for server processing)
        PlayerInput = {
            move_x = 0,      -- World-space X direction
            move_z = 0,      -- World-space Z direction
            yaw = 0,         -- Camera yaw for rotation sync
            jump = false,
            sprint = false
        }
    }):id()
    
    print(string.format("[PLAYER_CONTROLLER] Spawned player at (%.1f, %.1f, %.1f) entity_id=%d",
        spawn_pos.x, spawn_pos.y, spawn_pos.z, entity_id))
    
    return entity_id
end

--------------------------------------------------------------------------------
-- despawn_player (despawns all entities owned by client)
--------------------------------------------------------------------------------

--- @param client_id number
function M.despawn_player(client_id)
    -- Get all entities owned by this client from NetSync
    local net_ids = NetSync.get_net_ids_for_client(client_id)
    
    print(string.format("[CONFLUX_SERVER_PLAYER_CONTROLLER] Despawning %d entities for client %s", 
        #net_ids, client_id))
    
    -- Despawn each entity - NetSync.outbound_system will automatically detect
    -- the removed NetSync components and send despawn messages
    for _, net_id in ipairs(net_ids) do
        local entity_id = NetSync.get_entity(net_id)
        if entity_id then
            despawn(entity_id)
            print(string.format("[CONFLUX_SERVER_PLAYER_CONTROLLER] Despawned entity for net_id=%d", net_id))
        end

        -- If we despawned our own character, wait for it to respawn
        if net_id == NetSync.get_my_net_id() then
            wait_my_character()
        end
    end
    
    -- Release the spawn claim
    SpawnSystem.release_spawn(client_id)
    print(string.format("[CONFLUX_SERVER_PLAYER_CONTROLLER] Released spawn for client %s", client_id))
end

return M