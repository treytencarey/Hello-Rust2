-- Network Server Module v2
-- Handles server hosting with ECS-based message processing
--
-- Usage:
--   local NetServer2 = require("modules/net_server2.lua")
--   NetServer2.start(5000, 10)
--   register_system("Update", function(world) NetServer2.update(world) end)

local NetSync2 = require("modules/net_sync2.lua")

local NetServer2 = {}

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

-- Global filter function (optional)
local global_filter_fn = nil

--- Set callback for client connect
function NetServer2.on_client_connect(fn)
    callbacks.on_client_connect = fn
end

--- Set callback for client disconnect
function NetServer2.on_client_disconnect(fn)
    callbacks.on_client_disconnect = fn
end

--- Set global filter function
--- @param fn function(world, client_id, entity, msg) -> boolean
function NetServer2.set_filter(fn)
    global_filter_fn = fn
end

--------------------------------------------------------------------------------
-- Server Lifecycle
--------------------------------------------------------------------------------

--- Start the server
function NetServer2.start(port, max_clients)
    insert_resource("RenetServer", {})
    insert_resource("NetcodeServerTransport", {
        port = port,
        max_clients = max_clients or 10
    })
    print(string.format("[NET_SERVER2] Started on port %d", port))
end

--------------------------------------------------------------------------------
-- Send/Receive Wrappers
--------------------------------------------------------------------------------

--- Send message to client
local function send_to_client(world, client_id, channel, msg_str)
    print(string.format("[NET_SERVER2] Sending message to client %d: %s", client_id, msg_str))
    pcall(function()
        world:call_resource_method("RenetServer", "send_message", client_id, channel, msg_str)
    end)
end

--- Receive from client
local function receive_from_client(world, client_id, channel)
    local ok, msg = pcall(function()
        return world:call_resource_method("RenetServer", "receive_message", client_id, channel)
    end)
    if ok and msg then
        print(string.format("[NET_SERVER2] Received message from client %d: %s", client_id, msg))
        return msg
    end
    return nil
end

--- Get connected clients
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
-- Filter Logic
--------------------------------------------------------------------------------

--- Check if message passes filter for a specific client
local function passes_filter(world, client_id, msg)
    -- Get entity for per-entity filter
    local entity = nil
    if msg.owner_client then
        entity = world:get_entity(msg.owner_client)
    end

    -- Check per-entity filter first
    if entity then
        local sync = entity:get(NetSync2.MARKER)
        if sync and sync.filter_fn then
            local pass = sync.filter_fn(world, client_id, entity, msg)
            if not pass then
                return false
            end
        end
    end

    -- Check global filter
    if global_filter_fn then
        return global_filter_fn(world, client_id, entity, msg)
    end

    return true
end

--- Check and send despawns/spawns for reverse visibility
--- When entity owned by moving_client fails filter for other_client,
--- check if other_client's entities should be despawned from moving_client (or spawned if they should now be visible)
local function check_reverse_visibility(world, moving_client, other_client)
    if not moving_client or moving_client == other_client then
        return
    end

    -- Check all entities owned by other_client
    local other_net_ids = NetSync2.get_net_ids_for_client(other_client)

    for _, other_net_id in ipairs(other_net_ids) do
        local moving_client_knows = NetSync2.client_knows_entity(moving_client, other_net_id)

        -- Build synthetic message for filter check
        local other_entity_id = NetSync2.get_entity(other_net_id)
        local check_msg = {
            net_id = other_net_id,
            owner_client = other_entity_id,
        }

        -- Check if this entity passes filter for moving_client
        local filter_passes = passes_filter(world, moving_client, check_msg)

        if filter_passes and not moving_client_knows then
            -- Filter passes but client doesn't know entity - send spawn
            local entity = world:get_entity(other_entity_id)
            if entity then
                local spawn_msg = NetSync2.build_spawn_msg(world, entity, other_net_id)
                if spawn_msg then
                    local spawn_payload = {
                        type = "spawn",
                        owner_client = other_client,
                        net_id = other_net_id,
                        authority = spawn_msg.payload.authority,
                        parent_net_id = spawn_msg.payload.parent_net_id,
                        components = spawn_msg.payload.components,
                    }
                    local spawn_str = NetSync2.json_encode(spawn_payload)
                    send_to_client(world, moving_client, CHANNEL_RELIABLE, spawn_str)
                    NetSync2.add_to_client_scope(moving_client, other_net_id)
                end
            end
        elseif not filter_passes and moving_client_knows then
            -- Filter fails and client knows entity - send despawn
            local despawn_payload = {
                type = "despawn",
                owner_client = other_client,
                net_id = other_net_id,
            }
            local despawn_str = NetSync2.json_encode(despawn_payload)
            send_to_client(world, moving_client, CHANNEL_RELIABLE, despawn_str)
            NetSync2.remove_from_client_scope(moving_client, other_net_id)
        end
    end
