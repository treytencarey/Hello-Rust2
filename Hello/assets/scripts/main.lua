-- Hello Game-- Main entry point for Hello
-- This is your game's main Lua script

require("scripts/examples/sidebar_example.lua")

-- Disable VSync for higher FPS
local VSync = require("modules/VSync.lua")
VSync.disable()
