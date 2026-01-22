-- Network Client Module v2
-- Handles client connection with ECS-based message processing
--
-- Usage:
--   local NetClient2 = require("modules/net_client2.lua")
--   NetClient2.connect("127.0.0.1", 5000)
--   register_system("Update", function(world) NetClient2.update(world) end)

local NetSync2 = require("modules/net_sync2.lua")

local NetClient2 = {}

-- Channels
local CHANNEL_RELIABLE = 0
local CHANNEL_UNRELIABLE = 1

-- Connection state
local is_connected_flag = false

--------------------------------------------------------------------------------
-- Connection
--------------------------------------------------------------------------------

--- Connect to a server
function NetClient2.connect(server_addr, port)
    insert_resource("RenetClient", {})
    insert_resource("NetcodeClientTransport", {
        server_addr = server_addr,
        port = port
    })
    print(string.format("[NET_CLIENT2] Connecting to %s:%d", server_addr, port))
end

--- Check if connected
function NetClient2.is_connected(world)
    local ok, connected = pcall(function()
        return world:call_resource_method("RenetClient", "is_connected")
    end)
    is_connected_flag = ok and connected
    return is_connected_flag
end

--------------------------------------------------------------------------------
-- Send/Receive Wrappers
--------------------------------------------------------------------------------

--- Send to server
local function send_to_server(world, channel, msg_str)
    print(string.format("[NET_CLIENT2] Sending message to server: %s", msg_str))
    pcall(function()
        world:call_resource_method("RenetClient", "send_message", channel, msg_str)
    end)
end

--- Receive from server
local function receive_from_server(world, channel)
    local ok, msg = pcall(function()
        return world:call_resource_method("RenetClient", "receive_message", channel)
    end)
    if ok and msg then
        print(string.format("[NET_CLIENT2] Received message from server: %s", msg))
        return msg
    end
    return nil
end

--------------------------------------------------------------------------------
-- Receive System - Receive from Renet, spawn NetSyncInbound entities
--------------------------------------------------------------------------------

--- Receive messages from server and spawn NetSyncInbound entities
function NetClient2.receive_system(world)
    -- Reliable channel
    while true do
        local msg_str = receive_from_server(world, CHANNEL_RELIABLE)
        if not msg_str then break end

        local msg = NetSync2.json_decode(msg_str)
        if msg then
            spawn({
                [NetSync2.INBOUND] = {
                    msg_type = msg.type or msg.msg_type,
                    channel = CHANNEL_RELIABLE,
                    owner_client = msg.owner_client,
                    net_id = msg.net_id,
                    payload = msg,
                }
            })
        end
    end

    -- Unreliable channel
    while true do
        local msg_str = receive_from_server(world, CHANNEL_UNRELIABLE)
        if not msg_str then break end

        local msg = NetSync2.json_decode(msg_str)
        if msg then
            spawn({
                [NetSync2.INBOUND] = {
                    msg_type = msg.type or msg.msg_type,
                    channel = CHANNEL_UNRELIABLE,
                    owner_client = msg.owner_client,
                    net_id = msg.net_id,
                    payload = msg,
                }
            })
        end
    end
end

--------------------------------------------------------------------------------
-- Message Processor - Query NetSyncOutbound, send via Renet
--------------------------------------------------------------------------------

--- Process outbound messages and send to server
function NetClient2.message_processor(world)
    local outbound = world:query({NetSync2.OUTBOUND, "ScriptOwned"})

    for _, msg_entity in ipairs(outbound) do
        local msg = msg_entity:get(NetSync2.OUTBOUND)

        -- Skip messages from other instances (e.g. server/client instanced scripts)
        local script_owned = msg_entity:get("ScriptOwned")
        if script_owned.instance_id ~= __INSTANCE_ID__ then
            goto continue_entities
        end

        -- Serialize payload
        local payload = {
            type = msg.msg_type,
            net_id = msg.net_id,
            owner_client = msg.owner_client,
        }

        if msg.msg_type == "spawn" then
            payload.authority = msg.payload.authority
            payload.parent_net_id = msg.payload.parent_net_id
            payload.components = msg.payload.components
        elseif msg.msg_type == "update" then
            payload.components = msg.payload.components
            payload.seq = msg.payload.seq
        elseif msg.msg_type == "despawn" then
            -- Just type and net_id
        end

        local msg_str = NetSync2.json_encode(payload)
        send_to_server(world, msg.channel, msg_str)

        -- Despawn the message entity
        despawn(msg_entity:id())

        ::continue_entities::
    end
end

--------------------------------------------------------------------------------
-- Update System
--------------------------------------------------------------------------------

--- Main update - call in your Update system
function NetClient2.update(world)
    if not NetClient2.is_connected(world) then
        return
    end

    -- Step 1: Receive messages from server
    NetClient2.receive_system(world)

    -- Step 2: Process inbound messages (apply to ECS)
    NetSync2.inbound_system(world)

    -- Step 3: Run interpolation for remote entities
    NetSync2.interpolation_system(world)

    -- Step 4: Run prediction reconciliation for own entity
    -- DISABLED: Using modular_camera_controller.reconcile() instead which has proper replay support
    -- NetSync2.prediction_system(world)

    -- Step 5: Run outbound system (detect changes, spawn NetSyncOutbound)
    NetSync2.outbound_system(world, {
        get_clients = nil,
    })

    -- Step 6: Process outbound messages (send to server)
    NetClient2.message_processor(world)
end

--------------------------------------------------------------------------------
-- Helpers
--------------------------------------------------------------------------------

--- Get next net_id (delegates to NetSync2)
function NetClient2.next_net_id()
    return NetSync2.next_net_id()
end

--- Set client prefix (called when server assigns our ID)
function NetClient2.set_prefix(prefix)
    NetSync2.set_client_prefix(prefix)
end

--- Get my assigned client_id
function NetClient2.get_my_client_id()
    return NetSync2.get_my_client_id()
end

--- Check if entity is owned by me
--- @param world userdata
--- @param entity userdata|number
function NetClient2.is_my_entity(world, entity)
    return NetSync2.is_my_entity(world, entity)
end

print("[NET_CLIENT2] Module loaded, instance_id: " .. __INSTANCE_ID__)

return NetClient2