end

--------------------------------------------------------------------------------
-- Receive System - Receive from Renet, spawn NetSyncInbound entities
--------------------------------------------------------------------------------

--- Receive messages from clients and spawn NetSyncInbound entities
function NetServer2.receive_system(world)
    local clients = get_clients(world)

    for _, client_id in ipairs(clients) do
        -- Reliable channel
        while true do
            local msg_str = receive_from_client(world, client_id, CHANNEL_RELIABLE)
            if not msg_str then break end

            local msg = NetSync2.json_decode(msg_str)
            if msg then
                spawn({
                    [NetSync2.INBOUND] = {
                        msg_type = msg.type or msg.msg_type,
                        channel = CHANNEL_RELIABLE,
                        owner_client = client_id,
                        net_id = msg.net_id,
                        payload = msg,
                    }
                })
            end
        end

        -- Unreliable channel
        while true do
            local msg_str = receive_from_client(world, client_id, CHANNEL_UNRELIABLE)
            if not msg_str then break end

            local msg = NetSync2.json_decode(msg_str)
            if msg then
                spawn({
                    [NetSync2.INBOUND] = {
                        msg_type = msg.type or msg.msg_type,
                        channel = CHANNEL_UNRELIABLE,
                        owner_client = client_id,
                        net_id = msg.net_id,
                        payload = msg,
                    }
                })
            end
        end
    end
end

--------------------------------------------------------------------------------
-- Message Processor - Query NetSyncOutbound, send via Renet, apply filtering
--------------------------------------------------------------------------------

--- Process outbound messages and send via Renet
function NetServer2.message_processor(world)
    local outbound = world:query({NetSync2.OUTBOUND, "ScriptOwned"})
    
    for _, msg_entity in ipairs(outbound) do
        local msg = msg_entity:get(NetSync2.OUTBOUND)
        local net_id = msg.net_id

        -- Skip messages from other instances (e.g. server/client instanced scripts)
        local script_owned = msg_entity:get("ScriptOwned")
        if script_owned.instance_id ~= __INSTANCE_ID__ then
            goto continue_entities
        end

        -- Determine target clients
        local targets = msg.target_clients or get_clients(world)
        local exclude = msg.exclude_clients or {}
        local exclude_set = {}
        for _, cid in ipairs(exclude) do
            exclude_set[cid] = true
        end

        -- Serialize payload based on msg_type
        local payload = {
            type = msg.msg_type,
            net_id = net_id,
            owner_client = msg.owner_client,
        }

        if msg.msg_type == "spawn" then
            payload.authority = msg.payload.authority
            payload.parent_net_id = msg.payload.parent_net_id
            payload.components = msg.payload.components
        elseif msg.msg_type == "update" then
            payload.components = msg.payload.components
            payload.seq = msg.payload.seq
            -- ack_seq is per-client (for owner)
        elseif msg.msg_type == "despawn" then
            -- Just type and net_id
        elseif msg.msg_type == "owner_change" then
            payload.new_owner = msg.payload.new_owner
        elseif msg.msg_type == "client_id" then
            payload.client_id = msg.payload.client_id
        end

        -- Send to each target client
        for _, client_id in ipairs(targets) do
            if not exclude_set[client_id] then
                local client_knew_entity = NetSync2.client_knows_entity(client_id, net_id)
                local filter_passes = passes_filter(world, client_id, msg)
                
                if filter_passes then
                    -- Handle spawn vs update scope
                    if msg.msg_type == "spawn" then
                        -- Add to scope
                        NetSync2.add_to_client_scope(client_id, net_id)
                        local msg_str = NetSync2.json_encode(payload)
                        send_to_client(world, client_id, msg.channel, msg_str)

                    elseif msg.msg_type == "update" then
                        if not client_knew_entity then
                            -- Client doesn't know entity - send spawn first
                            local entity = world:get_entity(msg.owner_client)
                            if entity then
                                local spawn_msg = NetSync2.build_spawn_msg(world, entity, net_id)
                                if spawn_msg then
                                    local spawn_payload = {
                                        type = "spawn",
                                        net_id = net_id,
                                        owner_client = spawn_msg.owner_client,
                                        authority = spawn_msg.payload.authority,
                                        parent_net_id = spawn_msg.payload.parent_net_id,
                                        components = spawn_msg.payload.components,
                                    }
                                    local spawn_str = NetSync2.json_encode(spawn_payload)
                                    send_to_client(world, client_id, CHANNEL_RELIABLE, spawn_str)
                                    NetSync2.add_to_client_scope(client_id, net_id)

                                    -- Check reverse visibility: if A spawned for B,
                                    -- also check if B's entities should be spawned for A
                                    local moving_client = NetSync2.get_net_id_owner(net_id)
                                    check_reverse_visibility(world, moving_client, client_id)
                                end
                            end
                        end

                        -- Add ack_seq for entity owner
                        local owner = msg.owner_client
                        if owner and owner == client_id then
                            payload.ack_seq = msg.payload.ack_seq
                        else
                            payload.ack_seq = nil
                        end

                        local msg_str = NetSync2.json_encode(payload)
                        send_to_client(world, client_id, msg.channel, msg_str)

                    elseif msg.msg_type == "despawn" then
                        if client_knew_entity then
                            local msg_str = NetSync2.json_encode(payload)
                            send_to_client(world, client_id, msg.channel, msg_str)
                            NetSync2.remove_from_client_scope(client_id, net_id)
                        end

                    elseif msg.msg_type == "client_id" then
                        -- client_id message: send the client_id in payload
                        local client_id_payload = {
                            type = "client_id",
                            client_id = msg.payload.client_id,
                        }
                        local msg_str = NetSync2.json_encode(client_id_payload)
                        send_to_client(world, client_id, msg.channel, msg_str)

                    else
                        -- owner_change, etc.
                        local msg_str = NetSync2.json_encode(payload)
                        send_to_client(world, client_id, msg.channel, msg_str)
                    end

                else
                    -- Filter failed
                    if client_knew_entity then
                        -- Client knew entity but filter now fails - send despawn
                        local despawn_payload = {
                            type = "despawn",
                            net_id = net_id,
                            owner_client = msg.owner_client,
                        }
                        local despawn_str = NetSync2.json_encode(despawn_payload)
                        send_to_client(world, client_id, CHANNEL_RELIABLE, despawn_str)
                        NetSync2.remove_from_client_scope(client_id, net_id)
                        
                        -- Check reverse visibility: if A moved out of B's range,
                        -- also check if B's entities should be despawned from A
                        local moving_client = NetSync2.get_net_id_owner(net_id)
                        check_reverse_visibility(world, moving_client, client_id)
                    end
                    -- If they didn't know entity and filter fails, just skip (don't send)
                end
            end
        end

        -- Despawn the message entity
        despawn(msg_entity:id())

        ::continue_entities::
    end
