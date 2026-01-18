-- Third Person Camera Movement
-- Positions camera behind and above player, looking at player
--
-- Movement Module Interface:
--   M.init(config)
--   M.calculate_position(target_pos, yaw, pitch, config) → { camera_pos, look_target }

local ThirdPersonCamera = {}

-- Default config
local config = {
    distance = 5.0,         -- Distance behind player
    height = 2.0,           -- Height above player
    look_at_height = 1.0,   -- Look at player's chest, not feet
    offset = { x = 0, y = 0, z = 0 },  -- Additional offset
}

--- Initialize with config
--- @param cfg table|nil { distance, height, look_at_height, offset }
function ThirdPersonCamera.init(cfg)
    if cfg then
        config.distance = cfg.distance or cfg.third_person_distance or config.distance
        config.height = cfg.height or cfg.third_person_height or config.height
        config.look_at_height = cfg.look_at_height or config.look_at_height
        config.offset = cfg.offset or cfg.third_person_offset or config.offset
    end
    print(string.format("[THIRD_PERSON] Initialized with distance=%.1f, height=%.1f", config.distance, config.height))
end

--- Get current config (for hot-reload inspection)
function ThirdPersonCamera.get_config()
    return config
end

--- Calculate camera position and look target
--- @param target_pos table { x, y, z } - Target entity position
--- @param yaw number - Horizontal rotation in radians
--- @param pitch number - Vertical rotation in radians
--- @param override_config table|nil - Optional config override
--- @return table { camera_pos = {x,y,z}, look_target = {x,y,z} }
function ThirdPersonCamera.calculate_position(target_pos, yaw, pitch, override_config)
    local cfg = override_config or config
    local dist = cfg.distance or config.distance
    local height = cfg.height or config.height
    local look_height = cfg.look_at_height or config.look_at_height
    local offset = cfg.offset or config.offset
    
    -- Calculate camera position based on yaw/pitch
    -- Camera orbits around player: yaw rotates horizontally, pitch adjusts vertical angle
    -- At yaw=0, camera is behind player (-Z direction)
    local camera_pos = {
        x = target_pos.x + math.sin(yaw) * dist * math.cos(pitch) + (offset.x or 0),
        y = target_pos.y + height + math.sin(pitch) * dist + (offset.y or 0),
        z = target_pos.z + math.cos(yaw) * dist * math.cos(pitch) + (offset.z or 0)  -- Negative: behind at yaw=0
    }
    
    -- Look at player (at chest height)
    local look_target = {
        x = target_pos.x,
        y = target_pos.y + look_height,
        z = target_pos.z
    }
    
    return {
        camera_pos = camera_pos,
        look_target = look_target
    }
end

--- Called when camera attaches to a target
--- Sets initial camera position behind the player
--- @param world userdata
--- @param camera_entity_id number
--- @param target_entity_id number
--- @return table|nil Optional {yaw=..., pitch=...} to set controller state
function ThirdPersonCamera.on_attach(world, camera_entity_id, target_entity_id)
    -- Get target position and rotation
    local target = world:get_entity(target_entity_id)
    if not target then return end
    
    local target_transform = target:get("Transform")
    if not target_transform then return end
    
    local target_pos = target_transform.translation
    
    -- Calculate initial yaw based on target orientation using Transform:forward()
    local current_yaw = 0
    
    -- Use Bevy's built-in forward calculation
    local forward = world:call_component_method(target_entity_id, "Transform", "forward")
    if forward then
        current_yaw = math.atan(forward[1].x, forward[1].z)
    end
    
    -- Calculate initial position
    local result = ThirdPersonCamera.calculate_position(target_pos, current_yaw, 0)
    
    -- Set camera position
    local camera = world:get_entity(camera_entity_id)
    if camera then
        camera:set({
            Transform = {
                translation = result.camera_pos
            }
        })
        
        -- Set camera to look at target
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
        
        print(string.format("[THIRD_PERSON] Camera positioned behind player (yaw=%.2f)", current_yaw))
    end
    
    -- Return new state for controller
    return { yaw = current_yaw, pitch = 0 }
end

print("[THIRD_PERSON] Module loaded")

return ThirdPersonCamera
