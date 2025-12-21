-- UFbx FBX Loading Example
-- Demonstrates loading an FBX file with bevy_ufbx + mouse drag rotation

local FPS = require("modules/FPS.lua")
FPS.setup()

print("=== UFbx FBX Loading Example ===")

-- Marker component for the draggable FBX scene
DraggableScene = {}

-- State for mouse drag rotation
local is_dragging = false
local target_rotation = { x = 0.0, y = 0.0, z = 0.0, w = 1.0 }  -- Target rotation (smoothed to)
local rotation_speed = 0.005  -- Sensitivity
local smoothing = 0.15  -- Lower = smoother (0.1-0.3 is good range)

-- Helper function: Quaternion multiplication (q1 * q2)
local function quat_mul(q1, q2)
    return {
        w = q1.w * q2.w - q1.x * q2.x - q1.y * q2.y - q1.z * q2.z,
        x = q1.w * q2.x + q1.x * q2.w + q1.y * q2.z - q1.z * q2.y,
        y = q1.w * q2.y - q1.x * q2.z + q1.y * q2.w + q1.z * q2.x,
        z = q1.w * q2.z + q1.x * q2.y - q1.y * q2.x + q1.z * q2.w
    }
end

-- Helper function: Quaternion from axis-angle
local function quat_from_axis_angle(axis_x, axis_y, axis_z, angle)
    local half_angle = angle / 2
    local s = math.sin(half_angle)
    return {
        w = math.cos(half_angle),
        x = axis_x * s,
        y = axis_y * s,
        z = axis_z * s
    }
end

-- Helper function: Spherical linear interpolation (slerp) for smooth rotation
local function quat_slerp(q1, q2, t)
    -- Normalize quaternions
    local function normalize(q)
        local len = math.sqrt(q.w*q.w + q.x*q.x + q.y*q.y + q.z*q.z)
        return { w = q.w/len, x = q.x/len, y = q.y/len, z = q.z/len }
    end
    
    q1 = normalize(q1)
    q2 = normalize(q2)
    
    -- Compute dot product
    local dot = q1.w*q2.w + q1.x*q2.x + q1.y*q2.y + q1.z*q2.z
    
    -- If negative dot, negate q2 to take shorter path
    if dot < 0.0 then
        q2 = { w = -q2.w, x = -q2.x, y = -q2.y, z = -q2.z }
        dot = -dot
    end
    
    -- If very close, use linear interpolation
    if dot > 0.9995 then
        return normalize({
            w = q1.w + t * (q2.w - q1.w),
            x = q1.x + t * (q2.x - q1.x),
            y = q1.y + t * (q2.y - q1.y),
            z = q1.z + t * (q2.z - q1.z)
        })
    end
    
    -- Slerp
    local theta = math.acos(dot)
    local sin_theta = math.sin(theta)
    local w1 = math.sin((1 - t) * theta) / sin_theta
    local w2 = math.sin(t * theta) / sin_theta
    
    return {
        w = q1.w * w1 + q2.w * w2,
        x = q1.x * w1 + q2.x * w2,
        y = q1.y * w1 + q2.y * w2,
        z = q1.z * w1 + q2.z * w2
    }
end

-- System: Handle mouse button events (track drag state)
function handle_mouse_buttons(world)
    local button_events = world:read_events("MouseButtonInput")
    for _, event in ipairs(button_events) do
        if event.button and event.button.Left then
            if event.state.Pressed then
                is_dragging = true
            elseif event.state.Released then
                is_dragging = false
            end
        end
    end
end

-- System: Apply mouse motion to rotate entity while dragging
function apply_drag_rotation(world)
    -- Find entities with DraggableScene marker
    local entities = world:query({"DraggableScene", "Transform"}, nil)
    
    if #entities == 0 then return end
    local entity = entities[1]
    
    if is_dragging then
        -- Read mouse motion events and apply to target rotation
        local motion_events = world:read_events("MouseMotion")
        for _, event in ipairs(motion_events) do
            if event.delta then
                -- Create rotation deltas relative to model's current orientation
                -- Horizontal drag (delta.x) -> rotate around world Y axis (yaw)
                local yaw_delta = event.delta.x * rotation_speed
                local pitch_delta = event.delta.y * rotation_speed  -- No inversion needed
                
                -- Create incremental rotations
                local yaw_quat = quat_from_axis_angle(0, 1, 0, yaw_delta)  -- Around Y (up)
                local pitch_quat = quat_from_axis_angle(1, 0, 0, pitch_delta)  -- Around X (right)
                
                -- Apply yaw first (world space), then pitch (local space)
                -- This gives intuitive rotation: drag left/right spins, drag up/down tilts
                target_rotation = quat_mul(yaw_quat, target_rotation)
                target_rotation = quat_mul(target_rotation, pitch_quat)
            end
        end
    end
    
    -- Smooth interpolation: gradually move current rotation toward target
    local transform = entity:get("Transform")
    if transform then
        local current_rot = transform.rotation
        local new_rot = quat_slerp(current_rot, target_rotation, smoothing)
        
        entity:set({
            Transform = {
                translation = transform.translation,
                rotation = new_rot,
                scale = transform.scale
            }
        })
    end
