-- VR Panel Module
-- Wraps any UI entity as a 3D VR panel with RTT (render-to-texture)
--
-- Features:
--   - Auto-sizing from ComputedNode
--   - Auto-resize when UI size changes
--   - Grip-to-move support
--   - VR pointer integration
--
-- Usage:
--   local VrPanel = require("modules/vr_panel.lua")
--   
--   -- Wrap an existing UI entity
--   local panel = VrPanel.wrap(ui_entity_id, {
--       position = {x = 0, y = 1.2, z = -0.5},
--       look_at_camera = true,
--   })
--   
--   -- In update system:
--   panel:update(world)
--   
--   -- For VR pointer:
--   VrPointer.update(world, {panel:get_surface()})

local VrInput = require("modules/vr_input.lua")

local VrPanel = {}
VrPanel.__index = VrPanel

-- Conversion factor: pixels to meters
local PIXELS_TO_METERS = 0.0008  -- 0.8mm per pixel for comfortable VR reading

-- Unique panel IDs
local panel_counter = 0

--- Create a VR panel wrapper around an existing UI entity
--- @param ui_entity number Entity ID of the UI root to wrap
--- @param options table {position, look_at_camera, parent_panel, initial_size}
--- @return table VrPanel instance
function VrPanel.wrap(ui_entity, options)
    options = options or {}
    
    local self = setmetatable({}, VrPanel)
    
    -- Generate unique panel ID
    panel_counter = panel_counter + 1
    self.panel_id = tostring(os.time()) .. "_" .. tostring(panel_counter)
    
    -- Store wrapped entity
    self.ui_entity = ui_entity
    
    -- Position and orientation
    self.position = options.position or { x = 0, y = 1.2, z = -0.5 }
    self.rotation = nil  -- Set after looking_at
    self.look_at_camera = options.look_at_camera or false
    self.needs_look_at = options.look_at_camera or false
    
    -- Parent panel (for spawning dialogs near parent)
    self.parent_panel = options.parent_panel
    
    -- Current size (will be read from ComputedNode or initial_size)
    self.current_width = 100  -- Initial guess, updated on first frame
    self.current_height = 100
    
    -- Entity references
    self.rtt_camera = nil
    self.rtt_image = nil
    self.panel_mesh = nil
    
    -- Grip-to-move state
    self.is_gripping = false
    self.grip_offset = nil
    
    -- If initial_size is provided, spawn infrastructure immediately
    if options.initial_size then
        self:_spawn_infrastructure(options.initial_size.width, options.initial_size.height)
        self.needs_spawn = false
        self.needs_retarget = true  -- Still need to set UiTargetCamera on next update
        self.is_visible = true
        print(string.format("[VR_PANEL] Panel %s spawned immediately with size %dx%d", 
            self.panel_id, options.initial_size.width, options.initial_size.height))
    else
        -- Spawn infrastructure on next update (when we can read ComputedNode)
        self.needs_spawn = true
        self.needs_retarget = true  -- Will need to set UiTargetCamera after spawn
        self.is_visible = false
    end
    
    return self
end

