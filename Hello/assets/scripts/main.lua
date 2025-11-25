-- Hello Game Main Script
-- This is your game's main Lua script

-- Example: Spawn a simple text entity
spawn({
    Text = { text = "Hello from Lua!" },
    TextFont = { font_size = 64 },
    TextColor = { color = {r = 1.0, g = 0.8, b = 0.2, a = 1.0} },
    TextLayout = {},
    Transform = { translation = {x = 0, y = 100, z = 0} }
})

print("Hello text spawned")
