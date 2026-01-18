-- Player Controller Module
-- Server-authoritative movement with client prediction
--
-- Usage (Client):
--   local controller = PlayerController.create_client_controller(entity, send_fn)
--   register_system("Update", function(world)
--       local input = InputManager.get_movement_input(world)
--       controller.process_input(input, world:delta_time())
--   end)
--
-- Usage (Server):
--   register_system("Update", function(world)
--       -- Process PlayerInput components from clients
--       local players = world:query({"NetworkSync", "PlayerInput", "Transform"}, {"PlayerInput"})
--       for _, entity in ipairs(players) do
--           local new_state = PlayerController.process_server_input(entity)
--           entity:set({ Transform = new_state.transform, PlayerState = new_state })
--       end
--   end)

local PlayerController = {}

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

PlayerController.config = {
    mode = "velocity",              -- "velocity" or "physics"
    walk_speed = 10.0,               -- Units per second
    run_speed = 20.0,               -- Units per second when sprinting
    acceleration = 50.0,            -- For physics mode
    deceleration = 40.0,            -- For physics mode
    gravity = 15,                  -- Gravity strength
    jump_velocity = 5.0,            -- Initial jump velocity
    
    -- Rotation
    rotation_mode = "face_movement", -- "none", "face_movement", "face_camera"
    rotation_speed = 15.0,          -- Radians per second for rotation interpolation
    
    -- Client prediction
    snap_threshold = 0.5,           -- Snap to server if error exceeds this
    lerp_speed = 10.0,              -- Speed for smoothing minor corrections
    
    -- Network
    network_send_rate = 20,         -- Send input N times per second
}

--------------------------------------------------------------------------------
-- Camera-Relative Input Transformation
--------------------------------------------------------------------------------

