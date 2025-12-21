-- Spawn sprite entities using component-based API

-- Spawn a simple sprite with color and animation timer
spawn({
    Sprite = { color = {r = 1.0, g = 0.0, b = 0.0, a = 1.0} },
    Transform = { 
        translation = {x = -200, y = 0, z = 0},
        scale = {x = 50.0, y = 50.0, z = 1.0}
    },
    Timer = { duration = 3.0, elapsed = 0.0 }
})

spawn({
    Sprite = { color = {r = 0.0, g = 1.0, b = 0.0, a = 1.0} },
    Transform = { 
        translation = {x = 0, y = 0, z = 0},
        scale = {x = 50.0, y = 50.0, z = 1.0}
    },
    Timer = { duration = 3.0, elapsed = 0.0 }
})

spawn({
    Sprite = { color = {r = 0.0, g = 0.0, b = 1.0, a = 1.0} },
    Transform = { 
        translation = {x = 200, y = 0, z = 0},
        scale = {x = 50.0, y = 50.0, z = 1.0}
    },
    Timer = { duration = 3.0, elapsed = 0.0 }
})

print("Sprite entities queued successfully!")

-- Global frame counter and time accumulator for animation system
frame_count = 0
elapsed_time = 0

-- Animation system defined purely in Lua
-- Now with actual color animation!
function animation_system(world)
    -- Get delta time
    local dt = world:delta_time()
    
    -- Update time
    elapsed_time = elapsed_time + dt
    frame_count = frame_count + 1
    
    -- Query for entities with Timer and Sprite components
    local entities = world:query({"Timer", "Sprite"}, nil)
    
    -- Animate each sprite
    for i, entity in ipairs(entities) do
        -- Calculate pulsing effect based on time
        local pulse = (math.sin(elapsed_time * 2.0) + 1.0) / 2.0  -- 0 to 1
        
        -- Create animated color - pulse between original color and white
        local base_colors = {
            {r = 1.0, g = 0.0, b = 0.0},  -- Red
            {r = 0.0, g = 1.0, b = 0.0},  -- Green
            {r = 0.0, g = 0.0, b = 1.0},  -- Blue
        }
        
        local base = base_colors[i]
        local animated_color = {
            r = base.r + (1.0 - base.r) * pulse * 0.5,
            g = base.g + (1.0 - base.g) * pulse * 0.5,
            b = base.b + (1.0 - base.b) * pulse * 0.5,
            a = 1.0
        }
        
        -- Update the sprite color!
        entity:set({ Sprite = { color = animated_color } })
    end
    
    -- Print status occasionally
    if frame_count % 120 == 0 then
        print(string.format("Animation running: %d sprites, time=%.2fs, pulse=%.2f", 
            #entities, elapsed_time, (math.sin(elapsed_time * 2.0) + 1.0) / 2.0))
    end
end

register_system("Update", animation_system)

print("Animation system registered!")
print("Sprites will pulse between their base colors and white!")
