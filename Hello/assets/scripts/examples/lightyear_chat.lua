-- Chat Example - Lightyear + Zero Rust Philosophy
-- All chat logic in Lua, reads MessageEvent<LuaMessage> from Bevy events
-- Sends messages via Lightyear Client/Server resources

print("=== Lightyear Chat Example Starting ===")
print("Mode: " .. (IS_CLIENT_MODE and "CLIENT" or "SERVER"))

-- Configuration
local MAX_MESSAGES = 50
local CHAT_WIDTH = 600
local CHAT_HEIGHT = 400
local INPUT_HEIGHT = 40

-- State
local chat_messages = {}
local current_input = ""
local message_entities = {}
local input_entity_ref = nil
local is_typing = false
local input_focused = false
local cursor_timer = 0
local cursor_visible = true
local cursor_x = 0
local cursor_y = 0

--------------------------------------------------------------------------------
-- Lightyear Message System (Zero Rust - all logic in Lua)
--------------------------------------------------------------------------------

-- Send a LuaMessage via Lightyear
-- On client: sends to server
-- On server: broadcasts to all clients
function send_message_internal(message_type, data)
    -- Convert Lua table to JSON string
    local data_json = "{" .. table.concat({
        '"sender": "' .. (data.sender or "Unknown") .. '"',
        '"content": "' .. (data.content or "") .. '"',
        data.messages and ('"messages": "' .. data.messages .. '"') or ''
    }, ", ") .. "}"
    
    -- Call Rust binding (registered globally)
    send_lua_message(message_type, data_json)
    
    if IS_CLIENT_MODE then
        print("📤 Client sent: " .. message_type)
    else
        print("📡 Server broadcast: " .. message_type)
    end
end

-- Read messages from queue
function receive_lua_messages_internal()
    -- Call Rust binding (registered globally)
    local messages = receive_lua_messages()
    
    if messages then
        for i, msg in ipairs(messages) do
            -- Parse JSON data
            local data = {}
            for key, value in msg.data:gmatch('"([^"]+)": "([^"]+)"') do
                data[key] = value
            end
            
            handle_message(msg.message_type, data)
        end
    end
end

--------------------------------------------------------------------------------
-- Chat-Specific Message Handlers (Zero Rust - all logic here)
--------------------------------------------------------------------------------

function handle_message(message_type, data)
    if message_type == "chat" then
        add_chat_message(data.sender, data.content)
    elseif message_type == "history" then
        print("📥 Received chat history")
        if data.messages then
            for msg in data.messages:gmatch("[^|]+") do
                local sender, content = msg:match("([^:]+):(.*)")
                if sender and content then
                    add_chat_message(sender, content)
                end
            end
        end
    end
end

function handle_server_message(client_id, message_type, data)
    if message_type == "chat" then
        local sender = data.sender or ("Player" .. client_id)
        local content = data.content or ""
        
        print("💬 Server: Chat from client " .. client_id .. ": " .. content)
        
        -- Store in history
        table.insert(chat_messages, {sender = sender, content = content})
        if #chat_messages > MAX_MESSAGES then
            table.remove(chat_messages, 1)
        end
        
        -- Broadcast to all clients
        send_message_internal("chat", {
            sender = sender,
            content = content
        })
        
    elseif message_type == "request_history" then
        -- Send chat history to new client
        local history_str = ""
        for i, msg in ipairs(chat_messages) do
            if i > 1 then history_str = history_str .. "|" end
            history_str = history_str .. msg.sender .. ":" .. msg.content
        end
        
        -- Note: Would need to send to specific client, not broadcast
        send_message_internal("history", {messages = history_str})
        print("📚 Sent history to client " .. client_id)
    end
end

--------------------------------------------------------------------------------
-- UI and Display
--------------------------------------------------------------------------------

function add_chat_message(sender, content)
    local message_text = sender .. ": " .. content
    print("💬 " .. message_text)
    
    table.insert(chat_messages, {sender = sender, content = content})
    if #chat_messages > MAX_MESSAGES then
        table.remove(chat_messages, 1)
    end
    
    update_chat_display()
end

