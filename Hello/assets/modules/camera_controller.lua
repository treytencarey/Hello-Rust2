-- Camera Controller Module
-- First/third person camera with VR detection
--
-- Usage:
--   local CameraController = require("modules/camera_controller.lua")
--   CameraController.attach(player_entity_id)
--   register_system("Update", function(world)
--       CameraController.update(world, world:delta_time())
--   end)

local NetRole = require("modules/net_role.lua")

local CameraController = {}

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

CameraController.config = {
    mode = "third_person",          -- "first_person" or "third_person"
    
    -- Third person settings
    third_person_distance = 5.0,
    third_person_height = 2.0,
    third_person_offset = {x = 0, y = 0, z = 0},
    
    -- First person settings
    first_person_height = 1.7,
    
    -- Input settings
    sensitivity = 1.0,
    invert_y = false,
    
    -- Collision for third person
    collision_enabled = true,
    collision_offset = 0.3,
}

--------------------------------------------------------------------------------
-- State
--------------------------------------------------------------------------------

local camera_entity_id = nil
local target_entity_id = nil
local yaw = 0      -- Horizontal rotation (radians)
local pitch = 0    -- Vertical rotation (radians)
local is_vr = false

-- Input mode: "game" (mouse locked, controls camera) or "ui" (mouse free, can click UI)
local input_mode = "game"
local cursor_configured = false

-- Smoothing state
local current_camera_pos = nil  -- Smoothed camera position
local smoothing_factor = 100.0   -- Higher = snappier (10 = quite responsive), lower = smoother (e.g. for mobile)

--------------------------------------------------------------------------------
-- VR Detection
--------------------------------------------------------------------------------

local function detect_vr(world)
    -- Check if VR mode is active
    local ok, result = pcall(function()
        return world:query_resource("XrViews")
    end)
    -- return ok and result ~= nil
    return false -- TODO: implement VR detection
end

--------------------------------------------------------------------------------
-- Cursor Control
--------------------------------------------------------------------------------

--- Set cursor visibility and grab mode based on input mode
--- @param world userdata
local function apply_cursor_settings(world)
    -- In Bevy 0.17, CursorOptions is a separate component on the window entity
    local windows = world:query({"Window", "CursorOptions"}, nil)
    if #windows == 0 then
        -- Fallback to just Window if CursorOptions not found yet
        windows = world:query({"Window"}, nil)
        if #windows == 0 then return end
    end
    
    local window = windows[1]
    
    if input_mode == "game" then
        -- Game mode: hide cursor and lock it
        -- Enum variants need table format: { VariantName = true } for unit variants
        window:set({
            CursorOptions = {
                visible = false,
                grab_mode = { Locked = true }
            }
        })
        print("[CAMERA_CONTROLLER] Cursor locked for game mode")
    else
        -- UI mode: show cursor and free it
        window:set({
            CursorOptions = {
                visible = true,
                grab_mode = { None = true }
            }
        })
        print("[CAMERA_CONTROLLER] Cursor unlocked for UI mode")
    end
end

--- Set input mode
--- @param mode string "game" or "ui"
function CameraController.set_input_mode(mode)
    if mode == "game" or mode == "ui" then
        input_mode = mode
        cursor_configured = false  -- Re-apply on next update
        print(string.format("[CAMERA_CONTROLLER] Input mode set to: %s", mode))
    end
end

--- Get current input mode
function CameraController.get_input_mode()
    return input_mode
end

--- Toggle input mode between game and UI
function CameraController.toggle_input_mode()
    if input_mode == "game" then
        CameraController.set_input_mode("ui")
    else
        CameraController.set_input_mode("game")
    end
end

--------------------------------------------------------------------------------
-- Camera Creation & Attachment
--------------------------------------------------------------------------------