end

-- Spawn the FBX scene using load_asset (generic asset loading)
local scene_id = load_asset("examples/ufbx/MiMi_idle_stance.glb#Scene0")
print("Loaded scene with ID:", scene_id)

local scene = spawn({
    SceneRoot = { 
        id = scene_id
    },
    Transform = {
        translation = { x = 0.0, y = -1.0, z = 0.0 },
        rotation = { x = 0.0, y = 0.0, z = 0.0, w = 1.0 },
        scale = { x = 0.5, y = 0.5, z = 0.5 }
    },
    DraggableScene = {}  -- Marker to find this entity
})
local scene_entity_id = scene:id()

print("✓ FBX scene spawned!")

-- Camera positioned to look at model
spawn({
    Camera3d = {},
    Transform = {
        translation = { x = 0.0, y = 0.0, z = 5.0 },
        rotation = { x = 0.0, y = 0.0, z = 0.0, w = 1.0 }
    }
})

spawn({
    Camera2d = {}
})

-- High ambient light to see textures clearly (pseudo-unlit)
spawn({
    AmbientLight = {
        color = { r = 1.0, g = 1.0, b = 1.0, a = 1.0 },
        brightness = 5000.0  -- Very bright = essentially unlit
    }
})

-- Add directional light for shadows/highlights
spawn({
    DirectionalLight = {
        illuminance = 10000,
        shadows_enabled = true
    },
    Transform = {
        translation = { x = 5, y = 10, z = 5 },
        rotation = { x = -0.3, y = 0.3, z = 0.0, w = 0.9 }
    }
})

print("✓ Lighting added")

-- === Material Override System ===
-- Create materials with our textures (order: floating, clothes, face, hair)
local texture_body = load_asset("examples/ufbx/mimi new body uvs_base color.png")
local texture_floating = load_asset("examples/ufbx/Mimi_Floating_BaseColor.png")
local texture_clothes = load_asset("examples/ufbx/Mimi_ClothesMain_BaseColor.png")
local texture_face = load_asset("examples/ufbx/Mimi_Face_BaseColor.png")
local texture_hair = load_asset("examples/ufbx/Mimi_Hair_BaseColor.png")
local texture_wand = load_asset("examples/ufbx/MimiWand_BaseColor.png")

print("[TEXTURES] Loaded texture IDs: floating=" .. tostring(texture_floating) .. 
      ", clothes=" .. tostring(texture_clothes) .. 
      ", face=" .. tostring(texture_face) .. 
      ", hair=" .. tostring(texture_hair) ..
      ", wand=" .. tostring(texture_wand))

-- Create StandardMaterial assets with the textures
local mat_body = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
    base_color_texture = texture_body,
    perceptual_roughness = 0.8,  -- Non-shiny skin
    metallic = 0.0,              -- Not metallic
    reflectance = 0.5,           -- Standard reflectance
    unlit = false,
    double_sided = true
})

local mat_floating = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
    base_color_texture = texture_floating,
    perceptual_roughness = 0.7,
    metallic = 0.0,
    reflectance = 0.5,
    unlit = false,
    double_sided = true
})

local mat_clothes = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
    base_color_texture = texture_clothes,
    perceptual_roughness = 0.9,  -- Cloth is quite rough
    metallic = 0.0,
    reflectance = 0.5,
    unlit = false,
    double_sided = true
})

local mat_face = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
    base_color_texture = texture_face,
    perceptual_roughness = 0.6,  -- Face slightly less rough than body
    metallic = 0.0,
    reflectance = 0.5,
    unlit = false,
    double_sided = true
})

local mat_hair = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
    base_color_texture = texture_hair,
    perceptual_roughness = 0.5,  -- Hair has some shine
    metallic = 0.0,
    reflectance = 0.5,
    unlit = false,
    double_sided = true,
    alpha_mode = "Blend"  -- Hair often needs transparency
})