function update_chat_display()
    -- Clear old message entities
    for _, entity_id in ipairs(message_entities) do
        despawn(entity_id)
    end
    message_entities = {}
    
    -- Display recent messages
    local display_count = math.min(#chat_messages, 15)
    local start_index = #chat_messages - display_count + 1
    
    for i = 0, display_count - 1 do
        local msg_index = start_index + i
        if chat_messages[msg_index] then
            local msg = chat_messages[msg_index]
            local y_pos = 20 + CHAT_HEIGHT - 40 - (i * 20)
            
            local entity_id = spawn({
                Node = {
                    position_type = "Absolute",
                    left = {Px = 30},
                    top = {Px = y_pos},
                    width = {Px = CHAT_WIDTH - 40},
                    height = {Px = 18}
                },
                Text = { text = msg.sender .. ": " .. msg.content },
                TextFont = { font_size = 14 },
                TextColor = { color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0} }
            })
            
            table.insert(message_entities, entity_id)
        end
    end
end

function update_input_display(world)
    -- Lazy-load the input entity reference on first call
    if not input_entity_ref then
        local entities = world:query({"InputField"}, nil)
        if #entities > 0 then
            input_entity_ref = entities[1]
        end
    end
    
    if input_entity_ref then
        local cursor = ""
        if input_focused then
            cursor = cursor_visible and "|" or " "
        end
        input_entity_ref:set("Text", { text = "> " .. current_input .. cursor })
    end
end

function send_current_message(world)
    if current_input ~= "" then
        send_message_internal("chat", {
            sender = "Player",
            content = current_input
        })
        current_input = ""
        is_typing = false
        update_input_display(world)
    end
end

--------------------------------------------------------------------------------
-- Initialize UI
--------------------------------------------------------------------------------

-- Chat background
spawn({
    Node = {
        position_type = "Absolute",
        left = {Px = 10},
        top = {Px = 10},
        width = {Px = CHAT_WIDTH},
        height = {Px = CHAT_HEIGHT}
    },
    BackgroundColor = { color = {r = 0.1, g = 0.1, b = 0.15, a = 0.95} },
    ZIndex = { index = -1 }
})

-- Chat title
spawn({
    Node = {
        position_type = "Absolute",
        left = {Px = 20},
        top = {Px = 15},
        width = {Px = CHAT_WIDTH - 20},
        height = {Px = 30}
    },
    Text = { text = (IS_CLIENT_MODE and "Chat Client" or "Chat Server") },
    TextFont = { font_size = 20 },
    TextColor = { color = {r = 0.3, g = 0.8, b = 1.0, a = 1.0} }
})

if IS_CLIENT_MODE then
    -- Input background
    spawn({
        Node = {
            position_type = "Absolute",
            left = {Px = 10},
            top = {Px = 20 + CHAT_HEIGHT},
            width = {Px = CHAT_WIDTH},
            height = {Px = INPUT_HEIGHT}
        },
        BackgroundColor = { color = {r = 0.15, g = 0.15, b = 0.2, a = 0.95} },
        ZIndex = { index = -1 }
    })
    
    -- Input text (will be queried and stored on first update)
    spawn({
        Node = {
            position_type = "Absolute",
            left = {Px = 20},
            top = {Px = 30 + CHAT_HEIGHT},
            width = {Px = CHAT_WIDTH - 40},
            height = {Px = INPUT_HEIGHT - 20}
        },
        Text = { text = "> " },
        TextFont = { font_size = 16 },
        TextColor = { color = {r = 0.9, g = 0.9, b = 0.9, a = 1.0} },
        InputField = {}  -- Tag component to identify the input field
    })
    
    -- Instructions
    spawn({
        Node = {
            position_type = "Absolute",
            left = {Px = 20},
            top = {Px = 40 + CHAT_HEIGHT + INPUT_HEIGHT},
            width = {Px = CHAT_WIDTH},
            height = {Px = 60}
        },
        Text = { text = "Type to chat, press ENTER to send\n(Simple demo: letters, numbers, space only)" },
        TextFont = { font_size = 12 },
        TextColor = { color = {r = 0.6, g = 0.6, b = 0.6, a = 1.0} }
    })
end

print("✅ UI Created")

--------------------------------------------------------------------------------
-- Systems
--------------------------------------------------------------------------------

