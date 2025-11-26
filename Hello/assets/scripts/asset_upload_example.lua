-- Networked File Upload Example
-- Drag an image onto the window to upload it to the server
-- The server will create a sprite at your mouse position and replicate it to all clients

print("=== Networked File Upload System ===")
print("Drag an image file onto this window to upload it to the server")
print("")

-- State
local is_server = nil
local is_client = nil
local initialized = false

-- Message types
local MSG_FILE_UPLOAD = 1

-- Last known mouse position
local last_mouse_pos = {x = 0, y = 0, z = 0}

-- Helper: Get mouse position in world coordinates
local function get_mouse_world_pos(world)
    -- Try to query for cursor position events
    -- Note: CursorMoved may not be registered for reflection
    local success, cursor_events = pcall(function()
        return world:read_events("bevy_window::event::CursorMoved")
    end)
    
    if success and cursor_events and #cursor_events > 0 then
        local last_event = cursor_events[#cursor_events]
        if last_event.CursorMoved then
            local pos = last_event.CursorMoved.position
            -- Convert screen coords to world coords (assuming default camera setup)
            -- Screen center is (0, 0) in world space
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
    
    -- In a real implementation, you'd copy the file here
    -- For now, we'll try to load it directly with absolute path
    -- The AssetPlugin is configured to allow unapproved paths
    
    return source_path  -- Return absolute path for now
end

-- Main system: Initialize and handle file uploads
register_system("handle_file_uploads", function(world)
    -- Initialize on first run
    if not initialized then
        if world:query_resource("RenetServer") then
            is_server = true
            print("🖥️  Running as SERVER")
        elseif world:query_resource("RenetClient") then
            is_client = true
            print("💻 Running as CLIENT")
        else
            print("⚠️  No networking detected - starting as SERVER")
            insert_resource("RenetServer", {})
            insert_resource("NetcodeServerTransport", { port = 5000, max_clients = 10 })
            is_server = true
        end
        initialized = true
    end
    
    -- Handle file drop events
    local events = world:read_events("bevy_window::event::FileDragAndDrop")
    
    for i, event in ipairs(events) do
        if event.DroppedFile then
            local path = strip_quotes(event.DroppedFile.path_buf)
            
            if is_server then
                -- SERVER: Load asset and spawn sprite directly
                print(string.format("📁 Server: File dropped: %s", path))
                
                -- Prepare asset path
                local asset_path = copy_to_assets(path)
                local filename = get_filename(path)
                
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
                
            elseif is_client then
                -- CLIENT: Send file data to server via network
                print(string.format("📁 Client: File dropped: %s", path))
                
                local filename = get_filename(path)
                
                -- Get mouse position
                local pos = get_mouse_world_pos(world)
                print(string.format("📍 Client: Mouse at position (%.1f, %.1f)", pos.x, pos.y))
                
                print("📤 Client: File upload to server...")
                print(string.format("   Filename: %s", filename))
                print(string.format("   Position: (%.1f, %.1f)", pos.x, pos.y))
                print("")
                print("⚠️  Network file transfer not yet implemented!")
                print("   To implement:")
                print("   1. Add file reading API to Lua (read_file_bytes)")
                print("   2. Send file data + filename + position to server via RenetClient")
                print("   3. Server receives, saves to assets/uploads/, loads asset, spawns sprite")
                print("   4. Sprite replicates to all clients via Replicated component")
                print("")
                print("   For now: Run the server separately and drag files on the server window")
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
        local last_event = cursor_events[#cursor_events]
        if last_event.position then
            local pos = last_event.position
            -- Convert screen coords to world coords
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
