-- Pure Lua File Upload Example
-- Demonstrates "Zero Rust" networking - all logic in Lua
-- Client uploads C:/Users/treyt/Desktop/clfonr.png to server

print("=== Pure Lua File Upload Example ===")
print("Client will upload: C:/Users/treyt/Desktop/clfonr.png")
print("")

-- Network configuration
local SERVER_PORT = 5001
local FILE_TO_UPLOAD = "C:/Users/treyt/Desktop/cIfonr.png"

-- Determine mode (server or client)
local is_server = not IS_CLIENT_MODE
local is_client = IS_CLIENT_MODE

-- Initialize networking with proper sequencing
if is_server then
    print("🖥️  Starting SERVER on port " .. SERVER_PORT)
    
    -- Create server resources in specific order:
    -- 1. First create RenetServer (this sets up the connection config)
    insert_resource("RenetServer", {})
    
    -- 2. Then create transport (this binds the socket and sets up authentication)
    insert_resource("NetcodeServerTransport", {
        port = SERVER_PORT,
        max_clients = 10
    })
    
    print("✅ Server ready - waiting for file upload...")
    print("   Listening on 0.0.0.0:" .. SERVER_PORT)
else
    print("💻 Starting CLIENT - connecting to 127.0.0.1:" .. SERVER_PORT)
    
    -- Create client resources in specific order:
    -- 1. First create RenetClient (this sets up the connection config)  
    insert_resource("RenetClient", {})
    
    -- 2. Then create transport with proper delay
    -- The transport generates a connect token with authentication
    insert_resource("NetcodeClientTransport", {
        server_addr = "127.0.0.1",
        port = SERVER_PORT
    })
    
    print("✅ Client ready - will upload file on startup")
    print("   Connecting to 127.0.0.1:" .. SERVER_PORT)
end

-- Client: Upload file on startup
if is_client then
    local upload_attempted = false
    local connection_confirmed = false
    
    local frame_count = 0
    register_system("upload_file", function(world)
        frame_count = frame_count + 1
        
        -- Only upload once
        if upload_attempted then
            return
        end
        
        -- Give server time to fully initialize (critical when running as same binary)
        -- Server needs time to bind socket, set up authentication, and be ready
        if frame_count < 300 then  -- Wait 5 seconds (60 fps * 5)
            if frame_count % 60 == 0 then
                print("CLIENT: Waiting for server initialization... (" .. math.floor(frame_count/60) .. "s)")
            end
            return
        end
        
        -- Check if connected
        local success, connected = pcall(function()
            return world:call_resource_method("RenetClient", "is_connected")
        end)
        
        if not success then
            if frame_count % 60 == 0 then
                print("CLIENT: RenetClient not ready yet... (error: " .. tostring(connected) .. ")")
            end
            return
        end
        
        if not connected then
            if frame_count % 60 == 0 then
                local elapsed = math.floor(frame_count/60)
                print("CLIENT: Still connecting to server... (" .. elapsed .. "s elapsed)")
            end
            return  -- Not connected yet
        end
        
        -- Connection confirmed - send file IMMEDIATELY before protocol check
        upload_attempted = true
        print("\n📤 CLIENT: Connected! Uploading file immediately...")
        
        -- Read file
        local file_success, file_data = pcall(function()
            return read_file_bytes(FILE_TO_UPLOAD)
        end)
        
        if not file_success then
            print("❌ CLIENT: Failed to read file: " .. tostring(file_data))
            return
        end
        
        local file_size = string.len(file_data)
        print("✅ CLIENT: Read " .. file_size .. " bytes from file")
        
        -- Extract filename
        local filename = FILE_TO_UPLOAD:match("([^/\\]+)$") or "uploaded_file.png"
        print("📝 CLIENT: Filename: " .. filename)
        
        -- Encode message: filename_length (2 bytes) + filename + file_data
        local filename_len = string.len(filename)
        local message = string.char(
            math.floor(filename_len / 256),  -- high byte
            filename_len % 256                -- low byte
        ) .. filename .. file_data
        
        -- Send to server on channel 1
        local send_success, send_error = pcall(function()
            world:call_resource_method("RenetClient", "send_message", 1, message)
        end)
        
        if send_success then
            print("✅ CLIENT: File uploaded successfully! (" .. file_size .. " bytes)")
        else
            print("❌ CLIENT: Failed to send: " .. tostring(send_error))
        end
    end)
end

