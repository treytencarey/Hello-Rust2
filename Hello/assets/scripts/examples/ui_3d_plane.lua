-- 3D UI Plane with Render-to-Texture
-- This demonstrates rendering a 2D UI onto a 3D plane using RTT
-- The 2D camera renders to a texture, which is applied to the 3D plane material

print("[UI_3D_PLANE] Setting up render-to-texture UI scene...")

-- Create render target texture using generic method binding
local ui_texture = create_render_target_image(1024, 768)
print("[UI_3D_PLANE] Created UI texture ID: " .. ui_texture)

-- Create UI root node
local ui_root = spawn({
    Node = {
        width = { Percent = 100 },
        height = { Percent = 100 },
        justify_content = "Center",
        align_items = "Center"
    }
})

print("[UI_3D_PLANE] Spawned UI root")

-- Create interactive button
local button = spawn({
    Button = {},
    Node = {
        width = { Px = 300 },
        height = { Px = 100 },
        justify_content = "Center",
        align_items = "Center"
    },
    BackgroundColor = { _0 = { r = 0.2, g = 0.6, b = 0.8, a = 1.0 } },
    BorderColor = { _0 = { r = 1.0, g = 1.0, b = 1.0, a = 1.0 } },
    BorderRadius = { 
        top_left = 10.0,
        top_right = 10.0,
        bottom_left = 10.0,
        bottom_right = 10.0
    }
})

-- Add button text
spawn({
    Text = "Click Me!",
    TextFont = {
        font_size = 40.0
    },
    TextColor = { _0 = { r = 1.0, g = 1.0, b = 1.0, a = 1.0 } },
    Parent = { _0 = button }
})

print("[UI_3D_PLANE] Spawned button with text")

-- Create 3D plane mesh
local plane_mesh = create_asset("bevy_mesh::mesh::Mesh", {
    primitive = { 
        Plane3d = { 
            normal = { x = 0.0, y = 0.0, z = 1.0 },
            half_size = { x = 5.12, y = 3.84 }
        }
    }
})

-- Create material using the UI texture
local plane_material = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
    base_color_texture = ui_texture,
    unlit = true,
    alpha_mode = "Blend"
})

-- Spawn the 3D plane
spawn({
    Mesh3d = { _0 = plane_mesh },
    ["MeshMaterial3d<StandardMaterial>"] = { _0 = plane_material },
    Transform = { 
        translation = { x = 0.0, y = 0.0, z = 0.0 },
        rotation = { x = 0.0, y = 0.0, z = 0.0, w = 1.0 },
        scale = { x = 1.0, y = 1.0, z = 1.0 }
    }
})

print("[UI_3D_PLANE] Spawned 3D plane with UI texture")

-- Add lighting
spawn({
    PointLight = {
        shadows_enabled = false,
        color = { r = 1.0, g = 1.0, b = 1.0, a = 1.0 },
        intensity = 500000.0,
        range = 50.0
    },
    Transform = {
        translation = { x = 5.0, y = 5.0, z = 8.0 }
    }
})

-- Spawn 3D camera
spawn({
    Camera3d = {},
    Transform = {
        translation = { x = 0.0, y = 2.0, z = 12.0 },
        rotation = { x = -0.0871557, y = 0.0, z = 0.0, w = 0.9961947 }
    }
})

print("[UI_3D_PLANE] Spawned 3D camera and light")

-- Spawn 2D camera rendering to texture
-- This uses the generic enum construction with auto-wrapping for ImageRenderTarget
spawn({
    Camera2d = {},
    Camera = {
        order = -1,
        target = { Image = ui_texture }, -- This triggers the auto-wrapping logic
        clear_color = { Custom = { r = 0.0, g = 0.0, b = 0.0, a = 0.0 } }
    }
})

print("[UI_3D_PLANE] Spawned 2D camera with render target")

-- Button interaction system
local click_count = 0
register_system("Update", function(world)
    local buttons = world:query({"Button", "Interaction"}, {"Interaction"})
    for _, btn in ipairs(buttons) do
        local interaction = btn:get("Interaction")
        
        if interaction == "Pressed" then
            click_count = click_count + 1
            print("[UI_3D_PLANE] Button clicked! Total clicks: " .. click_count)
            
            -- Alternate button colors
            local new_color = (click_count % 2 == 0) and 
                { r = 0.2, g = 0.6, b = 0.8, a = 1.0 } or
                { r = 0.8, g = 0.3, b = 0.3, a = 1.0 }
            btn:set("BackgroundColor", { _0 = new_color })
        end
    end
end)

print("[UI_3D_PLANE] Scene ready! Click the button on the 3D plane.")
