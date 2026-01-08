-- Input Manager Module
-- Abstracts input with configurable bindings and VR controller simulation
-- Uses Bevy's ButtonInput<KeyCode> resource for keyboard state
--
-- Usage:
--   local InputManager = require("modules/input_manager.lua")
--   local movement = InputManager.get_movement_input(world)
--   if movement.forward > 0 then ... end
--
-- VR Integration:
--   InputManager.simulate_key_down("W")  -- Called from VR controller handler
--   InputManager.simulate_key_up("W")

local InputManager = {}

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

-- Default key bindings (can be overridden)
InputManager.bindings = {
    forward = {"KeyW"},
    backward = {"KeyS"},
    left = {"KeyA"},
    right = {"KeyD"},
    jump = {"Space"},
    sprint = {"ShiftLeft"},
}

--------------------------------------------------------------------------------
-- VR Simulation State
--------------------------------------------------------------------------------

-- VR controllers simulate key presses here
local vr_simulated_keys = {}

--- Simulate a key being pressed (for VR controller bridge)
function InputManager.simulate_key_down(key)
    vr_simulated_keys[key] = true
end

--- Simulate a key being released (for VR controller bridge)
function InputManager.simulate_key_up(key)
    vr_simulated_keys[key] = nil
end

--- Clear all simulated keys
function InputManager.clear_simulated()
    vr_simulated_keys = {}
end

--------------------------------------------------------------------------------
-- Input Queries (uses ButtonInput<KeyCode> resource)
--------------------------------------------------------------------------------

--- Check if a specific key is pressed (keyboard or VR simulated)
local warned_no_input = false
function InputManager.is_key_pressed(world, key)
    -- Check VR simulated keys first
    if vr_simulated_keys[key] then
        return true
    end
    
    -- Get ButtonInput<KeyCode> resource via reflection
    local input = world:get_resource("ButtonInput<KeyCode>")
    if not input then
        if not warned_no_input then
            print("[INPUT] Warning: ButtonInput<KeyCode> not found - keyboard input unavailable")
            warned_no_input = true
        end
        return false
    end
    
    -- Check if key is in the 'pressed' set
    -- ButtonInput stores pressed keys in a HashSet field
    if input.pressed then
        for _, pressed_key in ipairs(input.pressed) do
            for key_code, _ in pairs(pressed_key) do
                if key_code == key then
                    return true
                end
            end
        end
    end
    return false
end

--- Check if a key was just pressed this frame
function InputManager.is_key_just_pressed(world, key)
    local input = world:get_resource("ButtonInput<KeyCode>")
    if not input or not input.just_pressed then return false end
    
    for _, pressed_key in ipairs(input.just_pressed) do
        if pressed_key == key then
            return true
        end
    end
    return false
end

--- Check if a key was just released this frame
function InputManager.is_key_just_released(world, key)
    local input = world:get_resource("ButtonInput<KeyCode>")
    if not input or not input.just_released then return false end
    
    for _, released_key in ipairs(input.just_released) do
        if released_key == key then
            return true
        end
    end
    return false
end

--- Check if an action is active (any bound key pressed)
function InputManager.is_action_pressed(world, action)
    local keys = InputManager.bindings[action]
    if not keys then return false end
    
    for _, key in ipairs(keys) do
        if InputManager.is_key_pressed(world, key) then
            return true
        end
    end
    return false
end

--- Get movement input as a normalized vector
--- @return table { forward = -1..1, right = -1..1, jump = bool, sprint = bool }
function InputManager.get_movement_input(world)
    local forward_pressed = InputManager.is_action_pressed(world, "forward")
    local backward_pressed = InputManager.is_action_pressed(world, "backward")
    local left_pressed = InputManager.is_action_pressed(world, "left")
    local right_pressed = InputManager.is_action_pressed(world, "right")
    
    local forward = (backward_pressed and 1 or 0) - (forward_pressed and 1 or 0)
    local right = (left_pressed and 1 or 0) - (right_pressed and 1 or 0)
    
    return {
        forward = forward,
        right = right,
        jump = InputManager.is_action_pressed(world, "jump"),
        sprint = InputManager.is_action_pressed(world, "sprint"),
    }
end

--------------------------------------------------------------------------------
-- Binding Management
--------------------------------------------------------------------------------

--- Set a custom binding for an action
function InputManager.set_binding(action, keys)
    InputManager.bindings[action] = keys
end

--- Add a key to an action's binding
function InputManager.add_binding(action, key)
    if not InputManager.bindings[action] then
        InputManager.bindings[action] = {}
    end
    table.insert(InputManager.bindings[action], key)
end

--- Get current bindings for an action
function InputManager.get_binding(action)
    return InputManager.bindings[action] or {}
end

print("[INPUT_MANAGER] Module loaded (using ButtonInput<KeyCode>)")

return InputManager
