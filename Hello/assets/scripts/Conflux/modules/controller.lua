-- Conflux Controller Module
-- Game-specific player controller configuration
-- Imports camera module directly for camera-relative movement
--
-- Hot-reload friendly: require_async callbacks recreate systems when modules update
--
-- ControllerModule Interface:
--   M.create_system(callbacks) → function(world)

--------------------------------------------------------------------------------
-- Configuration (Conflux-specific settings)
--------------------------------------------------------------------------------

local CONTROLLER_CONFIG = {
    mode = "velocity",              -- "velocity" or "physics"
    walk_speed = 10.0,
    run_speed = 20.0,
    acceleration = 50.0,
    deceleration = 40.0,
    gravity = 15,
    jump_velocity = 5.0,
    
    -- Rotation
    rotation_mode = "face_movement", -- "none", "face_movement", "face_camera"
    rotation_speed = 15.0,
    
    -- Client prediction
    snap_threshold = 0.5,
    lerp_speed = 10.0,
    
    -- Network
    network_send_rate = 20,
}

--------------------------------------------------------------------------------
-- Module
--------------------------------------------------------------------------------

local M = {}

--- Create client-side update system
--- Hot-reload friendly: modules are loaded via require_async, and when they
--- update, the internal system is recreated automatically.
--- @param callbacks table|nil { on_player_found = function(world, entity_id) }
--- @return function System function to call each frame
function M.create_system(callbacks)
    -- State table that require_async callbacks can write to
    local state = {
        camera_module = nil,
        player_controller = nil,
        internal_system = nil,
    }
    
    -- Track if modules have been loaded (to detect hot-reload vs first load)
    local modules_loaded = {
        camera = false,
        player_controller = false,
    }
    
    -- Return the system function (called every frame)
    return function(world)
        -- Load camera module (callback fires on load AND hot-reload)
        require_async("scripts/Conflux/modules/camera.lua", function(cam_module)
            if not cam_module then
                print("[CONFLUX_CONTROLLER] ERROR: Failed to load camera module")
                return
            end
            
            local camera_changed = (state.camera_module ~= cam_module)
            state.camera_module = cam_module
            
            -- Load PlayerController (nested so we have camera first)
            require_async("modules/player_controller.lua", function(ctrl_module)
                if not ctrl_module then
                    print("[CONFLUX_CONTROLLER] ERROR: Failed to load PlayerController")
                    return
                end
                
                local controller_changed = (state.player_controller ~= ctrl_module)
                
                -- Apply Conflux configuration
                for key, value in pairs(CONTROLLER_CONFIG) do
                    if ctrl_module.config[key] ~= nil then
                        ctrl_module.config[key] = value
                    end
                end
                
                state.player_controller = ctrl_module
                
                -- Only recreate system if this is first load OR if modules changed
                local is_first_load = not modules_loaded.camera or not modules_loaded.player_controller
                local needs_recreate = is_first_load or camera_changed or controller_changed
                
                if needs_recreate then
                    state.internal_system = ctrl_module.create_client_system({
                        get_camera_entity = function(w)
                            return state.camera_module and state.camera_module.get_camera_entity()
                        end,
                        on_player_found = callbacks and callbacks.on_player_found
                    })
                    
                    modules_loaded.camera = true
                    modules_loaded.player_controller = true
                    
                    if is_first_load then
                        print("[CONFLUX_CONTROLLER] Client system created")
                    else
                        print("[CONFLUX_CONTROLLER] Client system recreated (hot-reload)")
                    end
                end
            end)
        end)
        
        -- Run the internal system if it exists
        if state.internal_system then
            state.internal_system(world)
        end
    end
end

--- Get config for inspection
function M.get_config()
    return CONTROLLER_CONFIG
end

print("[CONFLUX_CONTROLLER] Module loaded")

return M