-- Server: Receive file uploads (runs in PreUpdate to process before replicon checks)
if is_server then
    local files_received = 0
    local debug_counter = 0
    local known_clients = {}  -- Cache client IDs to check messages even after disconnect
    
    -- Priority system - runs BEFORE replicon's protocol check
    register_system("receive_uploads_priority", function(world)
        debug_counter = debug_counter + 1
        
        -- Check if server resource exists
        if not world:query_resource("RenetServer") then
            return
        end
        
        -- Get current connected client IDs and add to known clients
        local success, clients = pcall(function()
            return world:call_resource_method("RenetServer", "clients_id")
        end)
        
        if success and clients then
            for _, client_id in ipairs(clients) do
                known_clients[client_id] = true
            end
        end
        
        -- Check messages from ALL known clients (even if currently disconnected)
        -- This ensures we process messages that arrived before disconnect
        for client_id, _ in pairs(known_clients) do
            local success, message = pcall(function()
                return world:call_resource_method("RenetServer", "receive_message", client_id, 1)
            end)
            
            if not success then
                -- Error calling receive_message - log it
                if debug_counter % 60 == 0 then
                    print("SERVER DEBUG: Failed to receive from client " .. client_id .. ": " .. tostring(message))
                end
            elseif message then
                if string.len(message) == 0 then
                    -- Empty message
                    if debug_counter % 60 == 0 then
                        print("SERVER DEBUG: Empty message from client " .. client_id)
                    end
                elseif string.len(message) < 3 then
                    -- Too short to be valid
                    print("SERVER DEBUG: Message too short from client " .. client_id .. " (" .. string.len(message) .. " bytes)")
                else
                    -- Valid message!
                    files_received = files_received + 1
                    
                    print("\n📥 SERVER: Received upload from client " .. client_id)
                    
                    -- Decode message
                    local filename_len_high = string.byte(message, 1)
                    local filename_len_low = string.byte(message, 2)
                    local filename_len = filename_len_high * 256 + filename_len_low
                    
                    local filename = string.sub(message, 3, 2 + filename_len)
                    local file_data = string.sub(message, 3 + filename_len)
                    local file_size = string.len(file_data)
                    
                    print("📝 SERVER: Filename: " .. filename)
                    print("📊 SERVER: Size: " .. file_size .. " bytes")
                    
                    -- Save file to server's uploads directory
                    local save_path = "Hello/assets/uploads/" .. filename
                    local save_success, save_error = pcall(function()
                        write_file_bytes(save_path, file_data)
                    end)
                    
                    if save_success then
                        print("✅ SERVER: Saved to " .. save_path)
                        print("🎉 SERVER: Upload #" .. files_received .. " complete!")
                        
                        -- Load as asset and spawn sprite
                        local asset_path = "uploads/" .. filename
                        local texture_id = load_asset(asset_path)
                        
                        spawn({
                            Sprite = {
                                image = texture_id,
                                custom_size = {x = 200, y = 200}
                            },
                            Transform = {
                                translation = {x = 0, y = 0, z = 0},
                                rotation = {x = 0.0, y = 0.0, z = 0.0, w = 1.0},
                                scale = {x = 1.0, y = 1.0, z = 1.0}
                            },
                            Replicated = {}
                        })
                        
                        print("🖼️  SERVER: Spawned sprite with uploaded image")
                    else
                        print("❌ SERVER: Failed to save file: " .. tostring(save_error))
                    end
                end
            end
        end
    end)
    
    -- Status display
    local frame_count = 0
    register_system("server_status", function(world)
        frame_count = frame_count + 1
        
        -- Debug: Check connected clients periodically
        if frame_count % 60 == 0 then
            local success, clients = pcall(function()
                return world:call_resource_method("RenetServer", "clients_id")
            end)
            
            if success and clients then
                local client_count = 0
                for i, client_id in ipairs(clients) do
                    client_count = client_count + 1
                end
                if client_count > 0 then
                    print("📡 SERVER: " .. client_count .. " client(s) connected")
                    for i, client_id in ipairs(clients) do
                        print("   - Client ID: " .. client_id)
                    end
                end
            end
        end
        
        if frame_count % 300 == 0 then  -- Every 5 seconds at 60fps
            if files_received > 0 then
                print("\n📊 SERVER STATUS: " .. files_received .. " file(s) received")
            else
                print("\n⏳ SERVER STATUS: Waiting for uploads...")
            end
        end
    end)
end

print("\n✨ Pure Lua networking initialized!")
print("   - No Rust code for file upload logic")
print("   - All networking handled in Lua")
print("   - Using auto-generated method bindings")
