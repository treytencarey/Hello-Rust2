-- Net3 State Module
-- Uses define_resource for hot-reload safe, instance-scoped state storage
--
-- All networking state is stored in a single resource per instance.
-- This ensures state survives hot-reload and server/client instances are isolated.

local State = {}

local state = define_resource("NetSyncState", {
    -- Entity tracking
    known_entities = {},      -- net_id -> entity_id
    entity_to_net = {},       -- entity_id -> net_id
    entity_sync_config = {},  -- entity_id -> { sync_components, last_sync_times, last_sync_hashes, dirty, spawned, created_locally }
    
    -- Client identity
    my_client_id = nil,
    client_prefix = 0,        -- Assigned by server for net_id generation
    local_counter = 0,        -- Local incrementing counter for net_id
    
    -- Mode tracking (for self-registering systems)
    mode = nil,               -- "server" | "client" | nil
    initialized = false,      -- Whether init_server/init_client has been called
    get_clients = nil,        -- function(world) -> { client_id, ... } (server only)
    filter_clients = nil,     -- function(world, client_id) -> boolean (server only)
    
    -- Server-only state
    client_scope = {},        -- client_id -> { net_id -> true } (what each client knows about)
    net_id_owners = {},       -- net_id -> client_id (who owns each entity)
    known_clients = {},       -- client_id -> true (connected clients)
    client_input_seq = {},    -- entity_id -> last processed input sequence
    
    -- Pending children (waiting for parent to spawn)
    pending_children = {},    -- net_id -> { msg, owner_client }
    pending_children_time = {},  -- net_id -> timestamp
    
    -- Sync types set (component names we're syncing)
    sync_types = {},          -- component_name -> count
})

--- Get or create the NetSyncState resource (idempotent)
--- @return table The state table (modifications persist)
function State.get()
    return state
end

--------------------------------------------------------------------------------
-- Net ID Management
--------------------------------------------------------------------------------

--- Generate a unique net_id for a new entity
--- @return number The new net_id
function State.next_net_id()
    state.local_counter = state.local_counter + 1
    return state.client_prefix * 1000000 + state.local_counter
end

--- Set the client prefix (called when server assigns our client_id)
--- @param prefix number The prefix to use for net_id generation
function State.set_client_prefix(prefix)
    state.client_prefix = prefix
end

--- Get/set client ID
--- @param id number|nil If provided, sets the client ID
--- @return number|nil The current client ID
function State.my_client_id(id)
    if id ~= nil then
        state.my_client_id = id
    end
    return state.my_client_id
end

--------------------------------------------------------------------------------
-- Entity Registration
--------------------------------------------------------------------------------

--- Register a known entity mapping
--- @param net_id number The network ID
--- @param entity_id number The ECS entity ID
function State.register_entity(net_id, entity_id)
    state.known_entities[net_id] = entity_id
    state.entity_to_net[entity_id] = net_id
end

--- Unregister an entity
--- @param net_id number The network ID
function State.unregister_entity(net_id)
    local entity_id = state.known_entities[net_id]
    if entity_id then
        state.entity_to_net[entity_id] = nil
    end
    state.known_entities[net_id] = nil
    state.entity_sync_config[entity_id] = nil
end

--- Get entity ID from net_id
--- @param net_id number
--- @return number|nil
function State.get_entity_id(net_id)
    return state.known_entities[net_id]
end

--- Get net_id from entity ID
--- @param entity_id number
--- @return number|nil
function State.get_net_id(entity_id)
    return state.entity_to_net[entity_id]
end

--- Check if entity is known
--- @param net_id number
--- @return boolean
function State.is_known(net_id)
    return state.known_entities[net_id] ~= nil
end

--------------------------------------------------------------------------------
-- Sync Config Management
--------------------------------------------------------------------------------

--- Get or create sync config for an entity
--- @param entity_id number
--- @param sync_components table|nil Default sync components if creating
--- @return table The sync config
function State.get_sync_config(entity_id, sync_components)
    if not state.entity_sync_config[entity_id] then
        state.entity_sync_config[entity_id] = {
            sync_components = sync_components or { Transform = {} },
            last_sync_times = {},
            last_sync_hashes = {},
            dirty = {},
            spawned = false,
            created_locally = true,
        }
    end
    return state.entity_sync_config[entity_id]
end

--- Mark entity as spawned (initial spawn message sent)
--- @param entity_id number
function State.mark_spawned(entity_id)
    local config = State.get_sync_config(entity_id)
    config.spawned = true
end

--- Check if entity has been spawned
--- @param entity_id number
--- @return boolean
function State.is_spawned(entity_id)
    local config = state.entity_sync_config[entity_id]
    return config and config.spawned or false
end

--------------------------------------------------------------------------------
-- Server-Only: Client Scope Management
--------------------------------------------------------------------------------

--- Initialize scope tracking for a client
--- @param client_id number
function State.init_client_scope(client_id)
    state.client_scope[client_id] = state.client_scope[client_id] or {}
    state.known_clients[client_id] = true
end

--- Add entity to client's known scope
--- @param client_id number
--- @param net_id number
function State.add_to_client_scope(client_id, net_id)
    if state.client_scope[client_id] then
        state.client_scope[client_id][net_id] = true
    end
end

--- Remove entity from client's known scope
--- @param client_id number
--- @param net_id number
function State.remove_from_client_scope(client_id, net_id)
    if state.client_scope[client_id] then
        state.client_scope[client_id][net_id] = nil
    end
end

--- Check if client knows about an entity
--- @param client_id number
--- @param net_id number
--- @return boolean
function State.client_knows_entity(client_id, net_id)
    return state.client_scope[client_id] and state.client_scope[client_id][net_id] or false
end

--- Remove client scope (on disconnect)
--- @param client_id number
function State.remove_client_scope(client_id)
    state.client_scope[client_id] = nil
    state.known_clients[client_id] = nil
end

--- Get all known clients
--- @return table client_id -> true
function State.get_known_clients()
    return state.known_clients
end

--------------------------------------------------------------------------------
-- Net ID Ownership
--------------------------------------------------------------------------------

--- Set owner of a net_id
--- @param net_id number
--- @param client_id number
function State.set_owner(net_id, client_id)
    state.net_id_owners[net_id] = client_id
end

--- Get owner of a net_id
--- @param net_id number
--- @return number|nil
function State.get_owner(net_id)
    return state.net_id_owners[net_id]
end

--- Clear owner of a net_id
--- @param net_id number
function State.clear_owner(net_id)
    state.net_id_owners[net_id] = nil
end

--------------------------------------------------------------------------------
-- Sync Types Tracking (for query optimization)
--------------------------------------------------------------------------------

--- Add sync types from a sync_components table
--- @param sync_components table
function State.add_sync_types(sync_components)
    for comp_name, _ in pairs(sync_components) do
        state.sync_types[comp_name] = (state.sync_types[comp_name] or 0) + 1
    end
end

--- Remove sync types (when entity despawns)
--- @param sync_components table
function State.remove_sync_types(sync_components)
    for comp_name, _ in pairs(sync_components) do
        local count = (state.sync_types[comp_name] or 1) - 1
        if count <= 0 then
            state.sync_types[comp_name] = nil
        else
            state.sync_types[comp_name] = count
        end
    end
end

--- Get all active sync types
--- @return table component_name -> count
function State.get_sync_types()
    return state.sync_types
end

--------------------------------------------------------------------------------
-- Pending Children (deferred spawn for parent dependencies)
--------------------------------------------------------------------------------

--- Add pending child
--- @param net_id number
--- @param msg table
--- @param owner_client number
function State.add_pending_child(net_id, msg, owner_client)
    state.pending_children[net_id] = { msg = msg, owner_client = owner_client }
    state.pending_children_time[net_id] = os.clock()
end

--- Get and remove pending child
--- @param net_id number
--- @return table|nil { msg, owner_client }
function State.take_pending_child(net_id)
    local child = state.pending_children[net_id]
    state.pending_children[net_id] = nil
    state.pending_children_time[net_id] = nil
    return child
end

--- Get all pending children for a parent
--- @param parent_net_id number
--- @return table net_id -> { msg, owner_client }
function State.get_pending_children_for_parent(parent_net_id)
    local result = {}
    for net_id, data in pairs(state.pending_children) do
        if data.msg.payload and data.msg.payload.parent_net_id == parent_net_id then
            result[net_id] = data
        end
    end
    return result
end

--- Clean up timed-out pending children
--- @param timeout number Seconds before timing out
--- @return table List of timed out net_ids
function State.cleanup_pending_children(timeout)
    local now = os.clock()
    local timed_out = {}
    
    for net_id, timestamp in pairs(state.pending_children_time) do
        if (now - timestamp) > timeout then
            table.insert(timed_out, net_id)
            state.pending_children[net_id] = nil
            state.pending_children_time[net_id] = nil
        end
    end
    
    return timed_out
end

return State
