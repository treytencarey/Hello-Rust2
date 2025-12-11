# Bevy-Lua-ECS Implementation Guide

**Core Philosophy: "Zero Rust"** - All game logic in Lua. Rust provides only generic ECS infrastructure.

## Lua API Reference

### 1. Spawning Entities

```lua
-- Basic entity with Bevy components
spawn({
    Sprite = { color = {r = 1.0, g = 0.0, b = 0.0, a = 1.0} },
    Transform = { 
        translation = {x = 0, y = 0, z = 0},
        rotation = {x = 0.0, y = 0.0, z = 0.0, w = 1.0},
        scale = {x = 1.0, y = 1.0, z = 1.0}
    }
})

-- Custom Lua components (any Lua value)
spawn({
    Transform = { translation = {x = 100, y = 50, z = 0} },
    Timer = { duration = 3.0, elapsed = 0.0 },
    OnClick = function() print("Clicked!") end
})

-- UI text
spawn({
    Text = { text = "Hello!" },
    TextFont = { font_size = 64 },
    TextColor = { color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0} },
    Transform = { translation = {x = 0, y = 100, z = 0} }
})
```

### 2. Script Importing (require)

**Synchronous Loading**:
```lua
-- Load a module and use it immediately
local helpers = require("math_helpers.lua")
local result = helpers.add(5, 3)  -- 8
```

**Asynchronous Loading**:
```lua
-- Load with callback (designed for future server downloads)
require_async("heavy_data.lua", function(data)
    print("Data loaded!")
end)
```

**Module Pattern**:
```lua
-- my_module.lua
local M = {}
function M.my_function() return "Hello" end
return M
```

**Hot Reload**: When a module file changes:
- The module cache is automatically invalidated
- All scripts that imported it are also invalidated
- Next `require()` will reload the module with new code
- Dependent scripts are automatically reloaded

**Path Resolution**: All paths relative to `assets/scripts/`
- `require("helpers.lua")` → `assets/scripts/helpers.lua`
- `require("utils/math.lua")` → `assets/scripts/utils/math.lua`

### 3. Assets

```lua
-- Load image
local texture_id = load_asset("path/to/image.png")

-- Create asset via reflection
local atlas_id = create_asset("bevy_sprite::texture_atlas::TextureAtlasLayout", {
    tile_size = {x = 16, y = 16},
    columns = 8,
    rows = 8
})

-- Auto-discovered constructor for opaque types (like Image)
-- build.rs scans for Image::new_target_texture() and generates bindings
local rtt_image = create_asset("bevy_image::image::Image", {
    width = 512,
    height = 512,
    format = "Bgra8UnormSrgb"  -- TextureFormat enum parsed from string
})

-- Sprite with texture slicing (pure Lua)
spawn({
    Sprite = {
        image = texture_id,
        rect = { min = {x = 0, y = 0}, max = {x = 16, y = 16} }
    },
    Transform = { translation = {x = 0, y = 0, z = 0} }
})
```

### 4. Systems

```lua
function my_system(world)
    local dt = world:delta_time()
    local entities = world:query({"Transform", "Sprite"}, nil)
    
    for i, entity in ipairs(entities) do
        -- Process entities
    end
end

register_system("Update", my_system)
```

### 5. Querying

```lua
-- Basic query
local entities = world:query({"Transform", "Sprite"}, nil)

-- With change detection
local changed = world:query({"Transform"}, {"Transform"})

-- Multiple components
local buttons = world:query({"Button", "Interaction", "OnClick"}, nil)
```

### 6. Entity Operations

```lua
-- Get component
local transform = entity:get("Transform")
local sprite = entity:get("Sprite")

-- Check component
if entity:has("Timer") then
    local timer = entity:get("Timer")
end

-- Update component (queued)
entity:set("Sprite", { color = {r = 1.0, g = 0.5, b = 0.0, a = 1.0} })
entity:set("Transform", { translation = {x = 100, y = 50, z = 0} })

-- Get entity ID
local id = entity:id()
```

### 7. Resources

```lua
-- Insert resource (must be registered in Rust)
insert_resource("MyGameConfig", { difficulty = "hard", volume = 0.8 })

-- Networking (with networking feature)
insert_resource("RenetServer", {})
insert_resource("NetcodeServerTransport", { port = 5000, max_clients = 10 })

-- Query resource
if world:query_resource("RenetServer") then
    print("Server running!")
end
```

## Common Patterns

### Using Helper Modules

