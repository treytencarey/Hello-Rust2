-- Camera3 Controller Module
-- Resource-based camera controller with hot-reload safe state
--
-- Usage:
--   local CameraController = require("modules/camera3/controller.lua")
--   CameraController.attach(world, target_entity_id)
--   CameraController.update_system(world)  -- or use register_system

local ThirdPerson = require("modules/camera3/third_person.lua")

local CameraController = {}

--------------------------------------------------------------------------------
-- State (using resources for hot-reload safety)
--------------------------------------------------------------------------------

local function get_state()
    return define_resource("CameraControllerState", {
        camera_entity_id = nil,
        target_entity_id = nil,
        mode = "third_person",  -- "third_person" | "first_person"
        input_mode = "game",  -- "game" | "ui"
        
        -- Camera angles
        yaw = 0,
        pitch = 0,
        
        -- Distance (third person)
        distance = 4.0,
        target_distance = 4.0,
        min_distance = 2.0,
        max_distance = 50.0,
        
        -- Sensitivity
        mouse_sensitivity = 0.003,
        zoom_speed = 2.0,
        
        -- Smoothing
        position_lerp = 100.0,
        rotation_lerp = 100.0,
        zoom_lerp = 20.0,
        
        -- Offset from target
        offset = { x = 1.0, y = 1.2, z = 0 },
        
        -- Input prediction
        predictions = {},
        current_sequence = 0,
        last_sent_input = nil,
        
        -- Flags
        enabled = true,
        attached = false,
    })
end

--- Get prediction state resource
local function get_prediction_state()
    return define_resource("CameraPredictionState", {
        predictions = {},      -- sequence -> { input, position, rotation }
        current_sequence = 0,
        last_acked_sequence = nil,
        server_state = nil,
    })
end

--------------------------------------------------------------------------------
-- Initialization
--------------------------------------------------------------------------------

--- Create or get the camera entity
--- @param world userdata
--- @return number The camera entity ID
function CameraController.ensure_camera(world)
    local state = get_state()
    
    if state.camera_entity_id then
        local entity = world:get_entity(state.camera_entity_id)
        if entity then
            return state.camera_entity_id
        end
    end
    
    -- Create camera entity
    local camera_id = spawn({
        Camera3d = {},
        Transform = {
            translation = { x = 0, y = 5, z = 10 },
            rotation = { x = 0, y = 0, z = 0, w = 1 },
            scale = { x = 1, y = 1, z = 1 },
        },
    }):id()
    
    state.camera_entity_id = camera_id
    print(string.format("[CAMERA3] Created camera entity %d", camera_id))
    
    return camera_id
end

--- Attach camera to follow a target entity
--- @param world userdata
--- @param target_entity_id number
function CameraController.attach(world, target_entity_id)
    local state = get_state()
    
    local camera_id = CameraController.ensure_camera(world)
    state.target_entity_id = target_entity_id
    state.attached = true

    CameraController.set_input_mode(world, state.input_mode)
    
    -- Initialize camera state from geometry module
    local init_state = ThirdPerson.get_initial_state(world, target_entity_id)
    if init_state then
        state.yaw = init_state.yaw or state.yaw
        state.pitch = init_state.pitch or state.pitch
    end
    
    -- Position camera immediately to avoid jump
    local target_entity = world:get_entity(target_entity_id)
    if target_entity then
        local target_transform = target_entity:get("Transform")
        if target_transform then
            local pos, rot = ThirdPerson.calculate(
                target_transform.translation,
                state.yaw,
                state.pitch,
                state.distance,
                state.offset
            )
            
            local camera = world:get_entity(camera_id)
            if camera then
                camera:set({
                    Transform = {
                        translation = pos,
                        rotation = rot,
                        scale = { x = 1, y = 1, z = 1 }
                    }
                })
            end
        end
    end
    
    print(string.format("[CAMERA3] Attached to target entity %d (yaw=%.2f)", target_entity_id, state.yaw))
end

--- Detach camera from target
function CameraController.detach()
    local state = get_state()
    state.target_entity_id = nil
    state.attached = false
    
    print("[CAMERA3] Detached from target")
end

--------------------------------------------------------------------------------
-- Input Handling
--------------------------------------------------------------------------------

--- Set input mode
--- @param world userdata
--- @param mode string "game" | "ui"
function CameraController.set_input_mode(world, mode)
    local state = get_state()
    if not state.enabled then return end
    
    state.input_mode = mode

    local windows = world:query({"Window", "CursorOptions"})
    if #windows == 0 then return end
    
    local window = windows[1]
    if state.input_mode == "game" then
        window:set({
            CursorOptions = {
                visible = false,
                grab_mode = { Locked = true }
            }
        })
    else
        window:set({
            CursorOptions = {
                visible = true,
                grab_mode = { None = true }
            }
        })
    end
