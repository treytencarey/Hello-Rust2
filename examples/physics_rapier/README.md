# Physics Plugin Example (bevy_rapier2d)

This example demonstrates the "Zero Rust" philosophy applied to an **external plugin**. All physics entities and behaviors are defined purely in Lua, using components from the `bevy_rapier2d` physics plugin.

## Key Concept

When you add the `RapierPhysicsPlugin` to your Bevy app, Rapier's components (like `RigidBody`, `Collider`, `Velocity`, etc.) are automatically registered in Bevy's `TypeRegistry` because they implement the `Reflect` trait. This means they become immediately available to Lua through the existing reflection-based infrastructure - **no custom Rust code needed!**

## Building and Running

```powershell
# From the example directory
cd examples/physics_rapier
cargo run

# Or from the project root
cargo run --example physics_rapier_example
```

## What This Example Shows

### Physics Entities (All in Lua!)

1. **Ground Platform** - Static rigid body that doesn't move
   - `RigidBody = "Fixed"` - Immovable
   - `Collider` with cuboid shape
   - `Sprite` for visual representation

2. **Falling Boxes** - Dynamic rigid bodies affected by physics
   - `RigidBody = "Dynamic"` - Simulated by physics engine
   - `Collider` for collision detection
   - `Velocity` for initial movement
   - `GravityScale` to control gravity effect
   - `Restitution` for bounciness
   - `Sprite` for visual representation

3. **Physics Monitor System** - Lua system that tracks physics state
   - Queries entities with `RigidBody` and `Velocity`
   - Calculates total kinetic energy
   - Logs physics state periodically

### Available Rapier Components (Use by Name in Lua)

All of these work automatically through reflection:

- `RigidBody` - Defines physics behavior: `"Dynamic"`, `"Fixed"`, `"KinematicPositionBased"`, `"KinematicVelocityBased"`
- `Collider` - Collision shape (cuboid, ball, capsule, etc.)
- `Velocity` - Linear and angular velocity
- `GravityScale` - Multiplier for gravity effect
- `Restitution` - Bounciness (0.0 = no bounce, 1.0 = perfect bounce)
- `Friction` - Surface friction
- `Damping` - Air resistance
- `LockedAxes` - Restrict movement on certain axes
- `Ccd` - Continuous collision detection for fast objects
- `Sensor` - Trigger collisions without physical response
- And many more!

## The "Zero Rust" Philosophy

### What's in Rust (Generic Infrastructure)

```rust
// Just add the plugin - that's it!
app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0));

// Component registry automatically includes Rapier's components
let component_registry = ComponentRegistry::from_type_registry(
    app.world().resource::<AppTypeRegistry>().clone()
);

// For non-Reflect components (like Collider), use the serde_components! macro
app.insert_resource(bevy_lua_entity::serde_components![
    Collider,
]);
```

> **Note**: Some Rapier components like `Collider` don't implement `Reflect` but do implement `Deserialize`. For these, we use the `serde_components!` macro to register them. This is still "Zero Rust" - just list the types once, and the macro handles everything automatically.

### Serde Components (Non-Reflect Types)

For components that implement `Deserialize` but not `Reflect`, use the `serde_components!` macro:

```rust
app.insert_resource(bevy_lua_entity::serde_components![
    Collider,
    // Add more serde-based components here
]);
```

In Lua, these components require a JSON-compatible format. Helper functions make this easy:

```lua
-- Helper function for Collider
function ColliderCuboid(hx, hy)
    return {
        raw = { Cuboid = { half_extents = {hx, hy} } },
        unscaled = { Cuboid = { half_extents = {hx, hy} } },
        scale = {1.0, 1.0}
    }
end

-- Usage
spawn({
    RigidBody = "Dynamic",
    Collider = ColliderCuboid(25.0, 25.0),
    -- ...
})
```

### What's in Lua (All Game Logic)

```lua
-- Spawn physics entities
spawn({
    RigidBody = "Dynamic",
    Collider = { shape = "Cuboid", hx = 25.0, hy = 25.0 },
    Velocity = { linvel = { x = 0.0, y = 0.0 }, angvel = 0.0 },
    GravityScale = 1.0,
    Restitution = 0.7,
    Transform = { translation = { x = 0.0, y = 200.0, z = 0.0 } },
    Sprite = { color = { r = 1.0, g = 0.0, b = 0.0, a = 1.0 } }
})

-- Monitor and manipulate physics at runtime
function physics_system(world)
    local entities = world:query({"RigidBody", "Velocity"}, nil)
    for _, entity in ipairs(entities) do
        local velocity = entity:get("Velocity")
        -- Read and modify physics state in Lua!
    end
end
```

## Extending the Example

You can modify `assets/scripts/physics_example.lua` to:

- Add more complex shapes (balls, capsules, polygons)
- Create joints between bodies
- Apply forces and impulses
- Implement character controllers
- Build physics-based puzzles
- Create ragdoll systems

All without touching Rust code - just edit the Lua script and re-run!

## Comparison with Traditional Approach

### Traditional Rust Approach
```rust
// Spawn physics entity in Rust
commands.spawn((
    RigidBody::Dynamic,
    Collider::cuboid(25.0, 25.0),
    Velocity::default(),
    GravityScale(1.0),
    Restitution::coefficient(0.7),
    TransformBundle::from_transform(Transform::from_xyz(0.0, 200.0, 0.0)),
    SpriteBundle { /* ... */ }
));
```

### Lua Approach (This Example)
```lua
-- Same entity, defined in Lua
spawn({
    RigidBody = "Dynamic",
    Collider = { shape = "Cuboid", hx = 25.0, hy = 25.0 },
    Velocity = { linvel = { x = 0.0, y = 0.0 }, angvel = 0.0 },
    GravityScale = 1.0,
    Restitution = 0.7,
    Transform = { translation = { x = 0.0, y = 200.0, z = 0.0 } },
    Sprite = { color = { r = 1.0, g = 0.0, b = 0.0, a = 1.0 } }
})
```

**Benefits:**
- No Rust compilation needed for changes
- Rapid iteration (edit Lua, re-run)
- Same functionality, more flexibility
- External plugin components work automatically

## Key Takeaway

This example proves that **any external Bevy plugin** that uses reflected ECS components can be used from Lua with **zero custom Rust code**. The reflection-based infrastructure is completely generic and works with any plugin!
