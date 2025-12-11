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
- **Newtype Wrappers**: Seamlessly construct complex enum types with newtype wrappers (e.g., `RenderTarget::Image(ImageRenderTarget)`)
- **Resource Management**: Insert and query Bevy resources from Lua via builders
- **Auto-Generated Bindings**: Build script automatically generates Lua method bindings for resource types
- **Auto-Generated Event Registration**: Build script generates event registration from metadata
- **Component Queries**: Query entities with filtering and change detection
- **Component Mutation**: Update components via `entity:set()`
- **Lua Systems**: Register Lua functions to run every frame
- **Generic Lua Components**: Store arbitrary Lua data as components
- **Marker Components**: Register serializable marker components for special purposes
- **Time Access**: Get delta time and frame information
- **Script Importing**: Load Lua modules with `require()` and `require_async()`
- **Hot Reload**: Automatic cache invalidation and dependent script reloading

### Lua API

#### Script Importing

Load Lua modules to organize code into reusable components:

```lua
-- Synchronous loading (blocks until loaded)
local helpers = require("math_helpers.lua")
local result = helpers.add(5, 3)

-- Optional: Disable automatic reloading (default: true)
local static = require("math_helpers.lua", { reload = false })

-- Asynchronous loading (callback-based)
require_async("heavy_data.lua", function(data)
    print("Data loaded!")
end)

-- Optional: Re-trigger callback on hot reload (default: false)
require_async("config.lua", function(config)
    print("Config updated!")
end, { reload = true })
```

**Module Pattern**:
```lua
-- math_helpers.lua
local M = {}
function M.add(a, b) return a + b end
return M
```

**Hot Reload Behavior**:
- When a module file changes, its cache is invalidated
- Scripts that imported it with `reload = true` (default for sync) are reloaded
- Callbacks registered with `reload = true` (default false for async) are re-triggered
- Next `require()` loads the updated version

**Path Resolution**:
1. **Relative**: Tried first, relative to the current script's directory
   - `require("helpers.lua")` inside `scripts/utils/main.lua` → `scripts/utils/helpers.lua`
2. **Root-Relative**: Fallback, relative to `assets/scripts/`
   - `require("utils/math.lua")` → `assets/scripts/utils/math.lua`

**Caching**: Modules load once and are cached (singleton pattern)
```lua
local h1 = require("helpers.lua")
local h2 = require("helpers.lua")  -- Same instance as h1
```

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

