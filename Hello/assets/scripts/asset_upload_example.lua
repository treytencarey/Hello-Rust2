-- Networked File Upload Example
-- Drag an image onto the window to upload it to the server
-- The server will create a sprite at your mouse position and replicate it to all clients
--
-- ARCHITECTURE:
-- This example demonstrates the "Zero Rust" philosophy for networking:
-- - Networking resources (RenetServer/RenetClient, transports) are created entirely from Lua
-- - Resource constructors are registered in Rust (Hello/src/networking.rs)
-- - Auto-generated method bindings come from Cargo.toml metadata:
--   [package.metadata.lua_resources]
--   types = ["renet::remote_connection::RenetClient", "renet::server::RenetServer", ...]
-- - The build script generates Lua bindings for resource methods at compile time
-- - All game logic for networking setup and message handling is in Lua

print("=== Networked File Upload System ===")
print("Drag an image file onto this window to upload it to the server")
print("")

-- Network configuration constants
local PROTOCOL_ID = 0
local SERVER_PORT = 5000
local MAX_CLIENTS = 10

-- State
local is_server = nil
local is_client = nil

-- Message types
local MSG_FILE_UPLOAD = 1

-- Last known mouse position
local last_mouse_pos = {x = 0, y = 0, z = 0}

-- Initialize networking resources based on mode
-- This runs immediately when the script loads (in PostStartup)
if IS_CLIENT_MODE then
    is_client = true
    print("💻 Initializing CLIENT...")
    
    -- Create client networking resources
    insert_resource("RenetClient", {})
    insert_resource("NetcodeClientTransport", {
        server_addr = "127.0.0.1",
        port = SERVER_PORT
    })
    
    print("✅ Client initialized - connecting to server at 127.0.0.1:" .. SERVER_PORT)
else
    is_server = true
    print("🖥️  Initializing SERVER...")
    
    -- Create server networking resources
    insert_resource("RenetServer", {})
    insert_resource("NetcodeServerTransport", {
        port = SERVER_PORT,
        max_clients = MAX_CLIENTS
    })
    
    print("✅ Server initialized - listening on port " .. SERVER_PORT)
end

