-- Hot Reload Example Script
-- Automatically reloads when you save changes to this file!
-- Everything spawned by this script is destroyed and recreated on reload

print("=== Hot Reload Script Starting ===")

-- Global state for reload count
if not _G.reload_count then
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
            custom_size = { x = 80, y = 80 }
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
    Text2d = { text = "Hot Reload Demo - Edit and save to auto-reload!" },
    TextFont = { font_size = 24 },
    TextColor = { color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0} },
    TextLayout = {},
    Transform = { translation = {x = 0, y = 280, z = 0} }
})

-- Spawn reload counter text
spawn({
    Text2d = { text = "Reload Count: " .. _G.reload_count },
    TextFont = { font_size = 32 },
    TextColor = { color = {r = 1.0, g = 1.0, b = 0.3, a = 1.0} },
    TextLayout = {},
    Transform = { translation = {x = 0, y = -280, z = 0} }
})

-- Spawn manual reload button
spawn({
    Button = {},
    BackgroundColor = { color = {r = 0.2, g = 0.6, b = 0.8, a = 1.0} },
    Text = { text = "Manual Reload" },
    TextFont = { font_size = 20 },
    TextColor = { color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0} },
    OnClick = function(world)
        print("=== Manual Reload Button Clicked ===")
        world:reload_current_script()
    end
})

-- Spawn stop button
spawn({
    Button = {},
    BackgroundColor = { color = {r = 0.8, g = 0.2, b = 0.2, a = 1.0} },
    Node = {
        top = { Px = 0 },
        left = { Px = 200 }
    },
    Text = { text = "Stop Script" },
    TextFont = { font_size = 20 },
    TextColor = { color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0} },
    OnClick = function(world)
        print("=== Stop Script Button Clicked ===")
        world:stop_current_script()
    end
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
            entity:set({ Transform = {
                translation = {x = new_x, y = new_y, z = 0},
                rotation = transform.rotation,
                scale = transform.scale
            }})

            -- Update velocity if bounced
            if new_vx ~= velocity.x or new_vy ~= velocity.y then
                entity:set({ Velocity = {x = new_vx, y = new_vy} })
            end
        end
    end
end

-- Button click handling system
function button_system(world)
    local buttons = world:query({"Button", "Interaction", "OnClick"}, {"Interaction"})
    
    for i, entity in ipairs(buttons) do
        local interaction = entity:get("Interaction")
        if interaction == "Pressed" then
            local on_click = entity:get("OnClick")
            if on_click and type(on_click) == "function" then
                on_click(world)
            end
        end
    end
end

-- Register systems
register_system("Update", movement_system)
register_system("Update", button_system)

print("=== Hot Reload Script Initialized ===")
print("💡 Try these:")
print("   1. Edit this file and save to see auto-reload")
print("   2. Click 'Manual Reload' button to trigger reload via Lua (sets _G.manual_reload = true)")
print("   3. Change colors, positions, or text and see them update instantly!")
