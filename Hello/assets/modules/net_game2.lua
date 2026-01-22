-- Network Game Module v2
-- High-level game hosting/joining abstraction using ECS-based message system
--
-- Usage (Server):
--   local NetGame2 = require("modules/net_game2.lua")
--   NetGame2.host(world, {
--       port = 5000,
--       max_players = 10,
--       on_player_join = function(client_id) ... end,
--       on_player_leave = function(client_id) ... end
--   })
--
-- Usage (Client):
--   NetGame2.join(world, {
--       server_addr = "127.0.0.1",
--       port = 5000,
--       on_connected = function() ... end,
--       on_disconnected = function() ... end
--   })

local NetSync2 = require("modules/net_sync2.lua")
local NetServer2 = require("modules/net_server2.lua")
local NetClient2 = require("modules/net_client2.lua")
local NetRole = require("modules/net_role.lua")

local NetGame2 = {}

--------------------------------------------------------------------------------
-- State Management (module-local for instance isolation)
--------------------------------------------------------------------------------

local state = {
    server_config = nil,
    client_config = nil
}

--------------------------------------------------------------------------------
-- Server Hosting
--------------------------------------------------------------------------------

--- Host a game server
--- @param world userdata The ECS world
--- @param config table { port, max_players, on_player_join(client_id), on_player_leave(client_id), filter_fn }
---   filter_fn: optional function(world, client_id, entity, msg) -> boolean for spatial filtering
function NetGame2.host(world, config)
    if NetRole.is_server() then
        print("[NET_GAME2] Already hosting, ignoring")
        return
    end

    local port = config.port or 5000
    local max_players = config.max_players or 10

    state.server_config = config
    NetRole.set_hosted(true)

    -- Start server
    NetServer2.start(port, max_players)

    -- Set up callbacks
    if config.on_player_join then
        NetServer2.on_client_connect(function(client_id)
            print(string.format("[NET_GAME2] Player joined: %d", client_id))
            config.on_player_join(client_id)
        end)
    end

    if config.on_player_leave then
        NetServer2.on_client_disconnect(function(client_id)
            print(string.format("[NET_GAME2] Player left: %d", client_id))
            config.on_player_leave(client_id)
        end)
    end

    -- Set global filter if provided
    if config.filter_fn then
        NetServer2.set_filter(config.filter_fn)
        print("[NET_GAME2] Global filter configured")
    end

    print(string.format("[NET_GAME2] Hosting on port %d (max %d players)", port, max_players))
end

--------------------------------------------------------------------------------
-- Client Joining
--------------------------------------------------------------------------------

--- Join a game as client
--- @param world userdata The ECS world
--- @param config table { server_addr, port, on_connected(), on_disconnected() }
function NetGame2.join(world, config)
    if NetRole.is_client() then
        print("[NET_GAME2] Already joined, ignoring")
        return
    end

    local server_addr = config.server_addr or "127.0.0.1"
    local port = config.port or 5000

    state.client_config = config
    NetRole.set_joined(true)

    -- Connect to server
    NetClient2.connect(server_addr, port)

    print(string.format("[NET_GAME2] Connecting to %s:%d", server_addr, port))
end

--------------------------------------------------------------------------------
-- Update System
--------------------------------------------------------------------------------

--- Main update - call this in your game's Update system
--- Routes to appropriate networking backend (server/client) based on role
--- @param world userdata The ECS world
function NetGame2.update(world)
    -- Server update
    if NetRole.is_server() then
        NetServer2.update(world)
    end

    -- Client update
    if NetRole.is_client() then
        -- Check connection state for callbacks
        local connected = NetClient2.is_connected(world)

        if connected and not state.was_connected then
            -- Just connected
            state.was_connected = true
            if state.client_config and state.client_config.on_connected then
                state.client_config.on_connected()
            end
        elseif not connected and state.was_connected then
            -- Just disconnected
            state.was_connected = false
            if state.client_config and state.client_config.on_disconnected then
                state.client_config.on_disconnected()
            end
        end

        if connected then
            NetClient2.update(world)
        end
    end
end

--------------------------------------------------------------------------------
-- Helpers - Expose NetSync2 utilities
--------------------------------------------------------------------------------

--- Get next net_id
function NetGame2.next_net_id()
    return NetSync2.next_net_id()
end

--- Register entity with net_id
function NetGame2.register_entity(net_id, entity_id)
    NetSync2.register_entity(net_id, entity_id)
end

--- Get entity_id from net_id
function NetGame2.get_entity(net_id)
    return NetSync2.get_entity(net_id)
end

--- Get this client's assigned client_id
function NetGame2.get_my_client_id()
    return NetSync2.get_my_client_id()
end

--- Check if entity is owned by this client
--- @param world userdata
--- @param entity userdata|number Entity or entity_id
--- @return boolean
function NetGame2.is_my_entity(world, entity)
    return NetSync2.is_my_entity(world, entity)
end

--- Set entity owner (server-side)
function NetGame2.set_net_id_owner(net_id, client_id)
    NetSync2.set_net_id_owner(net_id, client_id)
end

--- Get entity owner (server-side)
function NetGame2.get_net_id_owner(net_id)
    return NetSync2.get_net_id_owner(net_id)
end

--- Get all net_ids owned by a client (server-side)
function NetGame2.get_net_ids_for_client(client_id)
    return NetSync2.get_net_ids_for_client(client_id)
end

--- Set client prefix (for net_id generation)
function NetGame2.set_client_prefix(prefix)
    NetSync2.set_client_prefix(prefix)
end

--------------------------------------------------------------------------------
-- Component Names - For use in entity definitions
--------------------------------------------------------------------------------

NetGame2.MARKER = NetSync2.MARKER
NetGame2.PREDICTION = NetSync2.PREDICTION
NetGame2.INTERPOLATION = NetSync2.INTERPOLATION

print("[NET_GAME2] Module loaded")

return NetGame2
