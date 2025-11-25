-- Check what components are available

-- Spawn button with OnClick callback
spawn({
    Button = {},
    BackgroundColor = {
        color = { r = 0.2, g = 0.6, b = 0.8, a = 1.0 }
    },
    Text = { text = "Click Me!" },
    TextFont = { font_size = 32 },
    TextColor = { color = { r = 1.0, g = 1.0, b = 1.0, a = 1.0 } },
    
    -- Lua function as a component!
    OnClick = function()
        print("Button clicked")
    end
})

-- Define the system entirely in Lua!
function button_system(world)
    local entities = world:query(
        {"Button", "Interaction", "OnClick"},
        {"Interaction"}
    )
    
    if #entities > 0 then
        local entity = entities[1]
        
        local interaction = entity:get("Interaction")
        if interaction then
            if interaction == "Pressed" then
                entity:get("OnClick")()
            end
        end
    end
end

-- Register the system to run every frame
register_system("Update", button_system)
