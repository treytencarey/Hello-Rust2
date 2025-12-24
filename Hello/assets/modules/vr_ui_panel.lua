-- VR UI Panel Module
-- Creates and manages expandable UI panels in VR
--
-- Usage:
--   local VrPanel = require("modules/vr_ui_panel.lua")
--   local panel = VrPanel.create()
--   panel:show(world)  -- Spawns in front of left controller
--   panel:get_surface_info()  -- For raycasting

local VrInput = require("modules/vr_input.lua")

local VrPanel = {}
VrPanel.__index = VrPanel

-- Panel instances
local panels = {}
local next_panel_id = 1

-- Base texture settings (256x256 per expansion level)
local BASE_TEXTURE_WIDTH = 256
local BASE_TEXTURE_HEIGHT = 256
local BASE_PANEL_WIDTH = 0.2   -- meters per expansion level
local BASE_PANEL_HEIGHT = 0.2  -- meters

--- Create a new VR UI panel
--- @param position table Translation {x, y, z} in world space
--- @return table Panel instance
function VrPanel.create(position)
    local self = setmetatable({}, VrPanel)
    
    self.id = next_panel_id
    next_panel_id = next_panel_id + 1
    
    self.position = position or {x = 0, y = 1.0, z = -0.5}
    self.normal = {x = 0, y = 0, z = 1}  -- Panel faces +Z by default
    self.is_visible = false
    self.expansion_level = 1
    
    -- Calculated dimensions (updated when showing/expanding)
    self.texture_width = BASE_TEXTURE_WIDTH
    self.texture_height = BASE_TEXTURE_HEIGHT
    self.panel_width = BASE_PANEL_WIDTH
    self.panel_height = BASE_PANEL_HEIGHT
    
    -- Entity references
    self.panel_entity = nil
    self.camera_entity = nil
    self.ui_root_entity = nil
    self.rtt_image = nil
    
    -- Button entities (for cleanup)
    self.button_entities = {}
    
    panels[self.id] = self
    return self
end

--- Calculate dimensions based on expansion level
function VrPanel:_calculate_dimensions()
    self.texture_width = BASE_TEXTURE_WIDTH * self.expansion_level
    self.texture_height = BASE_TEXTURE_HEIGHT
    self.panel_width = BASE_PANEL_WIDTH * self.expansion_level
    self.panel_height = BASE_PANEL_HEIGHT
end

--- Calculate quaternion to make a Plane3d face a direction with proper up constraint
--- For Plane3d, +Y is the normal (the face of the panel)
--- This is equivalent to looking_at with Y as the forward axis
--- @param forward table {x, y, z} direction the panel should face (normalized)
--- @param up table {x, y, z} world up vector, typically {x=0, y=1, z=0}
--- @return table {x, y, z, w} quaternion
function VrPanel._quat_looking_at_y(forward, up)
    -- We want the panel's +Y to point toward 'forward'
    -- And the panel's local +Z to be as close to world 'up' as possible
    
    -- Build an orthonormal basis:
    -- Y axis = forward (normalized) - this is where +Y will point
    local y = forward
    
    -- X axis = up × Y (normalized) - perpendicular to both
    local x = {
        x = up.y * y.z - up.z * y.y,
        y = up.z * y.x - up.x * y.z,
        z = up.x * y.y - up.y * y.x
    }
    local x_len = math.sqrt(x.x*x.x + x.y*x.y + x.z*x.z)
    if x_len < 0.0001 then
        -- forward is parallel to up, use a fallback
        x = { x = 1, y = 0, z = 0 }
    else
        x.x = x.x / x_len
        x.y = x.y / x_len
        x.z = x.z / x_len
    end
    
    -- Z axis = Y × X (normalized) - completes the basis
    local z = {
        x = y.y * x.z - y.z * x.y,
        y = y.z * x.x - y.x * x.z,
        z = y.x * x.y - y.y * x.x
    }
    
    -- Convert rotation matrix to quaternion
    -- Matrix is [X, Y, Z] where each is a column
    -- m00=x.x, m01=y.x, m02=z.x
    -- m10=x.y, m11=y.y, m12=z.y
    -- m20=x.z, m21=y.z, m22=z.z
    local m00, m01, m02 = x.x, y.x, z.x
    local m10, m11, m12 = x.y, y.y, z.y
    local m20, m21, m22 = x.z, y.z, z.z
    
    local trace = m00 + m11 + m22
    local qx, qy, qz, qw
    
    if trace > 0 then
        local s = math.sqrt(trace + 1.0) * 2  -- s = 4 * qw
        qw = 0.25 * s
        qx = (m21 - m12) / s
        qy = (m02 - m20) / s
        qz = (m10 - m01) / s
    elseif m00 > m11 and m00 > m22 then
        local s = math.sqrt(1.0 + m00 - m11 - m22) * 2  -- s = 4 * qx
        qw = (m21 - m12) / s
        qx = 0.25 * s
        qy = (m01 + m10) / s
        qz = (m02 + m20) / s
    elseif m11 > m22 then
        local s = math.sqrt(1.0 + m11 - m00 - m22) * 2  -- s = 4 * qy
        qw = (m02 - m20) / s
        qx = (m01 + m10) / s
        qy = 0.25 * s
        qz = (m12 + m21) / s
    else
        local s = math.sqrt(1.0 + m22 - m00 - m11) * 2  -- s = 4 * qz
        qw = (m10 - m01) / s
        qx = (m02 + m20) / s
        qy = (m12 + m21) / s
        qz = 0.25 * s
    end
    
    return { x = qx, y = qy, z = qz, w = qw }
