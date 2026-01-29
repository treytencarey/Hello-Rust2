-- Physics 3D Example Script
-- Demonstrates spawning 3D entities with Rapier physics components

-- Helper functions to create 3D Collider components
function ColliderCuboid(hx, hy, hz)
    return {
        raw = { Cuboid = { half_extents = {hx, hy, hz} } },
        unscaled = { Cuboid = { half_extents = {hx, hy, hz} } },
        scale = {1.0, 1.0, 1.0}
    }
end

function ColliderBall(radius)
    return {
        raw = { Ball = { radius = radius } },
        unscaled = { Ball = { radius = radius } },
        scale = {1.0, 1.0, 1.0}
    }
end

print("[PHYSICS_3D] Setting up 3D physics scene...")

-- Spawn point light to illuminate the scene
spawn({
    PointLight = {
        shadows_enabled = true,
        color = { r = 1.0, g = 1.0, b = 1.0, a = 1.0 },
        intensity = 800000.0,  -- Bevy 0.15+ uses lumens
        range = 100.0,
        radius = 0.0
    },
    Transform = {
        translation = { x = 4.0, y = 8.0, z = 4.0 }
    }
})

-- Create ground platform mesh and material
local ground_mesh = create_asset("bevy_mesh::mesh::Mesh", {
    primitive = { 
        Cuboid = { 
            half_size = { x = 25.0, y = 0.5, z = 25.0 }  -- Large flat platform
        }
    }
})

local ground_material = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
    base_color = { r = 0.3, g = 0.8, b = 0.3, a = 1.0 },  -- Green platform
    perceptual_roughness = 0.8,
    metallic = 0.0,
    unlit = false  -- Use PBR but shadows are disabled on light
})

-- Spawn ground platform with physics
spawn({
    Mesh3d = { _0 = ground_mesh },
    ["MeshMaterial3d<StandardMaterial>"] = { _0 = ground_material },
    Transform = { 
        translation = { x = 0.0, y = -0.5, z = 0.0 },
        rotation = { x = 0.0, y = 0.0, z = 0.0, w = 1.0 },
        scale = { x = 1.0, y = 1.0, z = 1.0 }
    },
    RigidBody = "Fixed",
    Collider = ColliderCuboid(25.0, 0.5, 25.0)
})

print("[PHYSICS_3D] Ground platform spawned")

-- Create cube mesh and materials for falling boxes
local cube_mesh = create_asset("bevy_mesh::mesh::Mesh", {
    primitive = { 
        Cuboid = { 
            half_size = { x = 0.5, y = 0.5, z = 0.5 }
        }
    }
})

-- Spawn falling boxes with different colors
local colors = {
    { r = 1.0, g = 0.3, b = 0.3, a = 1.0 },  -- Red
    { r = 0.3, g = 0.3, b = 1.0, a = 1.0 },  -- Blue
    { r = 1.0, g = 1.0, b = 0.3, a = 1.0 },  -- Yellow
    { r = 1.0, g = 0.5, b = 0.0, a = 1.0 },  -- Orange
    { r = 0.8, g = 0.3, b = 1.0, a = 1.0 }   -- Purple
}

for i = 1, 5 do
    local x_offset = (i - 3) * 2.0  -- Spread boxes horizontally
    local y_offset = 2.0 + i * 2.0  -- Stack them at different heights
    
    local box_material = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
        base_color = colors[i],
        perceptual_roughness = 0.5,
        metallic = 0.0,
        unlit = false  -- Use PBR but shadows are disabled on light
    })
    
    spawn({
        Mesh3d = { _0 = cube_mesh },
        ["MeshMaterial3d<StandardMaterial>"] = { _0 = box_material },
        Transform = { 
            translation = { x = x_offset, y = y_offset, z = 0.0 },
            rotation = { x = 0.0, y = 0.0, z = 0.0, w = 1.0 },
            scale = { x = 1.0, y = 1.0, z = 1.0 }
        },
        RigidBody = "Dynamic",
        Collider = ColliderCuboid(0.5, 0.5, 0.5),
        Restitution = { coefficient = 0.3 },
        GravityScale = 1.0
    })
end

print("[PHYSICS_3D] Falling boxes spawned")

-- Spawn camera positioned to view the scene
-- Looking at the center from an angle
spawn({
    Camera3d = {},
    Transform = {
        translation = { x = -8.0, y = 6.0, z = 12.0 },  -- Position camera back and up
        rotation = { x = -0.15, y = 0.25, z = 0.04, w = 0.96 }  -- Rotate to look at center (approximate)
    }
})

register_system("Update", function(world)
    local entities = world:query({"RigidBody"})
    print(#entities)
end)

print("[PHYSICS_3D] 3D physics scene ready - watch the boxes fall!")