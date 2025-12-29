-- Sidebar Menu System
-- Extensible icon bar that discovers SidebarButton entities and opens panel scripts
--
-- Usage:
--   local SidebarMenu = require("scripts/ui/sidebar_menu.lua")
--   local menu = SidebarMenu.new()
--   -- Spawn button entities with SidebarButton component to add items

local SidebarMenu = {}
SidebarMenu.__index = SidebarMenu

-- Load tooltip module
local Tooltip = require("scripts/ui/tooltip.lua")

-- Constants
local ICON_BAR_WIDTH = 50
local ICON_SIZE = 36
local ICON_PADDING = 7

-- Colors (matching file_browser theme)
local COLORS = {
    icon_bar_bg = {r = 0.08, g = 0.08, b = 0.10, a = 1.0},
    icon_bg = {r = 0.15, g = 0.15, b = 0.18, a = 1.0},
    icon_hover = {r = 0.22, g = 0.22, b = 0.26, a = 1.0},
    icon_active = {r = 0.25, g = 0.35, b = 0.5, a = 1.0},
    icon_text = {r = 0.7, g = 0.7, b = 0.7, a = 1.0},
}

-- Global state that persists across hot reloads
_G._SIDEBAR_OPEN_PANELS = _G._SIDEBAR_OPEN_PANELS or {}

--- Create a new sidebar menu
function SidebarMenu.new()
    local self = setmetatable({}, SidebarMenu)
    
    -- State
    self.is_visible = false
    self.icon_bar_entity = nil
    self.entities = {}
    
    -- Button tracking
    self.buttons = {}  -- Discovered SidebarButton data: [{entity_id, icon, script, icon_entity}]
    self.known_button_ids = {}  -- Set of entity IDs we've already processed
    
    -- Open panels tracking - use global state to survive hot reloads
    self.open_panels = _G._SIDEBAR_OPEN_PANELS
    
    -- Deferred operations
    self.should_hide = false  -- Set to true to trigger hide in update with world access
    self.pending_button_click = nil  -- Index of button clicked, processed in update
    
    -- Register update system
    local menu = self
    register_system("Update", function(world)
        menu:update(world)
    end)
    
    return self
end

--- Show the sidebar (icon bar only - panels position themselves)
--- @param parent_entity number|nil Optional parent entity ID for VR integration
function SidebarMenu:show(parent_entity)
    if self.is_visible then return end
    
    -- Store parent for panels to use
    self.parent_entity = parent_entity
    
    -- Icon bar (narrow vertical strip on left)
    local builder = spawn({
        Node = {
            position_type = "Absolute",
            left = {Px = 0},
            top = {Px = 0},
            bottom = {Px = 0},
            width = {Px = ICON_BAR_WIDTH},
            flex_direction = "Column",
            align_items = "Center",
            padding = {top = {Px = 8}},
            row_gap = {Px = 4},
        },
        BackgroundColor = { color = COLORS.icon_bar_bg },
        GlobalZIndex = { value = 100 },
    })
    if parent_entity then
        builder = builder:with_parent(parent_entity)
    end
    self.icon_bar_entity = builder:id()
    table.insert(self.entities, self.icon_bar_entity)
    
    self.is_visible = true
    
    -- Render any buttons we already know about
    self:render_buttons()
end

--- Hide the sidebar and all panels
--- Note: This just despawns UI. To stop open panel scripts, use hide_with_world()
--- Hide the sidebar
--- @param skip_despawn boolean If true, only clear state without despawning (caller will despawn parent)
function SidebarMenu:hide(skip_despawn)
    if not self.is_visible then return end
    
    -- Clear open panels list (scripts should have been stopped already via hide_with_world)
    self.open_panels = {}
    
    -- Only despawn if not skipping (when parent will handle despawn via cascade)
    if not skip_despawn and self.icon_bar_entity then
        despawn(self.icon_bar_entity)
    end
    
    -- Clear tracking lists (entities are already despawned by cascade)
    self.entities = {}
    self.icon_bar_entity = nil
    
    -- Clear button icon entities (they were children and despawned)
    for _, btn in ipairs(self.buttons) do
        btn.icon_entity = nil
    end
    
    -- Clear parent entity to prevent stale reference on reopen
    self.parent_entity = nil
    
    self.is_visible = false
end