end

--- Calculate quaternion to rotate from +Y axis to target direction
--- Simple rotation arc formula - good for horizontal directions
--- @param target table {x, y, z} target direction (normalized)
--- @return table {x, y, z, w} quaternion
function VrPanel._quat_from_y_to_dir(target)
    local from = { x = 0, y = 1, z = 0 }  -- +Y axis
    
    -- Dot product
    local dot = from.x * target.x + from.y * target.y + from.z * target.z
    
    -- If vectors are nearly parallel, return identity
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

--- Multiply two quaternions (q1 * q2)
--- @param q1 table {x, y, z, w} first quaternion
--- @param q2 table {x, y, z, w} second quaternion
--- @return table {x, y, z, w} result quaternion
function VrPanel._quat_multiply(q1, q2)
    return {
        x = q1.w * q2.x + q1.x * q2.w + q1.y * q2.z - q1.z * q2.y,
        y = q1.w * q2.y - q1.x * q2.z + q1.y * q2.w + q1.z * q2.x,
        z = q1.w * q2.z + q1.x * q2.y - q1.y * q2.x + q1.z * q2.w,
        w = q1.w * q2.w - q1.x * q2.x - q1.y * q2.y - q1.z * q2.z
    }
end

--- Calculate quaternion to rotate from -Z axis to target direction
--- Rectangle mesh has normal along +Z, and looking_at points -Z toward target
--- @param target table {x, y, z} target direction (normalized)
--- @return table {x, y, z, w} quaternion
function VrPanel._quat_from_neg_z_to_dir(target)
    local from = { x = 0, y = 0, z = -1 }  -- -Z axis
    
    -- Dot product
    local dot = from.x * target.x + from.y * target.y + from.z * target.z
    
    -- If vectors are nearly parallel, return identity
    if dot > 0.9999 then
        return { x = 0, y = 0, z = 0, w = 1 }
    elseif dot < -0.9999 then
        -- 180 degree rotation around Y axis
        return { x = 0, y = 1, z = 0, w = 0 }
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

