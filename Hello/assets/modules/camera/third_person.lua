-- Camera Third Person Math Module
-- Pure math utilities for third-person camera positioning

local ThirdPerson = {}

--- Calculate camera position behind and above target
--- @param target_pos table { x, y, z }
--- @param yaw number Horizontal angle (radians)
--- @param pitch number Vertical angle (radians)
--- @param distance number Distance from target
--- @return table position { x, y, z }
--- @return table rotation { x, y, z, w } quaternion
--- Calculate camera position and rotation with a local framing offset
--- @param target_pos table { x, y, z } Orbit pivot
--- @param yaw number Horizontal angle (radians)
--- @param pitch number Vertical angle (radians)
--- @param distance number Orbital distance
--- @param offset table|nil { x, y, z } Framing offset (x: right, y: up, z: forward)
--- @return table position { x, y, z }
--- @return table rotation { x, y, z, w } quaternion
function ThirdPerson.calculate(target_pos, yaw, pitch, distance, offset)
    offset = offset or { x = 0, y = 0, z = 0 }
    
    -- Local orientation components
    local cos_y = math.cos(yaw)
    local sin_y = math.sin(yaw)
    
    -- 1. Calculate base orbital position (purely behind the pivot)
    local bx = target_pos.x + distance * math.cos(pitch) * math.sin(yaw)
    local by = target_pos.y + distance * math.sin(pitch)
    local bz = target_pos.z + distance * math.cos(pitch) * math.cos(yaw)
    
    -- 2. Rotate framing offset by camera yaw to world space
    -- This keeps the character framed consistently (e.g. over-the-shoulder)
    local ox = offset.x * cos_y + (offset.z or 0) * sin_y
    local oy = offset.y
    local oz = -offset.x * sin_y + (offset.z or 0) * cos_y
    
    local position = {
        x = bx + ox,
        y = by + oy,
        z = bz + oz
    }
    
    -- 3. Look at the offset pivot
    local look_target = {
        x = target_pos.x + ox,
        y = target_pos.y + oy,
        z = target_pos.z + oz
    }
    
    local rotation = ThirdPerson.look_at(position, look_target)
    
    return position, rotation
end

--- Get initial camera state for this module
--- @param world userdata
--- @param target_id number
--- @return table { yaw, pitch }
function ThirdPerson.get_initial_state(world, target_id)
    local target = world:get_entity(target_id)
    if not target then return { yaw = 0, pitch = 0 } end
    
    local current_yaw = 0
    
    -- Align with target's forward vector
    local forward = world:call_component_method(target_id, "Transform", "forward")
    if forward then
        -- Bevy uses -Z forward. atan2(-dx, -dz) gives 0 for (0, 0, -1)
        current_yaw = math.atan2(-forward[1].x, forward[1].z)
    end
    
    return {
        yaw = current_yaw,
        pitch = 0
    }
end

--- Calculate quaternion to look from eye to target
--- @param eye table { x, y, z }
--- @param target table { x, y, z }
--- @return table { x, y, z, w } quaternion
function ThirdPerson.look_at(eye, target)
    -- Direction vector (from eye to target)
    local dx = target.x - eye.x
    local dy = target.y - eye.y
    local dz = target.z - eye.z
    
    -- Normalize
    local len = math.sqrt(dx*dx + dy*dy + dz*dz)
    if len < 0.0001 then
        return { x = 0, y = 0, z = 0, w = 1 }
    end
    
    dx, dy, dz = dx/len, dy/len, dz/len
    
    -- Calculate yaw and pitch from direction
    -- In Bevy, forward is -Z. We want yaw=0 when looking towards -Z.
    -- math.atan2(dx, dz) gives pi for (0, 0, -1). 
    -- math.atan2(-dx, -dz) gives 0 for (0, 0, -1).
    local yaw = math.atan2(-dx, -dz)
    
    -- In Bevy, positive pitch (rotation around X) rotates -Z towards +Y (UP).
    -- If target is below eye (dy < 0), we want to look down (negative pitch).
    -- asin(-dy) would give positive for dy < 0.
    -- asin(dy) gives negative for dy < 0.
    local pitch = math.asin(dy)
    
    return ThirdPerson.yaw_pitch_to_quat(yaw, pitch)
end

--- Convert yaw/pitch angles to quaternion
--- @param yaw number Horizontal angle (radians)
--- @param pitch number Vertical angle (radians)
--- @return table { x, y, z, w } quaternion
function ThirdPerson.yaw_pitch_to_quat(yaw, pitch)
    -- Half angles
    local hy = yaw * 0.5
    local hp = pitch * 0.5
    
    local cy = math.cos(hy)
    local sy = math.sin(hy)
    local cp = math.cos(hp)
    local sp = math.sin(hp)
    
    -- Quaternion from Euler (YXZ order)
    return {
        x = cy * sp,
        y = sy * cp,
        z = -sy * sp,
        w = cy * cp,
    }
end

--- Slerp between two quaternions
--- @param q1 table { x, y, z, w }
--- @param q2 table { x, y, z, w }
--- @param t number 0-1
--- @return table { x, y, z, w }
function ThirdPerson.slerp(q1, q2, t)
    -- Dot product
    local dot = q1.x*q2.x + q1.y*q2.y + q1.z*q2.z + q1.w*q2.w
    
    -- If negative dot, negate one quaternion
    local q2_adj = q2
    if dot < 0 then
        q2_adj = { x = -q2.x, y = -q2.y, z = -q2.z, w = -q2.w }
        dot = -dot
    end
    
    -- If very close, linear interpolation
    if dot > 0.9995 then
        return {
            x = q1.x + (q2_adj.x - q1.x) * t,
            y = q1.y + (q2_adj.y - q1.y) * t,
            z = q1.z + (q2_adj.z - q1.z) * t,
            w = q1.w + (q2_adj.w - q1.w) * t,
        }
    end
    
    -- Spherical interpolation
    local theta_0 = math.acos(dot)
    local theta = theta_0 * t
    local sin_theta = math.sin(theta)
    local sin_theta_0 = math.sin(theta_0)
    
    local s0 = math.cos(theta) - dot * sin_theta / sin_theta_0
    local s1 = sin_theta / sin_theta_0
    
    return {
        x = s0 * q1.x + s1 * q2_adj.x,
        y = s0 * q1.y + s1 * q2_adj.y,
        z = s0 * q1.z + s1 * q2_adj.z,
        w = s0 * q1.w + s1 * q2_adj.w,
    }
end

--- Lerp between two positions
--- @param p1 table { x, y, z }
--- @param p2 table { x, y, z }
--- @param t number 0-1
--- @return table { x, y, z }
function ThirdPerson.lerp_pos(p1, p2, t)
    return {
        x = p1.x + (p2.x - p1.x) * t,
        y = p1.y + (p2.y - p1.y) * t,
        z = p1.z + (p2.z - p1.z) * t,
    }
end

return ThirdPerson