--- Create the camera entity (client-only)
--- @return number|nil camera_entity_id
function CameraController.create_camera()
    -- Only create camera on client
    if not (NetRole.is_client() or NetRole.is_offline()) then
        return nil
    end
    
    local spawn_data = {
        Camera3d = {},
        Transform = {
            translation = {x = 0, y = 5, z = 10},
            rotation = {x = 0, y = 0, z = 0, w = 1},
            scale = {x = 1, y = 1, z = 1}
        }
    }
    
    camera_entity_id = spawn(spawn_data):id()
    print(string.format("[CAMERA_CONTROLLER] Created camera entity %s", tostring(camera_entity_id)))
    return camera_entity_id
end

--- Attach camera to follow a target entity
--- @param player_entity_id number
function CameraController.attach(player_entity_id)
    target_entity_id = player_entity_id
    
    -- Create camera if not exists
    if not camera_entity_id then
        CameraController.create_camera()
    end
    
    print(string.format("[CAMERA_CONTROLLER] Attached to entity %d", player_entity_id))
end

--- Detach camera from target
function CameraController.detach()
    target_entity_id = nil
    print("[CAMERA_CONTROLLER] Detached from target")
end

--------------------------------------------------------------------------------
-- Mode Switching
--------------------------------------------------------------------------------

--- Set camera mode
--- @param mode string "first_person" or "third_person"
function CameraController.set_mode(mode)
    if mode == "first_person" or mode == "third_person" then
        CameraController.config.mode = mode
        print(string.format("[CAMERA_CONTROLLER] Mode set to %s", mode))
    end
end

--- Toggle between first and third person
function CameraController.toggle_mode()
    if CameraController.config.mode == "first_person" then
        CameraController.set_mode("third_person")
    else
        CameraController.set_mode("first_person")
    end
end

--------------------------------------------------------------------------------
-- Input Processing
--------------------------------------------------------------------------------

--- Process mouse input for camera rotation (call this in update)
--- @param world userdata
function CameraController.process_mouse_input(world)
    -- Only process mouse in game mode
    if input_mode ~= "game" then return end
    
    -- Read mouse motion events
    -- In Bevy, MouseMotion has: { delta: Vec2 }
    -- Vec2 is serialized as table with x,y OR as array [1]=x, [2]=y depending on reflection
    local motion_events = world:read_events("MouseMotion")
    for i, event in ipairs(motion_events) do
        -- Debug: print the entire event structure
        if i == 1 then
            local function dump(t, indent)
                indent = indent or ""
                if type(t) ~= "table" then return tostring(t) end
                local s = "{\n"
                for k, v in pairs(t) do
                    s = s .. indent .. "  " .. tostring(k) .. " = " .. dump(v, indent .. "  ") .. ",\n"
                end
                return s .. indent .. "}"
            end
        end
        
        local dx, dy = 0, 0
        
        if event.delta then
            -- Try table-style access
            if event.delta.x then
                dx = event.delta.x
                dy = event.delta.y or 0
            -- Try array-style access (Vec2 as tuple)
            elseif event.delta[1] then
                dx = event.delta[1]
                dy = event.delta[2] or 0
            end
        end
        
        if dx ~= 0 or dy ~= 0 then
            CameraController.rotate(-dx, -dy)
        end
    end
end

--- Process keyboard input for mode toggles (call this in update)
--- @param world userdata
function CameraController.process_keyboard_input(world)
    -- Check for Escape key to toggle input mode
    local key_events = world:read_events("KeyboardInput")
    for _, event in ipairs(key_events) do
        if event.key_code and event.key_code.Escape then
            if event.state and event.state.Pressed then
                CameraController.toggle_input_mode()
            end
        end
    end
end

--------------------------------------------------------------------------------
-- Camera Update
--------------------------------------------------------------------------------