```lua
-- math_helpers.lua
local M = {}
function M.add(a, b) return a + b end
function M.clamp(v, min, max)
    if v < min then return min end
    if v > max then return max end
    return v
end
return M
```

```lua
-- main.lua
local math_helpers = require("math_helpers.lua")
local result = math_helpers.add(10, 20)
local clamped = math_helpers.clamp(15, 0, 10)
```

### Animation
```lua
local elapsed = 0
function animation_system(world)
    elapsed = elapsed + world:delta_time()
    local entities = world:query({"Sprite"}, nil)
    
    for i, entity in ipairs(entities) do
        local pulse = (math.sin(elapsed * 2.0) + 1.0) / 2.0
        entity:set("Sprite", { color = {r = pulse, g = 0.5, b = 1.0, a = 1.0} })
    end
end
register_system("Update", animation_system)
```

### Button Handling
```lua
spawn({
    Button = {},
    BackgroundColor = { color = {r = 0.2, g = 0.6, b = 0.8, a = 1.0} },
    Text = { text = "Click Me!" },
    OnClick = function() print("Clicked!") end
})

function button_system(world)
    local buttons = world:query({"Button", "Interaction", "OnClick"}, {"Interaction"})
    for i, entity in ipairs(buttons) do
        if entity:get("Interaction") == "Pressed" then
            entity:get("OnClick")()
        end
    end
end
register_system("Update", button_system)
```

### Tilemap
```lua
local tileset = load_asset("tileset.png")
local tile_w, tile_h, columns = 16, 16, 48

for y = 0, height - 1 do
    for x = 0, width - 1 do
        local tile_id = get_tile(x, y)
        local tile_x = (tile_id % columns) * tile_w
        local tile_y = math.floor(tile_id / columns) * tile_h
        
        spawn({
            Sprite = {
                image = tileset,
                rect = { min = {x = tile_x, y = tile_y}, max = {x = tile_x + tile_w, y = tile_y + tile_h} }
            },
            Transform = { translation = {x = x * tile_w, y = y * tile_h, z = 0} }
        })
    end
end
```

### Physics (Rapier)
```lua
function ColliderCuboid(hx, hy)
    return {
        raw = { Cuboid = { half_extents = {hx, hy} } },
        unscaled = { Cuboid = { half_extents = {hx, hy} } },
        scale = {1.0, 1.0}
    }
end

spawn({
    Sprite = { color = {r = 0.3, g = 0.7, b = 0.3, a = 1.0}, custom_size = {x = 600, y = 20} },
    Transform = { translation = {x = 0, y = -200, z = 0} },
    RigidBody = "Fixed",
    Collider = ColliderCuboid(300.0, 10.0)
})

spawn({
    Sprite = { color = {r = 1.0, g = 0.3, b = 0.3, a = 1.0}, custom_size = {x = 40, y = 40} },
    Transform = { translation = {x = 0, y = 200, z = 0} },
    RigidBody = "Dynamic",
    Collider = ColliderCuboid(20.0, 20.0),
    Restitution = { coefficient = 0.7 },
    GravityScale = 1.0
})
```

### Networking
```lua
-- Server
insert_resource("RenetServer", {})
insert_resource("NetcodeServerTransport", { port = 5000, max_clients = 10 })

register_system("server_init", function(world)
    if world:query_resource("RenetServer") then
        spawn({
            Transform = { translation = {x = 0, y = 0, z = 0} },
            Sprite = { color = {r = 0.2, g = 0.8, b = 0.3, a = 1.0} },
            Replicated = {}  -- Marker for replication
        })
    end
end)

-- Client
insert_resource("RenetClient", {})
insert_resource("NetcodeClientTransport", { server_addr = "127.0.0.1", port = 5000 })
```

## Component Structures

```lua
Transform = {
    translation = {x = 0, y = 0, z = 0},
    rotation = {x = 0, y = 0, z = 0, w = 1},  -- Quaternion
    scale = {x = 1, y = 1, z = 1}
}

Sprite = {
    color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0},
    custom_size = {x = 50, y = 50},
    image = asset_id,
    rect = { min = {x = 0, y = 0}, max = {x = 16, y = 16} },
    texture_atlas = atlas_id,
    index = 0
}

Text = { text = "Hello!" }
TextFont = { font_size = 32 }
TextColor = { color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0} }

Button = {}
BackgroundColor = { color = {r = 0.5, g = 0.5, b = 0.5, a = 1.0} }
Interaction = "None" | "Hovered" | "Pressed"  -- Read-only

RigidBody = "Fixed" | "Dynamic" | "Kinematic"
Restitution = { coefficient = 0.7 }
GravityScale = 1.0

-- Camera with RenderTarget enum (uses newtype wrapper)
Camera = {
    target = { Image = asset_id }  -- RenderTarget::Image(ImageRenderTarget(Handle<Image>))
}
```

