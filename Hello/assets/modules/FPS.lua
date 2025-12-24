-- FPS Display Module
-- Displays current FPS in the top-left corner using Text2d

local FPS = {}

-- Configuration
local UPDATE_INTERVAL = 0.5  -- Update FPS display every 0.5 seconds
local font_size = 24.0

-- State
local fps_entity = nil
local time_accumulator = 0.0
local frame_count = 0
local last_fps = 0

-- Initialize the FPS display
function FPS.init()
    -- Spawn Text2d entity for FPS display
    fps_entity = spawn({
        Text2d = {
            text = "FPS: --"
        },
        Transform = { 
            translation = {x = 0, y = 100, z = 0},
            rotation = {x = 0, y = 0, z = 0, w = 1},
            scale = {x = 1, y = 1, z = 1}
        }
    })
    
    print("✓ FPS display initialized")
end

-- Update system - calculates and displays FPS
function FPS.update(world)
    if not fps_entity then 
        return 
    end
    
    local delta = world:delta_time()
    time_accumulator = time_accumulator + delta
    frame_count = frame_count + 1
    
    -- Update display at the specified interval
    if time_accumulator >= UPDATE_INTERVAL then
        -- Calculate FPS
        local fps = frame_count / time_accumulator
        last_fps = math.floor(fps + 0.5)  -- Round to nearest integer
        
        -- Update the Text2d component directly
        fps_entity:set({
            Text2d = { text = "FPS: " .. tostring(last_fps) }
        })
        
        -- Reset counters
        time_accumulator = 0.0
        frame_count = 0
    end
end

-- Register the update system
function FPS.register()
    register_system("Update", FPS.update)
end

-- Convenience function to set up everything
function FPS.setup()
    FPS.init()
    FPS.register()
end

return FPS
