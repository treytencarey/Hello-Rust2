-- Player Controller 3 Module
-- ECS-pure player controller with resource-based state
-- Uses query-based camera attachment (no repeated system registration)
--
-- Usage:
--   local PlayerController3 = require("modules/shared/player_controller3.lua")
--   PlayerController3.register_systems()

local NetSync3 = require("modules/net3/init.lua")
local CameraController = require("modules/camera3/controller.lua")
local Movement = require("modules/shared/movement.lua")

local PlayerController3 = {}

--------------------------------------------------------------------------------
-- State (using resources for hot-reload safety)
--------------------------------------------------------------------------------

local state = define_resource("PlayerController3State", {
    attached_entities = {},   -- entity_id -> true (entities with camera attached)
    my_player_entity = nil,   -- Our controlled player entity ID
    movement_speed = 5.0,
    jump_force = 8.0,
    current_sequence = 0,     -- Input sequence number
    last_input = nil,         -- LAST SENT input to server (for optimization)
    rotation_mode = "face_movement", -- "face_camera" | "face_movement"
    enabled = false,
})

--------------------------------------------------------------------------------
-- Camera Attachment System
--------------------------------------------------------------------------------

--- System that attaches camera to owned player entities
--- Uses ECS query instead of callbacks - survives hot-reload
local function camera_attachment_system(world)
    if not state.enabled then
        return
    end
    
    local my_client_id = NetSync3.get_my_client_id()
    if not my_client_id then
        return
    end
    
    -- Query for entities with NetworkSync and PlayerState
    local entities = world:query({ NetSync3.MARKER, "PlayerState", "ScriptOwned" })
    
    for _, entity in ipairs(entities) do
        local entity_id = entity:id()
        
        -- Skip if already processed
        if state.attached_entities[entity_id] then
            goto continue_attachment
        end
        
        -- Check if owned by us
        local sync = entity:get(NetSync3.MARKER)
        if sync.owner_client == my_client_id then
            print(string.format("[PLAYER3] Found my player entity: %d", entity_id))
            
            -- Attach camera
            CameraController.attach(world, entity_id)
            state.my_player_entity = entity_id
            state.attached_entities[entity_id] = true
            
            -- Set authority to local so we can send input
            -- BUT: remove Transform from sync_components so we don't send predicted transforms back!
            local sync = entity:get(NetSync3.MARKER)
            local new_sync_comps = {}
            for k, v in pairs(sync.sync_components or {}) do
                if k ~= "Transform" then
                    new_sync_comps[k] = v
                end
            end
            
            entity:patch({
                [NetSync3.MARKER] = {
                    authority = "local",
                    sync_components = new_sync_comps
                }
            })
            
            -- Ensure PredictionState component exists
            if not entity:has(NetSync3.PREDICTION) then
                entity:patch({
                    [NetSync3.PREDICTION] = {
                        predictions = {},
                        last_acked_sequence = 0,
                        snap_threshold = 2.0
                    }
                })
            end
        end
        
        ::continue_attachment::
    end
end

--------------------------------------------------------------------------------
-- Input Helpers
--------------------------------------------------------------------------------

local function is_key_pressed(world, key_code)
    local input = world:get_resource("ButtonInput<KeyCode>")
    if input and input.pressed then
        for _, pressed_key in ipairs(input.pressed) do
            for k, _ in pairs(pressed_key) do
                if k == key_code then
                    return true
                end
            end
        end
    end
    return false
end

local function is_key_just_pressed(world, key_code)
    local input = world:get_resource("ButtonInput<KeyCode>")
    if input and input.just_pressed then
        for _, pressed_key in ipairs(input.just_pressed) do
            for k, _ in pairs(pressed_key) do
                if k == key_code then
                    return true
                end
            end
        end
    end
    return false
end

--------------------------------------------------------------------------------
-- Movement System
--------------------------------------------------------------------------------

