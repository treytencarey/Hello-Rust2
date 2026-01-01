-- VR Pointer Module
-- Provides VR controller raycasting, laser pointer visual, and PointerInput writing for UI interaction
-- Automatically detects VR panels via VrPanelMarker component on mesh entities.
--
-- Usage:
--   local VrPointer = require("modules/vr_pointer.lua")
--   VrPointer.init()  -- Spawn custom PointerId and laser visual
--   register_system("First", function(world)
--       VrPointer.update(world)  -- Auto-detects panels with VrPanelMarker
--   end)

local VrInput = require("modules/vr_input.lua")
local VrUI = nil  -- Lazy-loaded to avoid circular dependency

print(string.format("[VR_POINTER_DEBUG] ===== Module loaded at %f =====", os.clock()))

local VrPointer = {}

-- Custom pointer UUID for VR controller (matches Rust: 90870999)
local VR_POINTER_UUID = 90870999

-- Trigger state for edge detection
-- We compute edges in Lua because Rust's just_pressed is only true for 1 frame,
-- but Lua systems don't run every frame, so we'd miss edge events.
local trigger_state = {
    pressed = false,
    last_pressed = false,  -- Previous Lua frame's pressed state
    just_pressed = false,
    just_released = false,
    pending_press = false,   -- True if we pressed but weren't aiming at panel yet
    pending_release = false, -- True if we released but weren't aiming at panel yet
    -- Right-click simulation via trigger hold
    hold_start_time = nil,      -- Time when trigger was pressed (os.clock())
    hold_start_pos = nil,       -- Position {x, y} when trigger was pressed
    is_long_press = false,      -- True if we detected a long press (0.5s+ with small movement)
    long_press_sent = false,    -- True if we already sent the Secondary press for this gesture
}

-- Long press configuration
local LONG_PRESS_TIME = 0.5        -- Seconds to hold for right-click
local LONG_PRESS_TOLERANCE = 15    -- Pixels of movement allowed during hold

-- Entity references
local pointer_entity = nil
local laser_entity = nil
local debug_ray_entity = nil  -- Debug: visualize MeshRayCast ray

-- Hover and grip state for component-based communication
local hovered_entity = nil      -- Currently hovered panel mesh entity ID
local gripped_entity = nil      -- Currently gripped panel mesh entity ID

-- Laser settings
local LASER_LENGTH = 2.0  -- 2 meters
local LASER_RADIUS = 0.002  -- 2mm

--- Get the active pointer hand ("left" or "right")
local function get_active_hand()
    -- Lazy-load VrUI to avoid circular dependency
    if not VrUI then
        VrUI = require("modules/vr_ui.lua")
    end
    return VrUI.get_pointer_hand() or "right"
end

--- Get active controller position based on which hand is pointing
local function get_active_controller_position(world)
    local hand = get_active_hand()
    if hand == "left" then
        return VrInput.get_left_position(world)
    else
        return VrInput.get_right_position(world)
    end
end

--- Get active controller forward based on which hand is pointing
local function get_active_controller_forward(world)
    local hand = get_active_hand()
    if hand == "left" then
        return VrInput.get_left_forward(world)
    else
        return VrInput.get_right_forward(world)
    end
end

--- Get active trigger state based on which hand is pointing
local function get_active_trigger_pressed(world)
    local hand = get_active_hand()
    local vr_state = VrInput.get_buttons(world)
    if not vr_state then return false end
    if hand == "left" then
        return vr_state.left_trigger_pressed or false
    else
        return vr_state.right_trigger_pressed or false
    end
end

--- Get active grip just pressed state based on which hand is pointing
local function get_active_grip_just_pressed(world)
    local hand = get_active_hand()
    if hand == "left" then
        return VrInput.is_left_grip_just_pressed(world)
    else
        return VrInput.is_right_grip_just_pressed(world)
    end
end

--- Get active grip just released state based on which hand is pointing
local function get_active_grip_just_released(world)
    local hand = get_active_hand()
    if hand == "left" then
        return VrInput.is_left_grip_just_released(world)
    else
        return VrInput.is_right_grip_just_released(world)
    end
end

