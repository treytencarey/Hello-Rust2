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

return VrInput
