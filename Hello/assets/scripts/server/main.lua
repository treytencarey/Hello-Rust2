-- Hello Server Main Script
-- This runs when the application starts in server mode
-- The server hosts assets but doesn't render a game UI

print("=== Server Mode Started ===")

-- require("scripts/server/Conflux2/main.lua")

-- Test Net - Conflux
require("scripts/server/Conflux/main.lua")

-- local FPS = require("modules/FPS.lua")
-- FPS.setup()

-- local VSync = require("modules/VSync.lua")
-- VSync.disable()

-- print("Listening for client connections...")
-- print("Files will be served from assets/ directory")
-- print("File changes will be broadcast to subscribed clients")

-- -- Server-specific logic can go here
-- -- For example: game state management, authoritative game logic, etc.

-- -- Register server update system
-- register_system("Update", function(world)
--     NetServer.update(world)
-- end)

-- Note: Most server functionality is handled by Rust systems:
-- - handle_asset_requests_global: Serves file requests
-- - broadcast_file_updates: Pushes file changes to clients
-- - cleanup_disconnected_clients: Cleans up on disconnect
