-- Spawn text entities using component-based API for 2D world text

print("Text spawn script loaded!")

-- Spawn entity with Text2d for 2D world rendering
spawn({
    Text = { text = "Hello from Lua!" },
    TextFont = { font_size = 64 },
    TextColor = { color = {r = 1.0, g = 0.8, b = 0.2, a = 1.0} },
    TextLayout = {},
    Transform = { translation = {x = 0, y = 100, z = 0} }
})

spawn({
    Text = { text = "Component-Based!" },
    TextFont = { font_size = 48 },
    TextColor = { color = {r = 0.2, g = 0.8, b = 1.0, a = 1.0} },
    TextLayout = {},
    Transform = { translation = {x = 0, y = 0, z = 0} }
})

spawn({
    Text = { text = "Zero Definitions!" },
    TextFont = { font_size = 32 },
    TextColor = { color = {r = 0.5, g = 1.0, b = 0.5, a = 1.0} },
    TextLayout = {},
    Transform = { translation = {x = 0, y = -100, z = 0} }
})

print("Text entities queued successfully!")