end

--------------------------------------------------------------------------------
-- Update System
--------------------------------------------------------------------------------

--- Main update - call in your Update system
--- @param world userdata The ECS world
function NetServer2.update(world)
    local clients = get_clients(world)

    -- Helper for getting clients
    local function get_clients_fn()
        return clients
    end

    -- Initialize new clients
    for _, client_id in ipairs(clients) do
        local known = NetSync2.get_known_clients()
        local is_known = false
        for _, kc in ipairs(known) do
            if kc == client_id then
                is_known = true
                break
            end
        end

        if not is_known then
            -- Assign client prefix
            local prefix = next_client_prefix
            next_client_prefix = next_client_prefix + 1
            print(string.format("[NET_SERVER2] New client %s, assigned prefix %d", client_id, prefix))

            -- Initialize in NetSync2 (spawns client_id and entity spawn messages)
            NetSync2.on_client_connected(client_id, world)

            -- Call connect callback
            if callbacks.on_client_connect then
                callbacks.on_client_connect(client_id)
            end
        end
    end

    -- Step 1: Receive messages from clients
    NetServer2.receive_system(world)

    -- Step 2: Process inbound messages (apply to ECS)
    NetSync2.inbound_system(world)

    -- Step 3: Run outbound system (detect changes, spawn NetSyncOutbound)
    NetSync2.outbound_system(world, {
        get_clients = get_clients_fn,
    })

    -- Step 4: Process outbound messages (send via Renet)
    NetServer2.message_processor(world)

    -- Step 5: Detect disconnected clients
    local active = {}
    for _, client_id in ipairs(clients) do
        active[client_id] = true
    end

    for _, client_id in ipairs(NetSync2.get_known_clients()) do
        if not active[client_id] then
            print(string.format("[NET_SERVER2] Client %s disconnected", client_id))

            -- Handle disconnect in NetSync2
            NetSync2.on_client_disconnected(client_id, world, get_clients_fn)

            -- Process any despawn messages that were spawned
            NetServer2.message_processor(world)

            -- Call disconnect callback
            if callbacks.on_client_disconnect then
                callbacks.on_client_disconnect(client_id)
            end
        end
    end
end

print("[NET_SERVER2] Module loaded")

return NetServer2