--- System that handles player movement input
local function movement_system(world)
    if not state.enabled or not state.my_player_entity then
        return
    end
    
    local entity = world:get_entity(state.my_player_entity)
    if not entity then
        -- Entity despawned, reset state
        state.my_player_entity = nil
        state.attached_entities = {}
        return
    end
    
    -- Capture input
    local move_x = 0
    local move_z = 0
    if is_key_pressed(world, "KeyW") or is_key_pressed(world, "ArrowUp") then move_z = move_z + 1 end
    if is_key_pressed(world, "KeyS") or is_key_pressed(world, "ArrowDown") then move_z = move_z - 1 end
    if is_key_pressed(world, "KeyD") or is_key_pressed(world, "ArrowRight") then move_x = move_x + 1 end
    if is_key_pressed(world, "KeyA") or is_key_pressed(world, "ArrowLeft") then move_x = move_x - 1 end
    
    local sprint = is_key_pressed(world, "ShiftLeft") or is_key_pressed(world, "ShiftRight")
    local jump = is_key_just_pressed(world, "Space")
    local yaw = CameraController.get_yaw()
    local is_moving = (move_x ~= 0 or move_z ~= 0 or jump)
    
    -- Mode toggle (KeyV)
    if is_key_just_pressed(world, "KeyV") then
        if state.rotation_mode == "face_movement" then
            state.rotation_mode = "face_camera"
        else
            state.rotation_mode = "face_movement"
        end
        print("[PLAYER3] Rotation mode changed to: " .. state.rotation_mode)
    end
    
    -- Optimization: Only send if input changed
    local input_changed = false
    if not state.last_input then
        input_changed = true
    else
        local li = state.last_input
        if li.move_x ~= move_x or li.move_z ~= move_z or  
           li.sprint ~= sprint or li.jump ~= jump or
           li.rotation_mode ~= state.rotation_mode or
           (is_moving and math.abs(li.yaw - yaw) > 0.01) then -- Send if yaw changed by ~0.5 degrees, only while moving
            input_changed = true
        end
    end

    -- Sequence number still increments every frame for local prediction buffer
    state.current_sequence = state.current_sequence + 1
    local seq = state.current_sequence
    
    local input = {
        move_x = move_x,
        move_z = move_z,
        yaw = is_moving and yaw or (state.last_input and state.last_input.yaw or 0), -- Only send yaw if moving
        jump = jump,
        sprint = sprint,
        rotation_mode = state.rotation_mode,
        sequence = is_moving and seq or (state.last_input and state.last_input.sequence or 0), -- Only send sequence if moving
    }
    
    -- 1. Send input to server via PlayerInput component if it changed and if moving
    if input_changed then
        state.last_input = input
        entity:patch({ PlayerInput = input })
    end
    
    -- 2. Immediate local prediction (always happens every frame)
    local transform = entity:get("Transform")
    if transform then
        local dt = world:delta_time()
        local move_config = { rotation_mode = state.rotation_mode }
        local new_pos, new_rot = Movement.apply(world, transform, input, state.movement_speed, dt, true, move_config)
        
        -- Apply predicted transform locally, only if changed meaningfully (epsilon check)
        local eps = 0.0001
        local pos_changed = math.abs(new_pos.x - transform.translation.x) > eps or
                            math.abs(new_pos.y - transform.translation.y) > eps or
                            math.abs(new_pos.z - transform.translation.z) > eps
        
        local rot_changed = math.abs(new_rot.x - transform.rotation.x) > eps or
                            math.abs(new_rot.y - transform.rotation.y) > eps or
                            math.abs(new_rot.z - transform.rotation.z) > eps or
                            math.abs(new_rot.w - transform.rotation.w) > eps

        if pos_changed or rot_changed then
            entity:patch({
                Transform = {
                    translation = new_pos,
                    rotation = new_rot,
                }
            })
        end
        
        -- 3. Buffer prediction for reconciliation
        local pred_state = entity:get(NetSync3.PREDICTION) or { predictions = {} }
        pred_state.predictions[seq] = {
            input = input,
            position = new_pos,
            rotation = new_rot
        }
        entity:patch({ [NetSync3.PREDICTION] = pred_state })
    end
end

--------------------------------------------------------------------------------
-- Initialization (self-registering at module load)
--------------------------------------------------------------------------------

-- Register systems at module load
register_system("Update", camera_attachment_system)
register_system("Update", movement_system)

-- Systems are registered but disabled by default - call enable() when ready
print("[PLAYER3] Systems registered (disabled by default, call enable())")

--- Enable player controller (call this when player entity is ready)
function PlayerController3.enable()
    state.enabled = true
    print("[PLAYER3] Enabled")
end

--- Disable player controller
function PlayerController3.disable()
    state.enabled = false
    print("[PLAYER3] Disabled")
end

--- Get the controlled player entity ID
--- @return number|nil
function PlayerController3.get_my_player()
    return state.my_player_entity
end

--- Set movement speed
--- @param speed number
function PlayerController3.set_movement_speed(speed)
    state.movement_speed = speed
end

return PlayerController3
