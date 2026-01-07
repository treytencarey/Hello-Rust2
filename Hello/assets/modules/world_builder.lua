-- World Builder Module
-- Role-aware static geometry creation (floors, walls, etc.)
--
-- Usage:
--   local WorldBuilder = require("modules/world_builder.lua")
--   WorldBuilder.create_floor({
--       size = {x = 50, y = 1, z = 50},
--       position = {x = 0, y = -0.5, z = 0},
--       color = {r = 0.3, g = 0.8, b = 0.3, a = 1}
--   })
--   -- Visuals auto-applied based on network role

local NetRole = require("modules/net_role.lua")

local WorldBuilder = {}

--------------------------------------------------------------------------------
-- Collider Helpers
--------------------------------------------------------------------------------

--- Create a cuboid collider definition
function WorldBuilder.ColliderCuboid(hx, hy, hz)
    return {
        raw = { Cuboid = { half_extents = {hx, hy, hz} } },
        unscaled = { Cuboid = { half_extents = {hx, hy, hz} } },
        scale = {1.0, 1.0, 1.0}
    }
end

--- Create a ball collider definition
function WorldBuilder.ColliderBall(radius)
    return {
        raw = { Ball = { radius = radius } },
        unscaled = { Ball = { radius = radius } },
        scale = {1.0, 1.0, 1.0}
    }
end

--------------------------------------------------------------------------------
-- Mesh/Material Creation
--------------------------------------------------------------------------------

local function create_cuboid_mesh(size)
    return create_asset("bevy_mesh::mesh::Mesh", {
        primitive = {
            Cuboid = {
                half_size = { x = size.x / 2, y = size.y / 2, z = size.z / 2 }
            }
        }
    })
end

local function create_material(color)
    return create_asset("bevy_pbr::pbr_material::StandardMaterial", {
        base_color = color,
        perceptual_roughness = 0.8,
        metallic = 0.0,
        unlit = false
    })
end

--------------------------------------------------------------------------------
-- Floor Creation
--------------------------------------------------------------------------------

--- Create a floor entity with physics and optional visuals
--- @param config table { size = {x,y,z}, position = {x,y,z}, color = {r,g,b,a} }
--- @return number entity_id
function WorldBuilder.create_floor(config)
    local size = config.size or {x = 50, y = 1, z = 50}
    local position = config.position or {x = 0, y = -0.5, z = 0}
    local color = config.color or {r = 0.3, g = 0.8, b = 0.3, a = 1.0}
    
    local spawn_data = {
        Transform = {
            translation = position,
            rotation = {x = 0, y = 0, z = 0, w = 1},
            scale = {x = 1, y = 1, z = 1}
        },
        RigidBody = "Fixed",
        Collider = WorldBuilder.ColliderCuboid(size.x / 2, size.y / 2, size.z / 2)
    }
    
    -- Add visuals only if we're a client (or offline for testing)
    if NetRole.is_client() or NetRole.is_offline() then
        local mesh = create_cuboid_mesh(size)
        local material = create_material(color)
        spawn_data.Mesh3d = { _0 = mesh }
        spawn_data["MeshMaterial3d<StandardMaterial>"] = { _0 = material }
    end
    
    local entity_id = spawn(spawn_data)
    print(string.format("[WORLD_BUILDER] Created floor at (%.1f, %.1f, %.1f) size (%.1f, %.1f, %.1f) visuals=%s",
        position.x, position.y, position.z,
        size.x, size.y, size.z,
        tostring(NetRole.is_client() or NetRole.is_offline())))
    
    return entity_id
end

--------------------------------------------------------------------------------
-- Wall Creation
--------------------------------------------------------------------------------

--- Create a wall entity with physics and optional visuals
--- @param config table { size = {x,y,z}, position = {x,y,z}, rotation = {x,y,z,w}, color = {r,g,b,a} }
--- @return number entity_id
function WorldBuilder.create_wall(config)
    local size = config.size or {x = 10, y = 3, z = 0.5}
    local position = config.position or {x = 0, y = 1.5, z = 0}
    local rotation = config.rotation or {x = 0, y = 0, z = 0, w = 1}
    local color = config.color or {r = 0.5, g = 0.5, b = 0.5, a = 1.0}
    
    local spawn_data = {
        Transform = {
            translation = position,
            rotation = rotation,
            scale = {x = 1, y = 1, z = 1}
        },
        RigidBody = "Fixed",
        Collider = WorldBuilder.ColliderCuboid(size.x / 2, size.y / 2, size.z / 2)
    }
    
    -- Add visuals only if we're a client (or offline for testing)
    if NetRole.is_client() or NetRole.is_offline() then
        local mesh = create_cuboid_mesh(size)
        local material = create_material(color)
        spawn_data.Mesh3d = { _0 = mesh }
        spawn_data["MeshMaterial3d<StandardMaterial>"] = { _0 = material }
    end
    
    local entity_id = spawn(spawn_data)
    print(string.format("[WORLD_BUILDER] Created wall at (%.1f, %.1f, %.1f)",
        position.x, position.y, position.z))
    
    return entity_id
end

--------------------------------------------------------------------------------
-- Lighting
--------------------------------------------------------------------------------

--- Create a point light (client-only)
--- @param config table { position = {x,y,z}, intensity = number, color = {r,g,b,a} }
--- @return number|nil entity_id (nil if server-only)
function WorldBuilder.create_light(config)
    -- Lights are client-only
    if not (NetRole.is_client() or NetRole.is_offline()) then
        return nil
    end
    
    local position = config.position or {x = 4, y = 8, z = 4}
    local intensity = config.intensity or 800000.0
    local color = config.color or {r = 1.0, g = 1.0, b = 1.0, a = 1.0}
    
    local entity_id = spawn({
        PointLight = {
            shadows_enabled = true,
            color = color,
            intensity = intensity,
            range = 100.0,
            radius = 0.0
        },
        Transform = {
            translation = position
        }
    })
    
    print(string.format("[WORLD_BUILDER] Created light at (%.1f, %.1f, %.1f)",
        position.x, position.y, position.z))
    
    return entity_id
end

print("[WORLD_BUILDER] Module loaded")

return WorldBuilder
