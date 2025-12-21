-- Hello Server Main Script
-- This runs when the application starts in server mode
-- The server hosts assets but doesn't render a game UI

print("=== Server Mode Started ===")
print("Listening for client connections...")
print("Files will be served from assets/ directory")
print("File changes will be broadcast to subscribed clients")

-- Server-specific logic can go here
-- For example: game state management, authoritative game logic, etc.

-- Note: Most server functionality is handled by Rust systems:
-- - handle_asset_requests_global: Serves file requests
-- - broadcast_file_updates: Pushes file changes to clients
-- - cleanup_disconnected_clients: Cleans up on disconnect