local mat_wand = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
    base_color_texture = texture_wand,
    perceptual_roughness = 0.3,  -- Wand might be shinier
    metallic = 0.1,              -- Slight metallic for magical effect
    reflectance = 0.5,
    unlit = false,
    double_sided = true
})

-- Store materials for assignment (6 meshes total)
-- Common order: body, face, hair, clothes, floating, (extra - maybe eyes or accessories)
--                                                                           wand
local materials_to_assign = { mat_hair, mat_face, mat_clothes, mat_floating, mat_wand, mat_body }
local materials_assigned = false
local frames_waited = 0
local FRAMES_TO_WAIT = 0  -- Wait 60 frames (~1 second) for scene to load

-- System: Assign materials to meshes after scene loads
function assign_materials(world)
    if materials_assigned then return end
    
    -- Wait for scene to load
    frames_waited = frames_waited + 1
    if frames_waited < FRAMES_TO_WAIT then
        return
    end
    
    -- Only print debug every 60 frames to reduce spam
    local should_debug = (frames_waited % 60 == 0)
    
    -- Try different component queries to find the meshes
    local mesh3d_entities = world:query({"Mesh3d"}, nil)
    local mesh_entities = world:query({"MeshMaterial3d"}, nil)
    local name_entities = world:query({"Name"}, nil)
    
    if should_debug then
        print("[DEBUG] Mesh3d: " .. #mesh3d_entities .. ", MeshMaterial3d: " .. #mesh_entities .. ", Named: " .. #name_entities)
    end
    
    -- Try to assign to Mesh3d entities instead
    if #mesh3d_entities > 0 then
        print("✓ Found " .. #mesh3d_entities .. " Mesh3d entities!")
        
        -- First pass: print all mesh names and existing materials to see the order
        print("\n[MESH ORDER] Listing all " .. #mesh3d_entities .. " meshes:")
        for i, entity in ipairs(mesh3d_entities) do
            local name = entity:get("Name")
            local name_str = name and name.name or "unnamed"
            
            -- Try to get parent to see hierarchy
            local parent = entity:get("Parent")
            local parent_str = "no parent"
            if parent then
                local parent_name = world:entity(parent.entity):get("Name")
                parent_str = parent_name and ("parent: " .. parent_name.name) or "parent: unnamed"
            end
            
            -- Try to get existing material (if GLTF imported one)
            local existing_mat = entity:get("MeshMaterial3d<StandardMaterial>")
            local mat_info = existing_mat and ("material ID: " .. tostring(existing_mat._0)) or "no material"
            
            print("  [" .. i .. "] " .. name_str .. " (" .. parent_str .. ") - " .. mat_info)
        end
        print("\nNote: You have " .. #mesh3d_entities .. " meshes but " .. #materials_to_assign .. " materials in the array")
        print("")
        
        -- for i, entity in ipairs(mesh3d_entities) do
        --     local mat_index = ((i - 1) % #materials_to_assign) + 1
        --     local mat = materials_to_assign[mat_index]
            
        --     local name = entity:get("Name")
        --     local name_str = name and name.name or "unnamed"
            
        --     -- Debug: print material assignment
        --     print("[ASSIGN] Mesh #" .. i .. " (" .. name_str .. ") <- Material #" .. mat_index)
            
        --     -- Insert MeshMaterial3d<StandardMaterial> component with our material
        --     entity:set({ "MeshMaterial3d<StandardMaterial>" = { _0 = mat } })
        -- end
        
        materials_assigned = true
        print("✓ Materials assigned!")
    end
end

-- Register the material assignment system
register_system("Update", assign_materials)

-- System: Check for AnimationPlayer and play animation
local animation_started = false
function play_animation(world)
    if animation_started then return end
    
    -- Query for AnimationPlayer
    -- Note: It might be on a child entity, so we query all of them
    local players = world:query({"AnimationPlayer"}, nil)
    
    if #players > 0 then
        print("✓ Found " .. #players .. " AnimationPlayer(s)!")
        
        for _, player in ipairs(players) do
            if not player:is_playing_animation() then
                print("Starting animation...")
                -- TODO: We need to know the animation clip handle
                -- Usually bevy_ufbx loads clips as separate assets or embedded
                -- For now, just printing that we found the player
                animation_started = true
            end
        end
    end
end

-- Register systems
register_system("Update", handle_mouse_buttons)
register_system("Update", apply_drag_rotation)
-- register_system("Update", play_animation)

print("=== UFbx example loaded successfully ===")
print("🖱️ Drag with left mouse button to rotate the model!")


