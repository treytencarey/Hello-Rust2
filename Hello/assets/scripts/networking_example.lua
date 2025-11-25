local role = "server"  -- Change to "client" to test client mode

if role == "server" then
    print("Starting as SERVER...")
    print("")
    
    -- Use GENERIC insert_resource API to create server resources
    -- These are registered via builders in the example code
    insert_resource("RenetServer", {})
    insert_resource("NetcodeServerTransport", {
        port = 5000,
        max_clients = 10
    })
    
    print("Server resources inserted")
    print("  Port: 5000")
    print("  Max clients: 10")
    print("")
    
    -- Flag to ensure we only initialize once
    local server_initialized = false
    
    -- Register a system to check when server is ready
    register_system("check_server_status", function(world)
        -- Use GENERIC query_resource to check if server started
        if not server_initialized and world:query_resource("RenetServer") then
            server_initialized = true
            
            print("Server is running!")
            
            -- Spawn an entity with explicit Replicated component
            spawn({
                Transform = {
                    translation = { x = 0.0, y = 0.0, z = 0.0 },
                    rotation = { x = 0.0, y = 0.0, z = 0.0, w = 1.0 },
                    scale = { x = 1.0, y = 1.0, z = 1.0 }
                },
                Sprite = {
                    color = { r = 0.2, g = 0.8, b = 0.3, a = 1.0 },
                    custom_size = { x = 50.0, y = 50.0 }
                },
                Replicated = {}  -- Explicitly mark for replication
            })
            
            print("Spawned entity with Replicated marker")
            print("")
            print("=== Server Ready ===")
            print("Clients can now connect to 127.0.0.1:5000")
            print("")
        end
    end)
    
    -- Register a system to move the replicated entity
    register_system("move_replicated_entity", function(world)
        -- Query for entities with Transform AND Replicated
        local replicated_entities = world:query({"Transform", "Replicated"})
        
        local moved_count = 0
        -- Filter out the camera (it's at z=999.9 by default in 2D)
        for _, entity in ipairs(replicated_entities) do
            local transform = entity:get("Transform")
            if transform and transform.translation and transform.translation.z < 100.0 then
                -- Simple movement: oscillate left and right
                local time = os.clock()
                local new_x = math.sin(time * 2) * 300.0  -- Oscillate 300 pixels
                local new_y = math.cos(time) * 150.0      -- Also move up/down
                
                -- Update transform - this will be replicated to clients!
                entity:set("Transform", {
                    translation = { x = new_x, y = new_y, z = 0.0 },
                    rotation = { x = 0.0, y = 0.0, z = 0.0, w = 1.0 },
                    scale = { x = 1.0, y = 1.0, z = 1.0 }
                })
                moved_count = moved_count + 1
            end
        end
    end)
    
elseif role == "client" then
    print("Starting as CLIENT...")
    print("")
    
    -- Use GENERIC insert_resource API to create client resources
    insert_resource("RenetClient", {})
    insert_resource("NetcodeClientTransport", {
        server_addr = "127.0.0.1",
        port = 5000
    })
    
    print("Client resources inserted")
    print("  Server: 127.0.0.1:5000")
    print("")
    print("=== Attempting to Connect ===")
    print("Waiting for replicated entities from server...")
    print("(If no entities appear, the server may not be running)")
    print("")
    
    local last_entity_count = 0
    
    -- Register a system to monitor replicated entities
    register_system("monitor_replicated_entities", function(world)
        -- Query for entities with Transform AND Replicated (server entities)
        local entities = world:query({"Transform", "Replicated"})
        
        -- Only print when entity count changes
        if #entities > 0 and (#entities ~= last_entity_count) then
            last_entity_count = #entities
            print(string.format("Receiving %d replicated entities from server", #entities))
        end
    end)
    
    -- Register a system to add sprites to replicated entities on client
    register_system("add_sprites_to_replicated", function(world)
        -- Query for entities with Transform AND Replicated (server entities)
        local entities = world:query({"Transform", "Replicated"})
        
        for _, entity in ipairs(entities) do
            -- Try to add sprite (will fail if already exists, which is fine)
            pcall(function()
                entity:set("Sprite", {
                    color = { r = 0.2, g = 0.8, b = 0.3, a = 1.0 },
                    custom_size = { x = 50.0, y = 50.0 }
                })
            end)
        end
    end)
    
    -- Register a debug system to monitor entity positions
    local last_debug_time = 0
    local last_positions = {}
    register_system("debug_entity_positions", function(world)
        local current_time = os.clock()
        if current_time - last_debug_time > 1.0 then  -- Print every second
            last_debug_time = current_time
            
            local entities = world:query({"Transform", "Replicated"})
            if #entities > 0 then
                print(string.format("📍 Client sees %d entities:", #entities))
                for i, entity in ipairs(entities) do
                    local transform = entity:get("Transform")
                    if transform and transform.translation then
                        local pos_x = transform.translation.x
                        local pos_y = transform.translation.y
                        local z = transform.translation.z
                        
                        -- Only print non-camera entities (z < 100)
                        if z < 100.0 then
                            local last_pos = last_positions[i]
                            local moving = ""
                            if last_pos then
                                local dx = math.abs(pos_x - last_pos.x)
                                local dy = math.abs(pos_y - last_pos.y)
                                if dx > 0.1 or dy > 0.1 then
                                    moving = "MOVING"
                                else
                                    moving = "STATIC (no updates from server?)"
                                end
                            end
                            print(string.format("  Entity %d: (%.1f, %.1f)%s", i, pos_x, pos_y, moving))
                            last_positions[i] = {x = pos_x, y = pos_y}
                        end
                    end
                end
            else
                print("No entities yet (waiting for server...)")
            end
        end
    end)
end