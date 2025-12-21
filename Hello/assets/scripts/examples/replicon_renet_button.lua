-- Hybrid Button Example
-- Uses Renet for Lua event messaging + Replicon ready for component replication

print("=== Hybrid Renet + Replicon Button ===")

-- Parse CLI arguments
local args = get_args()
local mode = "singleplayer"
local port = 5000  
local ip = "127.0.0.1"

if args[2] then
    mode = args[2]
    for i = 3, #args do
        if args[i] == "--port" or args[i] == "-p" then
            port = tonumber(args[i + 1]) or port
        elseif args[i] == "--ip" then
            ip = args[i + 1] or ip
        end
    end
end

print("Mode: " .. mode)
print("Port: " .. port)
if mode == "client" then print("Server IP: " .. ip) end

-- Set up networking (Renet for messages)
if mode == "server" then
    print("Starting server...")
    insert_resource("RenetServer", {})
    insert_resource("NetcodeServerTransport", { port = port, max_clients = 10 })
elseif mode == "client" then
    print("Connecting to server...")
    insert_resource("RenetClient", {})
    insert_resource("NetcodeClientTransport", { server_addr = ip, port = port })
end

-- Shared button state (local to each instance, synced via Renet events)
local button_enabled = false

-- Spawn UI
print("Spawning UI...")

-- Root container
spawn({
    Node = {
        width = { Percent = 100.0 },
        height = { Percent = 100.0 },
        align_items = "Center",
        justify_content = "Center",
    },
})

-- Button
local button_id = spawn({
    Button = {},
    Node = {
        width = { Px = 150.0 },
        height = { Px = 65.0 },
        border = { left = 5.0, right = 5.0, top = 5.0, bottom = 5.0 },
        justify_content = "Center",
        align_items = "Center",
    },
    BackgroundColor = { color = { r = 0.15, g = 0.15, b = 0.15, a = 1.0 } },
    BorderColor = { color = { r = 1.0, g = 1.0, b = 1.0, a = 1.0 } },
    BorderRadius = { 
        top_left = 16.0,
        top_right = 16.0, 
        bottom_left = 16.0,
        bottom_right = 16.0 
    },
    Text = { text = button_enabled and "On" or "Off" },
    TextFont = { font_size = 30.0 },
    TextColor = { color = { r = 1.0, g = 1.0, b = 1.0, a = 1.0 } },
})

-- Track button click state
local last_interaction = "None"

-- Update button background and handle clicks based on interaction
register_system("Update", function(world)
    local buttons = world:query({"Button", "Interaction"}, nil)
    
    for _, entity in ipairs(buttons) do
        local interaction = entity:get("Interaction")
        
        -- Detect click (transition from Pressed to Hovered/None)
        if last_interaction == "Pressed" and (interaction == "Hovered" or interaction == "None") then
            print("Button clicked!")
            
            if mode == "client" then
                -- Client: send toggle request to server via Renet
                print("Client: Sending toggle request to server")
                world:send_network_event({ action = "toggle" })
            elseif mode == "server" or mode == "singleplayer" then
                -- Server/singleplayer: toggle immediately and broadcast
                button_enabled = not button_enabled
                print("Server: Toggled button to " .. (button_enabled and "On" or "Off"))
                
                if mode == "server" then
                    -- Broadcast new state to all clients via Renet
                    world:broadcast_event({ action = "set_state", enabled = button_enabled })
                end
            end
        end
        
        last_interaction = interaction
        
        -- Update background color
        local color
        if interaction == "Pressed" then
            color = { r = 0.35, g = 0.75, b = 0.35, a = 1.0 }
        elseif interaction == "Hovered" then
            color = { r = 0.25, g = 0.25, b = 0.25, a = 1.0 }
        else
            color = { r = 0.15, g = 0.15, b = 0.15, a = 1.0 }
        end
        
        entity:set({BackgroundColor = { color = color }})
    end
end)

-- Update text based on button state
register_system("Update", function(world)
    local text_value = button_enabled and "On" or "Off"
    
    local texts = world:query({"Text"}, nil)
    for _, entity in ipairs(texts) do
        local current_text = entity:get("Text")
        if current_text.text ~= text_value then
            entity:set({Text = { text = text_value }})
        end
    end
end)

-- Handle incoming network events
if mode == "server" then
    -- Server: handle client requests and track connections
    local last_count = 0
    
    register_system("Update", function(world)
        -- Track connected clients
        if world:query_resource("RenetServer") then
            local clients = world:call_resource_method("RenetServer", "clients_id", {})
            if clients then
                local count = #clients
                if count ~= last_count then
                    last_count = count
                    print("🔗 Connected clients: " .. count)
                    for i, client_id in ipairs(clients) do
                        print("  - Client ID: " .. tostring(client_id))
                    end
                end
            end
        end
        
        -- Handle client events via Renet
        local events = world:read_network_events()
        for _, event in ipairs(events) do
            if event.action == "toggle" then
                print("Server: Received toggle request from client")
                button_enabled = not button_enabled
                print("Server: Toggled button to " .. (button_enabled and "On" or "Off"))
                
                -- Broadcast new state to all clients
                world:broadcast_event({ action = "set_state", enabled = button_enabled })
            end
        end
    end)
elseif mode == "client" then
    -- Client: handle server state updates
    register_system("Update", function(world)
        local events = world:read_network_events()
        for _, event in ipairs(events) do
            if event.action == "set_state" then
                print("Client: Received state update from server: " .. (event.enabled and "On" or "Off"))
                button_enabled = event.enabled
            end
        end
    end)
end

print("=== Systems Registered ===")
print("Click the button to toggle its state!")
if mode ~= "singleplayer" then
    print("State changes sync via Renet messaging!")
    print("Replicon available for future component replication.")
end
