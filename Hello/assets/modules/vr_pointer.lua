-- VR Pointer Module
-- Provides VR controller raycasting, laser pointer visual, and PointerInput writing for UI interaction
--
-- Usage:
--   local VrPointer = require("modules/vr_pointer.lua")
--   VrPointer.init()  -- Spawn custom PointerId and laser visual
--   register_system("First", function(world)
--       VrPointer.update(world, panels)  -- panels = list of VrUiSurface tables
--   end)

local VrInput = require("modules/vr_input.lua")

local VrPointer = {}

-- Custom pointer UUID for VR controller (matches Rust: 90870999)
local VR_POINTER_UUID = 90870999

-- Trigger state for edge detection
local trigger_state = {
    pressed = false,
    just_pressed = false,
    just_released = false
}

-- Entity references
local pointer_entity = nil
local laser_entity = nil
local debug_ray_entity = nil  -- Debug: visualize MeshRayCast ray

-- Laser settings
local LASER_LENGTH = 2.0  -- 2 meters
local LASER_RADIUS = 0.002  -- 2mm

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
    
    local right_pos = VrInput.get_right_position(world)
    local right_fwd = VrInput.get_right_forward(world)
    
    if not right_pos or not right_fwd then
        -- Hide laser (scale 0)
        -- Note: We can't easily update just scale, so we skip update
        return
    end
    
    -- Calculate laser center (halfway along the ray)
    local laser_center = {
        x = right_pos.x + right_fwd.x * (LASER_LENGTH / 2),
        y = right_pos.y + right_fwd.y * (LASER_LENGTH / 2),
        z = right_pos.z + right_fwd.z * (LASER_LENGTH / 2)
    }
    
    -- Calculate rotation to point laser along forward direction
    local rotation = quat_from_y_to_dir(right_fwd)
    
    -- Update laser transform using entity:set()
    -- We need to query the laser entity - for now we'll spawn a global update
    -- Note: This requires searching for LaserPointer entities
    local lasers = world:query({"LaserPointer", "Transform"}, nil)
    for _, laser in ipairs(lasers) do
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
--- @param surfaces table List of surface info tables {entity, transform, texture_width, texture_height, panel_half_width, panel_half_height, rtt_image}
function VrPointer.update(world, surfaces)
    -- Always update laser visual
    update_laser(world)
    
    if not surfaces or #surfaces == 0 then return end
    
    -- Get VrButtonState for trigger
    local vr_state = VrInput.get_buttons(world)
    if not vr_state then return end
    
    -- Use right_trigger_just_pressed from resource (polled in Rust)
    local trigger_current = (vr_state.right_trigger or 0) > 0.5
    trigger_state.just_pressed = vr_state.right_trigger_just_pressed or false
    trigger_state.just_released = not trigger_current and trigger_state.pressed
    trigger_state.pressed = trigger_current
    
    -- Get right controller position and forward from VrControllerState
    local ray_origin = VrInput.get_right_position(world)
    local forward = VrInput.get_right_forward(world)
    
    if not ray_origin or not forward then
        return  -- No controller data
    end
    
    -- Use MeshRayCast to get UV coordinates directly
    local ray = {
        origin = ray_origin,
        direction = forward
    }
    
    local result = world:call_systemparam_method("MeshRayCast", "cast_ray", ray)
    
    -- Result is now a structured Lua table:
    -- {
    --   [1] = { entity = <u64 bits>, data = { uv = {x, y}, distance = ..., point = {...}, normal = {...} } },
    --   [2] = ...
    -- }
    if result and type(result) == "table" and #result > 0 then
        -- Iterate through hits to find one with VrPanelMarker
        for _, hit in ipairs(result) do
            local entity_bits = hit.entity
            local hit_data = hit.data
            
            if entity_bits and hit_data and hit_data.uv then
                local u = hit_data.uv.x
                local v = hit_data.uv.y
                
                if u and v then
                    print(string.format("[HIT] entity=%d UV(%.2f, %.2f)", entity_bits, u, v))
                    
                    -- Get entity wrapper and check for VrPanelMarker directly
                    local entity = world:get_entity(entity_bits)
                    if entity and entity:has("VrPanelMarker") then
                        -- Found the panel!
                        local surface = surfaces[1]
                        if not surface then return end
                        
                        -- Convert UV to texture pixel coords
                        -- Bevy Plane3d: UV (0,0) is top-left, V increases downward
                        -- Bevy UI: Y=0 is top, Y increases downward
                        -- So no flip is needed
                        local tex_x = u * surface.texture_width
                        local tex_y = v * surface.texture_height
                        
                        print(string.format("[HIT PANEL] UV(%.2f, %.2f) -> pixel(%.1f, %.1f)", 
                            u, v, tex_x, tex_y))
                        
                        -- Write PointerInput messages
                        VrPointer._write_pointer_input(world, surface.rtt_image, tex_x, tex_y)
                        return
                    end
                end
            end
        end
    end
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
    
    -- Send Press/Release on edges
    if trigger_state.just_pressed then
        world:write_message("PointerInput", {
            pointer_id = pointer_id,
            location = location,
            action = { Press = "Primary" }
        })
        print(string.format("[VR_POINTER] Press at (%.1f, %.1f)", tex_x, tex_y))
    end
    
    if trigger_state.just_released then
        world:write_message("PointerInput", {
            pointer_id = pointer_id,
            location = location,
            action = { Release = "Primary" }
        })
        print(string.format("[VR_POINTER] Release at (%.1f, %.1f)", tex_x, tex_y))
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

