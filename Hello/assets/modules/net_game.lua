-- Network Game Module
-- High-level game hosting/joining abstraction
--
-- Usage (Server):
--   local NetGame = require("modules/net_game.lua")
--   NetGame.host({
--       port = 5000,
--       max_players = 10,
--       on_player_join = function(client_id) ... end,
--       on_player_leave = function(client_id) ... end
--   })
--
-- Usage (Client):
--   NetGame.join({
--       server_addr = "127.0.0.1",
--       port = 5000,
--       on_connected = function() ... end,
--       on_disconnected = function() ... end
--   })

local NetRole = require("modules/net_role.lua")
local NetSync = require("modules/net_sync.lua")
local NetServer = require("modules/net_server.lua")
local NetClient = require("modules/net_client.lua")

local NetGame = {}

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

NetGame.config = {
    reconnection = {
        enabled = false,                   -- Placeholder for future implementation
        grace_period_seconds = 300,        -- How long to preserve state (future)
        identification = "client_uid",     -- "client_uid" | "session_token" | "account" (future)
    }
}

--------------------------------------------------------------------------------
-- State Management (module-local since require({instanced=true}) provides isolation)
--------------------------------------------------------------------------------

-- State is now module-local - each instanced require gets its own copy
-- NOTE: hosted/joined state is now tracked in NetRole module
local state = {
    server_config = nil,
    client_config = nil,
    spatial_filter_script = nil,  -- Store script path, not module
    camera_script = nil,          -- Store script path, not module
    camera_entity_id = nil,
    target_entity_id = nil,
    player_system_created = false,
    interpolation_system_created = false
}

-- Module-local cached modules (loaded via require_async)
local network_module = nil      -- Server: loaded networking module
local camera_module = nil       -- Client: loaded camera module
local controller_module = nil   -- Client: loaded controller module
local controller_system = nil   -- Client: created system function
local interpolation_system = nil

--------------------------------------------------------------------------------
-- Server Hosting
--------------------------------------------------------------------------------

--- Host a game server
--- @param world userdata The ECS world
--- @param config table { name, port, max_players, network_script, on_player_join(client_id), on_player_leave(client_id) }
---   name: optional name for server instance (creates named instance, allows multiple servers)
---   network_script: path to network module (e.g. "scripts/server/Conflux/modules/networking.lua")
---                   Module must expose: init(), filter_fn(world, client_id, net_id) → boolean
function NetGame.host(world, config)
    if NetRole.is_server() then
        print("[NET_GAME] Already hosting, ignoring")
        return
    end

    local port = config.port or 5000
    local max_players = config.max_players or 10

    state.server_config = config

    -- Start server
        NetServer.start(port, max_players)
    
    -- Set up callbacks
    if config.on_player_join then
        NetServer.on_client_connect(function(client_id)
            print(string.format("[NET_GAME] Player joined: %d", client_id))
            config.on_player_join(client_id)
        end)
    end
    
    if config.on_player_leave then
        NetServer.on_client_disconnect(function(client_id)
            print(string.format("[NET_GAME] Player left: %d", client_id))
            config.on_player_leave(client_id)
        end)
    end
    
    -- Store network script path (will be loaded on-demand in update)
    if config.network_script then
        state.network_script = config.network_script
        print(string.format("[NET_GAME] Network script configured: %s", config.network_script))
    end
    
    NetRole.set_hosted(true)
    print(string.format("[NET_GAME] Hosting on port %d (max %d players)", port, max_players))
end

--------------------------------------------------------------------------------
-- Client Joining
--------------------------------------------------------------------------------

--- Join a game as client
--- @param world userdata The ECS world
--- @param config table { name, server_addr, port, camera_script, controller_script, on_connected(), on_disconnected() }
---   name: optional name for client instance (creates named instance, allows multiple clients)
---   camera_script: path to camera module (e.g. "scripts/Conflux/modules/camera.lua")
---                  Module must expose: init(), attach(entity_id), update(world, dt), get_camera_entity()
---   controller_script: path to controller module (e.g. "scripts/Conflux/modules/controller.lua")
---                      Module must expose: init(), create_system(callbacks) → function(world)
function NetGame.join(world, config)
    if NetRole.is_client() then
        print("[NET_GAME] Already joined, ignoring")
        return
    end

    local server_addr = config.server_addr or "127.0.0.1"
    local port = config.port or 5000

    state.client_config = config

    -- Create client resources
    insert_resource("RenetClient", {})
    insert_resource("NetcodeClientTransport", {
        server_addr = server_addr,
        port = port
    })
    
    -- Store module script paths (will be loaded on-demand in update)
    if config.camera_script then
        state.camera_script = config.camera_script
        print(string.format("[NET_GAME] Camera script configured: %s", config.camera_script))
    end
    
    if config.controller_script then
        state.controller_script = config.controller_script
        print(string.format("[NET_GAME] Controller script configured: %s", config.controller_script))
    end
    
    NetRole.set_joined(true)
    print(string.format("[NET_GAME] Connecting to %s:%d", server_addr, port))
    
    -- TODO: Set up connection status monitoring for callbacks
    -- For now, on_connected would need to be called after verifying connection
