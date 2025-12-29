-- Tooltip Module
-- A simple tooltip system for Bevy UI
--
-- Usage:
--   local Tooltip = require("scripts/ui/tooltip.lua")
--   
--   -- In Pointer<Over> handler:
--   Tooltip.show("Button Title", entity)
--   
--   -- In Pointer<Out> handler:
--   Tooltip.hide()

local Tooltip = {}

-- Tooltip state
local tooltip_entity = nil
local tooltip_text_entity = nil
local parent_entity = nil  -- Optional parent for VR integration

-- Tooltip styling
local TOOLTIP_STYLE = {
    bg_color = { r = 0.1, g = 0.1, b = 0.1, a = 0.95 },
    text_color = { r = 1.0, g = 1.0, b = 1.0, a = 1.0 },
    font_size = 14,
    padding = 8,
    offset_x = 50,  -- Offset from trigger element
    offset_y = 0,
}

--- Set the parent entity for tooltip spawning (for VR integration)
--- @param entity number|nil Parent entity ID, or nil to spawn as root
function Tooltip.set_parent(entity)
    -- If clearing parent, also clear tooltip entity references
    -- (parent will be despawned, cascading to tooltip - prevent double despawn)
    if entity == nil and parent_entity ~= nil then
        tooltip_entity = nil
        tooltip_text_entity = nil
    end
    parent_entity = entity
end

--- Show a tooltip with the given text at the specified position
--- @param text string The tooltip text to display
--- @param x number|nil X position (defaults to 100)
--- @param y number|nil Y position (defaults to 100)
function Tooltip.show(text, x, y)
    -- Hide any existing tooltip first
    Tooltip.hide()
    
    if not text or text == "" then return end
    
    -- Use provided position or defaults
    local pos_x = x or 100
    local pos_y = y or 100
    
    -- Offset slightly from cursor so it doesn't interfere with hover
    pos_x = pos_x + 15
    pos_y = pos_y + 10
    
    -- Create tooltip container at mouse position
    local builder = spawn({
        Node = {
            position_type = "Absolute",
            left = {Px = pos_x},
            top = {Px = pos_y},
            padding = {
                left = {Px = TOOLTIP_STYLE.padding},
                right = {Px = TOOLTIP_STYLE.padding},
                top = {Px = TOOLTIP_STYLE.padding / 2},
                bottom = {Px = TOOLTIP_STYLE.padding / 2},
            },
        },
        BackgroundColor = { color = TOOLTIP_STYLE.bg_color },
        BorderRadius = {
            top_left = {Px = 4}, top_right = {Px = 4},
            bottom_left = {Px = 4}, bottom_right = {Px = 4},
        },
        GlobalZIndex = { value = 1000 },  -- Ensure tooltip is on top of everything
    })
    if parent_entity then
        builder = builder:with_parent(parent_entity)
    end
    tooltip_entity = builder:id()
    
    -- Create tooltip text
    tooltip_text_entity = spawn({
        Text = { text = text },
        TextFont = { font_size = TOOLTIP_STYLE.font_size },
        TextColor = { color = TOOLTIP_STYLE.text_color },
    }):with_parent(tooltip_entity):id()
end

--- Hide the current tooltip
function Tooltip.hide()
    if tooltip_entity then
        despawn(tooltip_entity)
        tooltip_entity = nil
        tooltip_text_entity = nil
    end
end

--- Check if a tooltip is currently visible
--- @return boolean
function Tooltip.is_visible()
    return tooltip_entity ~= nil
end

return Tooltip