--- Update camera position and rotation
--- @param world userdata
--- @param dt number delta time
function CameraController.update(world, dt)
    -- Apply cursor settings if needed
    if not cursor_configured then
        apply_cursor_settings(world)
        cursor_configured = true
    end
    
    -- Process input
    CameraController.process_keyboard_input(world)
    CameraController.process_mouse_input(world)
    
    -- Skip camera follow if no camera or target, or if in VR
    if not camera_entity_id or not target_entity_id then return end
    
    -- Check for VR mode
    is_vr = detect_vr(world)
    if is_vr then
        -- In VR, default to first person and let VR system handle camera
        CameraController.config.mode = "first_person"
        return
    end
    
    -- Get target position
    local target = world:get_entity(target_entity_id)
    if not target then return end  
    
    local target_transform = target:get("Transform")
    if not target_transform then return end
    
    local target_pos = target_transform.translation
    
    -- Get camera entity
    local camera = world:get_entity(camera_entity_id)
    if not camera then return end
    
    -- Calculate DESIRED camera position based on mode
    local desired_pos
    local look_target
    
    if CameraController.config.mode == "first_person" then
        -- First person: camera at player head
        desired_pos = {
            x = target_pos.x,
            y = target_pos.y + CameraController.config.first_person_height,
            z = target_pos.z
        }
        -- Look in the direction the player is facing
        look_target = {
            x = desired_pos.x - math.sin(yaw),
            y = desired_pos.y + math.sin(pitch),
            z = desired_pos.z - math.cos(yaw)
        }
    else
        -- Third person: camera behind and above player
        local config = CameraController.config
        local dist = config.third_person_distance
        local height = config.third_person_height
        
        -- Calculate camera position based on yaw/pitch
        desired_pos = {
            x = target_pos.x + math.sin(yaw) * dist * math.cos(pitch),
            y = target_pos.y + height + math.sin(pitch) * dist,
            z = target_pos.z + math.cos(yaw) * dist * math.cos(pitch)
        }
        
        -- Look at player
        look_target = {
            x = target_pos.x,
            y = target_pos.y + 1.0,  -- Look at chest height
            z = target_pos.z
        }
    end
    
    -- Initialize smoothed position if not set
    if not current_camera_pos then
        current_camera_pos = desired_pos
    end
    
    -- Smoothly interpolate camera position (lerp)
    local t = math.min(1.0, smoothing_factor * dt)
    current_camera_pos = {
        x = current_camera_pos.x + (desired_pos.x - current_camera_pos.x),
        y = current_camera_pos.y + (desired_pos.y - current_camera_pos.y),
        z = current_camera_pos.z + (desired_pos.z - current_camera_pos.z)
    }
    
    -- Compute look direction from current_camera_pos to look_target
    -- Using looking_to(direction) instead of looking_at(target) avoids
    -- dependency on the Transform's stale translation
    local look_dir = {
        x = look_target.x - current_camera_pos.x,
        y = look_target.y - current_camera_pos.y,
        z = look_target.z - current_camera_pos.z
    }
    
    -- Update camera transform with smoothed position
    camera:set({
        Transform = {
            translation = current_camera_pos
        }
    })

    -- Call looking_to with direction (not target position)
    world:call_component_method(
        camera:id(),
        "Transform",
        "looking_to",
        look_dir,
        {x = 0, y = 1, z = 0}  -- Up vector (Y-up)
    )
end

--------------------------------------------------------------------------------
-- Input Handling (manual rotation)
--------------------------------------------------------------------------------

--- Process mouse/controller input for camera rotation
--- @param delta_x number horizontal mouse movement
--- @param delta_y number vertical mouse movement
function CameraController.rotate(delta_x, delta_y)
    local sensitivity = CameraController.config.sensitivity * 0.003
    
    yaw = yaw + delta_x * sensitivity
    
    local y_mult = CameraController.config.invert_y and 1 or -1
    pitch = pitch + delta_y * sensitivity * y_mult
    
    -- Clamp pitch to prevent flipping
    pitch = math.max(-math.pi * 0.45, math.min(math.pi * 0.45, pitch))
end

--- Get current yaw (for player rotation sync)
function CameraController.get_yaw()
    return yaw
end

--- Get current pitch
function CameraController.get_pitch()
    return pitch
end

--------------------------------------------------------------------------------
-- Getters
--------------------------------------------------------------------------------

function CameraController.get_camera_entity()
    return camera_entity_id
end

function CameraController.get_target_entity()
    return target_entity_id
end

function CameraController.is_vr_mode()
    return is_vr
end

print("[CAMERA_CONTROLLER] Module loaded")

return CameraController
