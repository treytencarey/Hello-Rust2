-- Network Server Module
-- Handles server hosting and message relay with scope tracking
--
-- Usage:
--   local NetServer = require("modules/net_server.lua")
--   NetServer.start(5000, 10)
--   register_system("Update", function(world) NetServer.update(world) end)

local NetSync = require("modules/net_sync.lua")

local NetServer = {}

-- Channels
local CHANNEL_RELIABLE = 0
local CHANNEL_UNRELIABLE = 1

-- Track which net_ids each client knows about
local client_scope = {}  -- client_id -> { net_id = true, ... }

-- Track which client owns which net_id
local net_id_owners = {}  -- net_id -> client_id

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

local function send_to_client(world, client_id, channel, message)
    print(string.format("[NET_SERVER] Sending message to client %d: %s", client_id, message))
    pcall(function()
        world:call_resource_method("RenetServer", "send_message", client_id, channel, message)
    end)
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
    -- Decode JSON to table
    local msg = NetSync.json_decode(msg_str)
    if not msg then return end
    
    local msg_type = msg.type
    local net_id = msg.net_id
    
    if msg_type == "spawn" and net_id then
        -- Track ownership
        net_id_owners[net_id] = sender_id
        
        -- Mark sender as knowing this entity
        client_scope[sender_id] = client_scope[sender_id] or {}
        client_scope[sender_id][net_id] = true
    end
    
    -- Relay to all other clients
    local clients = get_clients(world)
    for _, receiver_id in ipairs(clients) do
        if receiver_id ~= sender_id then
            -- For spawns, always send (receiver will learn about it)
            -- For updates, check if receiver knows the entity
            if msg_type == "spawn" then
                send_to_client(world, receiver_id, channel, msg_str)
                -- Mark receiver as now knowing this entity
                client_scope[receiver_id] = client_scope[receiver_id] or {}
                client_scope[receiver_id][net_id] = true
            elseif msg_type == "update" and net_id then
                local receiver_scope = client_scope[receiver_id] or {}
                if receiver_scope[net_id] then
                    send_to_client(world, receiver_id, channel, msg_str)
                else
                    -- Receiver doesn't know this entity - request spawn from owner
                    local owner = net_id_owners[net_id]
                    if owner then
                        -- TODO: Send NAK to owner requesting re-spawn
                        print(string.format("[NET_SERVER] Receiver %d doesn't know net_id=%d", receiver_id, net_id))
                    end
                end
            elseif msg_type == "despawn" and net_id then
                send_to_client(world, receiver_id, channel, msg_str)
                -- Remove from scope
                if client_scope[receiver_id] then
                    client_scope[receiver_id][net_id] = nil
                end
            else
                -- Other messages, just relay
                send_to_client(world, receiver_id, channel, msg_str)
            end
        end
    end
end

--------------------------------------------------------------------------------
-- Update System
--------------------------------------------------------------------------------

--- Main update - call in your Update system
function NetServer.update(world)
    local clients = get_clients(world)
    
    -- Initialize new clients
    for _, client_id in ipairs(clients) do
        if not client_scope[client_id] then
            client_scope[client_id] = {}
            -- Assign client prefix
            local prefix = next_client_prefix
            next_client_prefix = next_client_prefix + 1
            print(string.format("[NET_SERVER] New client %d, assigned prefix %d", client_id, prefix))
            
            -- Late-join sync: Send all existing entities to new client
            local all_net_ids = NetSync.get_all_net_ids()
            for _, net_id in ipairs(all_net_ids) do
                local spawn_msg = NetSync.get_spawn_message_for(world, net_id)
                if spawn_msg then
                    print(string.format("[NET_SERVER] Sending spawn message to client %d: %s", client_id, spawn_msg))
                    send_to_client(world, client_id, CHANNEL_RELIABLE, spawn_msg)
                    client_scope[client_id][net_id] = true
                end
            end
            if #all_net_ids > 0 then
                print(string.format("[NET_SERVER] Sent %d existing entities to new client %d", #all_net_ids, client_id))
            end
            
            -- Call connect callback
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
    
    -- Clean up disconnected clients
    local active = {}
    for _, client_id in ipairs(clients) do
        active[client_id] = true
    end
    
    for client_id, _ in pairs(client_scope) do
        if not active[client_id] then
            -- Client disconnected - notify others of their entities despawning
            for net_id, owner in pairs(net_id_owners) do
                if owner == client_id then
                    local despawn_msg = NetSync.json_encode({ type = "despawn", net_id = net_id })
                    for _, other_id in ipairs(clients) do
                        send_to_client(world, other_id, CHANNEL_RELIABLE, despawn_msg)
                    end
                    net_id_owners[net_id] = nil
                end
            end
            client_scope[client_id] = nil
            print(string.format("[NET_SERVER] Client %d disconnected", client_id))
            
            -- Call disconnect callback
            if callbacks.on_client_disconnect then
                callbacks.on_client_disconnect(client_id)
            end
        end
    end
end

print("[NET_SERVER] Module loaded")

return NetServer
