-- First Person Camera Movement
-- Positions camera at player's head, looking in yaw/pitch direction
--
-- Movement Module Interface:
--   M.init(config)
--   M.calculate_position(target_pos, yaw, pitch, config) → { camera_pos, look_target }

local FirstPersonCamera = {}

-- Default config
local config = {
    height = 1.7,           -- Camera height above target position
    look_distance = 10.0,   -- Distance to look_target point (for direction calculation)
}

--- Initialize with config
--- @param cfg table|nil { height, look_distance }
function FirstPersonCamera.init(cfg)
    if cfg then
        config.height = cfg.height or config.height
        config.look_distance = cfg.look_distance or config.look_distance
    end
    print(string.format("[FIRST_PERSON] Initialized with height=%.1f", config.height))
end

--- Get current config (for hot-reload inspection)
function FirstPersonCamera.get_config()
    return config
end

--- Calculate camera position and look target
--- @param target_pos table { x, y, z } - Target entity position
--- @param yaw number - Horizontal rotation in radians
--- @param pitch number - Vertical rotation in radians
--- @param override_config table|nil - Optional config override
--- @return table { camera_pos = {x,y,z}, look_target = {x,y,z} }
function FirstPersonCamera.calculate_position(target_pos, yaw, pitch, override_config)
    local cfg = override_config or config
    local height = cfg.height or config.height
    local look_dist = cfg.look_distance or config.look_distance
    
    -- Camera at player's head
    local camera_pos = {
        x = target_pos.x,
        y = target_pos.y + height,
        z = target_pos.z
    }
    
    -- Look in the direction based on yaw/pitch
    -- yaw = 0 looks toward -Z, yaw = pi/2 looks toward -X
    local look_target = {
        x = camera_pos.x - math.sin(yaw) * look_dist * math.cos(pitch),
        y = camera_pos.y - math.sin(pitch) * look_dist,
        z = camera_pos.z - math.cos(yaw) * look_dist * math.cos(pitch)
    }
    
    return {
        camera_pos = camera_pos,
        look_target = look_target
    }
end

--- Called when camera attaches to a target
--- Sets initial camera position at player's head
--- @param world userdata
--- @param camera_entity_id number
--- @param target_entity_id number
--- @return table|nil Optional {yaw=..., pitch=...} to set controller state
function FirstPersonCamera.on_attach(world, camera_entity_id, target_entity_id)
    -- Get target position
    local target = world:get_entity(target_entity_id)
    if not target then return end
    
    local target_transform = target:get("Transform")
    if not target_transform then return end
    
    local target_pos = target_transform.translation
    
    -- Calculate initial camera position (at player's head, looking forward)
    local result = FirstPersonCamera.calculate_position(target_pos, 0, 0)
    
    -- Set camera position
    local camera = world:get_entity(camera_entity_id)
    if camera then
        camera:set({
            Transform = {
                translation = result.camera_pos
            }
        })
        
        -- Set camera to look in direction
        local look_dir = {
            x = result.look_target.x - result.camera_pos.x,
            y = result.look_target.y - result.camera_pos.y,
            z = result.look_target.z - result.camera_pos.z
        }
        
        world:call_component_method(
            camera_entity_id,
            "Transform",
            "looking_to",
            look_dir,
            {x = 0, y = 1, z = 0}
        )
        
        print("[FIRST_PERSON] Camera positioned at player head")
    end
    
    return { yaw = 0, pitch = 0 }
end

print("[FIRST_PERSON] Module loaded")

return FirstPersonCamera
