# Hello

A Lua-first game framework built on [Bevy](https://bevyengine.org/) and [bevy-lua-ecs](./bevy-lua-ecs).

## Philosophy

Hello is a collection of optional game plugins with full Lua bindings support. Following the "Zero Rust" philosophy, all game logic is written in Lua while Rust provides generic, plugin-based infrastructure.

### Key Principles

- **Lua-First Development**: Write your entire game in Lua scripts
- **Plugin Architecture**: Optional feature modules (physics, audio, networking, etc.)
- **Self-Contained Plugins**: Each plugin handles its own setup and Lua integration
- **No Manual Registration**: Plugins automatically register their components for Lua

## Project Structure

```
Hello/
├── bevy-lua-ecs/              # Core Lua-ECS integration library
│   ├── src/                   # Generic reflection-based infrastructure
│   └── examples/              # Lua scripting examples
└── Hello/                     # Game framework with optional plugins
    ├── src/
    │   ├── main.rs            # CLI demo runner
    │   ├── lib.rs             # Library exports
    │   └── plugins/           # Plugin modules
    │       ├── core.rs        # HelloCorePlugin (required)
    │       ├── physics.rs     # HelloPhysicsPlugin
    │       ├── networking.rs  # HelloNetworkingPlugin
    │       └── tiled.rs       # HelloTiledPlugin
    └── assets/scripts/        # Lua game scripts
        ├── main.lua           # Default client script
        └── server/main.lua    # Server mode script
```

## Quick Start

### CLI Demo Runner

```powershell
# Show help
cargo run -- --help

# List available demos
cargo run -- --list

# Run a specific demo
cargo run -- --demo physics
cargo run -- --demo tilemap
cargo run -- --demo ui_3d

# Run a custom script
cargo run -- --script scripts/examples/my_script.lua
```

### Network Modes

```powershell
# Start as server only (serves assets, no game UI)
cargo run -- --network server

# Start as client only (connects to server for assets)
cargo run -- --network client --server-addr 192.168.1.100 --port 5000

# Start as full peer (both server and client - default)
cargo run -- --network both

# Run networking demo
cargo run -- --demo networking
```

**Script Paths by Mode:**
- **Server**: Runs `scripts/server/main.lua`
- **Client/Both**: Runs `scripts/main.lua`

## Features

### Default Features

All plugins are enabled by default via `all-plugins`:

```toml
[features]
default = ["all-plugins"]
all-plugins = ["physics", "tiled", "networking"]
```

### Available Plugins

#### Physics (Rapier 2D)
- 2D physics simulation with rigid bodies, colliders, and joints
- Run: `cargo run -- --demo physics`

#### Tiled Maps
- Load and render Tiled map editor (.tmx) files
- Run: `cargo run -- --demo tilemap`

#### Networking (Asset Peer)
- On-demand asset downloading from server
- Hot reload when server files change
- Three modes: `server`, `client`, `both` (peer)
- Broadcast loop prevention for peer mode
- Run: `cargo run -- --network both`

## Adding New Plugins

To add a new optional plugin following the Hello architecture:

1. **Create a feature in `Cargo.toml`**:
```toml
[features]
my_plugin = ["dep:some_crate"]

[dependencies]
some_crate = { version = "1.0", optional = true }
```

2. **Create a plugin module** (e.g., `src/my_plugin.rs`):
```rust
use bevy::prelude::*;
use bevy_lua_ecs::*;

pub struct MyPluginIntegration;

impl Plugin for MyPluginIntegration {
    fn build(&self, app: &mut App) {
        // Add external plugins
        app.add_plugins(SomeExternalPlugin);
        
        // Register serde components if needed
        app.insert_resource(bevy_lua_ecs::serde_components![
            SomeComponent,
        ]);
        
        info!("✓ My plugin integration enabled");
    }
}
```

3. **Conditionally add to main.rs**:
```rust
#[cfg(feature = "my_plugin")]
mod my_plugin;

#[cfg(feature = "my_plugin")]
app.add_plugins(my_plugin::MyPluginIntegration);
```

4. **Write Lua scripts** that use the plugin's components and systems.

That's it! No manual component registration, no game-specific Rust code.

## Examples

### Physics Simulation

```lua
-- Spawn a dynamic physics box
spawn({
    RigidBody = "Dynamic",
    Collider = ColliderCuboid(25.0, 25.0),
    Velocity = { linvel = { x = 0.0, y = 0.0 }, angvel = 0.0 },
    Transform = { translation = { x = 0.0, y = 200.0, z = 0.0 } },
    Sprite = { color = { r = 1.0, g = 0.0, b = 0.0, a = 1.0 } }
})

-- Monitor physics state
function physics_monitor(world)
    local entities = world:query({"RigidBody", "Transform"}, nil)
    for _, entity in ipairs(entities) do
        -- Read physics data
        local transform = entity:get("Transform")
        -- React to physics state
    end
end

register_system("Update", physics_monitor)
```

## Design Goals

### For Plugin Developers (Rust)

- Create self-contained plugins with minimal boilerplate
- Automatically expose components to Lua via reflection
- No game logic in Rust - only infrastructure

### For Game Developers (Lua)

- Write entire game in Lua scripts
- Use any plugin component without configuration
- Hot-reload scripts during development
- Full ECS query and mutation capabilities

## Dependencies

- [Bevy 0.16](https://bevyengine.org/) - Game engine
- [mlua 0.10](https://github.com/mlua-rs/mlua) - Lua bindings
- [bevy_rapier2d 0.31](https://github.com/dimforge/bevy_rapier) - Physics (optional)

## Related Projects

- [bevy-lua-ecs](./bevy-lua-ecs) - The core Lua-ECS integration library

## License

This project is private.
