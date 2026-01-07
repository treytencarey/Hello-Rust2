-- Network Role Module
-- Detects network role from CLI args (--network server|client|both)
--
-- Usage:
--   local NetRole = require("modules/net_role.lua")
--   if NetRole.is_server() then ... end
--   if NetRole.is_client() then ... end

local NetRole = {}

--------------------------------------------------------------------------------
-- Role Detection
--------------------------------------------------------------------------------

local role = "offline"  -- Default if no --network arg

-- Parse CLI arguments
local args = get_args()
for i, arg in ipairs(args) do
    if arg == "--network" and args[i + 1] then
        role = args[i + 1]  -- "server", "client", or "both"
        break
    end
end

--- Check if running as server (server or both)
function NetRole.is_server()
    return role == "server" or role == "both"
end

--- Check if running as client (client or both)
function NetRole.is_client()
    return role == "client" or role == "both"
end

--- Check if running offline (no networking)
function NetRole.is_offline()
    return role == "offline"
end

--- Get the raw role string
function NetRole.get_role()
    return role
end

print(string.format("[NET_ROLE] Detected role: %s (server=%s, client=%s)", 
    role, 
    tostring(NetRole.is_server()), 
    tostring(NetRole.is_client())))

return NetRole