--- Hide with world access - stops all open panel scripts first
--- @param skip_despawn boolean If true, skip despawning (caller will despawn parent which cascades)
function SidebarMenu:hide_with_world(world, skip_despawn)
    if not self.is_visible then return end
    
    -- Always stop scripts to prevent their Update systems from running after entities are gone
    -- This is critical - if we don't stop them, they'll try to access despawned entities
    local panels = world:query({"SidebarPanel"})
    for _, panel_entity in ipairs(panels) do
        local panel_id = panel_entity:id()
        world:stop_owning_script(panel_id)
    end
    
    -- Now hide the UI (skip_despawn controls if we despawn ourselves)
    self:hide(skip_despawn)
end

--- Toggle visibility (deferred - triggers in update loop)
function SidebarMenu:toggle()
    if self.is_visible then
        self.should_hide = true  -- Defer to update where we have world access
    else
        self:show()
    end
end

--- Toggle with world access (immediate)
function SidebarMenu:toggle_with_world(world)
    if self.is_visible then
        self:hide_with_world(world)
    else
        self:show()
    end
end

--- Update (called each frame)
function SidebarMenu:update(world)
    -- Handle deferred hide (from toggle())
    if self.should_hide then
        self.should_hide = false
        self:hide_with_world(world)
        return
    end
    
    -- Process pending button updates (queued from update_button_state)
    if self.pending_button_updates then
        for _, update in ipairs(self.pending_button_updates) do
            self:update_button_state_with_world(world, update.idx, update.is_active)
        end
        self.pending_button_updates = {}
    end
    
    -- Handle escape key
    local key_events = world:read_events("bevy_input::keyboard::KeyboardInput")
    for _, e in ipairs(key_events) do
        if e.key_code and e.key_code.Escape and e.state and e.state.Pressed then
            -- Check debounce (prevent double-toggling if event persists across frames)
            local now = os.clock()
            if self.last_action_time and (now - self.last_action_time < 0.2) then
                break
            end
            
            -- Query for open sidebar panels via ECS (survives hot-reloads)
            local open_panels = world:query({"SidebarPanel"})
            
            self.last_action_time = now
            
            if #open_panels > 0 then
                -- Close the first/most recent panel
                local panel = open_panels[#open_panels]
                local panel_id = panel:id()
                world:stop_owning_script(panel_id)
            else
                -- No panels open, toggle sidebar visibility
                self:toggle_with_world(world)
            end
            -- Break to prevent handling multiple events in the same frame
            break
        end
    end
    
    -- Process pending button click (from on_button_click)
    if self.pending_button_click then
        local idx = self.pending_button_click
        self.pending_button_click = nil
        
        local btn = self.buttons[idx]
        if btn then
            -- Check if a panel for this script is already open via ECS query
            local existing_panels = world:query({"SidebarPanel"})
            local found_panel = nil
            for _, panel_entity in ipairs(existing_panels) do
                local panel_data = panel_entity:get("SidebarPanel")
                if panel_data and panel_data.script == btn.script then
                    found_panel = panel_entity
                    break
                end
            end
            
            if found_panel then
                -- Panel exists, close it
                world:stop_owning_script(found_panel:id())
                self:update_button_state(idx, false)
            else
                -- Open new panel
                local menu = self
                local left_offset = ICON_BAR_WIDTH
                local parent = self.parent_entity  -- Capture for closure
                
                require_async(btn.script, function(ModuleClass)
                    local instance = ModuleClass.new()
                    local container = instance:show(left_offset, parent)
                    if container then
                        menu:update_button_state(idx, true)
                    end
                end, { network = true, reload = true })
            end
        end
    end
    
    if not self.is_visible then return end
    
    -- Discover new SidebarButton entities (query all, we track known IDs)
    local all_buttons = world:query({"SidebarButton"})
    for _, entity in ipairs(all_buttons) do
        local id = entity:id()
        if not self.known_button_ids[id] then
            self.known_button_ids[id] = true
            local data = entity:get("SidebarButton")
            if data then
                table.insert(self.buttons, {
                    entity_id = id,
                    icon = data.icon,
                    title = data.title or data.icon_text or "?",
                    script = data.script,
                    icon_entity = nil,
                    is_active = false,
                })
                -- Render the new button
                self:render_button(#self.buttons)
            end
        end
    end
    
    -- Handle panels marked for closing (from button clicks)
    for i = #self.open_panels, 1, -1 do
        local panel = self.open_panels[i]
        if panel.should_close then
            world:stop_owning_script(panel.entity_id)
            -- Entity will be despawned, polling below will remove from list
            panel.should_close = false  -- Clear flag in case stop fails
        end
    end
    
    -- Poll for closed panels (entities that no longer exist)
    for i = #self.open_panels, 1, -1 do
        local panel = self.open_panels[i]
        local entity = world:get_entity(panel.entity_id)
        if entity == nil then
            -- Panel was closed
            table.remove(self.open_panels, i)
        end
    end
    
    -- Sync button visual states based on which SidebarPanel entities exist (ECS-based)
    local open_panels = world:query({"SidebarPanel"})
    local open_scripts = {}
    for _, panel_entity in ipairs(open_panels) do
        local panel_data = panel_entity:get("SidebarPanel")
        if panel_data and panel_data.script then
            open_scripts[panel_data.script] = true
        end
    end
    
    -- Update each button's visual state based on whether its script has an open panel
    for idx, btn in ipairs(self.buttons) do
        if btn then
            local has_icon = btn.icon_entity ~= nil
            local should_be_active = open_scripts[btn.script] or false
            local current_active = btn.is_active or false

            -- Only update if state changed AND we have an icon entity
            if should_be_active ~= current_active and has_icon then
                btn.is_active = should_be_active
                self:update_button_state_with_world(world, idx, should_be_active)
            end
        end
    end
end

--- Render all known buttons
function SidebarMenu:render_buttons()
    for i, _ in ipairs(self.buttons) do
        self:render_button(i)
    end
end

--- Render a single button in the icon bar
function SidebarMenu:render_button(idx)
    if not self.icon_bar_entity then return end
    
    local btn = self.buttons[idx]
    if not btn then return end
    
    -- Check if panel for this button is open (use tracked state)
    local is_active = btn.is_active or false
    
    local menu = self
    local icon_btn = spawn({
        Button = {},
        Node = {
            width = {Px = ICON_SIZE},
            height = {Px = ICON_SIZE},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = is_active and COLORS.icon_active or COLORS.icon_bg },
        BorderRadius = {
            top_left = {Px = 6}, top_right = {Px = 6},
            bottom_left = {Px = 6}, bottom_right = {Px = 6},
        },
    })
        :with_parent(self.icon_bar_entity)
        :observe("Pointer<Click>", function(entity, event)
            menu:on_button_click(idx)
        end)
        :observe("Pointer<Over>", function(entity, event)
            -- Show tooltip with button title at pointer position
            local x, y = nil, nil
            if event and event.pointer_location then
                x = event.pointer_location.position and event.pointer_location.position.x
                y = event.pointer_location.position and event.pointer_location.position.y
            end
            Tooltip.show(btn.title, x, y)
            -- Only hover effect if not active
            if not btn.is_active then
                entity:set({ BackgroundColor = { color = COLORS.icon_hover } })
            end
        end)
        :observe("Pointer<Out>", function(entity, event)
            -- Hide tooltip
            Tooltip.hide()
            if not btn.is_active then
                entity:set({ BackgroundColor = { color = COLORS.icon_bg } })
            end
        end)
        :id()
    
    btn.icon_entity = icon_btn
    
    -- Load and display icon image
    if btn.icon then
        local icon_asset = load_asset(btn.icon)
        spawn({
            ImageNode = { image = icon_asset },
            Node = {
                width = {Px = ICON_SIZE - 8},
                height = {Px = ICON_SIZE - 8},
            },
        }):with_parent(icon_btn)
    else
        -- Fallback to title text if no icon
        spawn({
            Text = { text = btn.title or "?" },
            TextFont = { font_size = 18 },
            TextColor = { color = COLORS.icon_text },
        }):with_parent(icon_btn)
    end
end

--- Update button visual state (active/inactive) - queues update for next update loop
--- Note: The actual color update requires world access, so we queue the update
function SidebarMenu:update_button_state(idx, is_active)
    local btn = self.buttons[idx]
    if btn and btn.icon_entity then
        -- Queue the update for the next update loop where we have world access
        if not self.pending_button_updates then
            self.pending_button_updates = {}
        end
        table.insert(self.pending_button_updates, {idx = idx, is_active = is_active})
    end
end

--- Update button visual state with world access (called from update loop)
function SidebarMenu:update_button_state_with_world(world, idx, is_active)
    local btn = self.buttons[idx]
    if btn and btn.icon_entity then
        local color = is_active and COLORS.icon_active or COLORS.icon_bg
        local entity = world:get_entity(btn.icon_entity)
        if entity then
            entity:set({ BackgroundColor = { color = color } })
        end
    end
end

--- Handle button click (deferred - will be processed in update with world access)
function SidebarMenu:on_button_click(idx)
    local btn = self.buttons[idx]
    if not btn then return end
    
    -- Queue the click for processing in update (where we have world access)
    self.pending_button_click = idx
end

return SidebarMenu


