-- Server-side NetGame2 module for Conflux2
-- Uses ECS-based message system with spatial filtering

local NetGame2 = require("modules/net_game2.lua")
local NetSync2 = require("modules/net_sync2.lua")
local PlayerController = require("scripts/server/Conflux2/modules/shared/player_controller2.lua")
local QuadrantFilter = require("modules/spatial_filters/quadrants2.lua")

print(string.format("[CONFLUX_SERVER_NET_GAME2] instance: %s", __LUA_STATE_ID__))

--------------------------------------------------------------------------------
-- Spatial Filter
--------------------------------------------------------------------------------

--- Filter function that uses quadrant-based visibility
--- Owners always see their own entities
local function spatial_filter(world, client_id, entity, msg)
    if not msg or not msg.net_id then
        return true
    end

    -- Owners always receive their own entity updates
    local owner = NetGame2.get_net_id_owner(msg.net_id)
    if owner == client_id then
        return true
    end

    -- Apply quadrant filter
    return QuadrantFilter.filter_fn(world, client_id, msg.net_id)
end

--------------------------------------------------------------------------------
-- Server Module
--------------------------------------------------------------------------------

local M = {}

--- Host the server with spatial filtering
function M.host(world, config)
    config = config or {}

    NetGame2.host(world, {
        port = config.port or 5001,
        max_players = config.max_players or 10,

        -- Set spatial filter for all entities
        filter_fn = nil, -- spacial_filter

        on_player_join = function(client_id)
            print(string.format("[CONFLUX_SERVER_NET_GAME2] Client %s connected", client_id))
            PlayerController.spawn_player(client_id)
        end,

        on_player_leave = function(client_id)
            print(string.format("[CONFLUX_SERVER_NET_GAME2] Client %s disconnected", client_id))
            PlayerController.despawn_player(client_id)
        end,
    })
end

--- Update wrapper
function M.update(world)
    NetGame2.update(world)
end

print("[CONFLUX_SERVER_NET_GAME2] Module loaded")

return M
