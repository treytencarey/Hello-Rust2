# Bevy-Lua-ECS Quick Reference

**Core Philosophy: "Zero Rust"** - All game logic in Lua. Rust provides generic ECS infrastructure.

## World API Reference

### Entity Operations
```lua
spawn({ Component = {...} })  -- Create entity, returns builder
entity:get("Component")       -- Read component data
entity:has("Component")       -- Check component exists
entity:set({ Component = data }) -- Write (queued)
entity:id()                   -- Get entity ID
entity:with_parent(id)        -- Set parent
entity:observe("Event", fn)   -- Attach observer
```

### Query & Systems
```lua
register_system("Update", function(world)
    local dt = world:delta_time()
    local entities = world:query({"Transform"}, {"Changed"})  -- (with, changed)
    for i, entity in ipairs(entities) do
        entity:set({Transform = {...}})
    end
end)
```

### Events (Read/Write)
```lua
-- Read events (auto-discovered from bevy_window, bevy_input, etc.)
local events = world:read_events("MouseButtonInput")
local events = world:read_events("CursorMoved")
local events = world:query_events("KeyboardInput")  -- Alias

-- Write events
world:send_event("EventType", { field = value })
world:write_event("EventType", data)  -- Alias
```

### Messages (for Picking System)
```lua
-- Messages use MessageWriter<T> instead of EventWriter<T>
world:write_message("PointerInput", {
    pointer_id = { Custom = 12345 },
    location = { target = { Image = handle }, position = { x = 0, y = 0 } },
    action = { Move = { delta = { x = 1, y = 0 } } }
})
world:send_message("Type", data)  -- Alias

-- Unit enum variants as strings
action = { Press = "Primary" }
action = { Release = "Primary" }
```

### SystemParam Methods
```lua
-- Call methods on auto-discovered SystemParam types
local result = world:call_systemparam_method("MeshRayCast", "cast_ray", {
    origin = { x = 0, y = 0, z = 5 },
    direction = { x = 0, y = 0, z = -1 }
})
```

### Component Methods (Auto-Discovered)
```lua
-- Call methods on entity components (Transform/GlobalTransform currently supported)
world:call_component_method(entity_id, "Transform", "looking_at", {x=0,y=0,z=0}, {x=0,y=1,z=0})
world:call_component_method(entity_id, "Transform", "rotate", {x=0,y=0.5,z=0,w=0.87})
world:call_component_method(entity_id, "Transform", "with_translation", {x=1,y=2,z=3})

-- Check if entity has component
if world:has_component(entity_id, "Transform") then ... end

-- Get raw entity (for SystemParam methods like MeshRayCast)
local entity = world:get_entity(entity_id)
```

**Note:** Builder methods returning `Self` (looking_at, with_*) auto-write result back to component.

### Resources
```lua
insert_resource("TypeName", data)     -- Insert (needs Rust builder)
world:query_resource("TypeName")      -- Check exists
world:call_resource_method("Type", "method", args)  -- Call method
```

### Observers (Picking Callbacks)
```lua
spawn({ Button = {} })
    :observe("Pointer<Over>", function(entity, event) end)
    :observe("Pointer<Out>", function(entity, event) end)
    :observe("Pointer<Click>", function(entity, event) end)
    :observe("Pointer<Drag>", function(entity, event)
        -- event.x, event.y = pointer position
    end)

-- Manual invocation for RTT picking
world:invoke_observer(entity_id, "Pointer<Click>", { x = 100, y = 200 })
```

### Assets
```lua
local id = load_asset("image.png")
local id = create_asset("bevy_image::image::Image", { width=512, height=512, format="Bgra8UnormSrgb" })
```

### Script Management
```lua
local module = require("path.lua")        -- Sync load
require_async("path.lua", callback)       -- Async load
world:reload_current_script()             -- Hot reload
world:stop_current_script()               -- Stop and cleanup
world:despawn_all("TagComponent")         -- Despawn by tag
```

## Component Reference

| Component | Structure |
|-----------|----------|
| Transform | `{translation={x,y,z}, rotation={x,y,z,w}, scale={x,y,z}}` |
| Sprite | `{color={r,g,b,a}, image=id, rect={min,max}}` |
| Node | `{width={Px=n}, height={Percent=50}, position_type="Absolute"}` |
| Camera | `{target={Image=id}}` (auto-wraps in ImageRenderTarget) |

## Auto-Discovery (Build Script)

**These are discovered at compile time and require no manual setup:**

- **Component Methods**: Methods on `#[derive(Component)]` types (Transform/GlobalTransform)
- **SystemParams**: Types with `#[derive(SystemParam)]` and their methods
- **Events**: Types with `#[derive(Event)]` from bevy_window, bevy_input
- **Messages**: Types using `MessageWriter<T>` (e.g., PointerInput)
- **Assets**: Types implementing `Asset` trait
- **Entity Wrappers**: Newtypes around `Entity` with `#[derive(Component)]`
- **Handle Newtypes**: Types wrapping `Handle<T>` (e.g., ImageRenderTarget)
- **Bitflags**: Common bitflags types with their flag names

## Newtype Wrappers

**Automatic reflection-based construction for complex enums:**
```lua
spawn({ Camera = { target = { Image = rtt_image_id } } })
-- Automatically constructs RenderTarget::Image(ImageRenderTarget(handle))
```

## Debugging

```powershell
$env:RUST_LOG="bevy_lua_ecs=debug"; cargo run --example name
```

**Log prefixes:**
- `[ENUM_NEWTYPE]` - variant detection
- `[MESSAGE_WRITE]` - message dispatch
- `[SEND_EVENT]` - event dispatch
- `[READ_EVENTS]` - event reading
- `[LUA_OBSERVER]` - observer callbacks
