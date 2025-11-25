-- Physics simulation using bevy_rapier2d components
-- All physics entities and behavior defined purely in Lua!

-- Helper functions to create Collider components (wrapping the complex JSON structure)
function ColliderCuboid(hx, hy)
    return {
        raw = { Cuboid = { half_extents = {hx, hy} } },
        unscaled = { Cuboid = { half_extents = {hx, hy} } },
        scale = {1.0, 1.0}
    }
end

function ColliderBall(radius)
    return {
        raw = { Ball = { radius = radius } },
        unscaled = { Ball = { radius = radius } },
        scale = {1.0, 1.0}
    }
end

-- Spawn a static ground platform
spawn({
    RigidBody = "Fixed",
    Collider = ColliderCuboid(400.0, 20.0),
    Transform = {
        translation = { x = 0.0, y = -300.0, z = 0.0 },
        scale = { x = 1.0, y = 1.0, z = 1.0 }
    },
    Sprite = {
        color = { r = 0.5, g = 0.5, b = 0.5, a = 1.0 },
        custom_size = { x = 800.0, y = 40.0 }
    }
})

print("Ground platform spawned")

-- Spawn multiple falling boxes with physics
local box_positions = {
    { x = -150.0, y = 200.0 },
    { x = -50.0, y = 300.0 },
    { x = 50.0, y = 250.0 },
    { x = 150.0, y = 350.0 }
}

local box_colors = {
    { r = 1.0, g = 0.3, b = 0.3, a = 1.0 },  -- Red
    { r = 0.3, g = 1.0, b = 0.3, a = 1.0 },  -- Green
    { r = 0.3, g = 0.3, b = 1.0, a = 1.0 },  -- Blue
    { r = 1.0, g = 1.0, b = 0.3, a = 1.0 }   -- Yellow
}

for i, pos in ipairs(box_positions) do
    spawn({
        RigidBody = "Dynamic",
        Collider = ColliderCuboid(25.0, 25.0),
        Transform = {
            translation = { x = pos.x, y = pos.y, z = 0.0 },
        },
        Sprite = {
            color = box_colors[i],
            custom_size = { x = 50.0, y = 50.0 }
        }
    })
end

print("" .. #box_positions .. " physics boxes spawned")

-- Frame counter for periodic logging
local frame_count = 0

-- Optional: Lua system to monitor physics state
function physics_monitor_system(world)
    frame_count = frame_count + 1
    
    -- Log physics state every 2 seconds (assuming 60 FPS)
    if frame_count % 120 == 0 then
        local entities = world:query({"RigidBody", "Transform"}, nil)
        
        -- Show position of first dynamic body
        if #entities > 1 then
            local entity = entities[2]  -- Skip ground (index 1), get first box
            local transform = entity:get("Transform")
            if transform and transform.translation then
                print(string.format(
                    "Physics state: %d rigid bodies | First box Y position: %.1f",
                    #entities,
                    transform.translation.y
                ))
            end
        else
            print(string.format("Physics state: %d rigid bodies active", #entities))
        end
    end
end

register_system("Update", physics_monitor_system)

print("Physics monitor system registered")
