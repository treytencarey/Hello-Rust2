-- Conflux3 Client Main Entry Point
-- Entry point for client-side game using net3 modules
-- Just requiring modules registers their systems automatically

local NetGame3 = require("modules/net_game3.lua", { instanced = true })
local NetSync3 = require("modules/net3/init.lua")
local PlayerController3 = require("modules/shared/player_controller3.lua")
local CameraController = require("modules/camera3/controller.lua")

print("[CONFLUX3_CLIENT] Starting...")

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

local config = define_resource("Conflux3ClientConfig", {
    server_ip = "127.0.0.1",
    server_port = 5000,
})

--------------------------------------------------------------------------------
-- Parse command line arguments
--------------------------------------------------------------------------------

local args = get_args()
for i, arg in ipairs(args) do
    if arg == "--server" and args[i+1] then
        config.server_ip = args[i+1]
    elseif arg == "--port" and args[i+1] then
        config.server_port = tonumber(args[i+1])
    end
end

--------------------------------------------------------------------------------
-- Initialize
--------------------------------------------------------------------------------

-- Callback when connected to server
local function on_connected(client_id, world)
    print(string.format("[CONFLUX3_CLIENT] Connected as client %d", client_id))
    
    -- Enable player controller (systems are already registered, just enable them)
    PlayerController3.enable()
end

-- Join the server
NetGame3.join(config.server_ip, config.server_port, {
    on_connected = on_connected,
})

print("[CONFLUX3_CLIENT] Waiting for connection...")
