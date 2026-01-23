local Movement = {}

--- Apply movement and rotation to a transform based on input
--- @param world userdata The world object (for static methods)
--- @param transform table Bevy Transform component { translation, rotation, scale }
--- @param input table PlayerInput component { move_x, move_z, yaw, jump, sprint }
--- @param speed number Base movement speed
--- @param dt number Delta time
--- @param smooth_rotation boolean Whether to smoothly rotate or snap
--- @param config table|nil Optional config { rotation_mode = "face_camera" | "face_movement" }
--- @return table New translation { x, y, z }, table New rotation { x, y, z, w }
function Movement.apply(world, transform, input, speed, dt, smooth_rotation, config)
    config = config or { rotation_mode = "face_movement" }
    local rotation_mode = config.rotation_mode or "face_movement"
    local move_x = input.move_x or 0
    local move_z = input.move_z or 0
    local yaw = input.yaw or 0
    local sprint = input.sprint or false
    
    local multiplier = sprint and 2.0 or 1.0
    
    -- 1. Calculate Rotation
    local current_rot = transform.rotation
    
    local is_moving = (move_x ~= 0 or move_z ~= 0)
    local target_rot
    
    if rotation_mode == "face_movement" then
        if is_moving then
            -- atan2(x, z) gives angle relative to "forward" (+Z for the math, which we then map)
            local movement_yaw = math.atan2(move_x, move_z)
            -- target_yaw = camera_yaw + movement_rel_yaw + model_offset
            local target_yaw = yaw - movement_yaw + math.pi
            target_rot = world:call_static_method("Quat", "from_rotation_y", target_yaw)
        else
            -- If not moving, maintain current rotation
            target_rot = current_rot
        end
    else
        -- face_camera mode: face where the camera is looking (yaw + pi)
        local target_yaw = yaw + math.pi
        target_rot = world:call_static_method("Quat", "from_rotation_y", target_yaw)
    end
    
    local new_rot
    if smooth_rotation then
        -- Smoothly slerp towards target rotation
        local rotation_speed = 10.0
        new_rot = world:call_static_method("Quat", "slerp",
            current_rot,
            target_rot,
            math.min(1.0, rotation_speed * dt)
        )
    else
        -- Snap immediately to reduce bandwidth on server
        new_rot = target_rot
    end
    
    -- 2. Calculate Position
    if move_x == 0 and move_z == 0 then
        return transform.translation, new_rot
    end
    
    -- Direction vector relative to yaw
    local sin_yaw = math.sin(yaw)
    local cos_yaw = math.cos(yaw)
    
    -- Forward is -Z, Right is +X
    local dx = (move_x * cos_yaw - move_z * sin_yaw) * speed * multiplier * dt
    local dz = (-move_x * sin_yaw - move_z * cos_yaw) * speed * multiplier * dt
    
    local new_pos = {
        x = transform.translation.x + dx,
        y = transform.translation.y,
        z = transform.translation.z + dz,
    }
    
    return new_pos, new_rot
end

return Movement
