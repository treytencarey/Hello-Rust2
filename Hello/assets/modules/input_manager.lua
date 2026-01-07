-- Input Manager Module
-- Abstracts input with configurable bindings and VR controller simulation
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
    forward = {"W"},
    backward = {"S"},
    left = {"A"},
    right = {"D"},
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
-- Input Queries
--------------------------------------------------------------------------------

--- Check if a specific key is pressed (keyboard or VR simulated)
function InputManager.is_key_pressed(world, key)
    -- Check VR simulated keys first
    if vr_simulated_keys[key] then
        return true
    end
    
    -- Check actual keyboard
    local ok, pressed = pcall(function()
        return world:call_resource_method("ButtonInput<KeyCode>", "pressed", key)
    end)
    
    return ok and pressed
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
    
    local forward = (forward_pressed and 1 or 0) - (backward_pressed and 1 or 0)
    local right = (right_pressed and 1 or 0) - (left_pressed and 1 or 0)
    
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

print("[INPUT_MANAGER] Module loaded")

return InputManager