--- Spawn RTT camera and 3D mesh for this panel
function VrPanel:_spawn_infrastructure(width, height)
    self.current_width = width
    self.current_height = height
    
    -- Create RTT image
    self.rtt_image = create_asset("bevy_image::image::Image", {
        width = width,
        height = height,
        format = "Bgra8UnormSrgb"
    })
    
    -- Spawn RTT camera (Camera2d for UI)
    -- Use high negative order so it renders before main camera
    -- Match vr_simple.lua: minimal Camera2d setup with just RTT target
    local camera = spawn({
        Camera2d = {},
        Camera = {
            order = -100 - panel_counter,  -- Unique order per panel, all before main camera
            target = { Image = self.rtt_image }
        }
        -- Note: Don't set OrthographicProjection - use Camera2d defaults
    })
    self.rtt_camera = camera:id()
    print(string.format("[VR_PANEL] RTT camera spawned: %d for UI entity %s (RTT size %dx%d)", 
        self.rtt_camera, tostring(self.ui_entity), width, height))
    
    -- Create 3D plane mesh
    local panel_width = width * PIXELS_TO_METERS
    local panel_height = height * PIXELS_TO_METERS
    
    local mesh = create_asset("bevy_mesh::mesh::Mesh", {
        primitive = { 
            Plane3d = { 
                half_size = { x = panel_width / 2, y = panel_height / 2 } 
            } 
        }
    })
    
    local material = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
        base_color_texture = self.rtt_image,
        unlit = true,
        cull_mode = "None"
    })
    
    -- Initial rotation and visibility
    local rotation = self.rotation or { x = 0, y = 0, z = 0, w = 1 }
    local initial_visibility = self.needs_look_at and "Hidden" or "Visible"
    
    -- Spawn panel mesh
    local panel = spawn({
        Mesh3d = { _0 = mesh },
        ["MeshMaterial3d<StandardMaterial>"] = { _0 = material },
        Transform = {
            translation = self.position,
            rotation = rotation,
            scale = { x = 1, y = 1, z = 1 }
        },
        Visibility = initial_visibility,
        VrPanelMarker = { panel_id = self.panel_id }
    })
    self.panel_mesh = panel:id()
    
    -- Flag to retarget UI entity to our RTT camera in update()
    -- This needs to be done via entity:set() which requires world context
    self.needs_retarget = true
    
    self.is_visible = true
    print(string.format("[VR_PANEL] Created panel %s (%dx%d) at pos (%.2f, %.2f, %.2f)", 
        self.panel_id, width, height, self.position.x, self.position.y, self.position.z))
end

--- Resize the panel (recreate mesh at new size, keep camera alive)
function VrPanel:_resize(world, new_width, new_height)
    if new_width == self.current_width and new_height == self.current_height then
        return
    end
    
    print(string.format("[VR_PANEL] Resize %s: %dx%d -> %dx%d",
        self.panel_id, self.current_width, self.current_height, new_width, new_height))
    
    self.current_width = new_width
    self.current_height = new_height
    
    -- Create new RTT image at new size
    local new_rtt_image = create_asset("bevy_image::image::Image", {
        width = new_width,
        height = new_height,
        format = "Bgra8UnormSrgb"
    })
    self.rtt_image = new_rtt_image
    
    -- Update camera's render target to new image
    if self.rtt_camera then
        local camera_entity = world:get_entity(self.rtt_camera)
        if camera_entity then
            camera_entity:set({
                Camera = {
                    target = { Image = new_rtt_image }
                }
            })
            print(string.format("[VR_PANEL] Updated camera %d target to new RTT", self.rtt_camera))
        end
    end
    
    -- Create new material with new texture
    local new_material = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
        base_color_texture = new_rtt_image,
        unlit = true,
        cull_mode = "None"
    })
    
    -- Calculate new mesh size
    local panel_width = new_width * PIXELS_TO_METERS
    local panel_height = new_height * PIXELS_TO_METERS
    
    -- Create new mesh at new size
    local new_mesh = create_asset("bevy_mesh::mesh::Mesh", {
        primitive = { 
            Plane3d = { 
                half_size = { x = panel_width / 2, y = panel_height / 2 } 
            } 
        }
    })
    
    -- Read current transform from old panel before despawning
    local current_position = self.position
    local current_rotation = self.rotation or { x = 0, y = 0, z = 0, w = 1 }
    
    if self.panel_mesh then
        local old_entity = world:get_entity(self.panel_mesh)
        if old_entity then
            local transform = old_entity:get("Transform")
            if transform then
                if transform.translation then
                    current_position = transform.translation
                    self.position = current_position  -- Update cached position
                end
                if transform.rotation then
                    current_rotation = transform.rotation
                    self.rotation = current_rotation  -- Update cached rotation
                end
            end
        end
        despawn(self.panel_mesh)
    end
    
    -- Spawn new panel mesh with preserved transform and marker
    local panel = spawn({
        Mesh3d = { _0 = new_mesh },
        ["MeshMaterial3d<StandardMaterial>"] = { _0 = new_material },
        Transform = {
            translation = current_position,
            rotation = current_rotation,
            scale = { x = 1, y = 1, z = 1 }
        },
        Visibility = "Visible",
        VrPanelMarker = { panel_id = self.panel_id }
    })
    self.panel_mesh = panel:id()
    
    -- Need to re-apply UiTargetCamera after camera target changed
    self.needs_retarget = true
    
    print(string.format("[VR_PANEL] Resize complete for %s, new mesh=%d", self.panel_id, self.panel_mesh))
