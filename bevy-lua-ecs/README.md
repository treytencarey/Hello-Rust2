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
bevy_lua_ecs = { path = "../bevy-lua-ecs" }
mlua = "0.10"
```

### 2. Setup Your App

```rust
use bevy::prelude::*;
use bevy_lua_ecs::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    
    // Add Lua plugin - automatically registers common Bevy events
    // (CursorMoved, FileDragAndDrop, KeyboardInput, MouseButtonInput, etc.)
    app.add_plugins(LuaSpawnPlugin);
    
    app.add_systems(Startup, load_script)
        .run();
}

fn load_script(lua_ctx: Res<LuaScriptContext>) {
    let script = std::fs::read_to_string("assets/scripts/main.lua").unwrap();
    lua_ctx.execute_script(&script, "main.lua").unwrap();
}
```

### 3. Write Game Logic in Lua

```lua
-- Spawn an entity
spawn({
    Sprite = {
        color = {r = 1.0, g = 0.0, b = 0.0, a = 1.0}
    },
    Transform = {
        translation = {x = 0, y = 0, z = 0}
    }
})

-- Register a system
function my_system(world)
```

## Features

### Core Capabilities

- **Component Spawning**: Create entities with any reflected Bevy component
- **Asset Loading**: Load image files and create assets via reflection
- **Asset Creation**: Create any Bevy asset type (layouts, materials, etc.) from Lua
- **Resource Management**: Insert and query Bevy resources from Lua via builders
- **Networking Constructors**: Built-in generic builders for multiplayer (with `networking` feature)
- **OS Utilities**: Generic socket binding, time access, and address parsing
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

#### Reading Events

Read **any** Bevy event using generic reflection:

```lua
function handle_input(world)
    -- File drag and drop
    local file_events = world:read_events("bevy_window::event::FileDragAndDrop")
    for i, event in ipairs(file_events) do
        if event.DroppedFile then
            local path = event.DroppedFile.path_buf
            local image = load_asset(path)
            spawn({ Sprite = { image = image }, Transform = {...} })
        end
    end
    
    -- Mouse cursor movement
    local cursor_events = world:read_events("bevy_window::event::CursorMoved")
    for i, event in ipairs(cursor_events) do
        print("Cursor at:", event.position.x, event.position.y)
    end
    
    -- Keyboard input
    local key_events = world:read_events("bevy_input::keyboard::KeyboardInput")
    for i, event in ipairs(key_events) do
        print("Key:", event.key_code, "State:", event.state)
    end
    
    -- Mouse buttons
    local mouse_events = world:read_events("bevy_input::mouse::MouseButtonInput")
    for i, event in ipairs(mouse_events) do
        print("Button:", event.button, "State:", event.state)
    end
end

register_system("handle_input", handle_input)
```

**Event Registration**

Common Bevy events are **automatically registered** by `LuaSpawnPlugin`:
- Window: `CursorMoved`, `FileDragAndDrop`, `WindowResized`, `WindowFocused`, `WindowClosed`
- Keyboard: `KeyboardInput`
- Mouse: `MouseButtonInput`, `MouseWheel`, `MouseMotion`

For additional or custom events, use one of these methods:

```rust
// Option 1: Use the convenience function (registers all common events)
register_common_bevy_events(&mut app);

// Option 2: Register specific events with the macro
register_lua_events!(app,
    bevy::window::CursorMoved,
    MyCustomEvent,
);

// Option 3: Manually register (for fine control)
app.register_type::<MyCustomEvent>();
app.register_type::<Events<MyCustomEvent>>();
```

The generic event reader works with **any** event type that has `#[derive(Event, Reflect)]`.

#### Accessing Time

```lua
function my_system(world)
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

### Generic Resource Constructors (Library Infrastructure)

For common use cases like networking, the library provides **generic resource constructors** as reusable infrastructure. These constructors use OS-level utilities that work for ANY game.

#### Networking Resources (with `networking` feature)

The library includes built-in constructors for networking resources:

```rust
use bevy_lua_ecs::*;

fn setup(world: &mut World) {
    let builder_registry = ResourceBuilderRegistry::default();
    
    // Register generic networking constructors from library
    #[cfg(feature = "networking")]
    register_networking_constructors(&builder_registry);
    
    world.insert_resource(builder_registry);
}
```

This registers constructors for:
- `RenetServer` - Multiplayer server resource
- `RenetClient` - Multiplayer client resource  
- `NetcodeServerTransport` - Server network transport (with socket binding)
- `NetcodeClientTransport` - Client network transport (with socket binding)

**From Lua, just use them:**

```lua
-- Start a server (library handles socket binding, time access, etc.)
insert_resource("RenetServer", {})
insert_resource("NetcodeServerTransport", {
    port = 5001,
    max_clients = 10
})

-- Or start a client
insert_resource("RenetClient", {})
insert_resource("NetcodeClientTransport", {
    server_addr = "127.0.0.1",
    port = 5001
})

-- Check if server is ready
if world:query_resource("RenetServer") then
    print("Server is running!")
end
```

**Key benefit:** No game-specific Rust code needed! The library provides generic OS utilities (`OsUtilities::bind_udp_socket()`, `OsUtilities::current_time()`, etc.) that work for any game.

### Custom Resource Builders

For game-specific resources, register custom builders:

```rust
use bevy_lua_ecs::*;

fn setup(world: &mut World) {
    let builder_registry = ResourceBuilderRegistry::default();
    
    // Register a custom resource builder
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

### OS Utilities (Advanced)

The library exposes generic OS-level utilities through `OsUtilities`:

- `OsUtilities::bind_udp_socket(addr)` - Bind UDP sockets
- `OsUtilities::current_time()` - Get system time
- `OsUtilities::parse_socket_addr(addr)` - Parse network addresses

These are used internally by networking constructors but can be used for custom resource builders that need OS-level operations.

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

### Networking Example (Zero Rust!)

See `examples/networking.rs` and `assets/scripts/networking_example.lua` for a complete example of:
- **Generic networking constructors from library** - No game-specific resource code!
- Server/client setup purely in Lua using `insert_resource()`
- Socket binding and time access handled by library's `OsUtilities`
- Marker components for entity replication
- **80+ lines of game code eliminated** - moved to reusable library infrastructure

**The game code is just configuration:**
```rust
// That's it! No networking resource builders needed.
register_networking_constructors(&builder_registry);
```

**All networking logic in Lua:**
```lua
-- Start server
insert_resource("RenetServer", {})
insert_resource("NetcodeServerTransport", { port = 5001, max_clients = 10 })

-- Or start client  
insert_resource("RenetClient", {})
insert_resource("NetcodeClientTransport", { server_addr = "127.0.0.1", port = 5001 })
```

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
- **ResourceConstructorRegistry**: Reflection-based resource construction (for future use)
- **OsUtilities**: Generic OS-level operations (socket binding, time access, address parsing)
- **Networking Constructors**: Built-in generic builders for multiplayer resources
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
