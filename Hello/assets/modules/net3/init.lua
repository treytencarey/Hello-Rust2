-- Net3 Main Module (Orchestrator)
-- Central entry point that exports everything and registers systems
--
-- Usage:
--   local NetSync3 = require("modules/net3/init.lua")
--   NetSync3.init_server(get_clients)  -- or
--   NetSync3.init_client()

local Components = require("modules/net3/components.lua")
local State = require("modules/net3/state.lua")
local Messages = require("modules/net3/messages.lua")
local Outbound = require("modules/net3/outbound.lua")
local Inbound = require("modules/net3/inbound.lua")
local Prediction = require("modules/net3/prediction.lua")
local Interpolation = require("modules/net3/interpolation.lua")
local Server = require("modules/net3/server.lua", { instanced = true })
local Client = require("modules/net3/client.lua", { instanced = true })

local NetSync3 = {}

local state = State.get()

--------------------------------------------------------------------------------
-- Exports
--------------------------------------------------------------------------------

-- Component names
NetSync3.MARKER = Components.MARKER
NetSync3.OUTBOUND = Components.OUTBOUND
NetSync3.INBOUND = Components.INBOUND
NetSync3.PREDICTION = Components.PREDICTION
NetSync3.INTERPOLATION = Components.INTERPOLATION

-- Channel constants
NetSync3.CHANNEL_RELIABLE = Messages.CHANNEL_RELIABLE
NetSync3.CHANNEL_UNRELIABLE = Messages.CHANNEL_UNRELIABLE

-- State accessors
NetSync3.get_state = State.get
NetSync3.next_net_id = State.next_net_id
NetSync3.set_client_prefix = State.set_client_prefix
NetSync3.my_client_id = State.my_client_id
NetSync3.register_entity = State.register_entity
NetSync3.unregister_entity = State.unregister_entity
NetSync3.get_entity_id = State.get_entity_id
NetSync3.get_net_id = State.get_net_id
NetSync3.is_known = State.is_known
NetSync3.get_sync_config = State.get_sync_config
NetSync3.mark_spawned = State.mark_spawned
NetSync3.is_spawned = State.is_spawned

-- Server-only state accessors
NetSync3.init_client_scope = State.init_client_scope
NetSync3.add_to_client_scope = State.add_to_client_scope
NetSync3.remove_from_client_scope = State.remove_from_client_scope
NetSync3.client_knows_entity = State.client_knows_entity
NetSync3.remove_client_scope = State.remove_client_scope
NetSync3.get_known_clients = State.get_known_clients
NetSync3.set_owner = State.set_owner
NetSync3.get_owner = State.get_owner
NetSync3.clear_owner = State.clear_owner

-- Message builders
NetSync3.build_spawn_msg = Messages.build_spawn
NetSync3.build_update_msg = Messages.build_update
NetSync3.build_despawn_msg = Messages.build_despawn
NetSync3.build_owner_change_msg = Messages.build_owner_change
NetSync3.build_client_id_msg = Messages.build_client_id

-- Individual systems (for custom ordering)
NetSync3.outbound_system = Outbound.system
NetSync3.inbound_system = Inbound.system
NetSync3.prediction_system = Prediction.system
NetSync3.interpolation_system = Interpolation.system
NetSync3.server_send_system = Server.send_system
NetSync3.server_receive_system = Server.receive_system
NetSync3.client_send_system = Client.send_system
NetSync3.client_receive_system = Client.receive_system

-- Connection handlers
NetSync3.on_client_connected = Server.on_client_connected
NetSync3.on_client_disconnected = Server.on_client_disconnected

-- Message handlers (for external override)
NetSync3.handle_spawn = Inbound.handle_spawn
NetSync3.handle_update = Inbound.handle_update
NetSync3.handle_despawn = Inbound.handle_despawn
NetSync3.handle_owner_change = Inbound.handle_owner_change
NetSync3.handle_client_id = Inbound.handle_client_id

--------------------------------------------------------------------------------
-- Server Initialization
--------------------------------------------------------------------------------

--- Initialize NetSync3 in server mode (sets state, systems auto-register)
--- @param get_clients function Returns list of connected client IDs
--- @param filter_clients function|nil Optional filter for target clients
function NetSync3.init_server(get_clients, filter_clients)
    if state.initialized then
        print("[NET3] Already initialized")
        return
    end
    
    print("[NET3] Initializing server mode")
    
    state.mode = "server"
    state.get_clients = get_clients
    state.filter_clients = filter_clients
    state.initialized = true
    
    print("[NET3] Server mode configured (systems already registered)")
end

--------------------------------------------------------------------------------
-- Client Initialization
--------------------------------------------------------------------------------

--- Initialize NetSync3 in client mode (sets state, systems auto-register)
function NetSync3.init_client()
    if state.initialized then
        print("[NET3] Already initialized")
        return
    end
    
    print("[NET3] Initializing client mode")
    
    state.mode = "client"
    state.initialized = true
    
    print("[NET3] Client mode configured (systems already registered)")
end

--------------------------------------------------------------------------------
-- Helpers
--------------------------------------------------------------------------------

--- Check if an entity is owned by this client
--- @param world userdata
--- @param entity userdata
--- @return boolean
function NetSync3.is_my_entity(world, entity)
    local sync = entity:get(Components.MARKER)
    if not sync then return false end
    return sync.owner_client == State.my_client_id()
end

--- Get the current client ID
--- @return number|nil
function NetSync3.get_my_client_id()
    return State.my_client_id()
end

--- Set the current client ID
--- @param id number
function NetSync3.set_my_client_id(id)
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

print("[NET3] Systems registered (will activate when init_server/init_client called)")

return NetSync3
