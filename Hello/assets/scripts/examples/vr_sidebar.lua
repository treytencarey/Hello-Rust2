-- VR Sidebar Menu
-- Hosts the desktop sidebar menu in VR using render-to-texture on a 3D plane
--
-- Press X button on left VR controller to toggle the sidebar panel
-- Use right controller to interact with UI (laser pointer + trigger)

local VrInput = require("modules/vr_input.lua")
local VrPointer = require("modules/vr_pointer.lua")
local SidebarMenu = require("scripts/ui/sidebar_menu.lua")
local Tooltip = require("scripts/ui/tooltip.lua")

print("=== VR Sidebar Menu ===")

-- Constants
local UI_HEIGHT = 600  -- Fixed height in pixels
local PIXELS_TO_METERS = 0.0008  -- 0.8mm per pixel for comfortable VR reading
local MAX_RTT_WIDTH = 800  -- Max RTT width - accommodates icon bar (50) + multiple panels (280 each)

-- State
local vr_sidebar = {
    is_visible = false,
    
    -- Entity references
    rtt_camera = nil,
    rtt_image = nil,
    panel_mesh = nil,
    ui_root = nil,
    
    -- Sidebar instance
    menu = nil,
    
    -- Size tracking
    current_width = 50,  -- Start with just icon bar
    current_height = UI_HEIGHT,
    
    -- Positioning
    position = nil,
    rotation = nil,
    looking_at_target = nil,
    looking_at_up = nil,
    panel_marker_id = nil,
}

--- Calculate quaternion to face panel toward camera
local function quat_from_y_to_dir(target)
    local from = { x = 0, y = 1, z = 0 }
    local dot = from.x * target.x + from.y * target.y + from.z * target.z
    
    if dot > 0.9999 then
        return { x = 0, y = 0, z = 0, w = 1 }
    elseif dot < -0.9999 then
        return { x = 1, y = 0, z = 0, w = 0 }
    end
    
    local cross = {
        x = from.y * target.z - from.z * target.y,
        y = from.z * target.x - from.x * target.z,
        z = from.x * target.y - from.y * target.x
    }
    
    local w = 1 + dot
    local len = math.sqrt(cross.x*cross.x + cross.y*cross.y + cross.z*cross.z + w*w)
    
    return {
        x = cross.x / len,
        y = cross.y / len,
        z = cross.z / len,
        w = w / len
    }
end

--- Create the VR panel with RTT and mesh
function vr_sidebar:create_panel(world)
    if self.is_visible then return end
    
    -- Calculate spawn position from left controller
    local left_pos = VrInput.get_left_position(world)
    local left_fwd = VrInput.get_left_forward(world)
    
    if left_pos and left_fwd then
        self.position = {
            x = left_pos.x + left_fwd.x * 0.35,
            y = left_pos.y + left_fwd.y * 0.35,
            z = left_pos.z + left_fwd.z * 0.35
        }
        
        -- Get camera to face toward it
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
                
                self.looking_at_target = {
                    x = self.position.x,
                    y = self.position.y + 1,
                    z = self.position.z
                }
                self.looking_at_up = to_camera
            end
        end
    else
        -- Fallback position
        self.position = { x = 0, y = 1.2, z = -0.5 }
    end
    
    self:_spawn_infrastructure()
end

