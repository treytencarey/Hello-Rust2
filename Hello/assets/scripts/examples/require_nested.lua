-- Example: Nested requires
-- Demonstrates a module that imports other modules

print("=== Nested Require Example ===")

spawn({
    Camera2d = {}
})

-- Import utils, which itself imports math_helpers and color_helpers
local utils = require("require_utils.lua")

print("Utils module loaded")
print("Testing nested module access:")
print("  utils.math.add(7, 3) =", utils.math.add(7, 3))
print("  utils.color.colors.red =", "r=" .. utils.color.colors.red.r)

-- Use combined utilities from utils module
print("Creating grid using nested modules...")

utils.create_grid(3, 4, 80, "cyan")

-- Create some individual squares
utils.create_square(-200, 150, 50, "red")
utils.create_square(-100, 150, 50, "green")
utils.create_square(0, 150, 50, "blue")
utils.create_square(100, 150, 50, "yellow")
utils.create_square(200, 150, 50, "magenta")

-- Add a title
spawn({
    Text2d = { text = "Nested Require Test" },
    TextFont = { font_size = 36 },
    TextColor = { color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0} },
    Transform = { 
        translation = {x = 0, y = 250, z = 0},
        rotation = {x = 0, y = 0, z = 0, w = 1},
        scale = {x = 1, y = 1, z = 1}
    }
})

print("✓ Nested require example complete")