end

--- Handle mouse movement for camera rotation
--- @param world userdata
function CameraController.handle_mouse_move(world)
    local state = get_state()
    if not state.enabled then return end
    
    local motion_events = world:read_events("MouseMotion")
    for _, event in ipairs(motion_events) do
        if event.delta then
            local dx, dy = event.delta[1], event.delta[2]
            state.yaw = state.yaw - dx * state.mouse_sensitivity
            state.pitch = math.max(-1.4, math.min(1.4, state.pitch + dy * state.mouse_sensitivity))
        end
    end
end

--- Handle scroll for zoom
--- @param world userdata
function CameraController.handle_scroll(world)
    local state = get_state()
    if not state.enabled or state.mode ~= "third_person" then return end
    
    local wheel_events = world:read_events("MouseWheel")
    for _, event in ipairs(wheel_events) do
        state.target_distance = math.max(
            state.min_distance,
            math.min(state.max_distance, state.target_distance - event.y * state.zoom_speed)
        )
    end
end

--------------------------------------------------------------------------------
-- Update System
--------------------------------------------------------------------------------

--- Camera update system - handles positioning and smooth following
--- @param world userdata
function CameraController.update_system(world)
    local state = get_state()
    local dt = world:delta_time()

    if not state.attached or not state.target_entity_id then
        return
    end
    
    local target_entity = world:get_entity(state.target_entity_id)
    if not target_entity then
        state.attached = false
        return
    end
    
    local camera_entity = world:get_entity(state.camera_entity_id)
    if not camera_entity then
        return
    end
    
    local target_transform = target_entity:get("Transform")
    local camera_transform = camera_entity:get("Transform")
    
    if not target_transform or not camera_transform then
        return
    end

    CameraController.handle_mouse_move(world)
    CameraController.handle_scroll(world)
    
    -- Smoothly interpolate distance
    state.distance = state.distance + (state.target_distance - state.distance) * math.min(1.0, state.zoom_lerp * dt)
    
    -- Pivot is the character's base position
    -- The offset will be applied locally in calculate()
    local pivot = target_transform.translation
    
    local desired_pos, desired_rot
    
    if state.mode == "third_person" then
        desired_pos, desired_rot = ThirdPerson.calculate(
            pivot,
            state.yaw,
            state.pitch,
            state.distance,
            state.offset
        )
    else
        -- First person - center at head height but ignore X/Z offset
        local head_pos = {
            x = pivot.x,
            y = pivot.y + state.offset.y,
            z = pivot.z
        }
        desired_pos = head_pos
        desired_rot = ThirdPerson.yaw_pitch_to_quat(state.yaw, state.pitch)
    end
    
    -- Smooth interpolation
    local t = math.min(1.0, state.position_lerp * dt)
    local new_pos = {
        x = camera_transform.translation.x + (desired_pos.x - camera_transform.translation.x) * t,
        y = camera_transform.translation.y + (desired_pos.y - camera_transform.translation.y) * t,
        z = camera_transform.translation.z + (desired_pos.z - camera_transform.translation.z) * t,
    }
    
    -- Smoothly interpolate rotation
    local new_rot = ThirdPerson.slerp(camera_transform.rotation, desired_rot, math.min(1.0, state.rotation_lerp * dt))
    
    -- Apply to camera
    camera_entity:set({
        Transform = {
            translation = new_pos,
            rotation = new_rot,
            scale = camera_transform.scale,
        }
    })
end

--------------------------------------------------------------------------------
-- Registration (self-registering at module load)
--------------------------------------------------------------------------------

-- Register camera update system (called once at module load)
register_system("Update", function(world)
    CameraController.update_system(world)
end)

print("[CAMERA3] System registered")

--------------------------------------------------------------------------------
-- Accessors
--------------------------------------------------------------------------------

function CameraController.get_yaw() return get_state().yaw end
function CameraController.get_pitch() return get_state().pitch end
function CameraController.get_distance() return get_state().distance end
function CameraController.get_camera_id() return get_state().camera_entity_id end
function CameraController.get_target_id() return get_state().target_entity_id end
function CameraController.is_attached() return get_state().attached end

function CameraController.set_enabled(enabled)
    get_state().enabled = enabled
end

function CameraController.set_mode(mode)
    get_state().mode = mode
end

return CameraController
