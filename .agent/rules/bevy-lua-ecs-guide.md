---
trigger: always_on
---

# Bevy-Lua-ECS Quick Reference

**Core Philosophy: "Zero Rust"** - All game logic in Lua. Rust provides generic ECS infrastructure.

## API Reference

### Spawning Entities
```lua
spawn({
    Sprite = { color = {r = 1.0, g = 0.0, b = 0.0, a = 1.0} },
    Transform = { translation = {x = 0, y = 0, z = 0} },
    CustomData = { value = 42 },  -- Custom Lua components
    OnClick = function() end      -- Functions as components
})
```

### Module System
```lua
-- Sync: local module = require("path.lua")
-- Async: require_async("path.lua", callback)
-- Async with hot reload: require_async("path.lua", callback, { reload = true })
-- Paths relative to assets/scripts/
-- Hot reload: Module changes auto-reload dependents
```

### Assets
```lua
local texture_id = load_asset("image.png")
local atlas = create_asset("bevy_sprite::texture_atlas::TextureAtlasLayout", {...})
-- Auto-discovered constructors (build.rs scans for new_*/from_* methods):
local rtt = create_asset("bevy_image::image::Image", {width=512, height=512, format="Bgra8UnormSrgb"})
```

### Systems & Queries
```lua
register_system("Update", function(world)
    local dt = world:delta_time()
    local entities = world:query({"Transform", "Sprite"}, nil)  -- (with, changed)
    for i, entity in ipairs(entities) do
        entity:set({ Transform = {...} })  -- Queued update
    end
end)
```

### Entity Operations
```lua
entity:get("Component")  -- Read
entity:has("Component")  -- Check
entity:set({ Component = data })  -- Write (queued)
entity:id()  -- Get ID
```

### Resources
```lua
insert_resource("TypeName", {...})  -- Must be registered in Rust
world:query_resource("TypeName")    -- Check existence
```

## Component Reference

| Component | Structure |
|-----------|----------|
| Transform | `{translation={x,y,z}, rotation={x,y,z,w}, scale={x,y,z}}` |
| Sprite | `{color={r,g,b,a}, custom_size={x,y}, image=id, rect={min,max}, texture_atlas=id, index=n}` |
| Text | `{text="..."}` + `TextFont={font_size=32}` + `TextColor={color=...}` |
| Button | `{}` + `BackgroundColor={color=...}` + `Interaction` (readonly: "None"/"Hovered"/"Pressed") |
| Physics | `RigidBody="Fixed"/"Dynamic"`, `Collider=...`, `Restitution={coefficient=0.7}`, `GravityScale=1.0` |
| Camera | `{target={Image=asset_id}}` (RenderTarget enum, auto-wraps in ImageRenderTarget newtype) |

## Newtype Wrappers (Complex Enums)

**Problem:** Enum variants like `RenderTarget::Image(ImageRenderTarget)` where `ImageRenderTarget` wraps `Handle<Image>` + other fields.

**Solution:** Automatic reflection-based construction (no manual registration).

**Process:**
1. Lua passes `{Image = asset_id}` for enum field
2. System detects variant contains newtype wrapper (not raw Handle<T>)
3. Converts asset_id → UntypedHandle → Handle<Image>
4. Uses TypeRegistry to construct newtype:
   - TupleStruct: `DynamicTupleStruct` with handle at index 0
   - Struct: `DynamicStruct` inserting handle + defaults for other fields
5. Uses `from_reflect()` to create concrete enum

**Default strategies for non-handle fields:**
- ReflectDefault trait
- ReflectFromReflect with empty DynamicTupleStruct
- Fallback primitives: 1.0f32, 0.0f32, 1i32, 0i32

**Example:**
```lua
local rtt = create_asset("bevy_image::image::Image", {width=512, height=512, format="Bgra8UnormSrgb"})
spawn({Camera = {target = {Image = rtt}}, Camera2d = {}})  -- Auto-wraps ImageRenderTarget
```

**Files:** `asset_loading.rs` (try_wrap_in_newtype_with_reflection), `components.rs` (variant detection), `build.rs` (handle creators)

## Auto-Discovered Constructors

**For opaque types** (e.g., Image) that can't use reflection:
- `build.rs` scans impl blocks for `new_*`, `from_*`, `default` methods
- Generates bindings extracting Lua params and calling constructor
- Supported params: u32, i32, f32, f64, bool, String, TextureFormat, TextureDimension
- Add enum support by updating build.rs match statement

## Hot Reload

- **Main scripts:** Cleanup (despawn entities, remove resources) → re-execute with fresh state
- **Modules:** Cache invalidation cascades to dependents → auto-reload dependent scripts

## Best Practices

- Custom Lua components for game state
- Use change detection: `world:query({...}, {"ChangedComponent"})`
- Helper modules for reusable logic
- Avoid circular module dependencies

## Troubleshooting

**Common Issues:**
- Component not found → Check `#[reflect(Component)]` in Rust
- Asset not loading → Verify path relative to `assets/`
- Query returns nothing → Verify components exist
- Module not found → Path relative to `assets/scripts/`, include extension
- Enum/newtype not working → Enable debug logging

**Debug Logging:**
```powershell
$env:RUST_LOG="bevy_lua_ecs=debug"; cargo run --example name
```

**Log Markers:**
- `[ENUM_NEWTYPE]` - Variant/newtype detection
- `[NEWTYPE_WRAP_REFLECT]` - Newtype construction
- `[ENUM_SET]` - Enum application via from_reflect
- `[HANDLE_CREATE]` - Typed handle creation

**Successful RTT logs:**
```
[ENUM_NEWTYPE] Result: variant_is_newtype=true
[NEWTYPE_WRAP_REFLECT] ✓ Auto-discovered newtype wrapper
[ENUM_SET] Created concrete enum via from_reflect
```