--- Initialize the VR pointer (spawn PointerId entity and laser visual)
function VrPointer.init()
    if pointer_entity then return end
    
    -- Spawn custom PointerId for VR controller
    local entity = spawn({
        ["PointerId"] = { Custom = VR_POINTER_UUID }
    })
    pointer_entity = entity:id()
    print("[VR_POINTER] Spawned custom PointerId entity:", pointer_entity)
    
    -- Create laser mesh (cylinder)
    local laser_mesh = create_asset("bevy_mesh::mesh::Mesh", {
        primitive = { Cylinder = { radius = LASER_RADIUS, half_height = 0.5 } }  -- Half height = 0.5 for unit cylinder
    })
    
    -- Create laser material (red, semi-transparent, unlit)
    local laser_material = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
        base_color = { r = 1.0, g = 0.2, b = 0.2, a = 0.8 },
        unlit = true,
        alpha_mode = "Blend"
    })
    
    -- Spawn laser visual (initially hidden at origin with scale 0)
    local laser = spawn({
        Mesh3d = { _0 = laser_mesh },
        ["MeshMaterial3d<StandardMaterial>"] = { _0 = laser_material },
        Transform = {
            translation = { x = 0, y = 0, z = 0 },
            rotation = { x = 0, y = 0, z = 0, w = 1 },
            scale = { x = 0, y = 0, z = 0 }  -- Start hidden
        },
        LaserPointer = {}  -- Marker component
    })
    laser_entity = laser:id()
    print("[VR_POINTER] Spawned laser pointer visual:", laser_entity)
end

--- Get current trigger state
--- @return table {pressed, just_pressed, just_released}
function VrPointer.get_trigger_state()
    return trigger_state
end

--- Helper: Calculate quaternion from rotation arc (from Y axis to target direction)
--- @param target table {x, y, z} target direction (normalized)
--- @return table {x, y, z, w} quaternion
local function quat_from_y_to_dir(target)
    local from = { x = 0, y = 1, z = 0 }  -- Y axis
    
    -- Dot product
    local dot = from.x * target.x + from.y * target.y + from.z * target.z
    
    -- If vectors are nearly parallel, return identity or 180 degree rotation
    if dot > 0.9999 then
        return { x = 0, y = 0, z = 0, w = 1 }
    elseif dot < -0.9999 then
        -- 180 degree rotation around X axis
        return { x = 1, y = 0, z = 0, w = 0 }
    end
    
    -- Cross product: from × target
    local cross = {
        x = from.y * target.z - from.z * target.y,
        y = from.z * target.x - from.x * target.z,
        z = from.x * target.y - from.y * target.x
    }
    
    -- Quaternion: w = 1 + dot, xyz = cross, then normalize
    local w = 1 + dot
    local len = math.sqrt(cross.x*cross.x + cross.y*cross.y + cross.z*cross.z + w*w)
    
    return {
        x = cross.x / len,
        y = cross.y / len,
        z = cross.z / len,
        w = w / len
    }
end

--- Update laser visual based on right controller pose
--- @param world userdata The world object
local function update_laser(world)
    if not laser_entity then return end
    
    local controller_pos = get_active_controller_position(world)
    local controller_fwd = get_active_controller_forward(world)
    
    if not controller_pos or not controller_fwd then
        -- Hide laser (scale 0)
        -- Note: We can't easily update just scale, so we skip update
        return
    end
    
    -- Calculate laser center (halfway along the ray)
    local laser_center = {
        x = controller_pos.x + controller_fwd.x * (LASER_LENGTH / 2),
        y = controller_pos.y + controller_fwd.y * (LASER_LENGTH / 2),
        z = controller_pos.z + controller_fwd.z * (LASER_LENGTH / 2)
    }
    
    -- Calculate rotation to point laser along forward direction
    local rotation = quat_from_y_to_dir(controller_fwd)
    
    -- Update laser transform
    local laser = world:get_entity(laser_entity)
    if laser then
        laser:set({
            Transform = {
                translation = laser_center,
                rotation = rotation,
                scale = { x = 1, y = LASER_LENGTH, z = 1 }
            }
        })
    end
end