## Newtype Wrappers for Complex Enums

### The Problem

Some Bevy components use complex enum types with newtype wrappers. For example:

```rust
pub enum RenderTarget {
    Image(ImageRenderTarget),
    Window(WindowRef),
    TextureView(ManualTextureViewHandle),
}

// In Bevy 0.17+, ImageRenderTarget is a Struct (not TupleStruct):
pub struct ImageRenderTarget {
    pub handle: Handle<Image>,
    pub scale_factor: FloatOrd,
}
```

From Lua, you want to write:
```lua
Camera = { target = { Image = rtt_image_id } }
```

But this requires the system to:
1. Detect that `Image` variant contains a newtype `ImageRenderTarget`
2. Convert `rtt_image_id` (integer) → `UntypedHandle`
3. Convert `UntypedHandle` → `Handle<Image>` (typed)
4. Construct `ImageRenderTarget` with the handle and default values for other fields
5. Insert `ImageRenderTarget` into the `RenderTarget::Image` enum variant

### The Solution: Reflection-Based Auto-Discovery

**No manual registration required!** The system automatically discovers newtype wrappers using Bevy's reflection.

**Architecture:**

1. **Handle Creators** (auto-generated by `build.rs`): Registered for ALL asset types in `[package.metadata.lua_assets.types]`. Creates typed `Handle<T>` from `UntypedHandle`.

2. **Newtype Auto-Discovery** (`try_wrap_in_newtype_with_reflection`): Uses reflection to construct newtype wrappers dynamically without manual registration.

**Detection Logic:**

When encountering an enum variant with a single-field tuple:
- Extract the field's `type_path` (e.g., `"bevy_camera::camera::ImageRenderTarget"`)
- Look up the type in `TypeRegistry`
- Check if it's a `TupleStruct` or `Struct` containing a `Handle<T>` field

**Generic Construction Flow (Struct Types):**

```rust
// 1. Get TypeInfo for the newtype
let type_info = registration.type_info();

// 2. For Struct types (like ImageRenderTarget in Bevy 0.17+):
match type_info {
    TypeInfo::Struct(struct_info) => {
        let mut dynamic_struct = DynamicStruct::default();
        
        for field in struct_info.iter() {
            if field.type_path().contains("Handle<") {
                // Insert our typed handle
                dynamic_struct.insert_boxed(field_name, typed_handle);
            } else {
                // Get default value using multiple fallback strategies:
                // 1. ReflectDefault on field type
                // 2. ReflectFromReflect with empty tuple
                // 3. ReflectFromReflect with common primitives (1.0f32, 0.0f32, etc.)
            }
        }
    }
}

// 3. Use ReflectFromReflect to create concrete type
let concrete = reflect_from_reflect.from_reflect(&dynamic_struct);
```

**Generic Default Value Strategies:**

For struct fields without `ReflectDefault` (like `FloatOrd`), the system tries:
1. `ReflectFromReflect` with empty `DynamicTupleStruct` (works if type has Default impl)
2. `ReflectFromReflect` with `1.0f32` (works for FloatOrd-like types)
3. `ReflectFromReflect` with `0.0f32`, `1i32`, `0i32` as additional fallbacks
4. Skip field and let final `from_reflect` use target's Default

**Why This Design is Generic:**

1. **No hardcoded type names**: Works with any newtype containing `Handle<T>`
2. **Handles both TupleStruct and Struct**: Bevy's `ImageRenderTarget` changed from tuple to struct in 0.17
3. **Automatic field default discovery**: Uses reflection to find defaults, not hardcoded values
4. **Runtime auto-discovery**: Asset types discovered from TypeRegistry via `ReflectAsset`
5. **Optional config**: `Cargo.toml` config is optional - discovery supplements/overrides it

**Key Implementation Files:**

- `asset_loading.rs`: 
  - `discover_and_register_handle_creators()` - runtime asset type discovery
  - `discover_entity_components()` - finds Entity wrapper newtypes
  - `try_wrap_in_newtype_with_reflection()` - generic newtype construction
- `build.rs`: Generates handle creators from `[package.metadata.lua_assets.types]` (optional)
- `components.rs`: Enum variant detection calling the newtype wrapper

**Usage:**

The system auto-discovers asset types at runtime - no config needed for common Bevy asset types!