end

--- Update panel (call each frame)
--- @param world userdata The world object
function VrPanel:update(world)
    -- First update: read ComputedNode and spawn infrastructure
    if self.needs_spawn then
        print("needs_spawn")
        local entity = world:get_entity(self.ui_entity)
        if entity then
            local computed = entity:get("ComputedNode")
            if computed and computed.size then
                local raw_width = computed.size.x or 0
                local raw_height = computed.size.y or 0
                
                -- If size is zero or very small, ComputedNode isn't ready yet - wait
                if raw_width < 10 or raw_height < 10 then
                    print("Waiting")
                    -- Keep waiting, ComputedNode not computed yet
                    return
                end
                
                local width = math.floor(raw_width)
                local height = math.floor(raw_height)
                print(string.format("[VR_PANEL] Detected ComputedNode size %dx%d for panel %s", width, height, self.panel_id))
                
                self:_spawn_infrastructure(width, height)
                self.needs_spawn = false
            end
        end
        return  -- Wait for next frame to continue
    end
    
    if self.needs_retarget and self.rtt_camera then
        print(string.format("[VR_PANEL_DEBUG] Panel %s: needs_retarget=true, rtt_camera=%d, calling entity:set()", 
            self.panel_id, self.rtt_camera))
        local entity = world:get_entity(self.ui_entity)
        if entity then
            -- Phase 1: Set UiTargetCamera
            entity:set({
                UiTargetCamera = { entity = self.rtt_camera }
            })
            print(string.format("[VR_PANEL] Retargeted UI %s to RTT camera %d", 
                tostring(self.ui_entity), self.rtt_camera))
            self.needs_retarget = false
        else
            print(string.format("[VR_PANEL] Retarget failed: UI entity %s not found", tostring(self.ui_entity)))
        end
    elseif self.needs_retarget and not self.rtt_camera then
        print(string.format("[VR_PANEL_DEBUG] Panel %s: needs_retarget=true but rtt_camera is nil!", self.panel_id))
    end
    
    -- Monitor for size changes (e.g., child panels opening)
    if self.rtt_camera then
        local entity = world:get_entity(self.ui_entity)
        if entity then
            local computed = entity:get("ComputedNode")
            if computed and computed.size then
                local new_width = math.floor(computed.size.x or 0)
                local new_height = math.floor(computed.size.y or 0)
                
                -- Check for significant size change (> 10px threshold)
                local width_diff = math.abs(new_width - self.current_width)
                local height_diff = math.abs(new_height - self.current_height)
                
                if (width_diff > 10 or height_diff > 10) and new_width >= 10 and new_height >= 10 then
                    print(string.format("[VR_PANEL] Size change detected for %s: %dx%d -> %dx%d",
                        self.panel_id, self.current_width, self.current_height, new_width, new_height))
                    self:_resize(world, new_width, new_height)
                    -- Camera entity stays the same, no need to retarget UiTargetCamera
                end
            end
        end
    end
    
    -- Apply looking_at on first visible frame
    if self.needs_look_at and self.panel_mesh then
        local entity = world:get_entity(self.panel_mesh)
        if entity then
            -- Get camera position to face toward
            local cameras = world:query({"GlobalTransform", "Camera3d"}, nil)
            if cameras and #cameras > 0 then
                local camera_transform = cameras[1]:get("GlobalTransform")
                if camera_transform and camera_transform._0 and camera_transform._0.translation then
                    local camera_pos = camera_transform._0.translation
                    local to_camera = {
                        x = camera_pos.x - self.position.x,
                        y = 0,
                        z = camera_pos.z - self.position.z
                    }
                    local len = math.sqrt(to_camera.x * to_camera.x + to_camera.z * to_camera.z)
                    if len > 0.001 then
                        to_camera.x = to_camera.x / len
                        to_camera.z = to_camera.z / len
                    end
                    
                    local looking_at_target = {
                        x = self.position.x,
                        y = self.position.y + 1,
                        z = self.position.z
                    }
                    
                    world:call_component_method(
                        self.panel_mesh,
                        "Transform",
                        "looking_at",
                        looking_at_target,
                        to_camera
                    )
                    
                    -- Store rotation and reveal
                    local transform = entity:get("Transform")
                    if transform then
                        self.rotation = transform.rotation
                    end
                    entity:set({ Visibility = "Visible" })
                end
            end
            
            self.needs_look_at = false
        end
        return  -- Don't process other updates on looking_at frame
    end
    
    -- Grip-to-move logic
    local left_pos = VrInput.get_left_position(world)
    
    -- Handle grip start
    if VrInput.is_left_grip_just_pressed(world) then
        if left_pos and self.position then
            self.grip_offset = {
                x = self.position.x - left_pos.x,
                y = self.position.y - left_pos.y,
                z = self.position.z - left_pos.z
            }
            self.is_gripping = true
        end
    end
    
    -- Handle grip release
    if VrInput.is_left_grip_just_released(world) then
        self.is_gripping = false
        self.grip_offset = nil
    end
    
    -- While gripping: move panel to follow hand
    if self.is_gripping and left_pos and self.grip_offset then
        local new_pos = {
            x = left_pos.x + self.grip_offset.x,
            y = left_pos.y + self.grip_offset.y,
            z = left_pos.z + self.grip_offset.z
        }
        self.position = new_pos
        
        local entity = world:get_entity(self.panel_mesh)
        if entity then
            entity:set({
                Transform = {
                    translation = new_pos,
                    rotation = self.rotation,
                    scale = { x = 1, y = 1, z = 1 }
                }
            })
        end
    end
