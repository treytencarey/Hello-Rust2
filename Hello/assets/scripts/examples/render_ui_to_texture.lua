-- render_ui_to_texture.lua
-- Zero-Rust RTT UI Example with click-to-drag functionality
-- Matches Rust example approach: raycast to sphere, write PointerInput messages

local PI = 3.14159265359
local CUSTOM_POINTER_UUID = 90870988  -- Unique ID for our virtual pointer
local TEXTURE_SIZE = 512

-- State
local sphere_entity = nil
local texture_camera_entity = nil
local rtt_image_handle = nil  -- Store RTT image handle for PointerInput
local elapsed_time = 0
local cursor_position = { x = 0, y = 0 }
local cursor_last = { x = 0, y = 0 }

-- Marker component for the sphere
RotatingSphere = {}

-- Called once when script loads
function setup()
    print("[RTT_LUA] Setting up render-to-texture UI...")
    
    -- === Step 1: Create the RTT Image ===
    local rtt_image = create_asset("bevy_image::image::Image", {
        width = TEXTURE_SIZE,
        height = TEXTURE_SIZE,
        format = "Bgra8UnormSrgb"
    })
    rtt_image_handle = rtt_image  -- Store globally for PointerInput
    print("[RTT_LUA] Created RTT image handle: " .. tostring(rtt_image))
    
    -- === Step 2: Spawn directional light ===
    spawn({
        DirectionalLight = {}
    })
    
    -- === Step 3: Spawn the Camera2d that renders to texture ===
    local texture_camera = spawn({
        Camera = {
            order = -1,
            target = { Image = rtt_image }
        },
        Camera2d = {}
    })
    texture_camera_entity = texture_camera:id()
    print("[RTT_LUA] Spawned texture camera: " .. tostring(texture_camera_entity))
    
    -- === Step 4: Spawn UI root node ===
    local ui_root = spawn({
        Node = {
            width = { Percent = 100.0 },
            height = { Percent = 100.0 },
            flex_direction = "Column",
            justify_content = "Center",
            align_items = "Center"
        },
        BackgroundColor = { color = { r = 0.5, g = 0.5, b = 0.5, a = 1.0 } },
        UiTargetCamera = { entity = texture_camera:id() }
    })
    
    -- === Step 5: Spawn draggable button with observers ===
    local button = spawn({
        Button = {},
        Node = {
            position_type = "Absolute",
            width = { Percent = 50.0 },
            height = { Percent = 50.0 },
            align_items = "Center",
            padding = { left = { Px = 20.0 }, right = { Px = 20.0 }, top = { Px = 20.0 }, bottom = { Px = 20.0 } }
        },
        BorderRadius = { 
            top_left = { Px = 10.0 }, 
            top_right = { Px = 10.0 }, 
            bottom_left = { Px = 10.0 }, 
            bottom_right = { Px = 10.0 } 
        },
        BackgroundColor = { color = { r = 0.0, g = 0.0, b = 1.0, a = 1.0 } },
        UiTargetCamera = { entity = texture_camera:id() }
    })
        :with_parent(ui_root:id())
        :observe("Pointer<Over>", function(entity, event)
            print("[LUA_OBSERVER] Pointer Over - turning RED")
            entity:set("BackgroundColor", { color = { r = 1.0, g = 0.0, b = 0.0, a = 1.0 } })
        end)
        :observe("Pointer<Out>", function(entity, event)
            print("[LUA_OBSERVER] Pointer Out - turning BLUE")
            entity:set("BackgroundColor", { color = { r = 0.0, g = 0.0, b = 1.0, a = 1.0 } })
        end)
        :observe("Pointer<Drag>", function(entity, event)
            print("[LUA_OBSERVER] Drag at x=" .. tostring(event.x) .. " y=" .. tostring(event.y))
            -- Move the button to follow drag position
            if event.x and event.y then
                entity:set("Node", {
                    position_type = "Absolute",
                    left = { Px = event.x - 128 },  -- Offset to center button on cursor
                    top = { Px = event.y - 128 },
                    width = { Percent = 50.0 },
                    height = { Percent = 50.0 },
                    align_items = "Center",
                    padding = { left = { Px = 20.0 }, right = { Px = 20.0 }, top = { Px = 20.0 }, bottom = { Px = 20.0 } }
                })
            end
        end)
    
    -- === Step 6: Spawn button text ===
    spawn({
        Text = "Drag Me!",
        TextFont = { font_size = 20.0 },
        TextColor = { color = { r = 1.0, g = 1.0, b = 1.0, a = 1.0 } }
    }):with_parent(button:id())
    
    -- === Step 7: Create sphere mesh ===
    local sphere_mesh = create_asset("bevy_mesh::mesh::Mesh", {
        primitive = { Cuboid = { width = 1.0, height = 1.0, depth = 1.0 } }
    })
    
    -- === Step 8: Create material with RTT texture ===
    local material = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
        base_color_texture = rtt_image,
        reflectance = 0.02,
        unlit = false
    })
    
    -- === Step 9: Spawn 3D sphere with RotatingSphere marker ===
    local sphere = spawn({
        Mesh3d = { _0 = sphere_mesh },
        ["MeshMaterial3d<StandardMaterial>"] = { _0 = material },
        Transform = {
            translation = { x = 0.0, y = 0.0, z = 1.5 },
            rotation = { x = 1.0, y = 0.0, z = 0.0, w = 0.0 },  -- PI rotation around X
            scale = { x = 1.0, y = 1.0, z = 1.0 }
        },
        RotatingSphere = {}
    })
    sphere_entity = sphere:id()
    print("[RTT_LUA] Spawned sphere: " .. tostring(sphere_entity))
    
    -- === Step 10: Spawn main 3D camera ===
    spawn({
        Camera3d = {},
        Transform = {
            translation = { x = 0.0, y = 0.0, z = 5.0 },
            rotation = { x = 0.0, y = 0.0, z = 0.0, w = 1.0 }
        }
    })
    
    -- === Step 11: Spawn the custom PointerId entity (like Rust example) ===
    spawn({
        PointerId = { Custom = CUSTOM_POINTER_UUID }
    })
    print("[RTT_LUA] Spawned custom PointerId")
    
    print("[RTT_LUA] Setup complete!")
