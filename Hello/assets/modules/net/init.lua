-- NET Main Module (Orchestrator)
-- Central entry point that exports everything and registers systems
--
-- Usage:
--   local NetSync = require("modules/net/init.lua")
--   NetSync.init_server(get_clients)  -- or
--   NetSync.init_client()

local Components = require("modules/net/components.lua")
local State = require("modules/net/state.lua")
local Messages = require("modules/net/messages.lua")
local Outbound = require("modules/net/outbound.lua")
local Inbound = require("modules/net/inbound.lua")
local Prediction = require("modules/net/prediction.lua")
local Interpolation = require("modules/net/interpolation.lua")
local Server = require("modules/net/server.lua", { instanced = true })
local Client = require("modules/net/client.lua", { instanced = true })

local NetSync = {}

local state = State.get()

--------------------------------------------------------------------------------
-- Exports
--------------------------------------------------------------------------------

-- Component names
NetSync.MARKER = Components.MARKER
NetSync.OUTBOUND = Components.OUTBOUND
NetSync.INBOUND = Components.INBOUND
NetSync.PREDICTION = Components.PREDICTION
NetSync.INTERPOLATION = Components.INTERPOLATION

-- Channel constants
NetSync.CHANNEL_RELIABLE = Messages.CHANNEL_RELIABLE
NetSync.CHANNEL_UNRELIABLE = Messages.CHANNEL_UNRELIABLE

-- State accessors
NetSync.get_state = State.get
NetSync.next_net_id = State.next_net_id
NetSync.set_client_prefix = State.set_client_prefix
NetSync.my_client_id = State.my_client_id
NetSync.register_entity = State.register_entity
NetSync.unregister_entity = State.unregister_entity
NetSync.get_entity_id = State.get_entity_id
NetSync.get_net_id = State.get_net_id
NetSync.is_known = State.is_known
NetSync.get_sync_config = State.get_sync_config
NetSync.mark_spawned = State.mark_spawned
NetSync.is_spawned = State.is_spawned

-- Server-only state accessors
NetSync.init_client_scope = State.init_client_scope
NetSync.add_to_client_scope = State.add_to_client_scope
NetSync.remove_from_client_scope = State.remove_from_client_scope
NetSync.client_knows_entity = State.client_knows_entity
NetSync.remove_client_scope = State.remove_client_scope
NetSync.get_known_clients = State.get_known_clients
NetSync.set_owner = State.set_owner
NetSync.get_owner = State.get_owner
NetSync.clear_owner = State.clear_owner

-- Message builders
NetSync.build_spawn_msg = Messages.build_spawn
NetSync.build_update_msg = Messages.build_update
NetSync.build_despawn_msg = Messages.build_despawn
NetSync.build_owner_change_msg = Messages.build_owner_change
NetSync.build_client_id_msg = Messages.build_client_id

-- Individual systems (for custom ordering)
NetSync.outbound_system = Outbound.system
NetSync.inbound_system = Inbound.system
NetSync.prediction_system = Prediction.system
NetSync.interpolation_system = Interpolation.system
NetSync.server_send_system = Server.send_system
NetSync.server_receive_system = Server.receive_system
NetSync.client_send_system = Client.send_system
NetSync.client_receive_system = Client.receive_system

-- Connection handlers
NetSync.on_client_connected = Server.on_client_connected
NetSync.on_client_disconnected = Server.on_client_disconnected

-- Message handlers (for external override)
NetSync.handle_spawn = Inbound.handle_spawn
NetSync.handle_update = Inbound.handle_update
NetSync.handle_despawn = Inbound.handle_despawn
NetSync.handle_owner_change = Inbound.handle_owner_change
NetSync.handle_client_id = Inbound.handle_client_id

--------------------------------------------------------------------------------
-- Server Initialization
--------------------------------------------------------------------------------

--- Initialize NetSync in server mode (sets state, systems auto-register)
--- @param get_clients function Returns list of connected client IDs
--- @param filter_clients function|nil Optional filter for target clients
function NetSync.init_server(get_clients, filter_clients)
    if state.initialized then
        print("[NET] Already initialized")
        return
    end
    
    print("[NET] Initializing server mode")
    
    state.mode = "server"
    state.get_clients = get_clients
    state.filter_clients = filter_clients
    state.initialized = true
    
    print("[NET] Server mode configured (systems already registered)")
end

--------------------------------------------------------------------------------
-- Client Initialization
--------------------------------------------------------------------------------

--- Initialize NetSync in client mode (sets state, systems auto-register)
function NetSync.init_client()
    if state.initialized then
        print("[NET] Already initialized")
        return
    end

    print("[NET] Initializing client mode")

    state.mode = "client"
    state.initialized = true

    -- Initialize scope tracking for server (client_id 0)
    State.init_client_scope(0)

    print("[NET] Client mode configured (systems already registered)")
end

--------------------------------------------------------------------------------
-- Helpers
--------------------------------------------------------------------------------

--- Check if an entity is owned by this client
--- @param world userdata
--- @param entity userdata
--- @return boolean
function NetSync.is_my_entity(world, entity)
    local sync = entity:get(Components.MARKER)
    if not sync then return false end
    return sync.owner_client == State.my_client_id()
end

--- Get the current client ID
--- @return number|nil
function NetSync.get_my_client_id()
    return State.my_client_id()
end

--- Set the current client ID
--- @param id number
function NetSync.set_my_client_id(id)
    State.my_client_id(id)
    State.set_client_prefix(id)
end

--------------------------------------------------------------------------------
-- Self-Registering Systems (run based on state.mode)
--------------------------------------------------------------------------------

-- Outbound system: detect changes, spawn NetSyncOutbound entities
register_system("Update", function(world)
    if not state.initialized then return end
    
    local context = {
        is_server = state.mode == "server",
        get_clients = state.get_clients,
        filter_clients = state.filter_clients,
    }
    
    -- 1. Receive messages
    if state.mode == "server" then
        Server.receive_system(world, state.get_clients)
    elseif state.mode == "client" then
        Client.receive_system(world)
    end

    -- 2. Process messages
    Inbound.system(world)

    -- 3. Prediction and interpolation (only for clients) 
    if state.mode == "client" then
        Prediction.system(world)
        Interpolation.system(world)
    end

    -- 4. Prepare outbound messages
    Outbound.system(world, context)

    -- 5. Send messages
    if state.mode == "server" then
        Server.send_system(world)
    elseif state.mode == "client" then
        Client.send_system(world)
    end
end)

print("[NET] Systems registered (will activate when init_server/init_client called)")

return NetSync