--- Show the panel (spawns all necessary entities)
--- @param world userdata The world object (optional, used to get controller position)
function VrPanel:show(world)
    if self.is_visible then return end
    
    -- Use stored rotation if available (from rebuild), otherwise calculate new one
    local rotation = self.rotation or { x = 0, y = 0, z = 0, w = 1 }
    
    -- If world is provided and we don't have a stored rotation, calculate position and rotation
    if world and not self.rotation then
        local left_pos = VrInput.get_left_position(world)
        local left_fwd = VrInput.get_left_forward(world)
        
        if left_pos and left_fwd then
            -- Calculate spawn position: 0.3m in front of left controller
            self.position = {
                x = left_pos.x + left_fwd.x * 0.3,
                y = left_pos.y + left_fwd.y * 0.3,
                z = left_pos.z + left_fwd.z * 0.3
            }
            
            -- Get camera position to face it
            local cameras = world:query({"GlobalTransform", "Camera3d"}, nil)
            local camera_pos = nil
            
            if cameras and #cameras > 0 then
                local camera_transform = cameras[1]:get("GlobalTransform")
                -- With reflection_to_lua, GlobalTransform should have proper field names
                -- GlobalTransform -> _0 -> translation (Vec3A) -> x, y, z
                if camera_transform and camera_transform._0 then
                    local affine = camera_transform._0
                    if affine.translation then
                        camera_pos = {
                            x = affine.translation.x,
                            y = affine.translation.y,
                            z = affine.translation.z
                        }
                    end
                end
            end
            
            if camera_pos then
                -- For Plane3d (+Y normal), we use the direction TO camera as the "up" vector
                -- This makes the +Y normal (the face) point toward the camera
                -- The looking_at target should be straight down (-Y) so -Z points down
                -- and +Y (the face) points horizontally toward camera
                local to_camera = {
                    x = camera_pos.x - self.position.x,
                    y = 0,  -- horizontal only for upright panel
                    z = camera_pos.z - self.position.z
                }
                -- Normalize
                local len = math.sqrt(to_camera.x * to_camera.x + to_camera.z * to_camera.z)
                if len > 0.001 then
                    to_camera.x = to_camera.x / len
                    to_camera.z = to_camera.z / len
                end
                
                -- looking_at target: straight UP from panel position (flips the orientation)
                self.looking_at_target = {
                    x = self.position.x,
                    y = self.position.y + 1,  -- 1 unit above
                    z = self.position.z
                }
                -- up vector: direction toward camera (this is where +Y will point)
                self.looking_at_up = to_camera
                
                print(string.format("[VR_PANEL] Panel at (%.2f, %.2f, %.2f), face toward camera at (%.2f, %.2f, %.2f)",
                    self.position.x, self.position.y, self.position.z,
                    camera_pos.x, camera_pos.y, camera_pos.z))
            else
                print("[VR_PANEL] Warning: Could not find camera, using forward direction")
                -- Fallback: look up, face in left_fwd direction
                self.looking_at_target = {
                    x = self.position.x,
                    y = self.position.y + 1,  -- above
                    z = self.position.z
                }
                self.looking_at_up = { x = left_fwd.x, y = 0, z = left_fwd.z }
            end
        end
    end
    
    -- Use stored rotation if available (from previous looking_at), otherwise identity
    -- looking_at will be applied on first update if looking_at_target is set
    local rotation = self.rotation or { x = 0, y = 0, z = 0, w = 1 }
    
    self:_calculate_dimensions()
    
    print(string.format("[VR_PANEL] Creating panel at (%.2f, %.2f, %.2f) size %.2fx%.2f",
        self.position.x, self.position.y, self.position.z,
        self.panel_width, self.panel_height))
    
    -- Create RTT image
    self.rtt_image = create_asset("bevy_image::image::Image", {
        width = self.texture_width,
        height = self.texture_height,
        format = "Bgra8UnormSrgb"
    })
    
    -- Spawn RTT camera (Camera2d for UI)
    local camera = spawn({
        Camera2d = {},
        Camera = {
            order = -1,
            target = { Image = self.rtt_image }
        }
    })
    self.camera_entity = camera:id()
    
    -- Create mesh using Plane3d - normal is +Y
    local mesh = create_asset("bevy_mesh::mesh::Mesh", {
        primitive = { 
            Plane3d = { 
                half_size = { x = self.panel_width / 2, y = self.panel_height / 2 } 
            } 
        }
    })
    
    local material = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
        base_color_texture = self.rtt_image,
        unlit = true,
        cull_mode = "None"  -- Double-sided
    })
    
    -- Generate a unique panel ID for this instance
    -- This allows us to find the panel via marker component query
    self.panel_marker_id = tostring(os.time()) .. "_" .. tostring(math.random(1000, 9999))
    
    -- Visibility: Hidden if looking_at will be applied (first show), Visible if rotation already set (rebuild)
    local initial_visibility = self.looking_at_target and "Hidden" or "Visible"
    
    -- Spawn the 3D panel plane
    local panel = spawn({
        Mesh3d = { _0 = mesh },
        ["MeshMaterial3d<StandardMaterial>"] = { _0 = material },
        Transform = {
            translation = self.position,
            rotation = rotation,
            scale = { x = 1, y = 1, z = 1 }
        },
        Visibility = initial_visibility,
        VrPanelMarker = { panel_id = self.panel_marker_id }  -- Custom marker for querying
    })
    self.panel_entity = panel:id()
    print("[VR_PANEL] Spawned panel with entity ID:", self.panel_entity)
    
    -- Spawn UI root
    local ui_root = spawn({
        Node = {
            width = { Percent = 100.0 },
            height = { Percent = 100.0 },
            flex_direction = "Row",
            justify_content = "Center",
            align_items = "Center",
            padding = { left = { Px = 20 }, right = { Px = 20 }, top = { Px = 20 }, bottom = { Px = 20 } }
        },
        BackgroundColor = { color = { r = 0.2, g = 0.2, b = 0.3, a = 0.9 } },
        UiTargetCamera = { entity = self.camera_entity }
    })
    self.ui_root_entity = ui_root:id()
    
    -- Spawn buttons
    self:_spawn_buttons()
    
    self.is_visible = true
    print("[VR_PANEL] Panel shown with id:", self.id)
