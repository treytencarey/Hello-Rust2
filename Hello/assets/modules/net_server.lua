-- Network Server Module
-- Handles server hosting and message relay with scope tracking
--
-- Usage:
--   local NetServer = require("modules/net_server.lua")
--   NetServer.start(5000, 10)
--   register_system("Update", function(world) NetServer.update(world) end)

local NetSync = require("modules/net_sync.lua")
local NetGame = require("modules/net_game.lua")

local NetServer = {}

-- Channels
local CHANNEL_RELIABLE = 0
local CHANNEL_UNRELIABLE = 1

-- Next client prefix to assign
local next_client_prefix = 1

-- Callback hooks
local callbacks = {
    on_client_connect = nil,     -- function(client_id)
    on_client_disconnect = nil,  -- function(client_id)
}

--- Set callback for client connect
function NetServer.on_client_connect(fn)
    callbacks.on_client_connect = fn
end

--- Set callback for client disconnect
function NetServer.on_client_disconnect(fn)
    callbacks.on_client_disconnect = fn
end

--------------------------------------------------------------------------------
-- Server Lifecycle
--------------------------------------------------------------------------------

--- Start the server
function NetServer.start(port, max_clients)
    insert_resource("RenetServer", {})
    insert_resource("NetcodeServerTransport", {
        port = port,
        max_clients = max_clients or 10
    })
    print(string.format("[NET_SERVER] Started on port %d", port))
end

--------------------------------------------------------------------------------
-- Send/Receive Wrappers
--------------------------------------------------------------------------------

--- Send message to client (overridable for filtering)
--- @param world userdata
--- @param client_id number
--- @param channel number
--- @param msg_str string
function NetServer.send(world, client_id, channel, msg_str)
    print(string.format("[NET_SERVER] Sending to %s on channel %d: %s",
        client_id, channel, msg_str))
    pcall(function()
        world:call_resource_method("RenetServer", "send_message", client_id, channel, msg_str)
    end)
end

--- Handle client connection (overridable for custom logic)
--- @param client_id number
--- @param world userdata
--- @param send_fn function
function NetServer.on_client_connected(client_id, world, send_fn)
    -- Default: delegate to NetSync
    NetSync.on_client_connected(client_id, world, send_fn)
end

--- Handle client disconnection (overridable for custom logic)
--- @param client_id number
--- @param send_fn function
--- @param get_clients_fn function
function NetServer.on_client_disconnected(client_id, send_fn, get_clients_fn)
    -- Default: delegate to NetSync
    NetSync.on_client_disconnected(client_id, send_fn, get_clients_fn)
end

local function receive_from_client(world, client_id, channel)
    local ok, msg = pcall(function()
        return world:call_resource_method("RenetServer", "receive_message", client_id, channel)
    end)
    if ok and msg then
        return msg
    end
    return nil
end

local function get_clients(world)
    local ok, clients = pcall(function()
        return world:call_resource_method("RenetServer", "clients_id")
    end)
    if ok and clients then
        return clients
    end
    return {}
end

--------------------------------------------------------------------------------
-- Message Processing
--------------------------------------------------------------------------------

--- Process a message from a client
local function process_client_message(world, sender_id, channel, msg_str)
    print(string.format("[NET_SERVER] Received message from client %s: %s", sender_id, msg_str))
    -- Decode JSON to table
    local msg = NetSync.json_decode(msg_str)
    if not msg then return end
    
    local msg_type = msg.type
    local net_id = msg.net_id
    
    -- Process message on server (apply with authority validation)
    -- The outbound_system will detect changes and relay
    NetSync.process_message(world, msg)
end

--------------------------------------------------------------------------------
-- Update System
--------------------------------------------------------------------------------

--- Main update - call in your Update system
--- @param world userdata The ECS world
function NetServer.update(world)
    local clients = get_clients(world)
    
    -- Create send function wrapper that uses NetGame.send (supports overrides)
    local function send_fn(client_id, channel, msg_str)
        NetGame.send(world, client_id, channel, msg_str)
    end
    
    local function get_clients_fn()
        return clients
    end
    
    -- Initialize new clients
    for _, client_id in ipairs(clients) do
        if not NetSync.is_client_known(client_id) then
            -- Assign client prefix
            local prefix = next_client_prefix
            next_client_prefix = next_client_prefix + 1
            print(string.format("[NET_SERVER] New client %s, assigned prefix %d", client_id, prefix))
            
            -- Late-join sync - calls NetGame.on_client_connected (with any overrides)
            NetGame.on_client_connected(client_id, world, send_fn)
            
            -- Call connect callback (separate from override chain)
            if callbacks.on_client_connect then
                callbacks.on_client_connect(client_id)
            end
        end
    end
    
    -- Process messages from each client
    for _, client_id in ipairs(clients) do
        -- Reliable channel
        while true do
            local msg = receive_from_client(world, client_id, CHANNEL_RELIABLE)
            if not msg then break end
            process_client_message(world, client_id, CHANNEL_RELIABLE, msg)
        end
        
        -- Unreliable channel
        while true do
            local msg = receive_from_client(world, client_id, CHANNEL_UNRELIABLE)
            if not msg then break end
            process_client_message(world, client_id, CHANNEL_UNRELIABLE, msg)
        end
    end
    
    -- Run outbound system - NetSync handles scope internally
    NetSync.outbound_system(world, send_fn, get_clients_fn)
    
    -- Detect disconnected clients
    local active = {}
    for _, client_id in ipairs(clients) do
        active[client_id] = true
    end
    
    -- Find clients that NetSync knows about but are no longer active
    for _, client_id in ipairs(NetSync.get_known_clients()) do
        if not active[client_id] then
            print(string.format("[NET_SERVER] Client %s disconnected", client_id))
            
            -- NetSync handles cleanup - calls NetGame.on_client_disconnected (with any overrides)
            NetGame.on_client_disconnected(client_id, send_fn, get_clients_fn)
            
            -- Call disconnect callback (separate from override chain)
            if callbacks.on_client_disconnect then
                callbacks.on_client_disconnect(client_id)
            end
        end
    end
end

print("[NET_SERVER] Module loaded")

return NetServer
