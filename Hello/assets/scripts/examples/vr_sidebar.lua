-- VR Sidebar Menu
-- Hosts the desktop sidebar menu in VR using VrPanel wrapper
--
-- Controls:
--   X button (left controller): Toggle sidebar panel
--   Right trigger: Click on UI elements
--   Right trigger (hold 0.5s): Right-click for context menu
--   Left grip: Grab and move the panel

local VrInput = require("modules/vr_input.lua")
local VrPointer = require("modules/vr_pointer.lua")
local VrPanel = require("modules/vr_panel.lua")
local SidebarMenu = require("scripts/ui/sidebar_menu.lua")
local Tooltip = require("scripts/ui/tooltip.lua")

print("=== VR Sidebar Menu ===")

-- State
local vr_sidebar = {
    is_visible = false,
    
    -- VrPanel wrapper (handles RTT, mesh, grip-to-move)
    panel = nil,
    
    -- UI entities
    ui_root = nil,
    menu = nil,
    sidebar_buttons = {},
    
    -- Dialog tracking
    tracked_dialogs = {},  -- {[entity_id] = VrPanel}
}

--- Spawn the sidebar UI (desktop-compatible, no VR knowledge)
function vr_sidebar:spawn_ui()
    -- Create UI root container
    local ui_root = spawn({
        Node = {
            width = { Px = 50 },  -- Initial width, will grow with panels
            height = { Px = 600 },
            flex_direction = "Row",
            align_items = "Stretch",
        },
        BackgroundColor = { color = { r = 0.1, g = 0.1, b = 0.12, a = 1.0 } },
        VrSidebarRoot = {},
    })
    self.ui_root = ui_root:id()
    
    -- Create sidebar menu as child of UI root
    self.menu = SidebarMenu.new()
    self.menu:show(self.ui_root)
    
    -- Spawn sidebar button entities
    self.sidebar_buttons = {}
    local files_btn = spawn({
        SidebarButton = {
            icon = "icons/files.png",
            title = "Files",
            script = "scripts/ui/file_browser.lua",
        }
    })
    table.insert(self.sidebar_buttons, files_btn:id())
    
    return self.ui_root
end

--- Create the VR panel
function vr_sidebar:create_panel(world)
    if self.is_visible then return end
    
    -- Calculate spawn position from left controller
    local position = VrInput.get_spawn_position_in_front_of_left(world, 0.35)
    if not position then
        position = { x = 0, y = 1.2, z = -0.5 }
    end
    
    -- Spawn the UI first
    local ui_entity = self:spawn_ui()
    
    -- Wrap it in a VR panel
    self.panel = VrPanel.wrap(ui_entity, {
        position = position,
        look_at_camera = true,
    })
    
    self.is_visible = true
    print("[VR_SIDEBAR] Panel created")
end

--- Destroy the VR panel
function vr_sidebar:destroy_panel(world)
    if not self.is_visible then return end
    
    -- Destroy any tracked dialog panels first
    for entity_id, dialog_panel in pairs(self.tracked_dialogs) do
        dialog_panel:destroy()
    end
    self.tracked_dialogs = {}
    
    -- Hide sidebar menu
    if self.menu then
        self.menu:hide_with_world(world, true)
        self.menu = nil
    end
    
    -- Despawn sidebar buttons
    if self.sidebar_buttons then
        for _, btn_id in ipairs(self.sidebar_buttons) do
            despawn(btn_id)
        end
        self.sidebar_buttons = {}
    end
    
    -- Destroy VrPanel (cleans up RTT camera and mesh)
    if self.panel then
        self.panel:destroy()
        self.panel = nil
    end
    
    -- Despawn UI root
    if self.ui_root then
        despawn(self.ui_root)
        self.ui_root = nil
    end
    
    self.is_visible = false
    print("[VR_SIDEBAR] Panel destroyed")
end

