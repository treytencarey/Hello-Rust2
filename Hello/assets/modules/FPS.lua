-- FPS Display Module
-- Displays current FPS in the top-left corner using Text2d
-- Uses absolute time to remain accurate even if deferred by frame budget

local FPS = {}

-- Configuration
local UPDATE_INTERVAL = 0.5  -- Update FPS display every 0.5 seconds

--------------------------------------------------------------------------------
-- State (using resources for hot-reload safety)
--------------------------------------------------------------------------------

local state = define_resource("FPSState", {
    fps_entity = nil,
    last_update_time = 0.0,
    frame_count = 0,
    last_fps = 0,
    initialized = false
})

--------------------------------------------------------------------------------
-- Initialization
--------------------------------------------------------------------------------

-- Initialize the FPS display
function FPS.init()
    if state.initialized then return end

    -- Spawn Text2d entity for FPS display
    state.fps_entity = spawn({
        Text2d = {
            text = "FPS: --"
        },
        Transform = { 
            translation = {x = -500, y = 300, z = 0}, -- Adjusted to top-left-ish
            rotation = {x = 0, y = 0, z = 0, w = 1},
            scale = {x = 1, y = 1, z = 1}
        },
        -- Marker to prevent despawn during script cleanup if needed, 
        -- but usually we want it to persist or be recreated.
    }):id()
    
    state.initialized = true
    print("✓ FPS display initialized")
end

--------------------------------------------------------------------------------
-- Update System
--------------------------------------------------------------------------------

-- Update system - calculates and displays FPS
function FPS.update(world)
    if not state.fps_entity then 
        return 
    end
    
    -- world:delta_time() is now real-time aware (tracks time since last execution)
    -- This handles frame budget deferral correctly
    local delta = world:delta_time()
    state.last_update_time = state.last_update_time + delta -- Track real time since last FPS update
    state.frame_count = state.frame_count + 1
    
    -- Update display at the specified interval
    if state.last_update_time >= UPDATE_INTERVAL then
        -- Calculate FPS
        local fps = state.frame_count / state.last_update_time
        state.last_fps = math.floor(fps + 0.5)
        
        -- Update the Text2d component
        local entity = world:get_entity(state.fps_entity)
        if entity then
            entity:set({
                Text2d = { text = "FPS: " .. tostring(state.last_fps) }
            })
        else
            -- Entity lost, reset to re-init
            state.initialized = false
            state.fps_entity = nil
        end
        print("[FPS] FPS: " .. tostring(state.last_fps))
        
        -- Reset counters
        state.last_update_time = 0.0
        state.frame_count = 0
    end
end

-- Register the update system
function FPS.register()
    register_system("Update", function(world)
        FPS.update(world)
    end)
end

-- Convenience function to set up everything
function FPS.setup()
    FPS.init()
    FPS.register()
end

return FPS