```lua
-- Just works - Image handle creators are auto-discovered via ReflectAsset
Camera = { target = { Image = rtt_image } }
```

**Optional Override** (for custom asset types or explicit control):
```toml
# Cargo.toml - optional, for types not auto-discovered
[package.metadata.lua_assets.types]
types = ["my_crate::MyCustomAsset"]
```

## Auto-Discovered Asset Constructors

For **opaque types** (like `Image`) that can't be populated via reflection, the build script auto-discovers constructor methods.

**How it works:**
1. `build.rs` scans bevy crates for `impl TypeName` blocks
2. Finds methods matching: `new_*`, `from_*`, `default`
3. Parses parameter names and types using `syn`
4. Generates code that extracts params from Lua and calls the constructor

**Supported params:** `u32`, `i32`, `f32`, `f64`, `bool`, `String`, `TextureFormat`, `TextureDimension`

**Example - Render-to-Texture:**
```lua
-- Calls Image::new_target_texture(width, height, format)
local rtt_image = create_asset("bevy_image::image::Image", {
    width = 512,
    height = 512,
    format = "Bgra8UnormSrgb"
})

spawn({
    Camera = { target = { Image = rtt_image } },  -- Uses auto-wrapped ImageRenderTarget
    Camera2d = {}
})
```

**Adding enum support:** Update build.rs match statement for new enum types.

## Hot Reload Behavior


### Main Scripts
When a main script file changes:
- The script instance is cleaned up (entities despawned, resources removed)
- The script is re-executed with fresh state
- All spawned entities are recreated

### Module Files
When a module file changes:
- The module cache is invalidated
- All dependent modules are also invalidated
- Main scripts that imported the module are automatically reloaded
- Next `require()` will load the updated module

**Example**:
1. `main.lua` calls `require("helpers.lua")`
2. You edit `helpers.lua`
3. System detects change and invalidates cache for `helpers.lua`
4. System also invalidates `main.lua` (because it depends on `helpers.lua`)
5. `main.lua` is automatically reloaded
6. When `main.lua` runs `require("helpers.lua")` again, it gets the new version

## Best Practices

1. **Custom Lua components for game state**: `Health = { current = 100, max = 100 }`
2. **Organize systems by responsibility**: Separate movement, animation, collision, UI systems
3. **Use change detection**: `world:query({"Button"}, {"Interaction"})` for performance
4. **Helper functions**: Create reusable spawn functions in modules
5. **Debug with print**: Log entity counts and component values
6. **Module organization**: Keep related functions in the same module
7. **Avoid circular dependencies**: Don't have modules that import each other

## Troubleshooting

- **Component not found**: Check spelling, ensure `#[reflect(Component)]` in Rust
- **Asset not loading**: Verify path relative to assets folder
- **System not running**: Check `register_system()` call and Lua errors
- **Updates not applying**: Updates are queued, applied next frame
- **Query returns nothing**: Verify all required components exist on entities
- **Module not found**: Check path is relative to `assets/scripts/` and includes file extension
- **Module returns nil**: Ensure module has a `return` statement
- **Hot reload not working**: Check file is in `assets/scripts/` directory

### Enum and Newtype Issues

- **Enum variant not setting**: Enable debug logging to trace:
  ```powershell
  $env:RUST_LOG="bevy_lua_ecs=debug"; cargo run --example your_example
  ```
- **RenderTarget::Image not working**: Verify the asset was created successfully before using in Camera
- **Handle not preserved in enum**: The system uses `from_reflect` to create concrete enums - check for `[ENUM_SET] Created concrete enum via from_reflect` in logs
- **Newtype wrapper not detected**: Verify the newtype has `#[derive(Reflect)]` with `FromReflect` auto-derived

### Debug Log Markers

When debugging enum/newtype issues, look for these markers:
- `[ENUM_NEWTYPE]` - Enum variant detection and newtype analysis
- `[NEWTYPE_WRAP_REFLECT]` - Newtype construction via reflection
- `[ENUM_SET]` - Enum variant application to component fields
- `[HANDLE_CREATE]` - Typed handle creation from asset IDs

Example debug output for successful RTT setup:
```
[ENUM_NEWTYPE] Result: variant_is_newtype=true, inner_type=Some("Handle<Image>")
[NEWTYPE_WRAP_REFLECT] Inserted handle into field 'handle'
[NEWTYPE_WRAP_REFLECT] ✓ Auto-discovered newtype wrapper for 'ImageRenderTarget'
[ENUM_SET] Created concrete enum via from_reflect
```