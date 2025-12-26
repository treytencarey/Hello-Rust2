-- Asset Browser Example
-- Shows how to use the FileBrowser module for managing assets
--
-- Usage: Run this script to display the asset browser panel

local FileBrowser = require("scripts/ui/file_browser.lua")

-- Spawn a 2D camera (required for UI rendering)
spawn({
    Camera = {},
    Camera2d = {},
})

-- Create and show the file browser
local browser = FileBrowser.new()
browser:show()

-- Register update system to handle events
register_system("Update", function(world)
    browser:update(world)
end)

print("Asset Browser loaded - panel visible on left side")

