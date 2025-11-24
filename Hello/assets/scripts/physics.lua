-- Physics Example Script
-- Demonstrates spawning entities with Rapier physics components

print("Physics example initialized!")

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

-- Spawn ground platform
spawn({
    Sprite = { 
        color = {r = 0.3, g = 0.7, b = 0.3, a = 1.0},
        custom_size = {x = 600, y = 20}
    },
    Transform = { translation = {x = 0, y = -200, z = 0} },
    RigidBody = "Fixed",
    Collider = ColliderCuboid(300.0, 10.0)
})

print("✓ Ground platform spawned")

-- Spawn dynamic bouncing ball
spawn({
    Sprite = { 
        color = {r = 1.0, g = 0.3, b = 0.3, a = 1.0},
        custom_size = {x = 40, y = 40}
    },
    Transform = { translation = {x = 0, y = 200, z = 0} },
    RigidBody = "Dynamic",
    Collider = ColliderBall(20.0),
    Restitution = { coefficient = 0.7 },
    GravityScale = 1.0
})

print("✓ Bouncing ball spawned")

-- Spawn some falling boxes
for i = 1, 5 do
    local x_offset = (i - 3) * 60
    spawn({
        Sprite = { 
            color = {r = 0.2, g = 0.5, b = 1.0, a = 1.0},
            custom_size = {x = 50, y = 50}
        },
        Transform = { translation = {x = x_offset, y = 100 + i * 50, z = 0} },
        RigidBody = "Dynamic",
        Collider = ColliderCuboid(25.0, 25.0),
        Restitution = { coefficient = 0.3 }
    })
end

print("✓ Falling boxes spawned")

-- Add instruction text
spawn({
    Text = { text = "Physics Demo - Watch the objects fall and collide!" },
    TextFont = { font_size = 32 },
    TextColor = { color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0} },
    TextLayout = {},
    Transform = { translation = {x = 0, y = 300, z = 0} }
})

print("✓ Physics example ready!")