--- Transform camera-relative input (forward/right) to world-space direction
--- @param input table {forward, right} in camera space
--- @param world userdata World context for component method calls
--- @param camera_entity_id number Camera entity ID
--- @return table {x, z} world-space movement direction
function PlayerController.transform_input_by_camera(input, world, camera_entity_id)
    local forward = input.forward or 0
    local right = input.right or 0
    
    if forward == 0 and right == 0 then
        return { x = 0, z = 0 }
    end
    
    -- Get camera's forward and right vectors using Bevy Transform methods
    local cam_forward = world:call_component_method(camera_entity_id, "Transform", "forward")
    local cam_right = world:call_component_method(camera_entity_id, "Transform", "right")
    
    if not cam_forward or not cam_right then
        -- Fallback to world-space if camera not available
        return { x = right, z = -forward }
    end
    
    -- Extract components from Dir3 (Bevy's direction vector type)
    -- Access via string keys only (Dir3 uses named fields)
    local cam_forward_x = tonumber(cam_forward[1].x) or 0
    local cam_forward_z = tonumber(cam_forward[1].z) or 0
    local cam_right_x = tonumber(cam_right[1].x) or 0
    local cam_right_z = tonumber(cam_right[1].z) or 0
    
    -- Normalize projected vectors (since we ignored Y component)
    local forward_len = math.sqrt(cam_forward_x * cam_forward_x + cam_forward_z * cam_forward_z)
    local right_len = math.sqrt(cam_right_x * cam_right_x + cam_right_z * cam_right_z)
    
    if forward_len > 0.01 then
        cam_forward_x = cam_forward_x / forward_len
        cam_forward_z = cam_forward_z / forward_len
    end
    if right_len > 0.01 then
        cam_right_x = cam_right_x / right_len
        cam_right_z = cam_right_z / right_len
    end
    
    -- Combine input with camera directions
    -- To match Bevy's coordinate system, flip the sign of cam_forward and cam_right as needed
    -- Forward should move in the direction the camera is facing (-Z), right is +X
    local world_x = forward * cam_forward_x + right * cam_right_x
    local world_z = forward * cam_forward_z + right * cam_right_z
    
    return { x = -world_x, z = -world_z }
end

--- Calculate target rotation quaternion from movement direction using Bevy Quat
--- @param move_dir table {x, z} world-space movement direction
--- @param world userdata World context for static method calls
--- @return table|nil quaternion {x, y, z, w} or nil if not moving
function PlayerController.rotation_from_movement(move_dir, world)
    local len = math.sqrt(move_dir.x * move_dir.x + move_dir.z * move_dir.z)
    if len < 0.01 then
        return nil  -- Not moving, no target rotation
    end
    
    -- Normalize direction
    local dir_x = move_dir.x / len
    local dir_z = move_dir.z / len
    
    -- Calculate yaw angle from direction
    -- In Bevy: +X is right, -Z is forward
    -- atan2(-x, -z) gives us the angle from forward (-Z axis)
    local yaw = math.atan2(dir_x, dir_z)
    
    -- Use Bevy's Quat.from_rotation_y to create rotation around Y axis
    local target_rotation = world:call_static_method("Quat", "from_rotation_y", yaw)
    
    return target_rotation
end

--- Interpolate rotation toward target using Bevy Quat.slerp
--- @param current_rot table current quaternion
--- @param target_rot table target quaternion
--- @param dt number delta time
--- @param world userdata World context for static method calls
--- @return table interpolated quaternion
function PlayerController.interpolate_rotation(current_rot, target_rot, dt, world)
    local speed = PlayerController.config.rotation_speed
    local t = math.min(1.0, speed * dt)
    
    return world:call_static_method("Quat", "slerp",
        current_rot,
        target_rot,
        t
    )
end

--------------------------------------------------------------------------------
-- Pure Movement Calculation (Shared between client and server)
--------------------------------------------------------------------------------

--- Calculate movement based on input (deterministic, shared logic)
--- @param current_pos table {x, y, z}
--- @param current_velocity table {x, y, z}
--- @param input table {forward, right, jump, sprint} OR world-space {move_dir={x,z}, jump, sprint}
--- @param dt number delta time
--- @param is_grounded boolean whether player is on ground
--- @param move_dir table|nil pre-computed world-space movement direction {x, z}
--- @return table new_pos, table new_velocity
function PlayerController.calculate_movement(current_pos, current_velocity, input, dt, is_grounded, move_dir)
    local config = PlayerController.config
    
    local new_pos = {
        x = current_pos.x,
        y = current_pos.y,
        z = current_pos.z
    }
    local new_velocity = {
        x = current_velocity.x,
        y = current_velocity.y,
        z = current_velocity.z
    }
    
    -- Use pre-computed world-space direction if provided, otherwise compute from raw input
    local dir_x, dir_z
    if move_dir then
        dir_x = move_dir.x or 0
        dir_z = move_dir.z or 0
    else
        -- Fallback: treat input as world-space (forward = -Z, right = +X)
        dir_x = input.right or 0
        dir_z = -(input.forward or 0)
    end
    
    -- Calculate speed based on sprint
    local speed = input.sprint and config.run_speed or config.walk_speed
    
    if config.mode == "velocity" then
        -- Velocity mode: instant direction change
        new_velocity.x = dir_x * speed
        new_velocity.z = dir_z * speed
    else
        -- Physics mode: acceleration/deceleration
        local target_vx = dir_x * speed
        local target_vz = dir_z * speed
        
        local is_moving = (dir_x ~= 0 or dir_z ~= 0)
        local accel = is_moving and config.acceleration or config.deceleration
        
        -- Lerp towards target velocity
        new_velocity.x = new_velocity.x + (target_vx - new_velocity.x) * math.min(1, accel * dt)
        new_velocity.z = new_velocity.z + (target_vz - new_velocity.z) * math.min(1, accel * dt)
    end
    
    -- Handle jumping and gravity
    if is_grounded then
        if input.jump then
            new_velocity.y = config.jump_velocity
        else
            new_velocity.y = 0
        end
    else
        -- Apply gravity
        new_velocity.y = new_velocity.y - config.gravity * dt
    end
    
    -- Apply velocity to position
    new_pos.x = new_pos.x + new_velocity.x * dt
    new_pos.y = new_pos.y + new_velocity.y * dt
    new_pos.z = new_pos.z + new_velocity.z * dt
    
    -- Simple ground clamp (y >= 0)
    if new_pos.y < 0 then
        new_pos.y = 0
        new_velocity.y = 0
    end
    
    return new_pos, new_velocity
end

--------------------------------------------------------------------------------
-- Client Controller (Prediction)
--------------------------------------------------------------------------------

--- Create a client-side controller for an entity
--- @param entity userdata The player entity
--- @param send_fn function(input_msg) Function to send input to server
--- @param get_camera_entity_fn function|nil Optional function returning camera entity ID
--- @return table Controller object with process_input() and on_server_state()
function PlayerController.create_client_controller(entity, send_fn, get_camera_entity_fn)
    local controller = {
        entity = entity,
        send_fn = send_fn,
        get_camera_entity = get_camera_entity_fn,
        sequence = 0,
        predictions = {},          -- sequence -> predicted_state
        last_send_time = 0,
        velocity = {x = 0, y = 0, z = 0},
        rotation = {x = 0, y = 0, z = 0, w = 1},  -- Current rotation
    }
    
    --- Process input and apply prediction
    function controller.process_input(world, input, dt)
        -- Use stored position if available, otherwise read from entity
        local current_pos
        local current_rotation = controller.rotation
        if controller.predicted_pos then
            current_pos = controller.predicted_pos
        else
            local transform = controller.entity:get("Transform")
            if not transform then return end
            current_pos = transform.translation
            current_rotation = transform.rotation or current_rotation
        end
            
        local is_grounded = current_pos.y <= 0.01  -- Simple ground check
        
        -- Transform input to world-space using camera if available
        local move_dir = nil
        local camera_entity_id = controller.get_camera_entity and controller.get_camera_entity(world)
        
        print(camera_entity_id)
        if camera_entity_id then
            move_dir = PlayerController.transform_input_by_camera(input, world, camera_entity_id)
        end
            
        -- Calculate predicted movement with world-space direction
        local new_pos, new_velocity = PlayerController.calculate_movement(
            current_pos,
            controller.velocity,
            input,
            dt,
            is_grounded,
            move_dir
        )
        
        -- Calculate rotation based on rotation_mode
        local new_rotation = current_rotation
        local rotation_mode = PlayerController.config.rotation_mode
        
        print(rotation_mode, move_dir)
        if rotation_mode == "face_movement" and move_dir then
            -- Only rotate when actually moving (not just holding backward)
            local target_rot = PlayerController.rotation_from_movement(move_dir, world)
            if target_rot then
                new_rotation = PlayerController.interpolate_rotation(
                    current_rotation,
                    target_rot,
                    dt,
                    world
                )
            end
        elseif rotation_mode == "face_camera" and camera_entity_id then
            -- Always face camera direction (get camera's forward, project to XZ)
            local cam_forward = world:call_component_method(camera_entity_id, "Transform", "forward")
            if cam_forward then
                local cam_dir = {
                    x = cam_forward.x or cam_forward[1] or 0,
                    z = cam_forward.z or cam_forward[3] or 0
                }
                local target_rot = PlayerController.rotation_from_movement(cam_dir, world)
                if target_rot then
                    new_rotation = PlayerController.interpolate_rotation(
                        current_rotation,
                        target_rot,
                        dt,
                        world
                    )
                end
            end
        end
        -- rotation_mode == "none" keeps current rotation
            
        -- Store predicted state for next frame
        controller.predicted_pos = new_pos
        controller.rotation = new_rotation
        
        -- Apply prediction locally
        controller.velocity = new_velocity
        controller.entity:set({
            Transform = {
                translation = new_pos,
                rotation = new_rotation,
                scale = {x = 1, y = 1, z = 1}
            }
        })
        
        -- Rate-limit network sends
        local now = os.clock()
        local send_interval = 1.0 / PlayerController.config.network_send_rate
        
        if now - controller.last_send_time >= send_interval then
            -- Check if inputs actually changed
            local last_input = controller.last_input or {}
            local input_changed = (input.forward ~= (last_input.forward or 0)) or
                                (input.right ~= (last_input.right or 0)) or
                                (input.jump ~= (last_input.jump or false)) or
                                (input.sprint ~= (last_input.sprint or false))
        
            if input_changed then
                controller.sequence = controller.sequence + 1
                
                -- Store prediction for later reconciliation
                controller.predictions[controller.sequence] = {
                    position = new_pos,
                    velocity = new_velocity,
                    timestamp = now
                }
                
                -- Send WORLD-SPACE direction to server (already camera-transformed)
                -- Server doesn't have camera info, so we send the computed direction
                local send_forward = move_dir and -move_dir.z or input.forward
                local send_right = move_dir and move_dir.x or input.right
                
                if controller.send_fn then
                    controller.send_fn({
                        sequence = controller.sequence,
                        forward = send_forward,
                        right = send_right,
                        jump = input.jump,
                        sprint = input.sprint
                    })
                end
                
                -- Remember last sent input
                controller.last_input = {
                    forward = input.forward,
                    right = input.right,
                    jump = input.jump,
                    sprint = input.sprint
                }
                
                -- Clean up old predictions (keep last 60)
                local to_remove = {}
                local count = 0
                for seq, _ in pairs(controller.predictions) do
                    count = count + 1
                    if seq < controller.sequence - 60 then
                        table.insert(to_remove, seq)
                    end
                end
                for _, seq in ipairs(to_remove) do
                    controller.predictions[seq] = nil
                end
            end
            
            controller.last_send_time = now
        end
    end
    
    --- Handle server state update (reconciliation)
    function controller.on_server_state(state)
        if not state or not state.position then return end
        
        -- Compare CURRENT client position to server position
        local current_pos = controller.predicted_pos
        if not current_pos then
            local transform = controller.entity:get("Transform")
            if not transform then return end
            current_pos = transform.translation
        end
        
        -- Check for mismatch
        local dx = math.abs(current_pos.x - state.position.x)
        local dy = math.abs(current_pos.y - state.position.y)
        local dz = math.abs(current_pos.z - state.position.z)
        local error_dist = math.sqrt(dx*dx + dy*dy + dz*dz)
        
        -- DEBUG: Log reconciliation check (only occasionally to reduce spam)
        if error_dist > 0.05 then
            print(string.format("[RECONCILE] client=(%.2f,%.2f,%.2f) server=(%.2f,%.2f,%.2f) error=%.3f threshold=%.1f",
                current_pos.x, current_pos.y, current_pos.z,
                state.position.x, state.position.y, state.position.z,
                error_dist, PlayerController.config.snap_threshold))
        end
        
        if error_dist > PlayerController.config.snap_threshold then
            -- Large error: snap to server position
            print(string.format("[PLAYER_CONTROLLER] Large mismatch (%.2f), snapping to server", error_dist))
            controller.predicted_pos = state.position
            controller.entity:set({
                Transform = {
                    translation = state.position,
                    rotation = controller.entity:get("Transform").rotation,
                    scale = controller.entity:get("Transform").scale
                }
            })
            controller.velocity = state.velocity or {x = 0, y = 0, z = 0}
        elseif error_dist > 0.01 then
            -- Continuous smooth correction: lerp toward server position
            -- More aggressive lerp factor based on error size
            local lerp_factor = math.min(0.3, error_dist * 0.8)
            controller.predicted_pos = {
                x = current_pos.x + (state.position.x - current_pos.x) * lerp_factor,
                y = current_pos.y + (state.position.y - current_pos.y) * lerp_factor,
                z = current_pos.z + (state.position.z - current_pos.z) * lerp_factor,
            }
        end
    end
    
    return controller
end

--------------------------------------------------------------------------------
-- Server Processing
--------------------------------------------------------------------------------

--- Process input received from client (server-side)
--- @param entity userdata Entity with Transform and PlayerInput
--- @return table { transform, velocity, last_acked_seq }
function PlayerController.process_server_input(entity)
    local input_component = entity:get("PlayerInput")
    local transform = entity:get("Transform")
    local player_state = entity:get("PlayerState") or { velocity = {x = 0, y = 0, z = 0} }
    
    if not input_component or not transform then
        return nil
    end
    
    local input = {
        forward = input_component.forward or 0,
        right = input_component.right or 0,
        jump = input_component.jump or false,
        sprint = input_component.sprint or false
    }
    
    -- Use fixed dt - don't trust client's dt (security + determinism)
    local dt = 1.0 / 60.0
    local current_pos = transform.translation
    local current_velocity = player_state.velocity or {x = 0, y = 0, z = 0}
    local is_grounded = current_pos.y <= 0.01
    
    -- Calculate authoritative movement
    local new_pos, new_velocity = PlayerController.calculate_movement(
        current_pos,
        current_velocity,
        input,
        dt,
        is_grounded
    )
    
    return {
        transform = {
            translation = new_pos,
            rotation = transform.rotation,
            scale = transform.scale
        },
        position = new_pos,
        velocity = new_velocity,
        sequence = input_component.sequence or 0,
        last_acked_seq = input_component.sequence or 0
    }
end

--------------------------------------------------------------------------------
-- Client System Factory
--------------------------------------------------------------------------------

--- Create client-side update system that handles finding entity, prediction, and reconciliation
--- Hot-reload friendly: require_async callbacks recreate state when modules update
--- @param config table { get_camera_entity = function(world) -> entity_id, on_player_found = function(world, entity_id) }
--- @return function System function to be called each frame
function PlayerController.create_client_system(config)
    -- State table that require_async callbacks can write to
    local state = {
        NetRole = nil,
        NetSync = nil,
        InputManager = nil,
        my_controller = nil,
        my_entity_id = nil,
        setup_complete = false,
    }
    
    -- Track first load vs hot-reload
    local modules_loaded = {
        NetRole = false,
        NetSync = false,
        InputManager = false,
    }
    
    -- Reset controller state (called when modules hot-reload)
    local function reset_state()
        state.my_controller = nil
        state.my_entity_id = nil
        state.setup_complete = false
        print("[PLAYER_CONTROLLER] State reset for hot-reload")
    end
    
    return function(world)
        -- Load modules via require_async (callbacks fire on load AND hot-reload)
        require_async("modules/net_role.lua", function(module)
            if state.NetRole ~= module then
                local is_hot_reload = modules_loaded.NetRole
                state.NetRole = module
                modules_loaded.NetRole = true
                if is_hot_reload then
                    reset_state()
                end
            end
        end)
        
        require_async("modules/net_sync.lua", function(module)
            if state.NetSync ~= module then
                local is_hot_reload = modules_loaded.NetSync
                state.NetSync = module
                modules_loaded.NetSync = true
                if is_hot_reload then
                    reset_state()
                end
            end
        end)
        
        require_async("modules/input_manager.lua", function(module)
            if state.InputManager ~= module then
                local is_hot_reload = modules_loaded.InputManager
                state.InputManager = module
                modules_loaded.InputManager = true
                if is_hot_reload then
                    reset_state()
                end
            end
        end)
        
        -- Wait for all modules to load
        if not state.NetRole or not state.NetSync or not state.InputManager then
            return
        end
        
        local dt = world:delta_time()
        
        -- Step 1: Find our character entity if not found yet
        if not state.setup_complete then
            local my_net_id = state.NetSync.get_my_net_id()
            if not my_net_id then
                return  -- Wait for server assignment
            end
            
            local entity_id = state.NetSync.get_my_entity()
            if not entity_id then
                return  -- Wait for entity to spawn
            end
            
            local entity = world:get_entity(entity_id)
            if not entity or not entity:get("NetworkSync") then
                return  -- Wait for entity to be valid
            end
            
            state.my_entity_id = entity_id
            
            -- Create controller with input sender
            state.my_controller = PlayerController.create_client_controller(
                entity,
                function(input_msg)
                    entity:set({ PlayerInput = input_msg })
                end,
                config and config.get_camera_entity
            )
            
            -- Notify that player was found
            if config and config.on_player_found then
                config.on_player_found(world, entity_id)
            end
            
            state.setup_complete = true
            print(string.format("[PLAYER_CONTROLLER] Client controller ready for entity %d (net_id %d)", 
                entity_id, my_net_id))
        end
        
        -- Step 2: Process input and prediction
        if state.my_controller then
            local input = state.InputManager.get_movement_input(world)
            state.my_controller.process_input(world, input, dt)
        end
        
        -- Step 3: Reconciliation with server state (skip in "both" mode)
        if state.my_controller and state.my_entity_id and state.NetRole.get_role() ~= "both" then
            local server_state = state.NetSync.get_server_authoritative_state()
            if server_state and server_state.position then
                local entity = world:get_entity(state.my_entity_id)
                if entity then
                    local player_state = entity:get("PlayerState")
                    local velocity = player_state and player_state.velocity or {x = 0, y = 0, z = 0}
                    
                    state.my_controller.on_server_state({
                        position = server_state.position,
                        velocity = velocity
                    })
                end
            end
        end
    end
end

print("[PLAYER_CONTROLLER] Module loaded")

return PlayerController
