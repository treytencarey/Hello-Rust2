-- Conflux Client Main Entry Point
-- Entry point for client-side game using net modules
-- Just requiring modules registers their systems automatically

local NetGame = require("modules/net/net_game.lua", { instanced = true })
local PlayerController = require("modules/shared/player_controller.lua")

print("[CONFLUX_CLIENT] Starting...")

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

local config = define_resource("ConfluxClientConfig", {
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
    print(string.format("[CONFLUX_CLIENT] Connected as client %d", client_id))
end

-- Join the server
NetGame.join(config.server_ip, config.server_port, {
    on_connected = on_connected,
})

print("[CONFLUX_CLIENT] Waiting for connection...")