end

--- Hide the panel (despawns all entities)
function VrPanel:hide()
    if not self.is_visible then return end
    
    -- Clear button list (they will be despawned with ui_root)
    self.button_entities = {}
    
    if self.ui_root_entity then
        despawn(self.ui_root_entity)
        self.ui_root_entity = nil
    end
    if self.panel_entity then
        despawn(self.panel_entity)
        self.panel_entity = nil
    end
    if self.camera_entity then
        despawn(self.camera_entity)
        self.camera_entity = nil
    end
    
    self.rtt_image = nil
    self.is_visible = false
    self.expansion_level = 1  -- Reset on hide
    self.rotation = nil  -- Clear stored rotation so next show recalculates
    print("[VR_PANEL] Panel hidden")
end

--- Toggle panel visibility
--- @param world userdata The world object (optional, used to get controller position when showing)
function VrPanel:toggle(world)
    if self.is_visible then
        self:hide()
    else
        self:show(world)
    end
end

--- Update function - applies Transform::looking_at via call_component_method
--- Must be called each frame - will only act on first frame after show()
--- @param world userdata The world object
function VrPanel:update(world)
    -- Process any pending expand request (deferred from observer callback)
    if self.pending_expand then
        self.pending_expand = false
        self.expansion_level = self.expansion_level + 1
        print("[VR_PANEL] Expanding to level:", self.expansion_level)
        if self.is_visible then
            self:_rebuild(world)  -- Pass current world, not stored reference
        end
    end
    
    -- Apply Transform::looking_at on first update after spawn (entity now exists)
    if self.looking_at_target and self.looking_at_up and self.panel_entity then
        world:call_component_method(
            self.panel_entity,
            "Transform",
            "looking_at",
            self.looking_at_target,  -- target: Vec3 (point above panel)
            self.looking_at_up       -- up: Vec3 (direction toward camera)
        )
        print(string.format("[VR_PANEL] Applied Transform::looking_at with up=(%.2f, %.2f, %.2f)",
            self.looking_at_up.x, self.looking_at_up.y, self.looking_at_up.z))
        
        -- Query for our panel using the VrPanelMarker custom component
        local panels = world:query({"VrPanelMarker", "Transform", "Visibility"}, nil)
        for _, entity in ipairs(panels) do
            local marker = entity:get("VrPanelMarker")
            if marker and marker.panel_id == self.panel_marker_id then
                -- Store rotation for _rebuild()
                local transform = entity:get("Transform")
                if transform then
                    self.rotation = transform.rotation
                    print("[VR_PANEL] Stored rotation for expand")
                end
                
                -- Reveal the panel now that looking_at is applied
                entity:set({ Visibility = "Visible" })
                print("[VR_PANEL] Panel revealed after looking_at")
                break
            end
        end
        
        -- Clear so we only do this once
        self.looking_at_target = nil
        self.looking_at_up = nil
    end