--- Update pointer with raycasting and input events
--- @param world userdata The world object
--- @return number|nil Entity ID of hovered panel mesh, or nil
function VrPointer.update(world)
    
    -- Always update laser visual
    update_laser(world)
    
    -- Get VrButtonState for trigger
    local vr_state = VrInput.get_buttons(world)
    if not vr_state then return nil end
    
    -- Get current pressed state from active hand's trigger
    local current_pressed = get_active_trigger_pressed(world)
    
    -- Compute edges in Lua by comparing to last Lua frame's state
    -- This works even when Lua doesn't run every Rust frame
    trigger_state.just_pressed = current_pressed and not trigger_state.last_pressed
    trigger_state.just_released = not current_pressed and trigger_state.last_pressed
    trigger_state.pressed = current_pressed
    trigger_state.last_pressed = current_pressed  -- Save for next Lua frame
    
    -- Get active controller position and forward
    local ray_origin = get_active_controller_position(world)
    local forward = get_active_controller_forward(world)
    
    if not ray_origin or not forward then
        return nil  -- No controller data
    end
    
    -- Use MeshRayCast to get UV coordinates directly
    local ray = {
        origin = ray_origin,
        direction = forward,
        early_exit = false,
    }
    
    local result = world:call_systemparam_method("MeshRayCast", "cast_ray", ray)
    
    -- Store previous hovered entity to detect changes
    local prev_hovered_entity = hovered_entity
    
    local hit_point = nil  -- 3D world position where ray hit the panel
    local hit_panel_position = nil  -- Panel center position from Transform
    local new_entity = nil  -- Will be set if we hit a panel with VrPanelMarker
    
    -- Result is now a structured Lua table:
    -- {
    --   [1] = { entity = <u64 bits>, data = { uv = {x, y}, distance = ..., point = {...}, normal = {...} } },
    --   [2] = ...
    -- }
    if result and type(result) == "table" and #result > 0 then
        -- Iterate through all hits to find one with VrPanelMarker
        -- Raycast now returns ALL hits (piercing mode), so we skip non-panel entities
        for _, hit in ipairs(result) do
            local entity_bits = hit.entity
            local hit_data = hit.data
            
            if entity_bits and hit_data and hit_data.uv then
                local u = hit_data.uv.x
                local v = hit_data.uv.y
                
                if u and v then
                    -- Check if hit entity has VrPanelMarker component
                    local hit_entity = world:get_entity(entity_bits)
                    if hit_entity then
                        local marker = hit_entity:get("VrPanelMarker")
                        if marker and marker.texture_width and marker.rtt_image then
                            hit_point = hit_data.point  -- Store 3D hit point
                            
                            -- Get panel position from Transform for grip offset calculation
                            local transform = hit_entity:get("Transform")
                            if transform and transform.translation then
                                hit_panel_position = transform.translation
                            end
                            
                            -- Convert UV to texture pixel coords
                            -- Bevy Plane3d: UV (0,0) is top-left, V increases downward
                            -- Bevy UI: Y=0 is top, Y increases downward
                            -- So no flip is needed
                            local tex_x = u * marker.texture_width
                            local tex_y = v * marker.texture_height
                            
                            -- Write PointerInput messages
                            VrPointer._write_pointer_input(world, marker.rtt_image, tex_x, tex_y)
                            new_entity = entity_bits
                            break  -- Found a panel hit, stop iterating
                        end
                    end
                    -- Skip hits without valid UV or without VrPanelMarker
                end
            end
        end
    end
    
    -- Update hovered_entity (will be nil if no panel was hit this frame)
    hovered_entity = new_entity

    -- Update VrHovered component if hover state changed
    if hovered_entity ~= prev_hovered_entity then
        print(string.format("[VR_POINTER] Hover state changed: prev=%s, new=%s", tostring(prev_hovered_entity), tostring(hovered_entity)))
        -- Remove VrHovered from old entity
        if prev_hovered_entity then
            print(string.format("[VR_POINTER] Removing VrHovered from entity %d", prev_hovered_entity))
            local old_entity = world:get_entity(prev_hovered_entity)
            if old_entity then
                old_entity:set({ VrHovered = nil })
            end
        end
        
        -- Add VrHovered to new entity
        if hovered_entity then
            print(string.format("[VR_POINTER] Adding VrHovered to entity %d", hovered_entity))
            local new_entity = world:get_entity(hovered_entity)
            if new_entity then
                new_entity:set({ VrHovered = {} })
            end
        end
    end
    
    
    -- Handle grip for gripping the hovered panel
    local grip_pos = get_active_controller_position(world)
    local grip_forward = get_active_controller_forward(world)
    
    -- Grip start: if grip just pressed and we're hovering a panel
    local grip_just_pressed = get_active_grip_just_pressed(world)
    if grip_just_pressed then
        print(string.format("[VR_POINTER] Grip just pressed detected! hovered=%s hit_panel_position=%s hit_point=%s",
            tostring(hovered_entity ~= nil), tostring(hit_panel_position ~= nil), tostring(hit_point ~= nil)))
    end
    if grip_just_pressed and hovered_entity and hit_panel_position and hit_point then
        -- Calculate distance and offset from hit point to panel center
        if grip_pos then
            gripped_entity = hovered_entity
            local entity = world:get_entity(gripped_entity)
            if entity then
                -- Calculate distance from controller to hit point
                local dx = hit_point.x - grip_pos.x
                local dy = hit_point.y - grip_pos.y
                local dz = hit_point.z - grip_pos.z
                local distance = math.sqrt(dx*dx + dy*dy + dz*dz)
                
                -- Calculate offset from hit point to panel center
                local offset_x = hit_panel_position.x - hit_point.x
                local offset_y = hit_panel_position.y - hit_point.y
                local offset_z = hit_panel_position.z - hit_point.z
                
                -- Set VrGripping (overwrites any existing, no need to clear first)
                entity:set({
                    VrGripping = {
                        distance = distance,
                        offset_x = offset_x,
                        offset_y = offset_y,
                        offset_z = offset_z
                    }
                })
                print(string.format("[VR_POINTER] Started gripping entity %d, distance=%.3f, offset=(%.3f,%.3f,%.3f)", 
                    gripped_entity, distance, offset_x, offset_y, offset_z))
            end
        end
    end
    
    -- Grip release: remove VrGripping (only if we're actually gripping something)
    if gripped_entity and get_active_grip_just_released(world) then
        print(string.format("[VR_POINTER] Grip released, gripped_entity=%d", gripped_entity))
        local entity = world:get_entity(gripped_entity)
        if entity then
            entity:set({ VrGripping = {} })
            print(string.format("[VR_POINTER] Removed VrGripping from entity %d", gripped_entity))
        else
            print(string.format("[VR_POINTER] Warning: couldn't find entity %d to remove VrGripping", gripped_entity))
        end
        gripped_entity = nil
    end
    
    -- If we pressed but didn't hit a panel, queue the press for when we do hit
    if trigger_state.just_pressed and not hovered_entity then
        trigger_state.pending_press = true
    end
    
    -- If we released but didn't hit a panel, queue the release for when we do hit
    if trigger_state.just_released and not hovered_entity then
        trigger_state.pending_release = true
        trigger_state.pending_press = false  -- Cancel any pending press
    end
    
    -- Clear pending release once we're pressing again (new gesture)
    if trigger_state.pressed then
        trigger_state.pending_release = false
    end
    
    return hovered_entity
end

--- Internal: Raycast against a plane
--- @return table|nil Hit info {local_x, local_y} or nil if no hit
function VrPointer._raycast_plane(ray_origin, ray_dir, plane_pos, plane_normal, half_width, half_height)
    -- Dot product of normal and ray direction
    local denom = plane_normal.x * ray_dir.x + plane_normal.y * ray_dir.y + plane_normal.z * ray_dir.z
    
    if math.abs(denom) < 1e-6 then
        return nil  -- Ray parallel to plane
    end
    
    -- Distance from ray origin to plane
    local diff = {
        x = plane_pos.x - ray_origin.x,
        y = plane_pos.y - ray_origin.y,
        z = plane_pos.z - ray_origin.z
    }
    local t = (diff.x * plane_normal.x + diff.y * plane_normal.y + diff.z * plane_normal.z) / denom
    
    if t < 0 or t > 5.0 then
        return nil  -- Hit behind controller or too far
    end
    
    -- Hit point in world space
    local hit_world = {
        x = ray_origin.x + ray_dir.x * t,
        y = ray_origin.y + ray_dir.y * t,
        z = ray_origin.z + ray_dir.z * t
    }
    
    -- Convert to local space (simplified - assumes no rotation for now)
    local local_x = hit_world.x - plane_pos.x
    local local_y = hit_world.y - plane_pos.y
    
    -- Check bounds
    if math.abs(local_x) > half_width or math.abs(local_y) > half_height then
        return nil
    end
    
    return { local_x = local_x, local_y = local_y }
end

--- Internal: Write PointerInput messages
function VrPointer._write_pointer_input(world, rtt_image, tex_x, tex_y)
    local pointer_id = { Custom = VR_POINTER_UUID }
    local location = {
        target = { Image = rtt_image },
        position = { x = tex_x, y = tex_y }
    }
    
    -- Always send Move
    world:write_message("PointerInput", {
        pointer_id = pointer_id,
        location = location,
        action = { Move = { delta = { x = 0, y = 0 } } }
    })
    
    -- Long press detection logic
    local current_time = os.clock()
    
    -- On trigger press: start tracking hold
    if trigger_state.just_pressed then
        trigger_state.hold_start_time = current_time
        trigger_state.hold_start_pos = { x = tex_x, y = tex_y }
        trigger_state.is_long_press = false
        trigger_state.long_press_sent = false
    end
    
    -- While pressed: check for long press conditions
    if trigger_state.pressed and trigger_state.hold_start_time and not trigger_state.long_press_sent then
        local hold_duration = current_time - trigger_state.hold_start_time
        
        -- Check movement from start position
        local dx = tex_x - trigger_state.hold_start_pos.x
        local dy = tex_y - trigger_state.hold_start_pos.y
        local movement = math.sqrt(dx * dx + dy * dy)
        
        -- If held long enough with small movement, trigger right-click
        if hold_duration >= LONG_PRESS_TIME and movement <= LONG_PRESS_TOLERANCE then
            trigger_state.is_long_press = true
            trigger_state.long_press_sent = true
            
            -- Send Secondary (right-click) press and immediate release
            world:write_message("PointerInput", {
                pointer_id = pointer_id,
                location = location,
                action = { Press = "Secondary" }
            })
            world:write_message("PointerInput", {
                pointer_id = pointer_id,
                location = location,
                action = { Release = "Secondary" }
            })
            print(string.format("[VR_POINTER] Long press (right-click) at (%.1f, %.1f)", tex_x, tex_y))
        elseif movement > LONG_PRESS_TOLERANCE then
            -- Too much movement - cancel long press tracking, treat as drag
            trigger_state.hold_start_time = nil
            trigger_state.hold_start_pos = nil
        end
    end
    
    -- Send Press/Release on edges (or pending press)
    -- Skip if this was a long press - we don't want to send Primary click
    local should_press = (trigger_state.just_pressed or 
        (trigger_state.pending_press and trigger_state.pressed))
    
    if should_press then
        world:write_message("PointerInput", {
            pointer_id = pointer_id,
            location = location,
            action = { Press = "Primary" }
        })
        trigger_state.pending_press = false  -- Press was sent
        -- print(string.format("[VR_POINTER] Press at (%.1f, %.1f)", tex_x, tex_y))
    end
    
    if trigger_state.just_released or trigger_state.pending_release then
        -- Only send Release if we didn't do a long press
        if not trigger_state.is_long_press then
            world:write_message("PointerInput", {
                pointer_id = pointer_id,
                location = location,
                action = { Release = "Primary" }
            })
            -- print(string.format("[VR_POINTER] Release at (%.1f, %.1f)", tex_x, tex_y))
        end
        trigger_state.pending_release = false  -- Release was sent
        
        -- Reset long press state
        trigger_state.hold_start_time = nil
        trigger_state.hold_start_pos = nil
        trigger_state.is_long_press = false
        trigger_state.long_press_sent = false
    end
end

--- Cleanup (despawn pointer and laser entities)
function VrPointer.cleanup()
    if pointer_entity then
        despawn(pointer_entity)
        pointer_entity = nil
    end
    if laser_entity then
        despawn(laser_entity)
        laser_entity = nil
    end
    print("[VR_POINTER] Cleaned up pointer entities")
end

return VrPointer