end

--------------------------------------------------------------------------------
-- State Queries
--------------------------------------------------------------------------------

--- Send message to client (overridable for filtering)
--- @param world userdata
--- @param client_id number
--- @param channel number
--- @param msg_str string
function NetGame.send(world, client_id, channel, msg_str)
    NetServer.send(world, client_id, channel, msg_str)
end

--- Handle client connection (overridable for custom logic)
--- @param client_id number
--- @param world userdata
--- @param send_fn function
function NetGame.on_client_connected(client_id, world, send_fn)
    NetServer.on_client_connected(client_id, world, send_fn)
end

--- Handle client disconnection (overridable for custom logic)
--- @param client_id number
--- @param send_fn function
--- @param get_clients_fn function
function NetGame.on_client_disconnected(client_id, send_fn, get_clients_fn)
    NetServer.on_client_disconnected(client_id, send_fn, get_clients_fn)
end

--------------------------------------------------------------------------------
-- Update System (Single Entry Point)
--------------------------------------------------------------------------------

--- Main update - call this in your game's Update system
--- Routes to appropriate networking backend (server/client) based on role
--- @param world userdata The ECS world
function NetGame.update(world)
    local dt = world:delta_time()

    -- Server update (only if this instance is hosting)
    if NetRole.is_server() then
        NetServer.update(world)
    end
    
    if NetRole.is_client() then
        NetClient.update(world)
        
        -- -- Load controller module on-demand if script path configured
        -- if state.controller_script and not controller_module then
        --     require_async(state.controller_script, function(module)
        --         if module then
        --             if module.init then
        --                 module.init()
        --             end
        --             controller_module = module
        --             controller_system = controller_module.create_system({
        --                 on_player_found = function(w, entity_id)
        --                     state.target_entity_id = entity_id
        --                     print(string.format("[NET_GAME] Player entity found: %d", entity_id))
                            
        --                     -- Attach camera to player when found
        --                     if camera_module and camera_module.attach then
        --                         camera_module.attach(entity_id)
        --                     end
        --                 end
        --             })
        --             print("[NET_GAME] Controller system created")
        --             print(string.format("[NET_GAME] Controller module loaded: %s", state.controller_script))
        --         else
        --             print(string.format("[NET_GAME] ERROR: Failed to load controller module: %s", state.controller_script))
        --         end
        --     end)
        -- end
        
        -- -- Run controller system (input, prediction, reconciliation)
        -- if controller_system then
        --     controller_system(world)
        -- end
        
        -- Initialize interpolation system if not created
        if not interpolation_system then
            local TransformInterpolation = require("modules/transform_interpolation.lua")
            interpolation_system = TransformInterpolation.create_system()
        end

        -- Run interpolation for remote entities
        if interpolation_system then
            interpolation_system(world)
        end
        
        -- -- Load camera module on-demand if script path configured
        -- print(string.format("[NET_GAME] Camera script: %s, camera_module: %s", state.camera_script, camera_module))
        -- if state.camera_script and not camera_module then
        --     require_async(state.camera_script, function(module)
        --         if module then
        --             if module.init then
        --                 module.init()
        --             end
        --             camera_module = module
        --             print(string.format("[NET_GAME] Camera module loaded: %s", state.camera_script))
                    
        --             -- Attach to target if already found
        --             if state.target_entity_id and module.attach then
        --                 module.attach(state.target_entity_id)
        --             end
        --         else
        --             print(string.format("[NET_GAME] ERROR: Failed to load camera module: %s", state.camera_script))
        --         end
        --     end)
        -- end
        
        -- -- Update camera
        -- if camera_module and camera_module.update then
        --     camera_module.update(world, dt)
        -- end
    end
end

--------------------------------------------------------------------------------
-- Deprecated (kept for backward compatibility)
--------------------------------------------------------------------------------

--- @deprecated Use NetGame.update() instead
function NetGame.create_server_update()
    return function(world)
        NetGame.update(world)
    end
end

--- @deprecated Use NetGame.update() instead - networking is now automatic
function NetGame.create_sync_outbound()
    return function(world)
        -- No-op: handled by NetGame.update() internally
    end
end

--- @deprecated Use NetGame.update() instead - networking is now automatic
function NetGame.create_sync_inbound()
    return function(world)
        -- No-op: handled by NetGame.update() internally
    end
end

print("[NET_GAME] Module loaded")

return NetGame
