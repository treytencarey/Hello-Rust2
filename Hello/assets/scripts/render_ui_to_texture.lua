-- render_ui_to_texture.lua
-- Zero-Rust RTT UI Example
-- This example demonstrates rendering UI to a texture that's applied to a 3D object
-- all defined in Lua using the enhanced create_asset and reflection infrastructure.

local PI = 3.14159265359
local CUBE_POINTER_ID = 90870987  -- Custom UUID for the cube's virtual pointer

-- Store entity IDs for the rotating sphere
local sphere_entity = nil
local elapsed_time = 0
local elapsed_time = 0

-- Called once when script loads
function setup()
    print("[RTT_LUA] Setting up render-to-texture UI...")
    
    -- === Step 1: Create the RTT Image ===
    -- Uses auto-discovered constructor: Image::new_target_texture(width, height, format)
    local rtt_image = create_asset("bevy_image::image::Image", {
        width = 512,
        height = 512,
        format = "Bgra8UnormSrgb"
    })
    print("[RTT_LUA] Created RTT image handle: " .. tostring(rtt_image))
    
    -- === Step 2: Spawn directional light ===
    spawn({
        DirectionalLight = {}
    })
    print("[RTT_LUA] Spawned directional light")
    
    -- === Step 3: Spawn the Camera2d that renders to texture ===
    -- Camera is alphabetically before Camera2d, so with sorting:
    -- 1. Camera is inserted first (with our target and order settings)
    -- 2. Camera2d is inserted second. Since Camera exists, Bevy won't replace it
    --    BUT Camera2d's other required components (like CameraRenderGraph) SHOULD still be added
    local texture_camera = spawn({
        Camera = {
            order = -1,  -- Render before main camera
            -- RenderTarget::Image(ImageRenderTarget) - pass handle directly, newtype auto-wraps
            target = { Image = rtt_image }
        },
        Camera2d = {}  -- Adds the 2D render graph
    })
    print("[RTT_LUA] Spawned texture camera: " .. tostring(texture_camera))
    
    -- === Step 4: Spawn UI root node targeting the texture camera ===
    -- SIMPLIFIED TEST: Just a bright red background to verify RTT works
    local ui_root = spawn({
        Node = {
            width = { Percent = 100.0 },
            height = { Percent = 100.0 },
            flex_direction = "Column",
            justify_content = "Center",
            align_items = "Center"
        },
        -- Gray background like the Rust example
        BackgroundColor = { color = { r = 0.5, g = 0.5, b = 0.5, a = 1.0 } },
        UiTargetCamera = { entity = texture_camera:id() }  -- Target UI to RTT camera
    })
    print("[RTT_LUA] Spawned UI root with UiTargetCamera: " .. tostring(ui_root))
    
    -- === Step 5: Spawn draggable button with Lua observers ===
    -- Using chainable API: spawn():with_parent():observe()
    local button = spawn({
        Button = {},  -- Required for Bevy UI interactions
        Node = {
            width = { Percent = 40.0 },
            height = { Percent = 20.0 },
            align_items = "Center",
            justify_content = "Center"
        },
        BorderRadius = { 
            top_left = 10.0, 
            top_right = 10.0, 
            bottom_left = 10.0, 
            bottom_right = 10.0 
        },
        BackgroundColor = { color = { r = 0.0, g = 0.47, b = 0.84, a = 1.0 } },
        UiTargetCamera = { entity = texture_camera:id() }
    })
        :with_parent(ui_root:id())
        :observe("Pointer<Over>", function(entity, event)
            print("[LUA_OBSERVER] Pointer Over - turning RED")
            entity:set("BackgroundColor", { color = { r = 1.0, g = 0.0, b = 0.0, a = 1.0 } })
        end)
        :observe("Pointer<Out>", function(entity, event)
            print("[LUA_OBSERVER] Pointer Out - turning BLUE") 
            entity:set("BackgroundColor", { color = { r = 0.0, g = 0.47, b = 0.84, a = 1.0 } })
        end)
        :observe("Pointer<Drag>", function(entity, event)
            print("[LUA_OBSERVER] Drag at x=" .. tostring(event.x) .. " y=" .. tostring(event.y))
            -- Just log for now - moving requires more work
        end)
    
    print("[RTT_LUA] Spawned button with chainable API and Lua observers: " .. tostring(button:id()))
    
    -- === Step 6: Spawn button text (as child of button) ===
    spawn({
        Text = { text = "Drag Me!" },
        TextFont = { font_size = 40.0 },
        TextColor = { color = { r = 1.0, g = 1.0, b = 1.0, a = 1.0 } },
        UiTargetCamera = { entity = texture_camera:id() }
    }):with_parent(button:id())
    
    print("[RTT_LUA] Spawned button text with UiTargetCamera")
    
    -- === Step 7: Create mesh for the 3D object ===
    -- Note: bevy_mesh::mesh::Mesh is the correct type path per the registry
    local sphere_mesh = create_asset("bevy_mesh::mesh::Mesh", {
        primitive = { Sphere = { radius = 1.0 } }
    })
    print("[RTT_LUA] Created sphere mesh: " .. tostring(sphere_mesh))
    
    -- === Step 8: Create material with RTT texture ===
    local material = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
        -- Option<Handle<Image>> - setter now handles automatic wrapping
        base_color_texture = rtt_image,
        reflectance = 0.02,
        unlit = true  -- Keep unlit for debugging
    })
    print("[RTT_LUA] Created material with RTT texture: " .. tostring(material) .. " using image handle: " .. tostring(rtt_image))
    
    -- === Step 9: Spawn 3D sphere with the material ===
    sphere_entity = spawn({
        Mesh3d = { _0 = sphere_mesh },
        ["MeshMaterial3d<StandardMaterial>"] = { _0 = material },
        Transform = {
            translation = { x = 0.0, y = 0.0, z = 1.5 },  -- Matches Rust: 0, 0, 1.5
            rotation = { x = 1.0, y = 0.0, z = 0.0, w = 0.0 },  -- PI rotation around X
            scale = { x = 1.0, y = 1.0, z = 1.0 }
        },
        -- Custom marker for rotation system
        RotatingSphere = {}
    })
    print("[RTT_LUA] Spawned rotating sphere: " .. tostring(sphere_entity))
    
    -- === Step 10: Spawn main 3D camera ===
    spawn({
        Camera3d = {},
        Transform = {
            translation = { x = 0.0, y = 0.0, z = 5.0 },  -- Matches Rust: 0, 0, 5 looking at origin
            rotation = { x = 0.0, y = 0.0, z = 0.0, w = 1.0 }  -- Looking -Z (at sphere at z=1.5)
        }
    })
    print("[RTT_LUA] Spawned main camera")
    
    -- Note: PointerId doesn't have proper Reflect implementation for Lua creation
    -- The diegetic pointer system would require Rust-side setup
    print("[RTT_LUA] Skipping custom pointer (requires Rust setup)")
    
    print("[RTT_LUA] Setup complete!")