end

--- Get surface info for VR pointer integration
--- @return table Surface info compatible with VrPointer.update()
function VrPanel:get_surface()
    if not self.is_visible then return nil end
    
    return {
        entity = self.panel_mesh,
        position = self.position,
        texture_width = self.current_width,
        texture_height = self.current_height,
        panel_half_width = (self.current_width * PIXELS_TO_METERS) / 2,
        panel_half_height = (self.current_height * PIXELS_TO_METERS) / 2,
        rtt_image = self.rtt_image
    }
end

--- Get RTT image asset ID
function VrPanel:get_rtt_image()
    return self.rtt_image
end

--- Get position (for spawning child panels nearby)
function VrPanel:get_position()
    return self.position
end

--- Check if panel is visible
function VrPanel:is_active()
    return self.is_visible
end

--- Destroy the panel (cleanup all entities)
function VrPanel:destroy()
    if self.panel_mesh then 
        despawn(self.panel_mesh) 
        self.panel_mesh = nil 
    end
    if self.rtt_camera then 
        despawn(self.rtt_camera) 
        self.rtt_camera = nil 
    end
    
    self.rtt_image = nil
    self.is_visible = false
    
    print(string.format("[VR_PANEL] Destroyed panel %s", self.panel_id))
end

return VrPanel
