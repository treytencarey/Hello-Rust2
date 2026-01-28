-- Profiler View Example
-- Shows how to use the Profiler module as a standalone panel
--
-- Usage: Run this script to display the profiler panel
-- Note: You can also use F3 to toggle the profiler globally

local Profiler = require("modules/profiler/init.lua")

-- Spawn a 2D camera (required for UI rendering)
spawn({
    Camera = {},
    Camera2d = {},
})

-- Create and show the profiler
-- We can use the legacy toggle/show_ui or the new instance-based API
-- For this standalone view, we'll use the new API to test it
local profiler = Profiler.new()
profiler:show() -- Defaults to top-right absolute position

-- Note: Profiler registers its own Update system, so we don't need to register one here
-- unlike FileBrowser which requires the user to drive the update loop.

print("Profiler View loaded - visible on top-right")
print("Press F3 to toggle global visibility")
