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
    walk_speed = 5.0,               -- Units per second
    run_speed = 10.0,               -- Units per second when sprinting
    acceleration = 50.0,            -- For physics mode
    deceleration = 40.0,            -- For physics mode
    gravity = 9.8,                  -- Gravity strength
    jump_velocity = 5.0,            -- Initial jump velocity
    
    -- Client prediction
    snap_threshold = 0.5,           -- Snap to server if error exceeds this
    lerp_speed = 10.0,              -- Speed for smoothing minor corrections
    
    -- Network
    network_send_rate = 20,         -- Send input N times per second
}

--------------------------------------------------------------------------------
-- Pure Movement Calculation (Shared between client and server)
--------------------------------------------------------------------------------

--- Calculate movement based on input (deterministic, shared logic)
--- @param current_pos table {x, y, z}
--- @param current_velocity table {x, y, z}
--- @param input table {forward, right, jump, sprint}
--- @param dt number delta time
--- @param is_grounded boolean whether player is on ground
--- @return table new_pos, table new_velocity
function PlayerController.calculate_movement(current_pos, current_velocity, input, dt, is_grounded)
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
    
    -- Calculate speed based on sprint
    local speed = input.sprint and config.run_speed or config.walk_speed
    
    if config.mode == "velocity" then
        -- Velocity mode: instant direction change
        new_velocity.x = input.right * speed
        new_velocity.z = -input.forward * speed  -- Negative because forward is -Z
    else
        -- Physics mode: acceleration/deceleration
        local target_vx = input.right * speed
        local target_vz = -input.forward * speed
        
        local accel = (input.forward ~= 0 or input.right ~= 0) and config.acceleration or config.deceleration
        
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
--- @return table Controller object with process_input() and on_server_state()
function PlayerController.create_client_controller(entity, send_fn)
    local controller = {
        entity = entity,
        send_fn = send_fn,
        sequence = 0,
        predictions = {},          -- sequence -> predicted_state
        last_send_time = 0,
        velocity = {x = 0, y = 0, z = 0},
    }
    
    --- Process input and apply prediction
    function controller.process_input(world, input, dt)
        local transform = controller.entity:get("Transform")
        if not transform then return end
        
        local current_pos = transform.translation
        local is_grounded = current_pos.y <= 0.01  -- Simple ground check
        
        -- Calculate predicted movement
        local new_pos, new_velocity = PlayerController.calculate_movement(
            current_pos,
            controller.velocity,
            input,
            dt,
            is_grounded
        )
        
        -- Apply prediction locally
        controller.velocity = new_velocity
        controller.entity:set({
            Transform = {
                translation = new_pos,
                rotation = transform.rotation,
                scale = transform.scale
            }
        })
        
        -- Rate-limit network sends
        local now = os.clock()
        local send_interval = 1.0 / PlayerController.config.network_send_rate
        
        if now - controller.last_send_time >= send_interval then
            controller.sequence = controller.sequence + 1
            
            -- Store prediction for later reconciliation
            controller.predictions[controller.sequence] = {
                position = new_pos,
                velocity = new_velocity,
                timestamp = now
            }
            
            -- Send input to server
            if controller.send_fn then
                controller.send_fn({
                    sequence = controller.sequence,
                    forward = input.forward,
                    right = input.right,
                    jump = input.jump,
                    sprint = input.sprint,
                    dt = dt
                })
            end
            
            controller.last_send_time = now
            
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
    end
    
    --- Handle server state update (reconciliation)
    function controller.on_server_state(state)
        if not state or not state.sequence then return end
        
        local predicted = controller.predictions[state.sequence]
        if not predicted then return end
        
        -- Check for mismatch
        local dx = math.abs(predicted.position.x - state.position.x)
        local dy = math.abs(predicted.position.y - state.position.y)
        local dz = math.abs(predicted.position.z - state.position.z)
        local error_dist = math.sqrt(dx*dx + dy*dy + dz*dz)
        
        if error_dist > PlayerController.config.snap_threshold then
            -- Large error: snap to server position
            print(string.format("[PLAYER_CONTROLLER] Large mismatch (%.2f), snapping to server", error_dist))
            controller.entity:set({
                Transform = {
                    translation = state.position,
                    rotation = controller.entity:get("Transform").rotation,
                    scale = controller.entity:get("Transform").scale
                }
            })
            controller.velocity = state.velocity or {x = 0, y = 0, z = 0}
        elseif error_dist > 0.01 then
            -- Small error: could lerp, but for now just accept prediction
            -- (Smooth correction would happen in a render system)
        end
        
        -- Clear acknowledged predictions
        for seq, _ in pairs(controller.predictions) do
            if seq <= state.sequence then
                controller.predictions[seq] = nil
            end
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
    
    local dt = input_component.dt or (1.0 / 60.0)
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

print("[PLAYER_CONTROLLER] Module loaded")

return PlayerController