end

-- System: Rotate the sphere slowly
function rotate_sphere(world)
    local dt = world:delta_time()
    elapsed_time = elapsed_time + dt
    
    local rotation_speed = 1.0
    local entities = world:query({"RotatingSphere", "Transform"}, nil)
    
    for i, entity in ipairs(entities) do
        local transform = entity:get("Transform")
        if transform then
            local angle = elapsed_time * rotation_speed
            local half_angle = angle / 2
            local sin_y = math.sin(half_angle)
            local cos_y = math.cos(half_angle)
            
            entity:set("Transform", {
                translation = transform.translation,
                rotation = { x = 0.0, y = sin_y, z = 0.0, w = cos_y },
                scale = transform.scale
            })
        end
    end
end

-- System: Track cursor position from CursorMoved events
function track_cursor(world)
    local cursor_events = world:read_events("CursorMoved")
    for _, event in ipairs(cursor_events) do
        if event.position then
            cursor_position.x = event.position.x
            cursor_position.y = event.position.y
        end
    end
end

-- System: Drive diegetic pointer - raycast to sphere and write PointerInput messages
-- This is the core of the Zero-Rust approach matching the Rust example
function drive_diegetic_pointer(world)
    -- Get 3D camera and sphere for raycasting
    local cameras = world:query({"Camera3d", "Transform"}, nil)
    local spheres = world:query({"RotatingSphere", "Transform", "Mesh3d"}, nil)
    
    if #cameras == 0 or #spheres == 0 then
        return  -- Not ready yet
    end
    
    local camera = cameras[1]
    local camera_transform = camera:get("Transform")
    if not camera_transform then return end
    
    -- Get window size (assume standard window)
    local window_width = 1280
    local window_height = 720
    
    -- Convert cursor position to NDC (-1 to 1)
    local ndc_x = (cursor_position.x / window_width) * 2.0 - 1.0
    local ndc_y = 1.0 - (cursor_position.y / window_height) * 2.0  -- Y is inverted
    
    -- Camera is at (0, 0, 5) looking at origin
    local cam_pos = camera_transform.translation
    
    -- Simple ray direction calculation (perspective projection)
    local fov = 45.0 * PI / 180.0
    local aspect = window_width / window_height
    local dir_x = ndc_x * aspect * math.tan(fov / 2)
    local dir_y = ndc_y * math.tan(fov / 2)
    local dir_z = -1.0
    
    -- Normalize direction
    local len = math.sqrt(dir_x*dir_x + dir_y*dir_y + dir_z*dir_z)
    dir_x = dir_x / len
    dir_y = dir_y / len
    dir_z = dir_z / len
    
    -- Construct Ray3d
    local ray = {
        origin = { x = cam_pos.x, y = cam_pos.y, z = cam_pos.z },
        direction = { x = dir_x, y = dir_y, z = dir_z }
    }
    
    -- Call MeshRayCast::cast_ray via SystemParam
    local result = world:call_systemparam_method("MeshRayCast", "cast_ray", ray)
    
    if result and string.len(result) > 2 then
        -- Parse UV from result
        local u_match = string.match(tostring(result), "uv: Some%(Vec2%(([%d%.%-]+),")
        local v_match = string.match(tostring(result), "uv: Some%(Vec2%([%d%.%-]+, ([%d%.%-]+)%)")
        
        if u_match and v_match then
            local u = tonumber(u_match)
            local v = tonumber(v_match)
            
            if u and v then
                -- Convert UV to texture pixel position
                local rtt_x = u * TEXTURE_SIZE
                local rtt_y = v * TEXTURE_SIZE
                
                -- Only send if position changed
                local dx = rtt_x - cursor_last.x
                local dy = rtt_y - cursor_last.y
                
                if math.abs(dx) > 0.5 or math.abs(dy) > 0.5 then
                    -- Write PointerInput Move message using generic reflection
                    -- Enums use table format: { VariantName = data }
                    world:write_message("PointerInput", {
                        pointer_id = { Custom = CUSTOM_POINTER_UUID },
                        location = {
                            target = { Image = rtt_image_handle },
                            position = { x = rtt_x, y = rtt_y }
                        },
                        action = { Move = { delta = { x = dx, y = dy } } }
                    })
                    -- print("PointerInput Move: " .. tostring(rtt_x) .. ", " .. tostring(rtt_y))
                    
                    cursor_last.x = rtt_x
                    cursor_last.y = rtt_y
                end
            end
        end
    end
    
    -- Handle mouse button events for press/release
    local button_events = world:read_events("MouseButtonInput")
    for _, event in ipairs(button_events) do
        if event.button and event.button.Left then
            local action = nil
            if event.state.Pressed then
                -- PointerButton::Primary is a unit variant, so just use the string directly
                -- The reflection code should handle this as a tuple variant containing a unit enum
                action = { Press = "Primary" }
            elseif event.state.Released then
                action = { Release = "Primary" }
            end
            
            if action then
                print("[RTT_LUA] Sending PointerInput " .. tostring(action.Release))
                -- Write PointerInput Press/Release using generic reflection
                world:write_message("PointerInput", {
                    pointer_id = { Custom = CUSTOM_POINTER_UUID },
                    location = {
                        target = { Image = rtt_image_handle },
                        position = { x = cursor_last.x, y = cursor_last.y }
                    },
                    action = action
                })
            end
        end
    end
end

-- Register systems (order matters: track cursor, then raycast)
register_system("Update", track_cursor)
register_system("Update", rotate_sphere)
register_system("Update", drive_diegetic_pointer)

-- Run setup
setup()

print("[RTT_LUA] Script loaded - Zero Rust RTT picking with PointerInput enabled!")
