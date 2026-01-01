-- VR UI Module
-- Automatically detects root-level UI Nodes and wraps them as 3D VR panels
--
-- Features:
--   - Auto-detects new root Nodes via Changed query
--   - Wraps them with RTT + 3D mesh using VrPanel
--   - X/A buttons toggle menu via global state
--   - Z-index determines distance from HMD
--   - VR pointer integration for interaction
--
-- Usage:
--   require("modules/vr_ui.lua")  -- in main.lua
--   -- That's it! All UI is automatically wrapped in VR

local VrInput = require("modules/vr_input.lua")
local VrPanel = require("modules/vr_panel.lua")
local VrPointer = require("modules/vr_pointer.lua")

local VrUI = {}

-- State
local tracked_nodes = {}  -- entity_id -> { panel, z_index }
local known_entity_ids = {}  -- Set of entity IDs we've processed
local is_vr_available = nil  -- Cached VR availability check
local pending_spawn_hand = nil  -- "left" or "right" - which hand triggered the pending spawn
local active_pointer_hand = "right"  -- Which hand is currently the pointer (opposite of spawn hand)

-- Configuration
local BASE_DISTANCE = 0.6  -- Base distance from HMD in meters
local Z_INDEX_SCALE = 0.0003  -- How much closer per z-index point
local MIN_DISTANCE = 0.25  -- Minimum distance from HMD
local PENDING_DELAY_FRAMES = 10  -- Frames to wait before wrapping (allows Children component to be applied)

-- Pending nodes (waiting for delay)
local pending_nodes = {}  -- entity_id -> { frame_count, entity_data }

--- Calculate spawn distance based on GlobalZIndex
--- Higher z-index = closer to HMD
--- @param z_index number The GlobalZIndex value
--- @return number Distance in meters
local function calculate_distance(z_index)
    z_index = z_index or 0
    local distance = BASE_DISTANCE - (z_index * Z_INDEX_SCALE)
    return math.max(MIN_DISTANCE, distance)
end

--- Check if VR is available (caches result)
--- @param world userdata The world object
--- @return boolean
local function check_vr_available(world)
    if is_vr_available ~= nil then
        return is_vr_available
    end
    
    local buttons = VrInput.get_buttons(world)
    is_vr_available = buttons ~= nil
    
    if is_vr_available then
        print("[VR_UI] VR detected, enabling VR UI wrapping")
    end
    
    return is_vr_available
end

--- Check if a Node is a root node we should wrap
--- Uses multiple heuristics to filter out child nodes and system nodes
--- @param world userdata The world object
--- @param entity userdata The entity to check
--- @return boolean True if this is a root node we should track
local function is_root_node(world, entity)
    local entity_id = entity:id()
    
    -- Check for Bevy's parent components (ChildOf is the new name, Parent is old)
    if entity:has("ChildOf") then
        print(string.format("[VR_UI_DEBUG] Entity %d filtered: has ChildOf", entity_id))
        return false
    end
    if entity:has("Parent") then
        print(string.format("[VR_UI_DEBUG] Entity %d filtered: has Parent", entity_id))
        return false
    end
    
    -- Skip if already has UiTargetCamera (already being rendered to a specific camera)
    -- This filters out nodes we've already wrapped
    if entity:has("UiTargetCamera") then
        print(string.format("[VR_UI_DEBUG] Entity %d filtered: has UiTargetCamera", entity_id))
        return false
    end
    
    -- Check Node dimensions to skip full-screen desktop UI
    local node = entity:get("Node")
    if node then
        local width = node.width
        local height = node.height
        
        if width and height then
            local is_full_width = width.Percent and width.Percent >= 100
            local is_full_height = height.Percent and height.Percent >= 100
            
            if is_full_width and is_full_height then
                print(string.format("[VR_UI_DEBUG] Entity %d filtered: full-screen", entity_id))
                return false  -- Full-screen desktop UI
            end
        end
    end
    
    print(string.format("[VR_UI_DEBUG] Entity %d passed all filters!", entity_id))
    return true
end

--- Send a simulated Escape key press event
--- @param world userdata The world object
local function send_escape_key(world)
    
    
    -- Find the primary window entity, required for KeyboardInput event
    local window_entity_bits = nil
    local windows = world:query({"Window"}, nil)
    if #windows > 0 then
        window_entity_bits = windows[1]:id()
        
        -- The event structure matches bevy::input::keyboard::KeyboardInput
        world:send_event("KeyboardInput", {
            key_code = "Escape",
            logical_key = "Escape",  -- Key::Escape (unit variant)
            state = "Pressed",
            window = window_entity_bits,
            ["repeat"] = false,
        })
        print("[VR_UI] Sent Escape key event")
    else
        print("[VR_UI] No windows found, cannot send Escape key event")
    end
end

