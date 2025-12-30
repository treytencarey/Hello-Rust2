-- VR Input Module
-- Provides access to VR controller button states and positions
--
-- Usage:
--   local VrInput = require("modules/vr_input.lua")
--   register_system("Update", function(world)
--       if VrInput.is_x_just_pressed(world) then
--           print("X button pressed!")
--           -- Spawn something in front of left controller
--           local pos = VrInput.get_spawn_position_in_front_of_left(world, 0.3)
--       end
--   end)

local VrInput = {}

--- Get the VrButtonState resource from the world
--- @param world userdata The world object from a system callback
--- @return table|nil The VrButtonState table or nil if not available
function VrInput.get_buttons(world)
    return world:get_resource("VrButtonState")
end

--- Get the VrControllerState resource from the world
--- @param world userdata The world object from a system callback
--- @return table|nil The VrControllerState table or nil if not available
function VrInput.get_controllers(world)
    return world:get_resource("VrControllerState")
end

--- Check if X button was just pressed this frame
--- @param world userdata The world object
--- @return boolean
function VrInput.is_x_just_pressed(world)
    local state = VrInput.get_buttons(world)
    return state and state.x_just_pressed or false
end

--- Check if X button is currently held
--- @param world userdata The world object
--- @return boolean
function VrInput.is_x_pressed(world)
    local state = VrInput.get_buttons(world)
    return state and state.x_pressed or false
end

--- Check if Y button was just pressed this frame
--- @param world userdata The world object
--- @return boolean
function VrInput.is_y_just_pressed(world)
    local state = VrInput.get_buttons(world)
    return state and state.y_just_pressed or false
end

--- Check if A button was just pressed this frame
--- @param world userdata The world object
--- @return boolean
function VrInput.is_a_just_pressed(world)
    local state = VrInput.get_buttons(world)
    return state and state.a_just_pressed or false
end

--- Check if B button was just pressed this frame
--- @param world userdata The world object
--- @return boolean
function VrInput.is_b_just_pressed(world)
    local state = VrInput.get_buttons(world)
    return state and state.b_just_pressed or false
end

--- Get left controller position
--- @param world userdata The world object
--- @return table|nil {x, y, z} or nil if not available
function VrInput.get_left_position(world)
    local ctrl = VrInput.get_controllers(world)
    return ctrl and ctrl.left_position or nil
end

--- Get left controller forward direction
--- @param world userdata The world object
--- @return table|nil {x, y, z} normalized or nil if not available
function VrInput.get_left_forward(world)
    local ctrl = VrInput.get_controllers(world)
    return ctrl and ctrl.left_forward or nil
end

--- Get right controller position
--- @param world userdata The world object
--- @return table|nil {x, y, z} or nil if not available
function VrInput.get_right_position(world)
    local ctrl = VrInput.get_controllers(world)
    return ctrl and ctrl.right_position or nil
end

--- Get right controller forward direction
--- @param world userdata The world object
--- @return table|nil {x, y, z} normalized or nil if not available
function VrInput.get_right_forward(world)
    local ctrl = VrInput.get_controllers(world)
    return ctrl and ctrl.right_forward or nil
end

--- Calculate spawn position in front of left controller
--- @param world userdata The world object
--- @param distance number Distance in meters in front of controller
--- @return table|nil {x, y, z} or nil if controller not available
function VrInput.get_spawn_position_in_front_of_left(world, distance)
    local pos = VrInput.get_left_position(world)
    local fwd = VrInput.get_left_forward(world)
    if pos and fwd then
        return {
            x = pos.x + fwd.x * distance,
            y = pos.y + fwd.y * distance,
            z = pos.z + fwd.z * distance
        }
    end
    return nil
end

--- Calculate spawn position in front of right controller
--- @param world userdata The world object
--- @param distance number Distance in meters in front of controller
--- @return table|nil {x, y, z} or nil if controller not available
function VrInput.get_spawn_position_in_front_of_right(world, distance)
    local pos = VrInput.get_right_position(world)
    local fwd = VrInput.get_right_forward(world)
    if pos and fwd then
        return {
            x = pos.x + fwd.x * distance,
            y = pos.y + fwd.y * distance,
            z = pos.z + fwd.z * distance
        }
    end
    return nil
end

--- Get HMD (head-mounted display) position
--- @param world userdata The world object
--- @return table|nil {x, y, z} or nil if not available
function VrInput.get_hmd_position(world)
    local ctrl = VrInput.get_controllers(world)
    return ctrl and ctrl.hmd_position or nil
end

--- Calculate spawn position in front of HMD
--- @param world userdata The world object
--- @param distance number Distance in meters in front of HMD
--- @return table|nil {x, y, z} or nil if HMD not available
function VrInput.get_spawn_position_in_front_of_hmd(world, distance)
    -- Get camera forward direction and position from Camera3d entity
    local cameras = world:query({"GlobalTransform", "Camera3d"}, nil)
    if not cameras or #cameras == 0 then return nil end
    
    local camera_transform = cameras[1]:get("GlobalTransform")
    if not camera_transform or not camera_transform._0 then return nil end
    
    local affine = camera_transform._0
    
    -- Get actual head position from the affine transform's translation
    local head_pos = affine.translation
    if not head_pos then
        -- Fallback to resource if affine translation not available
        head_pos = VrInput.get_hmd_position(world)
        if not head_pos then return nil end
    end
    
    -- Extract forward direction from affine transform matrix
    -- Column 2 is the Z axis (forward is -Z in Bevy)
    local matrix = affine.matrix3
    if matrix and matrix.z_axis then
        local fwd = {
            x = -matrix.z_axis.x,
            y = -matrix.z_axis.y,
            z = -matrix.z_axis.z
        }
        -- Normalize
        local len = math.sqrt(fwd.x*fwd.x + fwd.y*fwd.y + fwd.z*fwd.z)
        if len > 0.001 then
            fwd.x = fwd.x / len
            fwd.y = fwd.y / len
            fwd.z = fwd.z / len
        end
        
        -- Use only horizontal forward (project to XZ plane) so Y stays at head level
        local horiz_len = math.sqrt(fwd.x*fwd.x + fwd.z*fwd.z)
        local horiz_fwd = { x = 0, z = -1 }  -- Fallback if looking straight up/down
        if horiz_len > 0.001 then
            horiz_fwd.x = fwd.x / horiz_len
            horiz_fwd.z = fwd.z / horiz_len
        end
        
        return {
            x = head_pos.x + horiz_fwd.x * distance,
            y = head_pos.y,  -- Keep at actual head height from camera transform
            z = head_pos.z + horiz_fwd.z * distance
        }
    end
    
    -- Fallback: spawn directly in front (negative Z)
    return {
        x = head_pos.x,
        y = head_pos.y,
        z = head_pos.z - distance
    }
