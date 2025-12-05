-- Example: Synchronous require() usage
-- Demonstrates importing modules and using them immediately

print("=== Synchronous Require Example ===")

-- Load math helpers module
local math_helpers = require("require_math_helpers.lua")
local color_helpers = require("require_color_helpers.lua")

print("Testing math_helpers module:")
print("  add(5, 3) =", math_helpers.add(5, 3))
print("  multiply(4, 7) =", math_helpers.multiply(4, 7))
print("  average({10, 20, 30, 40}) =", math_helpers.average({10, 20, 30, 40}))
print("  clamp(15, 0, 10) =", math_helpers.clamp(15, 0, 10))

-- Spawn a text entity to display the result
local result = math_helpers.add(100, 200)

spawn({
    Camera2d = {}
})

spawn({
    Text2d = { text = "Sync require() test: " .. result },
    TextFont = { font_size = 32 },
    TextColor = { color = {r = 0.2, g = 0.8, b = 0.3, a = 1.0} },
    Transform = { 
        translation = {x = 0, y = 50, z = 0},
        rotation = {x = 0, y = 0, z = 0, w = 1},
        scale = {x = 1, y = 1, z = 1}
    }
})

-- Spawn a square that uses calculated values
local size = math_helpers.clamp(100, 50, 150)

spawn({
    Sprite = { 
        color = {r = 0.3, g = 0.7, b = 0.9, a = 1.0},
        custom_size = {x = size, y = size}
    },
    Transform = { 
        translation = {x = 0, y = -50, z = 0},
        rotation = {x = 0, y = 0, z = 0, w = 1},
        scale = {x = 1, y = 1, z = 1}
    }
})

print("✓ Synchronous require example complete")
