-- Strafe Style Module
-- Player faces movement direction normally, but when moving backward
-- they face toward the camera instead (walking backward toward camera)
-- Common in action RPGs and third-person adventure games
--
-- Movement Module Interface:
--   M.init(config)
--   M.get_target_rotation(input, camera_yaw, current_rotation, world) → target_quat or nil

local StrafeStyle = {}

-- Default config
local config = {
    rotation_speed = 15.0,  -- Radians per second for rotation interpolation
    backward_threshold = -0.3,  -- Forward input below this triggers backward-facing mode
    strafe_threshold = 0.5,  -- When |right| > this with small forward, also face camera
    -- Physics settings
    walk_speed = 10.0,
    run_speed = 20.0,
    gravity = 15.0,
    jump_velocity = 5.0,
}

-- Export default config for external use
StrafeStyle.default_config = config

--- Initialize with config
--- @param cfg table|nil { rotation_speed, backward_threshold, strafe_threshold }
function StrafeStyle.init(cfg)
    if cfg then
        config.rotation_speed = cfg.rotation_speed or config.rotation_speed
        config.backward_threshold = cfg.backward_threshold or config.backward_threshold
        config.strafe_threshold = cfg.strafe_threshold or config.strafe_threshold
    end
    print(string.format("[STRAFE_STYLE] Initialized with rotation_speed=%.1f, backward_threshold=%.1f", 
        config.rotation_speed, config.backward_threshold))
end

--- Get current config
function StrafeStyle.get_config()
    return config
end

--- Calculate target rotation based on movement with backward special case
--- @param input table { forward, right } camera-relative input (-1 to 1)
--- @param camera_yaw number Camera's horizontal rotation in radians
--- @param current_rotation table Current player quaternion {x,y,z,w}
--- @param world userdata World for static method calls
--- @return table|nil target quaternion, or nil if not moving
function StrafeStyle.get_target_rotation(input, camera_yaw, current_rotation, world)
    local forward = input.forward or 0
    local right = input.right or 0
    
    -- Not moving? Keep current rotation
    local len = math.sqrt(forward * forward + right * right)
    if len < 0.01 then
        return nil
    end
    
    -- Check for backward/strafe mode:
    -- - Moving backward (forward < threshold with small strafe)
    -- - Pure strafing (high |right| with low |forward|)
    local is_backward = forward < config.backward_threshold and math.abs(right) < config.strafe_threshold
    local is_pure_strafe = math.abs(right) > config.strafe_threshold and math.abs(forward) < 0.3
    
    local target_yaw
    
    if is_backward then
        -- Walking backward: face toward camera (opposite of camera direction)
        -- Camera at yaw=0 looks at -Z, so player facing camera means yaw = PI
        target_yaw = camera_yaw + math.pi
    elseif is_pure_strafe then
        -- Pure strafing: face camera direction (like face_camera mode)
        target_yaw = camera_yaw
    else
        -- Normal forward/diagonal movement: face movement direction
        local input_angle = math.atan2(right, forward)
        target_yaw = camera_yaw + input_angle
    end
    
    -- Create rotation quaternion facing that direction
    local target_rotation = world:call_static_method("Quat", "from_rotation_y", target_yaw)
    
    return target_rotation
end

--- Interpolate rotation toward target
--- @param current_rot table current quaternion
--- @param target_rot table target quaternion
--- @param dt number delta time
--- @param world userdata World for static method calls
--- @return table interpolated quaternion
function StrafeStyle.interpolate_rotation(current_rot, target_rot, dt, world)
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
function StrafeStyle.get_world_movement(input, camera_yaw)
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
function StrafeStyle.calculate_physics(current_pos, current_velocity, move_dir, input, dt, is_grounded, cfg)
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

print("[STRAFE_STYLE] Module loaded")

return StrafeStyle