--- Process newly changed Nodes and wrap them as VR panels
--- @param world userdata The world object
local function process_new_nodes(world)
    -- Query for Nodes that have Changed this frame
    -- Also require ComputedNode so we can get the size
    local changed_nodes = world:query({"Node", "ComputedNode"}, {"Node"})
    
    if #changed_nodes > 0 then
        print(string.format("[VR_UI_DEBUG] Query returned %d changed nodes", #changed_nodes))
    end
    
    for _, entity in ipairs(changed_nodes) do
        local entity_id = entity:id()
        
        -- Skip already tracked or pending nodes
        if known_entity_ids[entity_id] or pending_nodes[entity_id] then
            goto continue
        end
        
        -- Only track root nodes
        if not is_root_node(world, entity) then
            known_entity_ids[entity_id] = true  -- Mark as seen even if not tracked
            goto continue
        end
        
        -- Get ComputedNode size NOW from the query entity (before we lose it)
        local computed = entity:get("ComputedNode")
        local initial_size = nil
        if computed and computed.size then
            local w = computed.size.x or 0
            local h = computed.size.y or 0
            -- Require valid size (>= 10px) - don't use 50x50 fallback
            if w >= 10 and h >= 10 then
                initial_size = {
                    width = math.floor(w),
                    height = math.floor(h)
                }
                print(string.format("[VR_UI] Found Node %d with size %dx%d", 
                    entity_id, initial_size.width, initial_size.height))
            else
                print(string.format("[VR_UI] Found Node %d but size too small (%dx%d), will retry", entity_id, w, h))
                goto continue  -- Don't mark as seen - will retry when size is computed
            end
        else
            print(string.format("[VR_UI] Found Node %d but no computed size yet", entity_id))
            goto continue  -- Don't mark as seen - will retry when size is computed
        end
        
        -- Add to pending (wait a few frames for with_parent to be called)
        pending_nodes[entity_id] = {
            frame_count = 0,
            initial_size = initial_size,
        }
        
        ::continue::
    end
end

--- Process pending nodes (wrap after delay to allow with_parent to be called)
--- @param world userdata The world object
local function process_pending_nodes(world)
    local to_remove = {}
    
    for entity_id, data in pairs(pending_nodes) do
        data.frame_count = data.frame_count + 1
        
        -- Wait for delay
        if data.frame_count < PENDING_DELAY_FRAMES then
            goto continue
        end
        
        -- Check if entity still exists and is still a root (no parent added)
        local entity = world:get_entity(entity_id)
        if not entity then
            table.insert(to_remove, entity_id)
            known_entity_ids[entity_id] = true
            goto continue
        end
        
        -- Recheck ChildOf/Parent - these may have been added after initial detection
        if entity:has("ChildOf") or entity:has("Parent") then
            print(string.format("[VR_UI_DEBUG] Entity %d now has ChildOf/Parent, skipping", entity_id))
            table.insert(to_remove, entity_id)
            known_entity_ids[entity_id] = true
            goto continue
        end
        
        -- Check if this entity is a child of another entity by querying Children component on parents
        -- Build a set of all child entity IDs by scanning entities with Children component
        local is_child = false
        local parents_with_children = world:query({"Children"})
        for _, parent_entity in ipairs(parents_with_children) do
            local children_comp = parent_entity:get("Children")
            if children_comp then
                -- Debug: show structure of Children component
                -- print(string.format("[VR_UI_DEBUG] Parent %d Children: %s", parent_entity:id(), vim.inspect and vim.inspect(children_comp) or tostring(children_comp)))
                
                -- Children component may store IDs in different formats:
                -- 1. Direct table of numbers: {123, 456, 789}
                -- 2. Keyed by index: {[1]=123, [2]=456}
                -- 3. Wrapped in a field like _0 or children
                local children_list = children_comp._0 or children_comp.children or children_comp
                
                if type(children_list) == "table" then
                    for key, child_id in pairs(children_list) do
                        if type(child_id) == "number" and child_id == entity_id then
                            is_child = true
                            print(string.format("[VR_UI_DEBUG] Entity %d is child of %d", entity_id, parent_entity:id()))
                            break
                        end
                    end
                end
                if is_child then break end
            end
        end
        
        print(string.format("[VR_UI_DEBUG] Entity %d after %d frames: is_child=%s", 
            entity_id, data.frame_count, tostring(is_child)))
        
        if is_child then
            print(string.format("[VR_UI] Node %d is a child node, skipping", entity_id))
            table.insert(to_remove, entity_id)
            known_entity_ids[entity_id] = true
            goto continue
        end
        
        -- Get z-index for distance calculation
        local z_index = 0
        if entity:has("GlobalZIndex") then
            local gzi = entity:get("GlobalZIndex")
            z_index = gzi and gzi.value or 0
        end
        
        -- Timeout: if waiting too long, skip this entity (probably invalid)
        if data.frame_count > 60 then
            print(string.format("[VR_UI] Timeout waiting for entity %d, skipping", entity_id))
            table.insert(to_remove, entity_id)
            known_entity_ids[entity_id] = true
            goto continue
        end
        
        -- Use the initial_size captured at detection time
        -- If no valid size was captured, skip
        if not data.initial_size or data.initial_size.width < 10 or data.initial_size.height < 10 then
            print(string.format("[VR_UI] Entity %d has invalid initial_size, skipping", entity_id))
            table.insert(to_remove, entity_id)
            known_entity_ids[entity_id] = true
            goto continue
        end
        
        -- Calculate spawn position based on z-index and which hand triggered the spawn
        local distance = calculate_distance(z_index)
        local position = nil
        
        -- Use the pending spawn hand to determine which controller to spawn in front of
        if pending_spawn_hand == "left" then
            position = VrInput.get_spawn_position_in_front_of_left(world, distance)
            active_pointer_hand = "right"  -- Use right hand to point at left-spawned panel
        elseif pending_spawn_hand == "right" then
            position = VrInput.get_spawn_position_in_front_of_right(world, distance)
            active_pointer_hand = "left"  -- Use left hand to point at right-spawned panel
        else
            -- Default to HMD position
            position = VrInput.get_spawn_position_in_front_of_hmd(world, distance)
        end
        
        if not position then
            -- Fallback position if controller/HMD not available
            position = { x = 0, y = 1.5, z = -distance }
        end
        
        pending_spawn_hand = nil  -- Clear after use
        
        -- Wrap the Node in a VR panel with initial size (captured at detection)
        local panel = VrPanel.wrap(entity_id, {
            position = position,
            look_at_camera = true,
            initial_size = data.initial_size,
        })
        
        -- Track this node
        tracked_nodes[entity_id] = {
            panel = panel,
            z_index = z_index,
        }
        known_entity_ids[entity_id] = true
        table.insert(to_remove, entity_id)  -- Remove from pending
        
        print(string.format("[VR_UI] Wrapped Node %d as VR panel (%dx%d, z=%d, distance=%.2fm)",
            entity_id, data.initial_size.width, data.initial_size.height, z_index, distance))
        
        ::continue::
    end
    
    -- Remove processed pending nodes
    for _, entity_id in ipairs(to_remove) do
        pending_nodes[entity_id] = nil
    end
end

--- Update all tracked panels and clean up destroyed ones
--- @param world userdata The world object
local function update_tracked_panels(world)
    local to_remove = {}
    
    for entity_id, data in pairs(tracked_nodes) do
        -- Check if entity still exists
        local entity = world:get_entity(entity_id)
        if not entity then
            -- Entity was destroyed, clean up panel
            if data.panel then
                data.panel:destroy()
            end
            table.insert(to_remove, entity_id)
        else
            -- Update the panel (handles ComputedNode resize, grip-to-move, etc.)
            if data.panel then
                data.panel:update(world)
            end
        end
    end
    
    -- Remove destroyed entries
    for _, entity_id in ipairs(to_remove) do
        tracked_nodes[entity_id] = nil
        known_entity_ids[entity_id] = nil
        print(string.format("[VR_UI] Cleaned up panel for destroyed Node %d", entity_id))
    end
end

--- Get all active panel surfaces for VR pointer
--- @return table List of surface info tables
local function get_all_surfaces()
    local surfaces = {}
    
    for _, data in pairs(tracked_nodes) do
        if data.panel and data.panel:is_active() then
            local surface = data.panel:get_surface()
            if surface then
                table.insert(surfaces, surface)
            end
        end
    end
    
    return surfaces
end

--- Handle X/A button presses to toggle menu
--- X = left hand panel (use right hand to point)
--- A = right hand panel (use left hand to point)
--- @param world userdata The world object
local function handle_menu_buttons(world)
    if VrInput.is_x_just_pressed(world) then
        pending_spawn_hand = "left"
        send_escape_key(world)
    elseif VrInput.is_a_just_pressed(world) then
        pending_spawn_hand = "right"
        send_escape_key(world)
    end
end

--- Get which hand should be used for pointing
--- @return string "left" or "right"
function VrUI.get_pointer_hand()
    return active_pointer_hand
end

-- Initialize VR pointer
VrPointer.init()

-- Main Update system
register_system("Update", function(world)
    -- Skip if VR not available
    if not check_vr_available(world) then
        return
    end
    
    -- Handle X/A -> toggle menu
    handle_menu_buttons(world)
    
    -- Detect new root Nodes (adds to pending)
    process_new_nodes(world)
    
    -- Process pending nodes (wrap after delay)
    process_pending_nodes(world)
    
    -- Update existing panels
    update_tracked_panels(world)
end)

-- VR Pointer system (First schedule for PointerInput processing)
register_system("First", function(world)
    -- Skip if VR not available
    if not is_vr_available then
        return
    end
    
    -- Update VR pointer (auto-detects panels via VrPanelMarker)
    VrPointer.update(world)
end)

print("[VR_UI] Module loaded - will auto-wrap UI nodes when VR is detected")

return VrUI
