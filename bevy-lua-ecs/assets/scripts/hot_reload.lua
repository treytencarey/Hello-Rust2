-- Hot Reload Example Script
-- Spawns moving rectangle sprites, then hot reloads after 5 seconds
-- Everything spawned by this script is destroyed and recreated on reload

print("=== Hot Reload Script Starting ===")

-- Global state for reload timer and count
if not _G.reload_timer then
    _G.reload_timer = 0
    _G.reload_count = 0
end

_G.reload_count = _G.reload_count + 1
print("Reload #" .. _G.reload_count)

-- Insert a test resource to verify it gets cleared on reload
insert_resource("TestResource", {
    reload_number = _G.reload_count,
    message = "This is reload #" .. _G.reload_count
})
print("Inserted TestResource for reload #" .. _G.reload_count)

-- Define colors for each reload cycle
local color_sets = {
    -- Cycle 1: Warm colors
    {
        {r = 1.0, g = 0.3, b = 0.3, a = 1.0},  -- Red
        {r = 1.0, g = 0.6, b = 0.0, a = 1.0},  -- Orange
        {r = 1.0, g = 1.0, b = 0.3, a = 1.0},  -- Yellow
        {r = 1.0, g = 0.3, b = 0.6, a = 1.0},  -- Pink
        {r = 1.0, g = 0.5, b = 0.3, a = 1.0},  -- Coral
        {r = 1.0, g = 0.0, b = 0.5, a = 1.0},  -- Magenta
    },
    -- Cycle 2: Cool colors
    {
        {r = 0.3, g = 0.3, b = 1.0, a = 1.0},  -- Blue
        {r = 0.3, g = 1.0, b = 1.0, a = 1.0},  -- Cyan
        {r = 0.3, g = 1.0, b = 0.3, a = 1.0},  -- Green
        {r = 0.0, g = 0.8, b = 0.6, a = 1.0},  -- Teal
        {r = 0.5, g = 0.3, b = 1.0, a = 1.0},  -- Purple
        {r = 0.3, g = 0.6, b = 1.0, a = 1.0},  -- Sky Blue
    },
    -- Cycle 3: Pastel colors
    {
        {r = 1.0, g = 0.7, b = 0.8, a = 1.0},  -- Light Pink
        {r = 0.7, g = 1.0, b = 0.8, a = 1.0},  -- Light Green
        {r = 0.7, g = 0.8, b = 1.0, a = 1.0},  -- Light Blue
        {r = 1.0, g = 1.0, b = 0.7, a = 1.0},  -- Light Yellow
        {r = 1.0, g = 0.8, b = 0.7, a = 1.0},  -- Peach
        {r = 0.9, g = 0.7, b = 1.0, a = 1.0},  -- Lavender
    },
}

local colors = color_sets[(_G.reload_count - 1) % 3 + 1]

-- Spawn rectangles in a circle pattern
for i = 1, 6 do
    local angle = (i - 1) * (math.pi * 2 / 6)
    local radius = 150
    local x = math.cos(angle) * radius
    local y = math.sin(angle) * radius
    
    spawn({
        Sprite = {
            color = colors[i],
            custom_size = { x = 80, y = 80 }  -- Both formats work now!
        },
        Transform = {
            translation = {x = x, y = y, z = 0}
        },
        -- Custom Lua component for velocity
        Velocity = {
            x = math.cos(angle + math.pi/2) * 100,
            y = math.sin(angle + math.pi/2) * 100
        }
    })
    
    print("Spawned rectangle " .. i .. " at (" .. math.floor(x) .. ", " .. math.floor(y) .. ")")
end

-- Spawn instruction text
spawn({
    Text = { text = "Hot Reload Demo - Everything despawns and respawns every 5 seconds!" },
    TextFont = { font_size = 28 },
    TextColor = { color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0} },
    TextLayout = {},
    Transform = { translation = {x = 0, y = 280, z = 0} }
})

-- Spawn reload counter text
spawn({
    Text = { text = "Reload Count: " .. _G.reload_count },
    TextFont = { font_size = 24 },
    TextColor = { color = {r = 1.0, g = 1.0, b = 0.3, a = 1.0} },
    TextLayout = {},
    Transform = { translation = {x = 0, y = -280, z = 0} }
})

-- Movement system
function movement_system(world)
    local dt = world:delta_time()
    local entities = world:query({"Transform", "Velocity"}, nil)
    
    for i, entity in ipairs(entities) do
        local transform = entity:get("Transform")
        local velocity = entity:get("Velocity")
        
        if transform and velocity then
            -- Update position
            local new_x = transform.translation.x + velocity.x * dt
            local new_y = transform.translation.y + velocity.y * dt
            
            -- Bounce off screen edges (assuming 800x600 window)
            local new_vx = velocity.x
            local new_vy = velocity.y
            
            if new_x > 400 or new_x < -400 then
                new_vx = -velocity.x
                new_x = math.max(-400, math.min(400, new_x))
            end
            
            if new_y > 300 or new_y < -300 then
                new_vy = -velocity.y
                new_y = math.max(-300, math.min(300, new_y))
            end
            
            -- Update transform
            entity:set("Transform", {
                translation = {x = new_x, y = new_y, z = 0},
                rotation = transform.rotation,
                scale = transform.scale
            })
            
            -- Update velocity if bounced
            if new_vx ~= velocity.x or new_vy ~= velocity.y then
                entity:set("Velocity", {x = new_vx, y = new_vy})
            end
        end
    end
end

-- Hot reload system - automatically reloads the script every 5 seconds
function hot_reload_system(world)
    local dt = world:delta_time()
    _G.reload_timer = _G.reload_timer + dt
    
    if _G.reload_timer >= 5.0 then
        print("=== Hot Reload Triggered! ===")
        print(_G.reload_timer, dt)
        
        -- Despawn ALL entities spawned by this script automatically
        world:reload_script()
        print("Despawned all entities from this script")
        
        -- Reset timer
        _G.reload_timer = 0
        
        -- Set flag to trigger re-execution from Rust
        _G.should_reload = true
        
        print("=== Reload Requested - Waiting for script re-execution ===")
    end
end

-- Register systems
register_system("Update", movement_system)
register_system("Update", hot_reload_system)

print("=== Hot Reload Script Initialized ===")