end

-- System: Rotate the sphere
function rotate_sphere(world)
    local dt = world:delta_time()
    elapsed_time = elapsed_time + dt
    
    local rotation_speed = 1.0  -- radians per second
    
    -- Query for entities with RotatingSphere and Transform
    local entities = world:query({"RotatingSphere", "Transform"}, nil)
    
    for i, entity in ipairs(entities) do
        local transform = entity:get("Transform")
        if transform then
            -- Calculate Y rotation angle from elapsed time
            local angle = elapsed_time * rotation_speed
            
            -- Quaternion for rotation around Y axis: (0, sin(a/2), 0, cos(a/2))
            local half_angle = angle / 2
            local sin_y = math.sin(half_angle)
            local cos_y = math.cos(half_angle)
            
            entity:set("Transform", {
                translation = transform.translation,
                rotation = {
                    x = 0.0,
                    y = sin_y,
                    z = 0.0,
                    w = cos_y
                },
                scale = transform.scale
            })
        end
    end
end

-- Note: Button hover/click is now handled via Rust observers in the example code
-- This is necessary for RTT UI picking because PointerInput events don't update Interaction component
-- The Lua system below is disabled - hover is handled by Pointer<Over>/Pointer<Out> observers

-- Register systems
register_system("Update", rotate_sphere)

-- Run setup
setup()

print("[RTT_LUA] Script loaded successfully!")