-- Helper: Get mouse position in world coordinates
local function get_mouse_world_pos(world)
    -- Try to query for cursor position events
    local success, cursor_events = pcall(function()
        return world:read_events("bevy_window::event::CursorMoved")
    end)
    
    if success and cursor_events and #cursor_events > 0 then
        local last_event = cursor_events[#cursor_events]
        if last_event.position then
            local pos = last_event.position
            -- Convert screen coords to world coords
            -- Bevy's window coordinates: (0,0) at top-left
            -- World coordinates: (0,0) at center
            -- Assuming 800x600 default window size
            last_mouse_pos = {x = pos.x - 400, y = 300 - pos.y, z = 0}
        end
    end
    
    return last_mouse_pos
end

-- Helper: Strip quotes from path
local function strip_quotes(path)
    if path:sub(1, 1) == '"' and path:sub(-1) == '"' then
        return path:sub(2, -2)
    end
    return path
end

-- Helper: Extract filename from full path
local function get_filename(path)
    local filename = path:match("([^/\\]+)$")
    return filename or path
end

-- Helper: Copy file to assets folder and return relative path
local function copy_to_assets(source_path)
    local filename = get_filename(source_path)
    local dest_path = "Hello/assets/uploads/" .. filename
    
    -- Copy the file using the Lua API
    local success, err = pcall(function()
        copy_file(source_path, dest_path)
    end)
    
    if not success then
        print(string.format("⚠️  Failed to copy file: %s", tostring(err)))
        return nil
    end
    
    -- Return relative path from assets root
    return "uploads/" .. filename
end

-- Main system: Handle file uploads
register_system("handle_file_uploads", function(world)
    -- Handle file drop events
    local events = world:read_events("bevy_window::event::FileDragAndDrop")
    
    for i, event in ipairs(events) do
        if event.DroppedFile then
            local path = strip_quotes(event.DroppedFile.path_buf)
            
            if is_server then
                -- SERVER: Load asset and spawn sprite directly
                print(string.format("📁 Server: File dropped: %s", path))
                
                -- Copy file to assets folder
                local asset_path = copy_to_assets(path)
                if not asset_path then
                    print("❌ Server: Failed to copy file to assets folder")
                    goto continue
                end
                
                local filename = get_filename(path)
                print(string.format("✅ Server: Copied to assets: %s", asset_path))
                
                -- Load the asset
                local texture_id = load_asset(asset_path)
                print(string.format("✅ Server: Loaded asset '%s' with ID: %s", filename, tostring(texture_id)))
                
                -- Get mouse position
                local pos = get_mouse_world_pos(world)
                print(string.format("📍 Server: Spawning at position (%.1f, %.1f)", pos.x, pos.y))
                
                -- Spawn sprite with replication
                -- Note: We replicate Transform but not Sprite (image data)
                -- Clients will need to load the same asset locally
                spawn({
                    Sprite = {
                        image = texture_id,
                        custom_size = {x = 100, y = 100}  -- Default size
                    },
                    Transform = {
                        translation = pos,
                        rotation = {x = 0.0, y = 0.0, z = 0.0, w = 1.0},
                        scale = {x = 1.0, y = 1.0, z = 1.0}
                    },
                    Replicated = {}  -- Mark for replication to clients
                })
                
                print("🌐 Server: Sprite spawned and marked for replication")
                print("   Note: Clients will see the entity but need the same asset locally")
                
                ::continue::
                
            elseif is_client then
                -- CLIENT: Send file data to server via network
                print(string.format("📁 Client: File dropped: %s", path))
                
                local filename = get_filename(path)
                
                -- Get mouse position
                local pos = get_mouse_world_pos(world)
                print(string.format("📍 Client: Mouse at position (%.1f, %.1f)", pos.x, pos.y))
                
                -- Read file contents
                local success, file_data = pcall(function()
                    return read_file_bytes(path)
                end)
                
                if not success then
                    print(string.format("❌ Client: Failed to read file: %s", tostring(file_data)))
                    goto continue
                end
                
                print(string.format("✅ Client: Read %d bytes from file", string.len(file_data)))
                
                -- Encode message: filename length (2 bytes) + filename + position (8 bytes) + file data
                local filename_len = string.len(filename)
                local msg = string.char(
                    math.floor(filename_len / 256),  -- high byte
                    filename_len % 256               -- low byte
                ) .. filename
                
                -- Add position as 4-byte floats (simplified - just pack as strings for now)
                msg = msg .. string.pack("ff", pos.x, pos.y)
                
                -- Add file data
                msg = msg .. file_data
                
                print("📤 Client: Sending file to server...")
                world:call_resource_method("RenetClient", "send_message", 1, msg)  -- Channel 1 for file uploads
                print("✅ Client: File upload sent!")
                
                ::continue::
            end
        end
    end
end)

-- Mouse tracking system: Update mouse position continuously
register_system("track_mouse", function(world)
    -- Try to update mouse position from events
    local success, cursor_events = pcall(function()
        return world:read_events("bevy_window::event::CursorMoved")
    end)
    
    if success and cursor_events and #cursor_events > 0 then
        -- Get the last cursor event (most recent mouse position)
        local last_event = cursor_events[#cursor_events]
        if last_event.position then
            local pos = last_event.position
            -- Convert screen coords to world coords (assuming 800x600 window)
            last_mouse_pos = {x = pos.x - 400, y = 300 - pos.y, z = 0}
        end
    end
end)

-- Display system: Show info about uploaded sprites
local frame_count = 0
register_system("display_sprites", function(world)
    frame_count = frame_count + 1
    
    -- Only print every 180 frames (3 seconds at 60fps)
    if frame_count % 180 == 0 then
        local sprites = world:query({"Sprite", "Transform", "Replicated"}, nil)
        if #sprites > 0 then
            print(string.format("🖼️  Currently showing %d replicated sprites", #sprites))
        end
    end
end)

-- Server system: Receive file uploads from clients
register_system("receive_file_uploads", function(world)
    if not is_server then
        return
    end
    
    -- Check if server resource exists
    if not world:query_resource("RenetServer") then
        return
    end
    
    -- Get list of connected clients and receive messages from each
    -- Note: We need to manually iterate through potential client IDs
    -- In a real implementation, you'd track connected clients
    local messages = {}
    
    -- Try to receive from client IDs 0-9 (simple approach for demo)
    for client_id = 0, 9 do
        local success, result = pcall(function()
            -- Try to receive a message from this client on channel 1
            return world:call_resource_method("RenetServer", "receive_message", client_id, 1)
        end)
        
        if success and result then
            -- Add to messages table with client_id
            table.insert(messages, {client_id, result})
        end
    end
    
    for i, msg_data in ipairs(messages) do
        -- msg_data is a table: {client_id, message_bytes}
        local client_id = msg_data[1]
        local msg = msg_data[2]
        
        print(string.format("📥 Server: Received file upload from client %d (%d bytes)", client_id, string.len(msg)))
        
        -- Decode message
        local filename_len_high = string.byte(msg, 1)
        local filename_len_low = string.byte(msg, 2)
        local filename_len = filename_len_high * 256 + filename_len_low
        
        local filename = string.sub(msg, 3, 2 + filename_len)
        local pos_x, pos_y = string.unpack("ff", msg, 3 + filename_len)
        local file_data = string.sub(msg, 3 + filename_len + 8)
        
        print(string.format("   Filename: %s", filename))
        print(string.format("   Position: (%.1f, %.1f)", pos_x, pos_y))
        print(string.format("   File size: %d bytes", string.len(file_data)))
        
        -- Save file to assets/uploads/
        local dest_path = "Hello/assets/uploads/" .. filename
        local success, err = pcall(function()
            write_file_bytes(dest_path, file_data)
        end)
        
        if not success then
            print(string.format("❌ Server: Failed to save file: %s", tostring(err)))
            goto continue_server
        end
        
        print(string.format("✅ Server: Saved file to %s", dest_path))
        
        -- Load the asset
        local asset_path = "uploads/" .. filename
        local texture_id = load_asset(asset_path)
        print(string.format("✅ Server: Loaded asset with ID: %s", tostring(texture_id)))
        
        -- Spawn sprite with replication
        spawn({
            Sprite = {
                image = texture_id,
                custom_size = {x = 100, y = 100}
            },
            Transform = {
                translation = {x = pos_x, y = pos_y, z = 0},
                rotation = {x = 0.0, y = 0.0, z = 0.0, w = 1.0},
                scale = {x = 1.0, y = 1.0, z = 1.0}
            },
            Replicated = {}
        })
        
        print("🌐 Server: Sprite spawned from client upload and marked for replication")
        
        ::continue_server::
    end
end)
