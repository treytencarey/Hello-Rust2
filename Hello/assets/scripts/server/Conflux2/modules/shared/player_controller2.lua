-- Player Controller for Conflux2 using NetSync2
-- Handles player spawning/despawning and camera attachment

local CameraController = require("modules/modular_camera_controller.lua")
local Camera = require("modules/cameras/third_person.lua")
local Movement = require("modules/movement/face_movement.lua")
local NetSync2 = require("modules/net_sync2.lua")
local NetGame2 = require("modules/net_game2.lua")
local NetRole = require("modules/net_role.lua")
local SpawnSystem = require("modules/spawn_system.lua")

local M = {}

-- Initialize camera controller
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
        show_debug = false
    })
    print("[PLAYER_CONTROLLER2] SpawnSystem initialized")
    return true
end)

--------------------------------------------------------------------------------
-- Wait for my character to spawn, attach camera (client only)
--------------------------------------------------------------------------------

local function wait_my_character()
    register_system("Update", function(world)
        if NetRole.is_server() then
            return true  -- Stop this system on server
        end

        local my_client_id = NetGame2.get_my_client_id()
        if not my_client_id then
            -- Haven't received our client_id from server yet
            return
        end

        local entities = world:query({NetSync2.MARKER, "PlayerState", "ScriptOwned"})

        for _, entity in ipairs(entities) do
            local player_state = entity:get("PlayerState")
            local entity_id = entity:id()

            print(string.format("[PLAYER_CONTROLLER2] Checking entity: entity_id=%s, owner_client=%s", entity_id, player_state.owner_client))
            if entity_id and player_state then
                -- Check if this entity belongs to us by owner_client
                if player_state.owner_client == my_client_id then
                    print(string.format("[PLAYER_CONTROLLER2] Found my character: entity_id=%s", entity_id))
                    return CameraController.attach(world, entity_id)
                end
            end

            ::continue_entities::
        end
    end)
end
wait_my_character()

--------------------------------------------------------------------------------
-- Validation function for PlayerInput
--------------------------------------------------------------------------------

--- Validate PlayerInput updates from clients
local function validate_player_input(world, entity, old_value, new_value, owner_client)
    -- Check ownership
    local sync = entity:get(NetSync2.MARKER)
    if sync and sync.owner_client ~= owner_client then
        print(string.format("[PLAYER_CONTROLLER2] Rejected input: client %s doesn't own entity", owner_client))
        return false, nil
    end

    -- Sanitize input values
    local sanitized = {
        move_x = math.max(-1, math.min(1, new_value.move_x or 0)),
        move_z = math.max(-1, math.min(1, new_value.move_z or 0)),
        yaw = new_value.yaw or 0,
        jump = new_value.jump == true,
        sprint = new_value.sprint == true,
    }

    return true, sanitized
end

--------------------------------------------------------------------------------
-- spawn_player
--------------------------------------------------------------------------------

--- Spawn a player entity
--- @param client_id number|nil The client_id (used for owner_client in PlayerState)
function M.spawn_player(client_id)
    print(string.format("[PLAYER_CONTROLLER2] Spawning player for client %s (role: %s)", client_id, NetRole.is_server() and "server" or "client"))
    local spawn_data = SpawnSystem.claim_spawn(client_id)
    if not spawn_data then
        print(string.format("[PLAYER_CONTROLLER2] No spawn available for client %s", client_id))
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
        -- Network sync using NetSync2
        [NetSync2.MARKER] = {
            authority = "server",
            owner_client = client_id,
            sync_components = {
                SceneRoot = { rate_hz = 100, reliable = true, authority = "server" },
                Transform = { rate_hz = 20, authority = "server", interpolate = true },
                PlayerState = { rate_hz = 20, reliable = true, authority = "server" },
                MovementConfig = { rate_hz = 1, authority = "server" },
                AnimationState = { rate_hz = 10, authority = "server" },
                PlayerInput = {
                    rate_hz = 60,
                    authority = "client",
                    validation = validate_player_input,  -- Server validates input
                }
            },
            -- Per-entity filter can be set here:
            -- filter_fn = function(world, client_id, entity, msg) return true end,
        },
        -- Movement configuration
        MovementConfig = {
            module_path = "modules/movement/strafe_style.lua",
            walk_speed = 10.0,
            run_speed = 20.0,
            gravity = 15.0,
            jump_velocity = 5.0,
            rotation_speed = 20.0,
        },
        -- Player state
        PlayerState = {
            velocity = {x = 0, y = 0, z = 0},
            owner_client = client_id,
            spawn_pos = spawn_pos
        },
        AnimationState = { current = "idle" },
        -- Player input
        PlayerInput = {
            move_x = 0,
            move_z = 0,
            yaw = 0,
            jump = false,
            sprint = false
        }
    }):id()

    print(string.format("[PLAYER_CONTROLLER2] Spawned player at (%.1f, %.1f, %.1f) entity_id=%d",
        spawn_pos.x, spawn_pos.y, spawn_pos.z, entity_id))

    return entity_id
end

--------------------------------------------------------------------------------
-- despawn_player
--------------------------------------------------------------------------------

--- @param client_id number
function M.despawn_player(client_id)
    -- Get all entities owned by this client
    local net_ids = NetGame2.get_net_ids_for_client(client_id)

    print(string.format("[PLAYER_CONTROLLER2] Despawning %d entities for client %s",
        #net_ids, client_id))

    -- Despawn each entity
    for _, net_id in ipairs(net_ids) do
        local entity_id = NetGame2.get_entity(net_id)
        if entity_id then
            despawn(entity_id)
            print(string.format("[PLAYER_CONTROLLER2] Despawned entity for net_id=%d", net_id))
        end

        -- If we despawned our own character, wait for respawn
        local my_client_id = NetGame2.get_my_client_id()
        if my_client_id and client_id == my_client_id then
            wait_my_character()
        end
    end

    -- Release the spawn claim
    SpawnSystem.release_spawn(client_id)
    print(string.format("[PLAYER_CONTROLLER2] Released spawn for client %s", client_id))
end

return M