-- Auto-discovered constructors for opaque types (like Image)
-- build.rs scans bevy crates for constructors and generates bindings
local rtt_image = create_asset("bevy_image::image::Image", {
    width = 512,
    height = 512,
    format = "Bgra8UnormSrgb"  -- Calls Image::new_target_texture()
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

Read **any** Bevy event using generic reflection. Both `read_events()` and `query_events()` are supported (they are aliases):

```lua
function handle_input(world)
    -- File drag and drop (using read_events)
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
    
    -- Keyboard input (using query_events alias)
    local key_events = world:query_events("bevy_input::keyboard::KeyboardInput")
    for i, event in ipairs(key_events) do
        print("Key:", event.key_code, "State:", event.state)
    end
    
    -- Mouse buttons
    local mouse_events = world:query_events("bevy_input::mouse::MouseButtonInput")
    for i, event in ipairs(mouse_events) do
        print("Button:", event.button, "State:", event.state)
    end
end

register_system("handle_input", handle_input)
```

**Event Registration**

Common Bevy events are **automatically registered** when you call `register_common_bevy_events()` before adding plugins:
- Window: `CursorMoved`, `FileDragAndDrop`, `WindowResized`, `WindowFocused`, `WindowClosed`
- Keyboard: `KeyboardInput`
- Mouse: `MouseButtonInput`, `MouseWheel`, `MouseMotion`

**IMPORTANT for networking:** If using Bevy Replicon, call `register_common_bevy_events()` **after** `DefaultPlugins` but **BEFORE** `RepliconPlugins` to ensure consistent event registration order between client and server.

```rust
app.add_plugins(DefaultPlugins);

// Register events BEFORE Replicon for consistent protocol
register_common_bevy_events(&mut app);

// Now add Replicon plugins
app.add_plugins(RepliconPlugins);
app.add_plugins(LuaSpawnPlugin);
```

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

#### Component Method Bindings

Call methods on entity components directly from Lua:

```lua
-- Query for entities with MessageSender component
local senders = world:query({"MessageSender"}, nil)
for _, sender in ipairs(senders) do
    -- Call send() method on the component
    world:call_component_method(sender, "MessageSender", "send", message_json)
end

-- Query for entities with MessageReceiver component  
local receivers = world:query({"MessageReceiver"}, nil)
for _, receiver in ipairs(receivers) do
    -- Call receive() method and get results
    local messages = world:call_component_method(receiver, "MessageReceiver", "receive")
    for _, msg in ipairs(messages) do
        print("Received:", msg)
    end
end
```

**Game developers register component methods in Rust**:

```rust
use bevy_lua_ecs::LuaComponentRegistry;

fn register_components(registry: Res<LuaComponentRegistry>) {
    // Register methods for your component types
    registry.register_component::<MyComponent, _>("MyComponent", |methods| {
        methods.add("do_something", |component, _lua, arg: String| {
            component.internal_method(arg);
            Ok(true)  // Return value must implement IntoLua
        });
    });
}
```

**Key Features**:
- 🎯 **Generic Infrastructure**: Library has ZERO knowledge of specific components/plugins
- ✅ **Type-Safe**: Methods use Rust's type system for safety
- 🔄 **Auto-Mutability Detection**: Only mutable components can register methods (Bevy 0.16)
- 🎮 **Game-Specific Registration**: Keep plugin bindings in your game code, not the library

## Advanced: Resource Management

The library provides a generic resource management system that allows Lua to insert and query Bevy resources. This is useful for game-level configuration and state that isn't tied to specific entities.

### Auto-Generated Method Bindings

The build script can automatically generate Lua bindings for resource methods by reading type metadata from dependent crates' `Cargo.toml`:

**In your game's `Cargo.toml`:**
```toml
[package.metadata.lua_resources]
types = [
    "renet::remote_connection::RenetClient",
    "renet::server::RenetServer",
]
```

The build script will:
1. Find the source files for these types in the Cargo registry
2. Parse the impl blocks using `syn`
3. Generate method bindings automatically
4. Make them available via `world:call_resource_method()`

**From Lua:**
```lua
-- Call any method on a resource
local connected = world:call_resource_method("RenetClient", "is_connected")
local rtt = world:call_resource_method("RenetClient", "rtt")

-- Methods with parameters work too
world:call_resource_method("RenetClient", "send_message", channel_id, data)
```

**Benefits:**
- No manual binding code needed
- Automatically discovers all public methods
- Works with any resource type in any crate
- Type-safe through mlua's automatic conversion

### Auto-Generated Constructor Bindings

The build script can also automatically generate Lua bindings for constructors and static functions:

**In your game's `Cargo.toml`:**
```toml
[package.metadata.lua_resources]
types = [
    "renet::remote_connection::RenetClient",
    "renet::server::RenetServer",
]

constructors = [
    "renet::ConnectionConfig::default",
    "bevy_renet::RenetClient::new",
]
```

The build script will:
1. Find the source files for these types
2. Parse the associated functions using `syn`
3. Extract function signatures automatically
4. Generate global Lua constructor functions

**From Lua:**
```lua
-- Auto-generated constructor functions
local config = create_connectionconfig()  -- Calls ConnectionConfig::default()
local client = create_renetclient(config)  -- Calls RenetClient::new(config)

-- Use them to create resources
insert_resource("RenetClient", client)
```

**Benefits:**
- No manual constructor code needed
- Automatically discovers function signatures
- Works with any static function or constructor
- Type-safe parameter handling

### Auto-Generated Event Registration

Events are automatically registered from `[package.metadata.lua_events]` in `Cargo.toml`:

```toml
[package.metadata.lua_events]
types = [
    "bevy::window::CursorMoved",
    "bevy::window::FileDragAndDrop",
    "bevy::input::keyboard::KeyboardInput",
    # Add your custom events here
]
```

The build script generates the registration code at compile time, ensuring consistent event order for networking protocols.

### Auto-Discovered Asset Constructors

**For opaque types** (like `Image`) that cannot be populated via reflection, the build script automatically discovers constructor methods:

**How it works:**
1. Build script scans bevy crates for `impl TypeName` blocks
2. Finds methods matching patterns: `new_*`, `from_*`, `default`
3. Parses method signatures to extract parameter names and types
4. Generates code that extracts params from Lua tables and calls the constructor

**Supported parameter types:**
- Primitives: `u32`, `i32`, `f32`, `f64`, `usize`, `bool`, `String`
- Enums: `TextureFormat`, `TextureDimension` (with generated match statements)

**Example - Render-to-Texture Image:**
```lua
-- Creates Image via Image::new_target_texture(width, height, format)
local rtt_image = create_asset("bevy_image::image::Image", {
    width = 512,
    height = 512,
    format = "Bgra8UnormSrgb"
})
```

**Benefits:**
- Zero configuration - constructors discovered at build time
- Works with opaque types that don't support `ReflectDefault`
- Parameter extraction is type-safe based on parsed signatures
- Enum parameters parsed from string variants

### Generic Resource Constructors (Moved to Game Code)

**Note:** As of the latest architecture, networking-specific resource constructors have been moved to game code (e.g., `Hello/src/networking.rs`). This keeps bevy-lua-ecs as a pure ECS-Lua bridge without game-specific dependencies.

The library now focuses on:
- Generic resource builder infrastructure (`ResourceBuilderRegistry`)
- Generic reflection-based resource construction (`ResourceConstructorRegistry`)
- Build script-based auto-binding generation

For networking or other game-specific resources, implement constructors in your game crate.

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

-- Call methods on resources (with auto-generated bindings)
local value = world:call_resource_method("MyGameConfig", "get_difficulty")
world:call_resource_method("MyGameConfig", "set_volume", 0.9)
```

### Build Script Architecture

The build script (`build.rs`) provides automatic code generation:

1. **Standalone Mode**: When building bevy-lua-ecs itself, only generates event registrations from its own metadata
2. **Dependency Mode**: When used as a dependency, the dependent crate can specify resource types for binding generation

**Key functions:**
- `find_source_file()`: Locates source files in the Cargo registry
- `extract_methods_for_type()`: Parses impl blocks with `syn`
- `generate_registration_code()`: Creates binding code with `quote`
- `generate_event_registrations()`: Generates event registration from metadata

The generated code is included via `include!(concat!(env!("OUT_DIR"), "/auto_bindings.rs"))`.

### OS Utilities

The library exposes a minimal `OsUtilities` struct reserved for future generic utilities. Game-specific utilities (like networking socket binding) should be implemented in game code.

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

### Newtype Wrappers for Complex Enums (Auto-Discovered!)

Some Bevy components use complex enum types with newtype wrappers. For example, `Camera::target` uses `RenderTarget::Image(ImageRenderTarget)` where `ImageRenderTarget` wraps a `Handle<Image>`.

**Zero configuration required!** The system automatically discovers and constructs newtype wrappers using Bevy's reflection.

**From Lua:**
```lua
-- Create render target image
local rtt_image = create_asset("bevy_image::image::Image", {
    size = {x = 512, y = 512}
})

-- Just pass the asset ID - the system handles everything!
spawn({
    Camera = {
        target = { Image = rtt_image }  -- Auto-constructs ImageRenderTarget
    },
    Transform = { translation = {x = 0, y = 0, z = 10} }
})
```

**How it works:**
1. Lua passes `{ Image = rtt_image }` (enum-style table)
2. The system detects `Image` variant contains `ImageRenderTarget`
3. Automatically converts asset ID → `Handle<Image>`
4. Constructs `ImageRenderTarget` via reflection (with default field values)
5. Inserts into `RenderTarget::Image` enum variant

**Key features:**
- ✅ **Runtime auto-discovery** - asset types found via `ReflectAsset` at startup
- ✅ **Zero config required** - common Bevy types just work out of the box
- ✅ **Handles TupleStruct and Struct newtypes** - adapts to Bevy version changes
- ✅ **Automatic field defaults** - uses `ReflectDefault` or reflection fallbacks

**Optional Cargo.toml override** (for custom asset types):
```toml
# Only needed for types NOT auto-discovered via ReflectAsset
[package.metadata.lua_assets.types]
types = ["my_crate::MyCustomAsset"]
```

This enables zero-friction usage of complex Bevy enums from Lua!

## Examples


### Asset Upload Example

See `Hello/examples/asset_upload.rs` for a complete example demonstrating the "Zero Rust" networking philosophy:
- **Networking setup entirely in Lua**: Server/client resources created via `insert_resource()`
- File drag-and-drop event handling in Lua
- Dynamic asset loading and sprite creation
- Client-server file transfer via custom networking channel
- Auto-generated method bindings from `[package.metadata.lua_resources]`

**Key Lua code:**
```lua
-- Network initialization in Lua
if IS_CLIENT_MODE then
    insert_resource("RenetClient", {})
    insert_resource("NetcodeClientTransport", {
        server_addr = "127.0.0.1",
        port = 5000
    })
else
    insert_resource("RenetServer", {})
    insert_resource("NetcodeServerTransport", {
        port = 5000,
        max_clients = 10
    })
end
```

**Rust provides only:**
- Resource constructor registration (`register_networking_constructors()`)
- Socket binding and low-level networking utilities (UDP sockets, connect tokens)
- Auto-generated method bindings via build script

**All networking logic in Lua:**
- Network initialization and configuration
- Message sending via `world:call_resource_method("RenetClient", "send_message", channel, data)`
- Message receiving via `world:call_resource_method("RenetServer", "receive_message", client_id, channel)`
- No message queue or custom Lua functions needed - just auto-generated bindings!

All game logic and networking setup is in `Hello/assets/scripts/asset_upload_example.lua`.

### Networking Example

See `Hello/examples/networking.rs` for another networking example with message passing.

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
- **LuaResourceRegistry**: Type-safe method registry for resource bindings
- **ResourceConstructorRegistry**: Reflection-based resource construction (for future use)
- **Build Script**: Auto-generates method bindings and event registrations at compile time
- **SerdeComponentRegistry**: Registers marker components for serialization
- **LuaSystemRegistry**: Manages Lua systems to run each frame
- **Query API**: Provides ECS queries to Lua
- **Asset Loading**: Generic `load_asset()` and `create_asset()` functions
- **Resource Management**: Generic `insert_resource()`, `query_resource()`, and `call_resource_method()` functions

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
6. **Build-Time Code Generation**: Auto-generate bindings and registrations at compile time
7. **Clean Architecture**: Library remains free of game-specific dependencies
8. **Macro-Based Asset Registration**: Use `register_handle_setters!` macro to declare which asset types your game needs

## Advanced: Bitflags Handling

The library provides generic bitflags support for asset construction. Use pipe-separated strings in Lua:

```lua
-- RTT Image with texture_usages
local image = create_asset("bevy_image::image::Image", {
    size = { width = 512, height = 512, depth_or_array_layers = 1 },
    dimension = "D2",
    format = "Bgra8UnormSrgb",
    asset_usage = "RENDER_WORLD|MAIN_WORLD",
    texture_usages = "TEXTURE_BINDING|COPY_DST|RENDER_ATTACHMENT"
})
```

### Setup

```rust
use bevy_lua_ecs::BitflagsRegistry;

fn main() {
    App::new()
        .init_resource::<BitflagsRegistry>()
        .add_systems(Startup, |registry: Res<BitflagsRegistry>| {
            crate::auto_resource_bindings::register_auto_bitflags(&registry);
        })
        // ...
}
```

### Adding Custom Bitflags

Update `get_known_type_definition()` in `bevy-lua-ecs/build.rs`:

```rust
"MyBitflags" => Some(TypeDefinition::Bitflags {
    name: "MyBitflags".to_string(),
    full_path: "my_crate::MyBitflags".to_string(),
    flags: vec!["FLAG_A".to_string(), "FLAG_B".to_string()],
})
```

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

**Note:** The networking examples have been moved to the `Hello` game crate. To run them:

```bash
# From the Hello directory
cd Hello

# Networking with server/client (run in two terminals)
cargo run --example networking --features networking
cargo run --example asset_upload --features networking
cargo run --example asset_upload --features networking client

# From bevy-lua-ecs directory
cd bevy-lua-ecs

# Tilemap with texture atlas slicing
cargo run --example tilemap

# Animated sprites with color pulsing
cargo run --example sprites

# Interactive button with click handling
cargo run --example button

# Basic Lua integration
cargo run --example basic
```

## Troubleshooting

### Debug Logging

Enable detailed logging to diagnose issues:
```powershell
$env:RUST_LOG="bevy_lua_ecs=debug"; cargo run --example your_example
```

### Common Issues

- **Component not found**: Check spelling matches Rust type name exactly
- **Asset not loading**: Verify path is relative to `assets/` folder
- **Enum variant not applying**: Look for `[ENUM_SET]` logs to verify application

### Enum and Newtype Debugging

For complex enum types like `RenderTarget::Image(ImageRenderTarget)`:

**Expected log sequence:**
```
[ENUM_NEWTYPE] Result: variant_is_newtype=true, inner_type=Some("Handle<Image>")
[NEWTYPE_WRAP_REFLECT] Inserted handle into field 'handle'
[NEWTYPE_WRAP_REFLECT] ✓ Auto-discovered newtype wrapper for 'ImageRenderTarget'
[ENUM_SET] Created concrete enum via from_reflect
```

**If RTT not rendering:**
1. Check asset creation succeeded: `[RTT_LUA] Created RTT image handle: 0`
2. Verify Camera target set: `[ENUM_SET] Created concrete enum via from_reflect`
3. Ensure UiTargetCamera points to correct entity

See `build-script-bindings.md` for detailed technical documentation.