--- Toggle visibility
function vr_sidebar:toggle(world)
    if self.is_visible then
        self:destroy_panel(world)
    else
        self:create_panel(world)
    end
end

--- Get all surfaces for VR pointer (main panel + dialogs)
function vr_sidebar:get_surfaces()
    local surfaces = {}
    
    if self.panel and self.panel:is_active() then
        local surface = self.panel:get_surface()
        if surface then
            table.insert(surfaces, surface)
        end
    end
    
    for _, dialog_panel in pairs(self.tracked_dialogs) do
        if dialog_panel:is_active() then
            local surface = dialog_panel:get_surface()
            if surface then
                table.insert(surfaces, surface)
            end
        end
    end
    
    return surfaces
end

--- Detect and wrap high-z dialogs as VR panels
function vr_sidebar:check_for_dialogs(world)
    if not self.panel then return end
    
    -- Query for high GlobalZIndex entities (dialogs use 700+)
    local entities = world:query({"GlobalZIndex", "Node", "ComputedNode"}, nil)
    
    for _, entity in ipairs(entities) do
        local z = entity:get("GlobalZIndex")
        local entity_id = entity:id()
        
        if z and z.value >= 700 and not self.tracked_dialogs[entity_id] then
            -- Get computed size to check if it's a real dialog (not just backdrop)
            local computed = entity:get("ComputedNode")
            if computed and computed.size and computed.size.x > 100 and computed.size.y > 50 then
                -- Spawn dialog panel near parent panel
                local parent_pos = self.panel:get_position()
                local dialog_pos = {
                    x = parent_pos.x + 0.15,  -- Offset to the right
                    y = parent_pos.y,
                    z = parent_pos.z - 0.05   -- Slightly forward
                }
                
                local dialog_panel = VrPanel.wrap(entity_id, {
                    position = dialog_pos,
                    look_at_camera = true,
                    parent_panel = self.panel,
                })
                
                self.tracked_dialogs[entity_id] = dialog_panel
                print(string.format("[VR_SIDEBAR] Wrapped dialog entity %d as VR panel", entity_id))
            end
        end
    end
    
    -- Cleanup destroyed dialogs
    local to_remove = {}
    for entity_id, dialog_panel in pairs(self.tracked_dialogs) do
        local entity = world:get_entity(entity_id)
        if not entity then
            dialog_panel:destroy()
            table.insert(to_remove, entity_id)
        end
    end
    for _, entity_id in ipairs(to_remove) do
        self.tracked_dialogs[entity_id] = nil
        print(string.format("[VR_SIDEBAR] Cleaned up dialog panel %d", entity_id))
    end
end

-- Initialize VR pointer
VrPointer.init()

-- Update system
register_system("Update", function(world)
    -- Handle X button toggle
    if VrInput.is_x_just_pressed(world) then
        print("[VR_SIDEBAR] X button pressed - toggling panel")
        vr_sidebar:toggle(world)
    end
    
    if not vr_sidebar.is_visible then return end
    
    -- Update main panel (handles grip-to-move, auto-resize)
    if vr_sidebar.panel then
        vr_sidebar.panel:update(world)
    end
    
    -- Update dialog panels
    for _, dialog_panel in pairs(vr_sidebar.tracked_dialogs) do
        dialog_panel:update(world)
    end
    
    -- Check for new dialogs to wrap
    vr_sidebar:check_for_dialogs(world)
end)

-- VR Pointer system (runs in First for PointerInput processing)
register_system("First", function(world)
    if not vr_sidebar.is_visible then return end
    
    -- Update VR pointer (auto-detects panels via VrPanelMarker)
    VrPointer.update(world)
end)

print("=== VR Sidebar script loaded ===")
print("Press X button on left controller to open/close sidebar")
print("Point right controller at panel and pull trigger to click")
print("Hold trigger 0.5s on an item for right-click (context menu)")
print("Grip left controller to grab and move the panel")
