# Bevy Lua Entity

A Lua scripting integration for Bevy that follows the **"Zero Rust" philosophy** - build entire games in Lua using generic, reflection-based ECS infrastructure.

## Philosophy

All game logic, systems, and behaviors are implemented purely in Lua. Rust code provides only generic ECS operations using Bevy's reflection system. This means:

- ✅ Game developers write **only Lua** for game features
- ✅ Rust provides **generic utilities** (queries, component mutation, time access)
- ✅ **Any** Bevy component can be used from Lua via reflection
- ✅ Custom Lua components work alongside Bevy components

## Quick Start

### 1. Add to Your Project

```toml
[dependencies]
bevy = "0.15"
mlua = "0.10"
```

### 2. Setup Your App

```rust
use bevy::prelude::*;
use bevy_lua_entity::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    
    // Create component registry from type registry
    let component_registry = ComponentRegistry::from_type_registry(
        app.world().resource::<AppTypeRegistry>().clone()
    );
    
    app.insert_resource(component_registry)
        .init_resource::<SpawnQueue>()
        .init_resource::<ComponentUpdateQueue>()
        .add_plugins(LuaSpawnPlugin)
        .add_systems(Update, (
            process_spawn_queue,
            run_lua_systems,
            process_component_updates,
        ))
        .add_systems(Startup, setup)
        .run();
}
```

### 3. Write Lua Scripts

```lua
-- Spawn entities with any Bevy component
spawn({
    Sprite = { color = {r = 1.0, g = 0.0, b = 0.0, a = 1.0} },
    Transform = { 
        translation = {x = 0, y = 0, z = 0},
        scale = {x = 50.0, y = 50.0, z = 1.0}
    }
})

-- Define systems entirely in Lua
function my_system(world)
    local dt = world:delta_time()
    local entities = world:query({"Sprite", "Transform"}, nil)
    
    for i, entity in ipairs(entities) do
        -- Update components
        entity:set("Sprite", { 
            color = {r = math.sin(dt), g = 0.5, b = 1.0, a = 1.0} 
        })
    end
end

register_system("Update", my_system)
```

## Features

### Core Capabilities

- **Component Spawning**: Create entities with any reflected Bevy component
- **Component Queries**: Query entities with filtering and change detection
- **Component Mutation**: Update components via `entity:set()`
- **Lua Systems**: Register Lua functions to run every frame
- **Generic Lua Components**: Store arbitrary Lua data as components
- **Time Access**: Get delta time and frame information

### Lua API

#### Spawning Entities

```lua
spawn({
    ComponentName = { field1 = value1, field2 = value2 },
    AnotherComponent = { ... }
})
```

#### Querying Entities

```lua
-- Query with required components
local entities = world:query({"Component1", "Component2"}, nil)

-- Query with change detection
local changed = world:query({"Transform"}, {"Transform"})
```

#### Updating Components

```lua
entity:set("ComponentName", { field = new_value })
```

#### Registering Systems

```lua
function my_system(world)
    -- System logic here
end

register_system("Update", my_system)
```

#### Accessing Time

```lua
local dt = world:delta_time()
```

## Examples

### Animated Sprites

See `examples/sprites.rs` and `assets/scripts/spawn_sprites.lua` for a complete example of:
- Spawning sprites with reflection
- Animating colors using Lua systems
- Component mutation every frame

### Interactive UI

See `examples/button.rs` and `assets/scripts/spawn_button.lua` for:
- UI component creation
- Click event handling in Lua
- Custom Lua components (callbacks)

## Architecture

### Rust Layer (Generic Infrastructure)

- **ComponentRegistry**: Auto-discovers Bevy components via reflection
- **SpawnQueue**: Queues entity creation from Lua
- **ComponentUpdateQueue**: Queues component updates from Lua
- **LuaSystemRegistry**: Manages Lua systems to run each frame
- **Query API**: Provides ECS queries to Lua

### Lua Layer (Game Logic)

- All game systems, behaviors, and logic
- Entity spawning and component updates
- Event handling and callbacks
- Animation and gameplay code

## Design Principles

1. **Generic Rust Code**: Rust provides only reflection-based ECS utilities
2. **Lua-First Features**: All game features implemented in Lua
3. **No Game-Specific Rust**: Animation, UI, gameplay all in Lua
4. **Automatic Component Discovery**: Any `#[reflect(Component)]` type works
5. **Type Safety**: Reflection ensures correct component structure

## Running Examples

```bash
# Animated sprites with color pulsing
cargo run --example sprites

# Interactive button with click handling
cargo run --example button

# Basic Lua integration
cargo run --example basic
```
