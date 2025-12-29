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