end

--- Expand the panel (deferred to Update for safe world access)
function VrPanel:expand()
    self.pending_expand = true
    print("[VR_PANEL] Expand requested, will process in Update")
end

--- Rebuild panel with new dimensions (for expansion)
--- Preserves current position and rotation
function VrPanel:_rebuild(world)
    -- Store state before rebuild
    local pos = self.position
    local rot = self.rotation  -- Use stored rotation from initial looking_at
    local level = self.expansion_level
    
    -- Hide (resets expansion_level)
    self:hide()
    
    -- Restore state and show WITHOUT world to preserve rotation
    -- (no looking_at recalculation)
    self.expansion_level = level
    self.position = pos
    self.rotation = rot
    self:show()  -- No world = no looking_at recalculation
end

--- Internal: Spawn button UI elements with observers
function VrPanel:_spawn_buttons()
    if not self.ui_root_entity then return end
    
    -- Clear old button references
    self.button_entities = {}
    
    local self_ref = self  -- Closure reference
    
    -- Spawn buttons based on expansion level
    for i = 1, self.expansion_level do
        local is_expand_button = (i == self.expansion_level)
        local button_label = is_expand_button and "Expand" or ("Item " .. i)
        
        -- Calculate button size
        local button_width = 180  -- Large buttons
        local button_height = 160
        
        -- Build button with observers - add all observers BEFORE getting id
        local button_builder = spawn({
            Button = {},
            Node = {
                width = { Px = button_width },
                height = { Px = button_height },
                margin = { left = { Px = 10 }, right = { Px = 10 }, top = { Px = 10 }, bottom = { Px = 10 } },
                justify_content = "Center",
                align_items = "Center"
            },
            BackgroundColor = { color = { r = 0.3, g = 0.5, b = 0.8, a = 1.0 } },
            BorderRadius = { 
                top_left = { Px = 16 }, 
                top_right = { Px = 16 }, 
                bottom_left = { Px = 16 }, 
                bottom_right = { Px = 16 } 
            },
            UiTargetCamera = { entity = self.camera_entity }
        })
            :with_parent(self.ui_root_entity)
            :observe("Pointer<Over>", function(entity, event)
                entity:set({ BackgroundColor = { color = { r = 0.8, g = 0.2, b = 0.2, a = 1.0 } } })
            end)
            :observe("Pointer<Out>", function(entity, event)
                entity:set({ BackgroundColor = { color = { r = 0.3, g = 0.5, b = 0.8, a = 1.0 } } })
            end)
            :observe("Pointer<Click>", function(entity, event)
                print("[VR_PANEL] Expand button clicked!")
                self_ref:expand()
            end)
        
        -- NOW get the id after all observers are attached
        local button_id = button_builder:id()
        table.insert(self.button_entities, button_id)
        
        -- Button text (also needs parent)
        spawn({
            Text = button_label,
            TextFont = { font_size = 32.0 },
            TextColor = { color = { r = 1.0, g = 1.0, b = 1.0, a = 1.0 } },
            UiTargetCamera = { entity = self.camera_entity }
        }):with_parent(button_id)
    end
end

--- Get surface info for raycasting
--- @return table|nil Surface info for VrPointer
function VrPanel:get_surface_info()
    if not self.is_visible then return nil end
    
    return {
        entity = self.panel_entity,
        position = self.position,
        normal = self.normal,
        rotation = self.rotation,  -- Quaternion for local space transform
        texture_width = self.texture_width,
        texture_height = self.texture_height,
        panel_half_width = self.panel_width / 2,
        panel_half_height = self.panel_height / 2,
        rtt_image = self.rtt_image
    }
end

--- Get all visible panels' surface info
--- @return table List of surface info tables
function VrPanel.get_all_surfaces()
    local surfaces = {}
    for _, panel in pairs(panels) do
        local info = panel:get_surface_info()
        if info then
            table.insert(surfaces, info)
        end
    end
    return surfaces
end

--- Get a panel by ID
--- @param id number Panel ID
--- @return table|nil Panel instance or nil
function VrPanel.get(id)
    return panels[id]
end

return VrPanel
