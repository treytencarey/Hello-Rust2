-- Hello Game-- Main entry point for Hello
-- This is your game's main Lua script

-- VR UI support (auto-detects and wraps UI nodes as VR panels)
require("modules/vr_ui.lua")

local FPS = require("modules/fps.lua")
FPS.setup()

require("scripts/examples/sidebar_example.lua")

-- Disable VSync for higher FPS
local VSync = require("modules/VSync.lua")
VSync.disable()