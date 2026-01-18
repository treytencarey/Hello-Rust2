-- Server Override for NetGame
-- Directly modifies NetGame functions with super references
-- This ensures all calls to NetGame use these overrides

local NetGame = require("modules/net_game.lua")
local NetSync = require("modules/net_sync.lua")
local PlayerController = require("scripts/server/Conflux2/modules/shared/player_controller.lua")
local QuadrantFilter = require("modules/spatial_filters/quadrants.lua")

print(string.format("[CONFLUX_SERVER_NET_GAME] instance: %s", __LUA_STATE_ID__))

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

local options = {
    filter_fn = QuadrantFilter.filter_fn
}

local CHANNEL_RELIABLE = 0

--------------------------------------------------------------------------------
-- Override: send (applies spatial filtering)
--------------------------------------------------------------------------------

-- Save super reference
local super_send = NetGame.send

--- @param world userdata
--- @param client_id number
--- @param channel number
--- @param msg_str string
function NetGame.send(world, client_id, channel, msg_str)
    -- Parse message to check if it's a sync message
    local msg = NetSync.json_decode(msg_str)
    local net_id = msg and msg.net_id
    
    if not net_id then
        -- Not a sync message (or no net_id), pass through to super
        super_send(world, client_id, channel, msg_str)
        return
    end
    
    -- Check if owner (owners always receive their own entity)
    local is_owner = (NetSync.get_net_id_owner(net_id) == client_id)
    
    -- Check filter
    local passes_filter = options.filter_fn(world, client_id, net_id)
    print(is_owner, passes_filter)
    
    if is_owner or passes_filter then
        -- Allowed, send normally via super
        super_send(world, client_id, channel, msg_str)
    elseif NetSync.client_knows_entity(client_id, net_id) then
        -- Filtered out but client knows entity → send despawn instead
        local despawn_msg = NetSync.json_encode({ type = "despawn", net_id = net_id })
        super_send(world, client_id, CHANNEL_RELIABLE, despawn_msg)
        NetSync.remove_from_scope(client_id, net_id)
        print(string.format("[CONFLUX_SERVER_NET_GAME] Filtered out net_id=%d for client %s, sent despawn", net_id, client_id))
    end
    -- Otherwise: filtered out, client doesn't know → drop silently
end

--------------------------------------------------------------------------------
-- Override: on_client_connected
--------------------------------------------------------------------------------

-- Save super reference
local super_on_client_connected = NetGame.on_client_connected

function NetGame.on_client_connected(client_id, world, send_fn)
    PlayerController.spawn_player(client_id)

    print(string.format("[CONFLUX_SERVER_NET_GAME] Client %s connected", client_id))
    
    -- Call super
    super_on_client_connected(client_id, world, send_fn)
end

--------------------------------------------------------------------------------
-- Override: on_client_disconnected
--------------------------------------------------------------------------------

-- Save super reference
local super_on_client_disconnected = NetGame.on_client_disconnected

function NetGame.on_client_disconnected(client_id, send_fn, get_clients_fn)
    PlayerController.despawn_player(client_id)

    print(string.format("[CONFLUX_SERVER_NET_GAME] Client %s disconnected", client_id))
    
    -- Call super
    super_on_client_disconnected(client_id, send_fn, get_clients_fn)
end

print("[CONFLUX_SERVER_NET_GAME] Module loaded (overrides applied to NetGame)")

-- Return NetGame so callers get the modified version
return NetGame