-- Face Movement Module
-- Player rotates to face the direction they're moving
-- Standard behavior for most third-person games
--
-- Movement Module Interface:
--   M.init(config)
--   M.get_target_rotation(input, camera_yaw, current_rotation, world) → target_quat or nil

local FaceMovement = {}

-- Default config (includes physics settings)
local config = {
    rotation_speed = 15.0,  -- Radians per second for rotation interpolation
    -- Physics settings
    walk_speed = 10.0,
    run_speed = 20.0,
    gravity = 15.0,
    jump_velocity = 5.0,
}

-- Export default config for external use
FaceMovement.default_config = config

--- Initialize with config
--- @param cfg table|nil { rotation_speed }
function FaceMovement.init(cfg)
    if cfg then
        config.rotation_speed = cfg.rotation_speed or config.rotation_speed
    end
    print(string.format("[FACE_MOVEMENT] Initialized with rotation_speed=%.1f", config.rotation_speed))
end

--- Get current config
function FaceMovement.get_config()
    return config
end

--- Calculate target rotation based on movement direction
--- @param input table { forward, right } camera-relative input (-1 to 1)
--- @param camera_yaw number Camera's horizontal rotation in radians
--- @param current_rotation table Current player quaternion {x,y,z,w}
--- @param world userdata World for static method calls
--- @return table|nil target quaternion, or nil if not moving
function FaceMovement.get_target_rotation(input, camera_yaw, current_rotation, world)
    local forward = input.forward or 0
    local right = input.right or 0
    
    -- Not moving? Keep current rotation
    local len = math.sqrt(forward * forward + right * right)
    if len < 0.01 then
        return nil
    end
    
    -- Calculate world-space movement angle from camera-relative input
    -- Input: forward = W key, right = D key (camera-relative)
    -- 
    -- camera_yaw = 0 means camera looks at -Z
    -- forward = 1 means move in camera's forward direction (-Z rotated by camera_yaw)
    -- right = 1 means move in camera's right direction (+X rotated by camera_yaw)
    --
    -- Movement angle relative to camera:
    local input_angle = math.atan2(right, forward)
    
    -- World-space angle = camera_yaw + input_angle
    local world_angle = camera_yaw + input_angle
    
    -- Create rotation quaternion facing that direction
    local target_rotation = world:call_static_method("Quat", "from_rotation_y", world_angle)
    
    return target_rotation
end

--- Interpolate rotation toward target
--- @param current_rot table current quaternion
--- @param target_rot table target quaternion
--- @param dt number delta time
--- @param world userdata World for static method calls
--- @return table interpolated quaternion
function FaceMovement.interpolate_rotation(current_rot, target_rot, dt, world)
    local speed = config.rotation_speed
    local t = math.min(1.0, speed * dt)
    
    return world:call_static_method("Quat", "slerp",
        current_rot,
        target_rot,
        t
    )
end

--------------------------------------------------------------------------------
-- World-Space Movement
--------------------------------------------------------------------------------

--- Transform camera-relative input to world-space direction
--- @param input table { forward, right } camera-relative input (-1 to 1)
--- @param camera_yaw number Camera's horizontal rotation in radians
--- @return table { x, z } world-space direction (unnormalized)
function FaceMovement.get_world_movement(input, camera_yaw)
    local forward = input.forward or 0
    local right = input.right or 0
    
    if forward == 0 and right == 0 then
        return { x = 0, z = 0 }
    end
    
    local sin_yaw = math.sin(camera_yaw)
    local cos_yaw = math.cos(camera_yaw)
    
    -- Transform by camera yaw
    -- Forward input (-1 = forward key) should move in camera's forward direction
    -- Right input (+1 = right key) should move in camera's right direction
    return {
        x = -forward * sin_yaw + right * cos_yaw,
        z = -forward * cos_yaw - right * sin_yaw
    }
end

--------------------------------------------------------------------------------
-- Physics Calculation
--------------------------------------------------------------------------------

--- Calculate physics (stateless - takes current state, returns new state)
--- @param current_pos table { x, y, z } current position
--- @param current_velocity table { x, y, z } current velocity
--- @param move_dir table { x, z } world-space movement direction
--- @param input table { jump, sprint } input flags
--- @param dt number delta time
--- @param is_grounded boolean whether player is on ground
--- @param cfg table|nil optional config override (uses default_config if nil)
--- @return table new_pos, table new_velocity
function FaceMovement.calculate_physics(current_pos, current_velocity, move_dir, input, dt, is_grounded, cfg)
    local c = cfg or config
    
    local speed = input.sprint and c.run_speed or c.walk_speed
    
    -- Horizontal velocity (instant direction change for responsive controls)
    local new_velocity = {
        x = move_dir.x * speed,
        y = current_velocity.y,
        z = move_dir.z * speed
    }
    
    -- Jumping and gravity
    if is_grounded then
        if input.jump then
            new_velocity.y = c.jump_velocity
        else
            new_velocity.y = 0
        end
    else
        new_velocity.y = new_velocity.y - c.gravity * dt
    end
    
    -- Apply velocity to position
    local new_pos = {
        x = current_pos.x + new_velocity.x * dt,
        y = current_pos.y + new_velocity.y * dt,
        z = current_pos.z + new_velocity.z * dt
    }
    
    -- Ground clamp (simple floor at y=0)
    if new_pos.y < 0 then
        new_pos.y = 0
        new_velocity.y = 0
    end
    
    return new_pos, new_velocity
end

print("[FACE_MOVEMENT] Module loaded")

return FaceMovement
