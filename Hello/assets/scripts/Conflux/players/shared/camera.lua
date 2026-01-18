-- Player Camera Configuration
-- Wrapper that loads camera implementation with config
-- Hot-reload this file to update camera settings in real-time

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

local CAMERA_CONFIG = {
    -- Choose implementation: "third_person_simple" or "camera_controller"
    implementation = "camera_controller",
    
    -- Configuration for "third_person_simple"
    simple = {
        offset = { x = 0, y = 2, z = -6 },
        lerp_speed = 5.0,
        look_at_offset = { x = 0, y = 1.5, z = 0 }
    },
    
    -- Configuration for "camera_controller"
    controller = {
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
}

--------------------------------------------------------------------------------
-- Module
--------------------------------------------------------------------------------

local M = {}

-- State
local camera_module = nil
local initialized = false
local loading = false

function M.init()
    if initialized or loading then
        return
    end
    
    loading = true
    
    if CAMERA_CONFIG.implementation == "third_person_simple" then
        -- Load simple third person camera synchronously
        local third_person = require("modules/cameras/third_person.lua")
        third_person.init(CAMERA_CONFIG.simple)
        camera_module = third_person
        initialized = true
        loading = false
        print("[CAMERA] Third-person simple camera initialized")
        
    elseif CAMERA_CONFIG.implementation == "camera_controller" then
        -- Load CameraController asynchronously
        require_async("modules/camera_controller.lua", function(module)
            if not module then
                print("[CAMERA] ERROR: Failed to load CameraController")
                loading = false
                return
            end
            
            -- Apply configuration to CameraController
            for key, value in pairs(CAMERA_CONFIG.controller) do
                if module.config[key] ~= nil then
                    module.config[key] = value
                end
            end
            
            camera_module = module
            initialized = true
            loading = false
            print("[CAMERA] CameraController initialized with config")
        end)
    else
        print("[CAMERA] ERROR: Unknown implementation: " .. tostring(CAMERA_CONFIG.implementation))
        loading = false
    end
end

function M.update(world, camera_entity_id, target_entity_id, dt)
    -- Ensure initialization started
    if not loading and not initialized then
        M.init()
        return
    end
    
    -- Wait for module to load
    if not initialized or not camera_module then
        return
    end
    
    -- Route to appropriate implementation
    if CAMERA_CONFIG.implementation == "third_person_simple" then
        camera_module.update(world, camera_entity_id, target_entity_id, dt)
        
    elseif CAMERA_CONFIG.implementation == "camera_controller" then
        -- CameraController needs to be attached to the target
        -- Check if already attached
        if camera_module.target_entity_id ~= target_entity_id then
            camera_module.attach(target_entity_id)
            print("[CAMERA] CameraController attached to entity: " .. tostring(target_entity_id))
        end
        
        -- Update CameraController (it manages its own camera entity)
        camera_module.update(world, dt)
    end
end

return M
