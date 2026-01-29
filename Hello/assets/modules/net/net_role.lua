-- Network Role Module
-- Detects network role from CLI args (--network server|client|both)
-- Also tracks per-instance hosted/joined state for instanced modules
--
-- Usage:
--   local NetRole = require("modules/net/net_role.lua")
--   if NetRole.is_server() then ... end     -- True if hosting
--   if NetRole.is_client() then ... end     -- True if joined
--   if NetRole.can_host() then ... end      -- True if CLI allows hosting
--   if NetRole.can_join() then ... end      -- True if CLI allows joining

local NetRole = {}

--------------------------------------------------------------------------------
-- CLI Role Detection (static, based on --network arg)
--------------------------------------------------------------------------------

local cli_role = "offline"  -- Default if no --network arg

-- Parse CLI arguments
local args = get_args()
for i, arg in ipairs(args) do
    if arg == "--network" and args[i + 1] then
        cli_role = args[i + 1]  -- "server", "client", or "both"
        break
    end
end

--- Check if CLI allows hosting (--network server or --network both)
function NetRole.can_host()
    return cli_role == "server" or cli_role == "both"
end

--- Check if CLI allows joining (--network client or --network both)
function NetRole.can_join()
    return cli_role == "client" or cli_role == "both"
end

--- Check if running offline (no networking)
function NetRole.is_offline()
    return cli_role == "offline"
end

--- Get the raw CLI role string
function NetRole.get_cli_role()
    return cli_role
end

--------------------------------------------------------------------------------
-- Per-Instance State (module-local, isolated via instanced require)
--------------------------------------------------------------------------------

local hosted = false  -- True if this instance called NetGame.host()
local joined = false  -- True if this instance called NetGame.join()

--- Check if this instance is waiting to host or join
function NetRole.is_waiting()
    return not hosted and not joined
end

--- Check if this instance is acting as server (has hosted)
function NetRole.is_server()
    return hosted
end

--- Check if this instance is acting as client (has joined)
function NetRole.is_client()
    return joined
end

--- Mark this instance as hosting (called by NetGame.host())
function NetRole.set_hosted(value)
    hosted = value
    print(string.format("[NET_ROLE] Instance %s set hosted=%s", __LUA_STATE_ID__ or 0, tostring(value)))
end

--- Mark this instance as joined (called by NetGame.join())
function NetRole.set_joined(value)
    joined = value
    print(string.format("[NET_ROLE] Instance %s set joined=%s", __LUA_STATE_ID__ or 0, tostring(value)))
end

--- Get current instance state
function NetRole.get_state()
    return { hosted = hosted, joined = joined }
end

print(string.format("[NET_ROLE] Module loaded (instance %s): cli_role=%s, can_host=%s, can_join=%s", 
    __LUA_STATE_ID__ or 0,
    cli_role, 
    tostring(NetRole.can_host()), 
    tostring(NetRole.can_join())))

return NetRole
