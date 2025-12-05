-- Hello Game Main Script
-- Testing with just light and camera first

-- Helper function to create mesh assets and spawn 3D entities
local function create_mesh_material_bundle(mesh_type, position, scale, color)
    print("[LUA] Creating " .. mesh_type .. " at position x=" .. position.x .. ", y=" .. position.y .. ", z=" .. position.z)
    -- Create mesh asset
    local mesh_id
    if mesh_type == "Cuboid" then
        mesh_id = create_asset("bevy_mesh::mesh::Mesh", {
            primitive = { 
                Cuboid = { 
                    half_size = { x = 0.5, y = 0.5, z = 0.5 }
                }
            }
        })
    elseif mesh_type == "Circle" then
        mesh_id = create_asset("bevy_mesh::mesh::Mesh", {
            primitive = { 
                Circle = { 
                    radius = 1.0
                }
            }
        })
    elseif mesh_type == "Sphere" then
        mesh_id = create_asset("bevy_mesh::mesh::Mesh", {
            primitive = { 
                Sphere = { 
                    radius = 1.0
                }
            }
        })
    end
    
    -- Create material asset
    local material_id = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
        base_color = color or { r = 0.0, g = 0.0, b = 1.0, a = 1.0 },
        perceptual_roughness = 0.5,
        metallic = 0.0
    })
    
    print("[LUA] Created mesh_id=" .. mesh_id .. ", material_id=" .. material_id)
    
    -- Spawn entity with mesh and material
    print("[LUA] Spawning entity with Mesh3d._0=" .. mesh_id .. ", MeshMaterial3d._0=" .. material_id)
    return spawn({
        Mesh3d = { _0 = mesh_id },  -- Tuple struct: Mesh3d(Handle<Mesh>)
        ["MeshMaterial3d<StandardMaterial>"] = { _0 = material_id },  -- Full generic type name
        Transform = {
            translation = position or { x = 0.0, y = 0.0, z = 0.0 },
            rotation = { x = 0.0, y = 0.0, z = 0.0, w = 1.0 },
            scale = scale or { x = 1.0, y = 1.0, z = 1.0 }
        }
        -- Note: Visibility components should be added automatically by Bevy
    })
end

--[[
-- Spawn ground plane (using Plane3d/Rectangle for proper PBR rendering)
print("[LUA] Creating ground plane (Rectangle 8x8)")
local ground_mesh = create_asset("bevy_mesh::mesh::Mesh", {
    primitive = { 
        Plane3d = { 
            normal = { x = 0.0, y = 1.0, z = 0.0 },
            half_size = { x = 4.0, y = 4.0 }
        }
    }
})
local ground_material = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
    base_color = { r = 0.8, g = 0.8, b = 0.8, a = 1.0 }
})
print("[LUA] Ground: mesh_id=" .. ground_mesh .. ", material_id=" .. ground_material)
print("[LUA] Spawning ground entity with Mesh3d._0=" .. ground_mesh .. ", MeshMaterial3d._0=" .. ground_material)
spawn({
    Mesh3d = { _0 = ground_mesh },
    ["MeshMaterial3d<StandardMaterial>"] = { _0 = ground_material },
    Visibility = "Inherited",
    Transform = {
        translation = { x = 0.0, y = 0.0, z = 0.0 },
        rotation = { x = 0.0, y = 0.0, z = 0.0, w = 1.0 },
        scale = { x = 1.0, y = 1.0, z = 1.0 }
    }
})
--]]

-- Spawn cube right in front of viewer
-- Spawn sphere and store its entity ID for bouncing
local sphere_entity = create_mesh_material_bundle("Sphere", 
    { x = 0.0, y = 1.5, z = -1.5 },  -- 3 meters in front, at eye level
    { x = 0.5, y = 0.5, z = 0.5 },  -- Half meter cube
    { r = 1.0, g = 0.0, b = 0.0, a = 1.0 }  -- Bright red for visibility
)

-- Bounce parameters
local bounce_amplitude = 50.0  -- meters
local bounce_speed = 1.0      -- radians per second
local base_y = 1.5            -- original Y position

-- Per-frame update system to bounce the sphere
register_system("Update", function(world)
    local dt = world:delta_time()
    if sphere_entity ~= nil then
        local bounce_y = base_y + math.sin(dt * bounce_speed) * bounce_amplitude
        local entities = world:query({"Transform", "Mesh3d"}, nil)
        if #entities > 0 then
            entities[1]:set("Transform", {
                translation = { x = 0.0, y = bounce_y, z = -1.5 },
                rotation = { x = 0.0, y = 0.0, z = 0.0, w = 1.0 },
                scale = { x = 0.5, y = 0.5, z = 0.5 }
            })
        end
    end
end)

--[[ SECOND CUBE DISABLED FOR INITIAL TESTING
-- Spawn another cube to the side
create_mesh_material_bundle("Cuboid", 
    { x = 2.0, y = 1.5, z = -3.0 },
    { x = 0.5, y = 0.5, z = 0.5 },
    { r = 0.0, g = 1.0, b = 0.0, a = 1.0 }  -- Green
)
--]]

print("[LUA] Spawning light, camera, and ONE test cube")

-- Spawn point light
spawn({
    PointLight = {
        shadows_enabled = true,
        color = { r = 1.0, g = 0.647, b = 0.0, a = 1.0 },
        intensity = 8000000.0,  -- Bevy 0.15+ uses lumens
        range = 100.0,
        radius = 0.0
    },
    Transform = {
        translation = { x = 4.0, y = 8.0, z = 4.0 }
    }
})

-- Spawn camera (VR will override this with its own cameras)
spawn({
    Camera3d = {},
    Transform = {
        translation = { x = -2.5, y = 4.5, z = 9.0 },
        rotation = { x = 0.0, y = 0.0, z = 0.0, w = 1.0 }
    }
})

print("3D scene spawned from Lua with proper mesh/material assets")