--- Spawn RTT camera, mesh, and UI root
function vr_sidebar:_spawn_infrastructure()
    -- Create RTT image at current size (dynamic sizing)
    self.rtt_image = create_asset("bevy_image::image::Image", {
        width = self.current_width,
        height = self.current_height,
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
    self.rtt_camera = camera:id()
    
    -- Create 3D plane mesh at current size
    local panel_width = self.current_width * PIXELS_TO_METERS
    local panel_height = self.current_height * PIXELS_TO_METERS
    
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
    
    -- Generate marker ID for raycasting
    self.panel_marker_id = tostring(os.time()) .. "_" .. tostring(math.random(1000, 9999))
    
    local rotation = self.rotation or { x = 0, y = 0, z = 0, w = 1 }
    local initial_visibility = self.looking_at_target and "Hidden" or "Visible"
    
    local panel = spawn({
        Mesh3d = { _0 = mesh },
        ["MeshMaterial3d<StandardMaterial>"] = { _0 = material },
        Transform = {
            translation = self.position,
            rotation = rotation,
            scale = { x = 1, y = 1, z = 1 }
        },
        Visibility = initial_visibility,
        VrPanelMarker = { panel_id = self.panel_marker_id }
    })
    self.panel_mesh = panel:id()
    
    -- Spawn UI root at current size (matches RTT)
    local ui_root = spawn({
        Node = {
            width = { Px = self.current_width },
            height = { Px = self.current_height },
            flex_direction = "Row",
            align_items = "Stretch",
        },
        BackgroundColor = { color = { r = 0.1, g = 0.1, b = 0.12, a = 1.0 } },
        UiTargetCamera = { entity = self.rtt_camera },
        VrSidebarRoot = {},
    })
    self.ui_root = ui_root:id()
    
    -- Set tooltip parent for VR
    Tooltip.set_parent(self.ui_root)
    
    -- Create sidebar menu as child of UI root
    self.menu = SidebarMenu.new()
    self.menu:show(self.ui_root)
    
    -- Spawn sidebar button entities and track for cleanup
    self.sidebar_buttons = {}
    local files_btn = spawn({
        SidebarButton = {
            icon = "icons/files.png",
            title = "Files",
            script = "scripts/ui/file_browser.lua",
        }
    })
    table.insert(self.sidebar_buttons, files_btn:id())
    
    self.is_visible = true
    print("[VR_SIDEBAR] Panel created")
end

--- Destroy the VR panel
function vr_sidebar:destroy_panel(world)
    if not self.is_visible then return end
    
    -- Hide sidebar menu (this stops any owning scripts too)
    if self.menu then
        self.menu:hide_with_world(world)
        self.menu = nil
    end
    
    -- Despawn sidebar buttons
    if self.sidebar_buttons then
        for _, btn_id in ipairs(self.sidebar_buttons) do
            despawn(btn_id)
        end
        self.sidebar_buttons = nil
    end
    
    -- Clear tooltip parent
    Tooltip.set_parent(nil)
    
    -- Despawn entities
    if self.ui_root then despawn(self.ui_root) self.ui_root = nil end
    if self.panel_mesh then despawn(self.panel_mesh) self.panel_mesh = nil end
    if self.rtt_camera then despawn(self.rtt_camera) self.rtt_camera = nil end
    
    self.rtt_image = nil
    self.is_visible = false
    self.rotation = nil  -- Clear so next show recalculates
    
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

--- Resize the panel (rebuild camera/mesh, update UI root - keep content intact)
function vr_sidebar:resize_panel(world, new_width, new_height)
    if new_width == self.current_width and new_height == self.current_height then
        return
    end
    
    print(string.format("[VR_SIDEBAR] Resize %dx%d -> %dx%d",
        self.current_width, self.current_height, new_width, new_height))
    
    -- Read current rotation from entity BEFORE despawning
    local saved_rotation = self.rotation  -- fallback
    if self.panel_mesh then
        local panels = world:query({"VrPanelMarker", "Transform"}, nil)
        for _, entity in ipairs(panels) do
            local marker = entity:get("VrPanelMarker")
            if marker and marker.panel_id == self.panel_marker_id then
                local transform = entity:get("Transform")
                if transform and transform.rotation then
                    saved_rotation = transform.rotation
                end
                break
            end
        end
    end
    
    -- Despawn camera and mesh only (NOT UI root - keep file browser intact!)
    if self.panel_mesh then despawn(self.panel_mesh) self.panel_mesh = nil end
    if self.rtt_camera then despawn(self.rtt_camera) self.rtt_camera = nil end
    
    -- Update dimensions
    self.current_width = new_width
    self.current_height = new_height
    
    -- Create new RTT image at new size
    self.rtt_image = create_asset("bevy_image::image::Image", {
        width = new_width,
        height = new_height,
        format = "Bgra8UnormSrgb"
    })
    
    -- Spawn new RTT camera
    local camera = spawn({
        Camera2d = {},
        Camera = {
            order = -1,
            target = { Image = self.rtt_image }
        }
    })
    self.rtt_camera = camera:id()
    
    -- Create new mesh at new size
    local panel_width = new_width * PIXELS_TO_METERS
    local panel_height = new_height * PIXELS_TO_METERS
    
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
    
    -- Spawn new panel mesh with SAVED rotation
    local rotation = saved_rotation or { x = 0, y = 0, z = 0, w = 1 }
    local panel = spawn({
        Mesh3d = { _0 = mesh },
        ["MeshMaterial3d<StandardMaterial>"] = { _0 = material },
        Transform = {
            translation = self.position,
            rotation = rotation,
            scale = { x = 1, y = 1, z = 1 }
        },
        Visibility = "Visible",
        VrPanelMarker = { panel_id = self.panel_marker_id }
    })
    self.panel_mesh = panel:id()
    
    -- Update existing UI root's Node size AND UiTargetCamera to use new camera
    if self.ui_root then
        local roots = world:query({"VrSidebarRoot", "Node"}, nil)
        for _, entity in ipairs(roots) do
            entity:set({
                Node = {
                    width = { Px = new_width },
                    height = { Px = new_height },
                    flex_direction = "Row",
                    align_items = "Stretch",
                },
                UiTargetCamera = { entity = self.rtt_camera }
            })
            break
        end
    end
    
    -- Store rotation for future resizes
    self.rotation = rotation
end

--- Get surface info for VR pointer raycasting
function vr_sidebar:get_surfaces()
    if not self.is_visible then return {} end
    
    -- Use current dimensions (panel resizes dynamically)
    return {{
        entity = self.panel_mesh,
        position = self.position,
        texture_width = self.current_width,
        texture_height = self.current_height,
        panel_half_width = (self.current_width * PIXELS_TO_METERS) / 2,
        panel_half_height = (self.current_height * PIXELS_TO_METERS) / 2,
        rtt_image = self.rtt_image
    }}
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
    
    -- Apply looking_at on first frame after spawn
    if vr_sidebar.looking_at_target and vr_sidebar.looking_at_up then
        -- Query for panel by marker (more reliable than stored ID which may be stale)
        local panels = world:query({"VrPanelMarker", "Transform", "Visibility"}, nil)
        for _, entity in ipairs(panels) do
            local marker = entity:get("VrPanelMarker")
            if marker and marker.panel_id == vr_sidebar.panel_marker_id then
                -- Call looking_at on this entity
                world:call_component_method(
                    entity:id(),
                    "Transform",
                    "looking_at",
                    vr_sidebar.looking_at_target,
                    vr_sidebar.looking_at_up
                )
                
                -- Store rotation and reveal
                local transform = entity:get("Transform")
                if transform then
                    vr_sidebar.rotation = transform.rotation
                end
                entity:set({ Visibility = "Visible" })
                
                -- Clear flags
                vr_sidebar.looking_at_target = nil
                vr_sidebar.looking_at_up = nil
                break
            end
        end
        -- Don't check for resize on same frame as looking_at
        return
    end
    
    -- Only check for resize after rotation is stored
    if not vr_sidebar.rotation then return end
    
    -- Calculate total width by querying actual ComputedNode sizes
    local ICON_BAR_WIDTH = 50
    local total_width = ICON_BAR_WIDTH
    
    -- Query all SidebarPanel entities and sum their computed widths
    local panels = world:query({"SidebarPanel", "ComputedNode"}, nil)
    for _, panel_entity in ipairs(panels) do
        local computed = panel_entity:get("ComputedNode")
        if computed and computed.size then
            total_width = total_width + math.floor(computed.size.x)
        end
    end
    
    -- Trigger resize if width changed significantly
    if math.abs(total_width - vr_sidebar.current_width) > 2 then
        print(string.format("[VR_SIDEBAR] Size change detected: %d panels, new width: %d", 
            #panels, total_width))
        vr_sidebar:resize_panel(world, total_width, UI_HEIGHT)
    end
end)

-- VR Pointer system (runs in First for PointerInput processing)
register_system("First", function(world)
    if not vr_sidebar.is_visible then return end
    
    local surfaces = vr_sidebar:get_surfaces()
    VrPointer.update(world, surfaces)
end)

print("=== VR Sidebar script loaded ===")
print("Press X button on left controller to open/close sidebar")
print("Point right controller at panel and pull trigger to click")
