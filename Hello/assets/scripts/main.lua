-- Hello Game-- Main entry point for Hello
-- This is your game's main Lua script

-- Load FPS counter
local FPS = require("scripts/examples/ufbx.lua")

-- Disable VSync for higher FPS
local VSync = require("modules/VSync.lua")
VSync.disable()