end


-- =============================================================================
-- Grip Button Accessors
-- =============================================================================

--- Check if left grip was just pressed this frame
--- @param world userdata The world object
--- @return boolean
function VrInput.is_left_grip_just_pressed(world)
    local state = VrInput.get_buttons(world)
    return state and state.left_grip_just_pressed or false
end

--- Check if left grip is currently held
--- @param world userdata The world object
--- @return boolean
function VrInput.is_left_grip_pressed(world)
    local state = VrInput.get_buttons(world)
    return state and state.left_grip_pressed or false
end

--- Check if left grip was just released this frame
--- @param world userdata The world object
--- @return boolean
function VrInput.is_left_grip_just_released(world)
    local state = VrInput.get_buttons(world)
    return state and state.left_grip_just_released or false
end

--- Get left grip analog value (0.0 to 1.0)
--- @param world userdata The world object
--- @return number
function VrInput.get_left_grip_value(world)
    local state = VrInput.get_buttons(world)
    return state and state.left_grip or 0.0
end

--- Check if right grip was just pressed this frame
--- @param world userdata The world object
--- @return boolean
function VrInput.is_right_grip_just_pressed(world)
    local state = VrInput.get_buttons(world)
    return state and state.right_grip_just_pressed or false
end

--- Check if right grip is currently held
--- @param world userdata The world object
--- @return boolean
function VrInput.is_right_grip_pressed(world)
    local state = VrInput.get_buttons(world)
    return state and state.right_grip_pressed or false
end

--- Check if right grip was just released this frame
--- @param world userdata The world object
--- @return boolean
function VrInput.is_right_grip_just_released(world)
    local state = VrInput.get_buttons(world)
    return state and state.right_grip_just_released or false
end

--- Get right grip analog value (0.0 to 1.0)
--- @param world userdata The world object
--- @return number
function VrInput.get_right_grip_value(world)
    local state = VrInput.get_buttons(world)
    return state and state.right_grip or 0.0
end

-- =============================================================================
-- Left Trigger Accessors
-- =============================================================================

--- Check if left trigger was just pressed this frame
--- @param world userdata The world object
--- @return boolean
function VrInput.is_left_trigger_just_pressed(world)
    local state = VrInput.get_buttons(world)
    return state and state.left_trigger_just_pressed or false
end

--- Check if left trigger is currently held
--- @param world userdata The world object
--- @return boolean
function VrInput.is_left_trigger_pressed(world)
    local state = VrInput.get_buttons(world)
    return state and state.left_trigger_pressed or false
end

--- Check if left trigger was just released this frame
--- @param world userdata The world object
--- @return boolean
function VrInput.is_left_trigger_just_released(world)
    local state = VrInput.get_buttons(world)
    return state and state.left_trigger_just_released or false
end

--- Get left trigger analog value (0.0 to 1.0)
--- @param world userdata The world object
--- @return number
function VrInput.get_left_trigger_value(world)
    local state = VrInput.get_buttons(world)
    return state and state.left_trigger or 0.0
end

--- Get right trigger analog value (0.0 to 1.0)
--- @param world userdata The world object
--- @return number
function VrInput.get_right_trigger_value(world)
    local state = VrInput.get_buttons(world)
    return state and state.right_trigger or 0.0
end

--- Check if right trigger was just pressed this frame
--- @param world userdata The world object
--- @return boolean
function VrInput.is_right_trigger_just_pressed(world)
    local state = VrInput.get_buttons(world)
    return state and state.right_trigger_just_pressed or false
end

--- Check if right trigger is currently held
--- @param world userdata The world object
--- @return boolean
function VrInput.is_right_trigger_pressed(world)
    local state = VrInput.get_buttons(world)
    return state and state.right_trigger_pressed or false
end

--- Check if right trigger was just released this frame
--- @param world userdata The world object
--- @return boolean
function VrInput.is_right_trigger_just_released(world)
    local state = VrInput.get_buttons(world)
    return state and state.right_trigger_just_released or false
end

-- =============================================================================
-- Controller Rotation Accessors
-- =============================================================================

--- Get left controller rotation (quaternion)
--- @param world userdata The world object
--- @return table|nil {x, y, z, w} quaternion or nil if not available
function VrInput.get_left_rotation(world)
    local ctrl = VrInput.get_controllers(world)
    return ctrl and ctrl.left_rotation or nil
end

--- Get right controller rotation (quaternion)
--- @param world userdata The world object
--- @return table|nil {x, y, z, w} quaternion or nil if not available
function VrInput.get_right_rotation(world)
    local ctrl = VrInput.get_controllers(world)
    return ctrl and ctrl.right_rotation or nil
end

return VrInput

