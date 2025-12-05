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
```

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