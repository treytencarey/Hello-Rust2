-- VSync Control Module
-- Disable VSync to allow higher than 60 FPS
--
-- Usage:
--   local VSync = require("modules/VSync.lua")
--   VSync.disable()  -- Call once at startup

local VSync = {}

-- Internal state
local _configured = false

-- Disable VSync (set present_mode to AutoNoVsync)
function VSync.disable()
    if _configured then return end
    
    -- Register a one-time system to set VSync
    register_system("Update", function(world)
        if _configured then return end
        
        local windows = world:query({"Window"}, nil)
        if #windows > 0 then
            windows[1]:set({
                Window = {
                    present_mode = "AutoNoVsync"
                }
            })
            print("✓ VSync disabled - uncapped FPS")
            _configured = true
        end
    end)
end

-- Enable VSync (set present_mode to Fifo)
function VSync.enable()
    if _configured then return end
    
    register_system("Update", function(world)
        if _configured then return end
        
        local windows = world:query({"Window"}, nil)
        if #windows > 0 then
            windows[1]:set({
                Window = {
                    present_mode = "Fifo"
                }
            })
            print("✓ VSync enabled - capped at monitor refresh rate")
            _configured = true
        end
    end)
end

return VSync
