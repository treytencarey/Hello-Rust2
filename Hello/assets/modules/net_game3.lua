-- Net Game 3 Module
-- High-level host/join API using net3 modules
-- Self-registering systems that check state at runtime
--
-- Usage:
--   local NetGame3 = require("modules/net_game3.lua")
--   NetGame3.host(port, callbacks)     -- Start server
--   NetGame3.join(ip, port, callbacks) -- Connect to server

local NetRole = require("modules/net_role.lua")
local NetSync3 = require("modules/net3/init.lua")

local NetGame3 = {}

--------------------------------------------------------------------------------
-- State (using resources for hot-reload safety)
--------------------------------------------------------------------------------

local function get_state()
    return define_resource("NetGame3State", {
        mode = nil,         -- "server" | "client" | nil
        server_port = nil,
        client_ip = nil,
        client_port = nil,
        on_game_ready = nil,
        on_connected = nil,
        on_player_joined = nil,
        on_player_left = nil,
        initialized = false,
        connected = false,           -- Client: connected to server
        connected_clients = {},      -- client_id -> true (tracked for connection diff)
    })
end

--------------------------------------------------------------------------------
-- Connection Handlers (internal)
--------------------------------------------------------------------------------

local function on_client_connected(client_id, world)
    local state = get_state()
    NetSync3.on_client_connected(client_id, world)
    if state.on_player_joined then
        state.on_player_joined(client_id, world)
    end
end

local function on_client_disconnected(client_id, world, get_clients)
    local state = get_state()
    NetSync3.on_client_disconnected(client_id, world, get_clients)
    if state.on_player_left then
        state.on_player_left(client_id, world)
    end
end

--------------------------------------------------------------------------------
-- Server Hosting
--------------------------------------------------------------------------------

--- Host a game server
--- @param port number The port to listen on
--- @param callbacks table { on_game_ready, on_player_joined, on_player_left }
function NetGame3.host(port, callbacks)
    callbacks = callbacks or {}
    local state = get_state()
    
    if state.initialized then
        print("[NET_GAME3] Already initialized")
        return
    end
    
    print(string.format("[NET_GAME3] Hosting on port %d", port))
    
    state.mode = "server"
    state.server_port = port
    state.on_game_ready = callbacks.on_game_ready
    state.on_player_joined = callbacks.on_player_joined
    state.on_player_left = callbacks.on_player_left

    insert_resource("RenetServer", {})
    insert_resource("NetcodeServerTransport", {
        port = port,
        max_clients = 10
    })
    
    -- Mark as hosting
    NetRole.set_hosted(true)
    
    -- Initialize as server with our client_id = 0
    NetSync3.set_my_client_id(0)
    
    -- Get clients function (stored in state for systems)
    local function get_clients_from_renet(world)
        return world:call_resource_method("RenetServer", "clients_id") or {}
    end
    
    -- Initialize NetSync3 in server mode
    NetSync3.init_server(get_clients_from_renet)
    
    state.initialized = true
    
    if state.on_game_ready then
        state.on_game_ready()
    end
    
    print("[NET_GAME3] Server initialized")
end

--------------------------------------------------------------------------------
-- Client Connection
--------------------------------------------------------------------------------

--- Join a game server
--- @param ip string Server IP address
--- @param port number Server port
--- @param callbacks table { on_connected, on_disconnected }
function NetGame3.join(ip, port, callbacks)
    callbacks = callbacks or {}
    local state = get_state()
    
    if state.initialized then
        print("[NET_GAME3] Already initialized")
        return
    end
    
    print(string.format("[NET_GAME3] Joining %s:%d", ip, port))
    
    state.mode = "client"
    state.client_ip = ip
    state.client_port = port
    state.on_connected = callbacks.on_connected

    insert_resource("RenetClient", {})
    insert_resource("NetcodeClientTransport", {
        server_addr = ip,
        port = port
    })
    
    -- Mark as joined
    NetRole.set_joined(true)
    
    -- Initialize NetSync3 in client mode
    NetSync3.init_client()
    
    state.initialized = true
    
    print("[NET_GAME3] Client connecting...")
end

--------------------------------------------------------------------------------
-- Utility Functions
--------------------------------------------------------------------------------

--- Check if hosting
--- @return boolean
function NetGame3.is_hosting()
    return get_state().mode == "server"
end

--- Check if connected as client
--- @return boolean
function NetGame3.is_client()
    return get_state().mode == "client"
end

--- Get current mode
--- @return string|nil "server" | "client" | nil
function NetGame3.get_mode()
    return get_state().mode
end

--- Check if fully initialized
--- @return boolean
function NetGame3.is_initialized()
    return get_state().initialized
end

--- Get my client ID
--- @return number|nil
function NetGame3.get_my_client_id()
    return NetSync3.get_my_client_id()
end

--------------------------------------------------------------------------------
-- Self-Registering Systems
--------------------------------------------------------------------------------

-- Server: Connection monitoring system
register_system("Update", function(world)
    local state = get_state()
    if state.mode ~= "server" then return end
    
    -- Guard: ensure RenetServer resource exists
    if not world:query_resource("RenetServer") then return end
    
    local current_clients = world:call_resource_method("RenetServer", "clients_id") or {}
    
    -- Build set of current clients
    local current_set = {}
    for _, client_id in ipairs(current_clients) do
        current_set[client_id] = true
    end
    
    -- Check for new connections (in current but not in known)
    for _, client_id in ipairs(current_clients) do
        if not state.connected_clients[client_id] then
            state.connected_clients[client_id] = true
            on_client_connected(client_id, world)
        end
    end
    
    -- Check for disconnections (in known but not in current)
    for client_id, _ in pairs(state.connected_clients) do
        if not current_set[client_id] then
            state.connected_clients[client_id] = nil
            on_client_disconnected(client_id, world, function() return current_clients end)
        end
    end
end)

-- Client: Connection detection system
register_system("Update", function(world)
    local state = get_state()
    if state.mode ~= "client" then return end
    if state.connected then return end
    
    local my_id = NetSync3.get_my_client_id()
    if my_id then
        state.connected = true
        print(string.format("[NET_GAME3] Connected with client_id %d", my_id))
        if state.on_connected then
            state.on_connected(my_id, world)
        end
    end
end)

print("[NET_GAME3] Systems registered (will activate when host/join called)")

return NetGame3
