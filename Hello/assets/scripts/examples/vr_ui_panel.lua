-- VR UI Panel Example (Lua)
-- Press X button on left VR controller to open an expandable UI panel
--
-- This example demonstrates:
-- - Reading VR button states from Lua via VrButtonState resource
-- - Getting controller positions via VrControllerState resource
-- - Creating RTT UI on a 3D plane
-- - VR controller raycasting for pointer input
-- - Modular script organization with separate input, panel, and pointer modules

local VrInput = require("modules/vr_input.lua")
local VrPanel = require("modules/vr_ui_panel.lua")
local VrPointer = require("modules/vr_pointer.lua")

print("=== VR UI Panel Example (Lua) ===")

-- Create main panel (position will be set when shown)
local main_panel = VrPanel.create()

-- Initialize VR pointer system
VrPointer.init()

-- Panel toggle system (Update schedule)
register_system("Update", function(world)
    -- Apply any pending Transform::looking_at via call_component_method
    main_panel:update(world)
    
    -- Check for X button to toggle panel
    if VrInput.is_x_just_pressed(world) then
        print("[VR_LUA] X button pressed - toggling panel")
        main_panel:toggle(world)
    end
end)

-- VR Pointer raycasting system (First schedule with PickingSystems::Input)
-- This needs to run in First schedule for PointerInput to be processed
register_system("First", function(world)
    if not main_panel.is_visible then return end
    
    -- Update VR pointer (auto-detects panels via VrPanelMarker)
    VrPointer.update(world)
end)

print("=== VR UI Panel script loaded ===")
print("Press X button on left controller to open/close panel")
print("Point right controller at panel and pull trigger to click")
print("Click 'Expand' button on panel to add more items")
