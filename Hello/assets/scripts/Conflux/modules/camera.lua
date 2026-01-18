-- Conflux Camera Module
-- Game-specific camera configuration
--
-- CameraModule Interface:
--   M.init()
--   M.attach(target_entity_id)
--   M.update(world, dt)
--   M.get_camera_entity() → entity_id
--   M.get_yaw() → number

--------------------------------------------------------------------------------
-- Configuration (Conflux-specific settings)
--------------------------------------------------------------------------------

local CAMERA_CONFIG = {
    mode = "third_person",          -- "first_person" or "third_person"
    
    -- Movement module paths (can swap for custom modules)
    movement_modules = {
        first_person = "modules/cameras/first_person.lua",
        third_person = "modules/cameras/third_person.lua",
    },
    
    -- Third person settings
    third_person_distance = 5.0,
    third_person_height = 2.0,
    third_person_offset = {x = 0, y = 0, z = 0},
    
    -- First person settings
    first_person_height = 1.7,
    
    -- Input settings
    sensitivity = 1.0,
    invert_y = false,
    
    -- Smoothing
    smoothing_factor = 100.0,
}

--------------------------------------------------------------------------------
-- Module
--------------------------------------------------------------------------------

local M = {}

-- State
local camera_controller = nil
local initialized = false
local loading = false

function M.init()
    if initialized or loading then
        return
    end
    
    loading = true
    
    -- Load CameraController via require_async
    require_async("modules/camera_controller.lua", function(module)
        if not module then
            print("[CONFLUX_CAMERA] ERROR: Failed to load CameraController")
            loading = false
            return
        end
        
        -- Apply Conflux configuration
        module.init(CAMERA_CONFIG)
        
        camera_controller = module
        initialized = true
        loading = false
        print("[CONFLUX_CAMERA] CameraController loaded and configured")
    end)
end

function M.attach(target_entity_id)
    if not initialized or not camera_controller then
        -- Queue attachment for when module loads
        print("[CONFLUX_CAMERA] Queuing attachment, module not yet loaded")
        return
    end
    
    camera_controller.attach(target_entity_id)
end

function M.update(world, dt)
    -- Ensure initialization started
    if not loading and not initialized then
        M.init()
        return
    end
    
    -- Wait for module to load
    if not initialized or not camera_controller then
        return
    end
    
    camera_controller.update(world, dt)
end

function M.get_camera_entity()
    if camera_controller then
        return camera_controller.get_camera_entity()
    end
    return nil
end

function M.get_yaw()
    if camera_controller then
        return camera_controller.get_yaw()
    end
    return 0
end

function M.get_pitch()
    if camera_controller then
        return camera_controller.get_pitch()
    end
    return 0
end

--- Check if camera is ready
function M.is_ready()
    return initialized and camera_controller ~= nil
end

--- Get config for hot-reload inspection
function M.get_config()
    return CAMERA_CONFIG
end

print("[CONFLUX_CAMERA] Module loaded")

return M
