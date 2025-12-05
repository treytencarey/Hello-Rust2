-- Example: Asynchronous require_async() usage
-- Demonstrates loading modules with callbacks

print("=== Asynchronous Require Example ===")

print("Before async load")

spawn({
    Camera2d = {}
})

-- Load math helpers asynchronously
require_async("require_math_helpers.lua", function(math_helpers)
    print("Callback: Module loaded!")
    print("  Testing in callback:")
    print("  add(10, 20) =", math_helpers.add(10, 20))
    print("  multiply(3, 7) =", math_helpers.multiply(3, 7))
    
    -- Spawn entity from within callback
    local result = math_helpers.multiply(50, 3)
    
    spawn({
        Text2d = { text = "Async callback result: " .. result },
        TextFont = { font_size = 28 },
        TextColor = { color = {r = 0.9, g = 0.5, b = 0.2, a = 1.0} },
        Transform = { 
            translation = {x = 0, y = 50, z = 0},
            rotation = {x = 0, y = 0, z = 0, w = 1},
            scale = {x = 1, y = 1, z = 1}
        }
    })

    require_async("require_color_helpers.lua", function(color_helpers)
        print("Callback: Color module loaded!")
        local mixed_color = color_helpers.mix(color_helpers.colors.red, color_helpers.colors.blue, 0.5)
        
        spawn({
            Sprite = { 
                color = mixed_color,
                custom_size = {x = 100, y = 100}
            },
            Transform = { 
                translation = {x = -150, y = 0, z = 0},
                rotation = {x = 0, y = 0, z = 0, w = 1},
                scale = {x = 1, y = 1, z = 1}
            }
        })
        
        print("✓ Color callback complete")
    end, { reload = true })

    print("✓ Callback complete")
end, { reload = true })

print("After async load (callback may run immediately)")

-- Spawn another entity to show execution continues
spawn({
    Sprite = { 
        color = {r = 0.8, g = 0.3, b = 0.5, a = 1.0},
        custom_size = {x = 80, y = 80}
    },
    Transform = { 
        translation = {x = 150, y = 0, z = 0},
        rotation = {x = 0, y = 0, z = 0, w = 1},
        scale = {x = 1, y = 1, z = 1}
    }
})

print("✓ Main script complete")