if IS_CLIENT_MODE then
    -- Client input system
    register_system("Update", function(world)
        -- Lazy-load the input entity reference on first update
        if not input_entity_ref then
            local entities = world:query({"InputField"}, nil)
            if #entities > 0 then
                input_entity_ref = entities[1]
            end
        end
        
        -- Handle blinking cursor (toggle every 0.5s)
        cursor_timer = (cursor_timer or 0) + (world:delta_time() or 0.016)
        if cursor_timer > 0.5 then
            cursor_timer = 0
            cursor_visible = not cursor_visible
            if input_focused and input_entity_ref then update_input_display(world) end
        end

        -- Track cursor position from CursorMoved events
        local cursor_events = world:query_events("bevy_window::event::CursorMoved")
        if cursor_events then
            for _, event in ipairs(cursor_events) do
                if event.position then
                    cursor_x = tonumber(event.position.x or event.position[1] or 0)
                    cursor_y = tonumber(event.position.y or event.position[2] or 0)
                end
            end
        end

        -- Handle mouse input for focus
        local mouse_events = world:query_events("bevy_input::mouse::MouseButtonInput")
        if mouse_events and input_entity_ref then
            for _, event in ipairs(mouse_events) do
                if event.state and event.state.Pressed and event.button and event.button.Left then
                    -- Since Node bounds may not be accessible, use hardcoded bounds from spawn
                    local left = 20
                    local top = 30 + CHAT_HEIGHT
                    local width = CHAT_WIDTH - 40
                    local height = INPUT_HEIGHT - 20
                    
                    print("click", cursor_x, cursor_y, left, top, width, height)
                    if cursor_x >= left and cursor_x <= left + width and cursor_y >= top and cursor_y <= top + height then
                        print("focused")
                        input_focused = true
                        update_input_display(world)
                    else
                        print("unfocused")
                        input_focused = false
                        update_input_display(world)
                    end
                end
            end
        end

        -- Handle keyboard input for typing (only if focused)
        if input_focused then
            local key_events = world:query_events("bevy_input::keyboard::KeyboardInput")
            for _, event in ipairs(key_events) do
                if event.state and event.state.Pressed then
                    local key = event.key_code
                    if key.Enter or key.Return then
                        send_current_message(world)
                    elseif key.Backspace or key.Back then
                        if #current_input > 0 then
                            current_input = current_input:sub(1, -2)
                            update_input_display(world)
                        end
                    elseif key.Space then
                        current_input = current_input .. " "
                        is_typing = true
                        update_input_display(world)
                    elseif event.text and #event.text.Some == 1 then
                        current_input = current_input .. event.text.Some[1]:sub(2,2)
                        is_typing = true
                        update_input_display(world)
                    end
                end
            end
        end

        -- Receive messages from server
        receive_lua_messages_internal()
    end)
    
    -- Request chat history on connection
    local requested_history = false
    register_system("Update", function(world)
        if not requested_history then
            -- TODO: Check if connected via Lightyear
            -- For now, just request once after a delay
            requested_history = true
            send_message_internal("request_history", {})
            print("📨 Requested chat history from server")
        end
    end)
    
else
    -- Server message handling system
    register_system("Update", function(world)
        receive_lua_messages_internal()
    end)
    
    -- Server status display
    local status_entity = spawn({
        Node = {
            position_type = "Absolute",
            left = {Px = 20},
            top = {Px = CHAT_HEIGHT - 30},
            width = {Px = CHAT_WIDTH - 40},
            height = {Px = 20}
        },
        Text = { text = "Lightyear Server Ready (awaiting Lua bindings)" },
        TextFont = { font_size = 14 },
        TextColor = { color = {r = 0.7, g = 0.7, b = 0.3, a = 1.0} }
    })
    
    register_system("Update", function(world)
        -- TODO: Get connected client count from Lightyear
        -- Expected API: world:call_resource_method("LightyearServer", "client_count")
        local count = 0  -- Placeholder
        
        local entities = world:query({"Text"}, nil)
        for _, entity in ipairs(entities) do
            if entity:id() == status_entity then
                entity:set("Text", { text = "Connected clients: " .. count .. " (awaiting bindings)" })
            end
        end
    end)
end

print("=== Lightyear Chat Example Ready ===")
print("⚠ NOTE: This script is ready but Lightyear Lua bindings need to be implemented")
print("Required:")
print("  1. Expose MessageEvent<LuaMessage> as Bevy event to Lua")
print("  2. Add send_message() API for Lightyear Client/Server in Lua")
print("  3. Once Lightyear compiles with Bevy 0.16, add Lua resource bindings")
print(IS_CLIENT_MODE and "Type to chat (local UI only until bindings ready)!" or "Server mode (awaiting bindings)")

