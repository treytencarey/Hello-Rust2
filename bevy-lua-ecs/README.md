# Bevy Lua Entity

A Lua scripting integration for Bevy that follows the **"Zero Rust" philosophy** - build entire games in Lua using generic, reflection-based ECS infrastructure.

## Philosophy

All game logic, systems, and behaviors are implemented purely in Lua. Rust code provides only generic ECS operations using Bevy's reflection system. This means:

- ✅ Game developers write **only Lua** for game features
- ✅ Rust provides **generic utilities** (queries, component mutation, time access, resource management)
- ✅ **Any** Bevy component can be used from Lua via reflection
- ✅ **Any** Bevy resource can be inserted from Lua via builders
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
    
    // Create asset registry for asset loading/creation
    let asset_registry = AssetRegistry::default();
    
    app.insert_resource(component_registry)
        .insert_resource(asset_registry)
        .init_resource::<SpawnQueue>()
        .init_resource::<ComponentUpdateQueue>()
        .init_resource::<ResourceQueue>()
        .init_resource::<ResourceBuilderRegistry>()
        .init_resource::<SerdeComponentRegistry>()
        .add_plugins(LuaSpawnPlugin)
        .add_systems(Update, (
            process_spawn_queue,
            process_resource_queue,  // Process resources inserted from Lua
            process_pending_assets,  // Process assets created from Lua
            run_lua_systems,
            process_component_updates,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    world: &mut World,
) {
    // Setup Lua context with asset loading
    let lua_ctx = LuaScriptContext::new().unwrap();
    let asset_server = world.resource::<AssetServer>().clone();
    let asset_registry = world.resource::<AssetRegistry>().clone();
    
    add_asset_loading_to_lua(&lua_ctx, asset_server, asset_registry.clone()).unwrap();
    
    // Link asset registry to component registry
    world.resource_mut::<ComponentRegistry>()
        .set_asset_registry(asset_registry);
    
    world.insert_resource(lua_ctx);
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
- **Asset Loading**: Load image files and create assets via reflection
- **Asset Creation**: Create any Bevy asset type (layouts, materials, etc.) from Lua
- **Resource Management**: Insert and query Bevy resources from Lua via builders
- **Component Queries**: Query entities with filtering and change detection
- **Component Mutation**: Update components via `entity:set()`
- **Lua Systems**: Register Lua functions to run every frame
- **Generic Lua Components**: Store arbitrary Lua data as components
- **Marker Components**: Register serializable marker components for special purposes
- **Time Access**: Get delta time and frame information

### Lua API

#### Loading and Creating Assets

```lua
-- Load image files
local texture = load_asset("path/to/image.png")

-- Create any Bevy asset via reflection
local layout = create_asset("bevy_sprite::texture_atlas::layout::TextureAtlasLayout", {
    tile_size = { x = 16, y = 16 },
    columns = 10,
    rows = 5
})

-- Use assets in components
spawn({
    Sprite = {
        image = texture,
        rect = {
            min = { x = 0, y = 0 },
            max = { x = 16, y = 16 }
        }
    }
})
```

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

#### Inserting Resources

```lua
-- Register resource builders in Rust first, then use from Lua
insert_resource("MyResource", { field1 = value1, field2 = value2 })
```

#### Querying Resources

```lua
-- Check if a resource exists
if world:query_resource("MyResource") then
    print("Resource exists!")
end
```

## Advanced: Resource Management

The library provides a generic resource management system that allows Lua to insert and query Bevy resources. This is useful for game-level configuration and state that isn't tied to specific entities.

### Registering Resource Builders

In your Rust code, register builders that construct resources from Lua data:

```rust
use bevy_lua_ecs::*;

fn setup(world: &mut World) {
    let mut builder_registry = ResourceBuilderRegistry::default();
    
    // Register a simple resource builder
    builder_registry.register("MyGameConfig", |_lua, data: LuaValue, world: &mut World| {
        let table = data.as_table()
            .ok_or_else(|| LuaError::RuntimeError("Expected table".to_string()))?;
        
        let difficulty: String = table.get("difficulty")?;
        let volume: f32 = table.get("volume")?;
        
        world.insert_resource(MyGameConfig { difficulty, volume });
        Ok(())
    });
    
    world.insert_resource(builder_registry);
}
```

### Using Resources from Lua

```lua
-- Insert a resource (calls the registered builder)
insert_resource("MyGameConfig", {
    difficulty = "hard",
    volume = 0.8
})

-- Query if a resource exists
if world:query_resource("MyGameConfig") then
    print("Game config loaded!")
end
```

### Marker Components

For components that need to be serialized/replicated (e.g., for networking), register them as marker components:

```rust
let mut serde_registry = SerdeComponentRegistry::default();
serde_registry.register_marker::<Replicated>("Replicated");
world.insert_resource(serde_registry);
```

Then use them in Lua:

```lua
spawn({
    Transform = { ... },
    Replicated = {}  -- Marker component for replication
})
```

## Examples

### Networking Example

See `examples/networking.rs` and `assets/scripts/networking_example.lua` for a complete example of:
- Using resource builders for server/client setup
- Generic `insert_resource()` API for any resource type
- Marker components for entity replication
- Zero game-specific networking code in Rust

### Tilemap Rendering

See `examples/tilemap.rs` and `assets/scripts/tilemap.lua` for a complete example of:
- Loading tileset textures from Tiled
- Using Sprite's `rect` field for tile slicing (pure Lua)
- Spawning thousands of tiles with texture atlas
- Zero game-specific Rust code

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
- **AssetRegistry**: Manages loaded images and created assets
- **SpawnQueue**: Queues entity creation from Lua
- **ComponentUpdateQueue**: Queues component updates from Lua
- **ResourceQueue**: Queues resource insertion from Lua
- **ResourceBuilderRegistry**: Registers constructors for Bevy resources
- **SerdeComponentRegistry**: Registers marker components for serialization
- **LuaSystemRegistry**: Manages Lua systems to run each frame
- **Query API**: Provides ECS queries to Lua
- **Asset Loading**: Generic `load_asset()` and `create_asset()` functions
- **Resource Management**: Generic `insert_resource()` and `query_resource()` functions

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
6. **Macro-Based Asset Registration**: Use `register_handle_setters!` macro to declare which asset types your game needs

## Advanced: Custom Asset Type Registration

For full Zero Rust compliance, you can customize which asset types are registered using the `register_handle_setters!` macro:

```rust
use bevy::prelude::*;
use bevy_lua_ecs::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    
    // Create empty asset registry
    let mut asset_registry = AssetRegistry::new();
    let type_registry = app.world().resource::<AppTypeRegistry>();
    
    // Register only the asset types your game needs
    let mut handle_setters = std::collections::HashMap::new();
    register_handle_setters!(
        handle_setters,
        type_registry,
        Image,              // For sprites and UI
        Mesh,               // For 3D models
        StandardMaterial,   // For PBR materials
        MyCustomAsset,      // Your own asset types!
    );
    
    // The macro generates type-specific code at compile time
    // This avoids hardcoding asset types in the library
    
    // Continue with app setup...
}
```

Alternatively, use the convenience method `AssetRegistry::from_type_registry()` which pre-registers common Bevy asset types:
- `Image`, `Mesh`, `StandardMaterial`, `Scene`, `AnimationClip`, `AudioSource`, `Font`

## Running Examples

```bash
# Networking with server/client resources (requires networking feature)
cargo run --example networking --features networking

# Tilemap with texture atlas slicing
cargo run --example tilemap

# Animated sprites with color pulsing
cargo run --example sprites

# Interactive button with click handling
cargo run --example button

# Basic Lua integration
cargo run --example basic
```
