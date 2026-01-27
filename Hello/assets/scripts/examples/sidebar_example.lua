-- Sidebar Example
-- Demonstrates the extensible sidebar menu system
--
-- Run this script to see the sidebar in action:
-- 1. Press Escape to toggle sidebar visibility
-- 2. Click icons to open panel scripts
-- 3. Press Escape to close panels (most recent first)

-- Spawn camera for UI
spawn({
    Camera2d = {},
})

-- Load the sidebar menu
local SidebarMenu = require("scripts/ui/sidebar_menu.lua")
local menu = SidebarMenu.new()

-- Spawn sidebar button entities
-- The sidebar menu will discover these via ECS queries

-- Files button - opens file browser
spawn({
    SidebarButton = {
        icon = "icons/files.png",
        title = "Files",
        script = "scripts/ui/file_browser.lua",
    }
})

-- Accounts button (placeholder)
spawn({
    SidebarButton = {
        icon = "icons/account.png", 
        title = "Account",
        script = "scripts/ui/file_browser1.lua",  -- Reusing for demo
    }
})

-- Settings button (placeholder)
spawn({
    SidebarButton = {
        icon = "icons/settings.png",
        title = "Settings",
        script = "scripts/ui/file_browser2.lua",  -- Reusing for demo
    }
})

print("Sidebar Example loaded - press Escape to toggle menu")
