-- Modular Camera Controller
-- Separates camera positioning (first_person, third_person) from movement behavior (face_movement, face_camera, strafe_style)
--
-- Usage:
--   local CameraController = require("modules/modular_camera_controller.lua")
--   local Camera = require("modules/cameras/third_person.lua")
--   local Movement = require("modules/movement/face_movement.lua")
--
--   CameraController.init({
--       camera = Camera,      -- Camera positioning module
--       movement = Movement,  -- Movement behavior module (optional)
--   })
--
--   CameraController.attach(world, player_entity_id)
--   register_system("Update", function(world)
--       CameraController.update(world, world:delta_time())
--   end)

local InputManager = require("modules/input_manager.lua")
local NetRole = require("modules/net_role.lua")
local NetSync = require("modules/net_sync.lua")

local CameraController = {}

CameraController.MARKER = "ModularCameraController"

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

CameraController.config = {
    -- Modules (set via init or hot-swap via set_camera/set_movement)
    camera = nil,       -- Camera positioning module (e.g., first_person, third_person)
    movement = nil,     -- Movement behavior module (e.g., face_movement, face_camera)
    
    -- Input settings
    sensitivity = 1.0,
    invert_y = false,
    
    -- Smoothing (10-15 for smooth follow, 50+ for snappy)
    smoothing_factor = 80.0,
}

--------------------------------------------------------------------------------
-- State
--------------------------------------------------------------------------------

local camera_entity_id = nil
local target_entity_id = nil
local yaw = 0      -- Horizontal rotation (radians)
local pitch = 0    -- Vertical rotation (radians)
local is_vr = false

-- Input mode: "game" (mouse locked, controls camera) or "ui" (mouse free, can click UI)
local input_mode = "game"
local cursor_configured = false

-- Smoothing state
local current_camera_pos = nil  -- Smoothed camera position
local current_look_target = nil  -- Smoothed look target
local current_player_rotation = nil  -- Smoothed player rotation

-- Prediction state (client only)
local predictions = {}         -- sequence -> { position, velocity, input }
local current_sequence = 0

-- Input state tracking for change detection
local last_sent_input = nil    -- Last PlayerInput values sent to network

-- Reconciliation config
local SNAP_THRESHOLD = 2.0     -- Snap to server if error exceeds this (increased to reduce snapping)
local BLEND_FACTOR = 0.1       -- How fast to blend toward server position (reduced for smoother correction)

-- Server: cached movement modules per entity
local movement_modules = {}    -- entity_id -> loaded module

--------------------------------------------------------------------------------
-- Initialization
--------------------------------------------------------------------------------

--- Initialize with config and modules
--- @param cfg table { camera, movement, sensitivity, invert_y, smoothing_factor }
function CameraController.init(cfg)
    if cfg then
        if cfg.camera then
            CameraController.config.camera = cfg.camera
            if cfg.camera.init then
                cfg.camera.init(cfg.camera_config)
            end
            print("[MODULAR_CAMERA] Camera module set")
        end
        
        if cfg.movement then
            CameraController.config.movement = cfg.movement
            if cfg.movement.init then
                cfg.movement.init(cfg.movement_config)
            end
            print("[MODULAR_CAMERA] Movement module set")
        end
        
        if cfg.sensitivity then CameraController.config.sensitivity = cfg.sensitivity end
        if cfg.invert_y ~= nil then CameraController.config.invert_y = cfg.invert_y end
        if cfg.smoothing_factor then CameraController.config.smoothing_factor = cfg.smoothing_factor end
    end
    
    print("[MODULAR_CAMERA] Initialized")
end

--- Hot-swap camera module
--- @param camera_module table Camera positioning module
function CameraController.set_camera(camera_module)
    CameraController.config.camera = camera_module
    if camera_module.init then
        camera_module.init()
    end
    print("[MODULAR_CAMERA] Camera module swapped")
end

--- Hot-swap movement module
--- @param movement_module table Movement behavior module
function CameraController.set_movement(movement_module)
    CameraController.config.movement = movement_module
    if movement_module.init then
        movement_module.init()
    end
    print("[MODULAR_CAMERA] Movement module swapped")
end

--------------------------------------------------------------------------------
-- VR Detection
--------------------------------------------------------------------------------

local function detect_vr(world)
    local ok, result = pcall(function()
        return world:query_resource("XrViews")
    end)
    return false -- TODO: implement VR detection
end

--------------------------------------------------------------------------------
-- Cursor Control
--------------------------------------------------------------------------------

local function apply_cursor_settings(world)
    local windows = world:query({"Window", "CursorOptions"}, nil)
    if #windows == 0 then
        windows = world:query({"Window"}, nil)
        if #windows == 0 then return end
    end
    
    local window = windows[1]
    
    if input_mode == "game" then
        window:set({
            CursorOptions = {
                visible = false,
                grab_mode = { Locked = true }
            }
        })
    else
        window:set({
            CursorOptions = {
                visible = true,
                grab_mode = { None = true }
            }
        })
    end
end

function CameraController.set_input_mode(mode)
    if mode == "game" or mode == "ui" then
        input_mode = mode
        cursor_configured = false
    end
end

function CameraController.get_input_mode()
    return input_mode
end

function CameraController.toggle_input_mode()
    if input_mode == "game" then
        CameraController.set_input_mode("ui")
    else
        CameraController.set_input_mode("game")
    end
end

--------------------------------------------------------------------------------
-- Camera Creation & Attachment
--------------------------------------------------------------------------------

function CameraController.create_camera()
    if not (NetRole.is_client() or NetRole.is_offline()) then
        return nil
    end
    
    local spawn_data = {
        [CameraController.MARKER] = {},
        Camera3d = {},
        Transform = {
            translation = {x = 0, y = 5, z = 10},
            rotation = {x = 0, y = 0, z = 0, w = 1},
            scale = {x = 1, y = 1, z = 1}
        }
    }
    
    camera_entity_id = spawn(spawn_data):id()
    print(string.format("[MODULAR_CAMERA] Created camera entity %s", tostring(camera_entity_id)))
    return camera_entity_id
end

--------------------------------------------------------------------------------
-- Helpers
--------------------------------------------------------------------------------

--- Get movement module for an entity (uses MovementConfig.module_path or fallback)
local function get_movement_module_for(entity)
    local move_config = entity:get("MovementConfig")
    if not move_config or not move_config.module_path then
        return CameraController.config.movement  -- Fallback to global
    end
    
    local entity_id = entity:id()
    if not movement_modules[entity_id] then
        movement_modules[entity_id] = require(move_config.module_path)
    end
    return movement_modules[entity_id]
end

--- Get movement config for an entity (uses MovementConfig component or module default)
local function get_movement_config_for(entity)
    local move_config = entity:get("MovementConfig")
    if move_config then return move_config end

    local module = get_movement_module_for(entity)
    return module and module.default_config or {}
end

--- Check if entity is grounded using MeshRayCast
local function check_grounded(world, entity_id)
    local entity = world:get_entity(entity_id)
    if not entity then return false end
    
    local transform = entity:get("Transform")
    if not transform then return false end
    
    local pos = transform.translation
    local ray = {
        origin = { x = pos.x, y = pos.y + 0.1, z = pos.z },
        direction = { x = 0, y = -1, z = 0 },
        early_exit = true,
    }
    
    local ok, result = pcall(function()
        return world:call_systemparam_method("MeshRayCast", "cast_ray", ray)
    end)
    
    if ok and result and result.distance and result.distance < 0.3 then
        return true
    end
    
    -- Fallback: simple y check when raycast not available
    return pos.y <= 0.01
end

--- Unified physics update - used by both client prediction and server authority
--- @param world userdata
--- @param entity userdata
--- @param player_input table PlayerInput component data
--- @param dt number delta time
--- @return table new_pos, table new_velocity, table new_rotation
local function calculate_player_physics(world, entity, player_input, dt)
    local transform = entity:get("Transform")
    local player_state = entity:get("PlayerState")
    if not transform or not player_state then
        return nil, nil, nil
    end

    local movement_module = get_movement_module_for(entity)
    if not movement_module or not movement_module.calculate_physics then
        return nil, nil, nil
    end

    local move_config = get_movement_config_for(entity)
    local move_dir = { x = player_input.move_x or 0, z = player_input.move_z or 0 }
    local is_grounded = check_grounded(world, entity:id())

    -- Calculate physics (authoritative)
    local new_pos, new_velocity = movement_module.calculate_physics(
        transform.translation,
        player_state.velocity or {x = 0, y = 0, z = 0},
        move_dir, player_input, dt, is_grounded,
        move_config
    )

    -- Calculate rotation from yaw
    local new_rotation = world:call_static_method("Quat", "from_rotation_y", player_input.yaw or 0)

    return new_pos, new_velocity, new_rotation
end

function CameraController.attach(world, player_entity_id)
    target_entity_id = player_entity_id
    
    -- Create or use existing camera
    local entities = world:query({CameraController.MARKER})
    if #entities == 0 then
        CameraController.create_camera()
    else
        camera_entity_id = entities[1]:id()
    end
    
    -- Call camera module's on_attach if it has one
    local camera_module = CameraController.config.camera
    if camera_module and camera_module.on_attach then
        local new_state = camera_module.on_attach(world, camera_entity_id, target_entity_id)
        if new_state then
            if new_state.yaw then yaw = new_state.yaw end
            if new_state.pitch then pitch = new_state.pitch end
            print(string.format("[MODULAR_CAMERA] Updated state from module: yaw=%.2f, pitch=%.2f", yaw, pitch))
        end
    end
    
    print(string.format("[MODULAR_CAMERA] Attached to entity %d", player_entity_id))
end

function CameraController.detach()
    target_entity_id = nil
end

--------------------------------------------------------------------------------
-- Input Processing
--------------------------------------------------------------------------------

function CameraController.process_mouse_input(world)
    if input_mode ~= "game" then return end
    
    local motion_events = world:read_events("MouseMotion")
    for _, event in ipairs(motion_events) do
        local dx, dy = 0, 0
        
        if event.delta then
            if event.delta.x then
                dx = event.delta.x
                dy = event.delta.y or 0
            elseif event.delta[1] then
                dx = event.delta[1]
                dy = event.delta[2] or 0
            end
        end
        
        if dx ~= 0 or dy ~= 0 then
            CameraController.rotate(-dx, -dy)
        end
    end
end

function CameraController.process_keyboard_input(world)
    local key_events = world:read_events("KeyboardInput")
    for _, event in ipairs(key_events) do
        if event.key_code and event.key_code.Escape then
            if event.state and event.state.Pressed then
                CameraController.toggle_input_mode()
            end
        end
    end
end

--------------------------------------------------------------------------------
-- Camera Update
--------------------------------------------------------------------------------

--- Process server-side player movement for all players
local function process_server_players(world, dt)
    -- Process all players every frame (no change detection)
    -- This ensures continuous movement while keys are held
    local players = world:query({"PlayerInput", "Transform", "PlayerState", "MovementConfig"})

    for _, entity in ipairs(players) do
        local player_input = entity:get("PlayerInput")
        if not player_input then
            goto continue_player
        end

        -- Use unified physics calculation
        local new_pos, new_velocity, new_rotation = calculate_player_physics(world, entity, player_input, dt)
        if not new_pos then
            goto continue_player
        end

        -- Debug: log server processing
        print(string.format("[SERVER] Processing player entity_id=%s pos=(%.2f,%.2f,%.2f)",
            tostring(entity:id()), new_pos.x, new_pos.y, new_pos.z))

        -- Apply authoritatively using :set() to trigger ECS change detection
        local transform = entity:get("Transform")
        entity:set({
            Transform = {
                translation = new_pos,
                rotation = new_rotation,
                scale = transform.scale  -- Preserve scale
            }
        })

        local player_state = entity:get("PlayerState")
        entity:set({
            PlayerState = {
                velocity = new_velocity,
                owner_client = player_state.owner_client,  -- Preserve owner
                spawn_pos = player_state.spawn_pos  -- Preserve spawn_pos
            }
        })

        -- Debug: verify set applied
        local verify_transform = entity:get("Transform")
        print(string.format("[SERVER] After set: pos=(%.2f,%.2f,%.2f)",
            verify_transform.translation.x, verify_transform.translation.y, verify_transform.translation.z))

        ::continue_player::
    end
end

--- Handle reconciliation with server authoritative state using sequence-based prediction
local function reconcile(world, entity, movement_module, move_config)
    local server_state = NetSync.get_server_authoritative_state()
    if not server_state or not server_state.position then return end

    -- Get the acknowledged sequence from server
    local entity_id = entity:id()
    local ack_seq = NetSync.get_last_acked_sequence(entity_id)
    if not ack_seq then
        NetSync.clear_server_authoritative_state()
        return
    end

    -- Find the prediction that matches the server's acknowledged sequence
    local acked_prediction = predictions[ack_seq]
    if not acked_prediction then
        -- No matching prediction (too old), just clear
        NetSync.clear_server_authoritative_state()
        return
    end

    -- Compare server's position with our prediction at that sequence
    local sp = server_state.position
    local pp = acked_prediction.position
    local dx = pp.x - sp.x
    local dy = pp.y - sp.y
    local dz = pp.z - sp.z
    local error_dist = math.sqrt(dx*dx + dy*dy + dz*dz)

    if error_dist > SNAP_THRESHOLD then
        -- Significant misprediction: correct and replay
        print(string.format("[MODULAR_CAMERA] Reconcile SNAP: error=%.2f at seq=%d", error_dist, ack_seq))

        -- Start from server's authoritative state
        local corrected_pos = sp
        local corrected_vel = acked_prediction.velocity

        -- Replay all predictions after the acknowledged one
        local sequences_to_replay = {}
        for seq, _ in pairs(predictions) do
            if seq > ack_seq then
                table.insert(sequences_to_replay, seq)
            end
        end
        table.sort(sequences_to_replay)

        for _, seq in ipairs(sequences_to_replay) do
            local pred = predictions[seq]
            if pred and movement_module and movement_module.calculate_physics then
                local dt = world:delta_time()  -- Approximate dt for replay
                corrected_pos, corrected_vel = movement_module.calculate_physics(
                    corrected_pos, corrected_vel,
                    pred.move_dir, pred.input, dt,
                    pred.is_grounded, move_config
                )
                -- Update the prediction with corrected values
                pred.position = corrected_pos
                pred.velocity = corrected_vel
            end
        end

        -- Apply corrected position
        local transform = entity:get("Transform")
        entity:set({
            Transform = {
                translation = corrected_pos,
                rotation = transform.rotation,
                scale = transform.scale
            },
            PlayerState = {
                velocity = corrected_vel
            }
        })
    elseif error_dist > 0.01 then
        -- Small error: just note it (prediction is working well)
        print(string.format("[MODULAR_CAMERA] Reconcile OK: error=%.2f at seq=%d", error_dist, ack_seq))
    end

    -- Clean up old predictions (keep a reasonable history)
    for seq, _ in pairs(predictions) do
        if seq < ack_seq - 10 then
            predictions[seq] = nil
        end
    end

    -- Clear after processing
    NetSync.clear_server_authoritative_state()
end

--- Update camera position, player movement (prediction), and server processing
--- @param world userdata
function CameraController.update(world)
    local dt = world:delta_time()

    -- Apply cursor settings if needed (client only)
    if NetRole.is_client() or NetRole.is_offline() then
        if not cursor_configured then
            apply_cursor_settings(world)
            cursor_configured = true
        end
        
        -- Process input
        CameraController.process_keyboard_input(world)
        CameraController.process_mouse_input(world)
    end
    
    -- SERVER: Process all player inputs
    if NetRole.is_server() then
        process_server_players(world, dt)
    end
    
    -- CLIENT/OFFLINE: Camera positioning and player prediction
    if (NetRole.is_client() or NetRole.is_offline()) and target_entity_id and camera_entity_id then
        -- Check for VR
        is_vr = detect_vr(world)
        if is_vr then return end
        
        -- Get target entity
        local target = world:get_entity(target_entity_id)
        if not target then return end
        
        local target_transform = target:get("Transform")
        if not target_transform then return end
        
        local player_state = target:get("PlayerState")
        
        local target_pos = target_transform.translation
        local target_rotation = target_transform.rotation or {x = 0, y = 0, z = 0, w = 1}
        local velocity = (player_state and player_state.velocity) or {x = 0, y = 0, z = 0}
        
        -- Get camera entity
        local camera = world:get_entity(camera_entity_id)
        if not camera then return end
        
        -- Get input
        local input = InputManager.get_movement_input(world)
        
        -- Get movement module and config for this entity
        local movement_module = get_movement_module_for(target)
        local move_config = get_movement_config_for(target)
        
        -- Calculate world-space movement direction
        local move_dir = { x = 0, z = 0 }
        if movement_module and movement_module.get_world_movement then
            move_dir = movement_module.get_world_movement(input, yaw)
        end
        
        -- Calculate physics (prediction)
        local new_pos = target_pos
        local new_velocity = velocity
        local is_grounded = check_grounded(world, target_entity_id)
        if movement_module and movement_module.calculate_physics then
            new_pos, new_velocity = movement_module.calculate_physics(
                target_pos, velocity, move_dir, input, dt, is_grounded, move_config
            )
        end

        -- Calculate rotation
        local new_rotation = target_rotation
        if movement_module and movement_module.get_target_rotation then
            local target_rot = movement_module.get_target_rotation(input, yaw, target_rotation, world)
            if target_rot then
                if not current_player_rotation then
                    current_player_rotation = target_rotation
                end
                new_rotation = movement_module.interpolate_rotation(
                    current_player_rotation, target_rot, dt, world
                )
                current_player_rotation = new_rotation
            end
        end

        -- Build current input state
        local current_input = {
            move_x = move_dir.x,
            move_z = move_dir.z,
            yaw = yaw,
            jump = input.jump,
            sprint = input.sprint
        }

        -- Detect if movement input changed (exclude yaw from change detection)
        local movement_changed = false
        if not last_sent_input then
            movement_changed = true  -- First frame
        else
            -- Check movement fields only (not yaw - it changes too frequently)
            movement_changed = (
                current_input.move_x ~= last_sent_input.move_x or
                current_input.move_z ~= last_sent_input.move_z or
                current_input.jump ~= last_sent_input.jump or
                current_input.sprint ~= last_sent_input.sprint
            )
        end

        -- Check if yaw changed significantly (only matters when moving)
        local yaw_changed = false
        if last_sent_input and is_moving then
            local yaw_delta = math.abs(current_input.yaw - last_sent_input.yaw)
            yaw_changed = yaw_delta > 0.05  -- ~3 degrees threshold
        end

        -- Send PlayerInput when movement changed OR yaw changed significantly while moving
        local should_send = movement_changed or yaw_changed
        if should_send then
            target:patch({ PlayerInput = current_input })
            -- Store copy of sent input for next frame comparison
            last_sent_input = {
                move_x = current_input.move_x,
                move_z = current_input.move_z,
                yaw = current_input.yaw,
                jump = current_input.jump,
                sprint = current_input.sprint
            }
        end

        -- Only update Transform/PlayerState and store predictions when actually moving
        local is_moving = math.abs(move_dir.x) > 0.001 or math.abs(move_dir.z) > 0.001 or input.jump
        if is_moving then
            -- Get sequence number from NetSync (will be incremented when sending)
            local seq = NetSync.get_current_sequence(target_entity_id) + 1

            -- Store prediction with all data needed for replay
            predictions[seq] = {
                position = new_pos,
                velocity = new_velocity,
                move_dir = move_dir,
                input = {
                    jump = input.jump,
                    sprint = input.sprint,
                    forward = input.forward,
                    right = input.right
                },
                is_grounded = is_grounded
            }

            -- Apply Transform immediately using call_component_method (same frame as camera)
            world:call_component_method(
                target:id(),
                "Transform",
                "with_translation",
                new_pos
            )
            world:call_component_method(
                target:id(),
                "Transform",
                "with_rotation",
                new_rotation
            )

            -- Queue PlayerState update (doesn't need frame sync)
            target:patch({
                PlayerState = {
                    velocity = new_velocity
                }
            })
        end
        
        -- Reconciliation with server
        reconcile(world, target, movement_module, move_config)
        
        -- Camera positioning
        local camera_module = CameraController.config.camera
        local desired_pos, look_target
        
        if camera_module and camera_module.calculate_position then
            local result = camera_module.calculate_position(new_pos, yaw, pitch)
            desired_pos = result.camera_pos
            look_target = result.look_target
        else
            -- Fallback: basic third person
            local dist = 5.0
            local height = 2.0
            
            desired_pos = {
                x = new_pos.x + math.sin(yaw) * dist * math.cos(pitch),
                y = new_pos.y + height + math.sin(pitch) * dist,
                z = new_pos.z + math.cos(yaw) * dist * math.cos(pitch)
            }
            look_target = {
                x = new_pos.x,
                y = new_pos.y + 1.0,
                z = new_pos.z
            }
        end
        
        -- Initialize smoothed position if not set
        if not current_camera_pos then
            current_camera_pos = desired_pos
        end
        if not current_look_target then
            current_look_target = look_target
        end
        
        -- Smooth camera position and look target together
        local smoothing = CameraController.config.smoothing_factor
        local t = math.min(1.0, smoothing * dt)
        current_camera_pos = {
            x = current_camera_pos.x + (desired_pos.x - current_camera_pos.x) * t,
            y = current_camera_pos.y + (desired_pos.y - current_camera_pos.y) * t,
            z = current_camera_pos.z + (desired_pos.z - current_camera_pos.z) * t
        }
        current_look_target = {
            x = current_look_target.x + (look_target.x - current_look_target.x) * t,
            y = current_look_target.y + (look_target.y - current_look_target.y) * t,
            z = current_look_target.z + (look_target.z - current_look_target.z) * t
        }
        
        -- Compute look direction using smoothed values
        local look_dir = {
            x = current_look_target.x - current_camera_pos.x,
            y = current_look_target.y - current_camera_pos.y,
            z = current_look_target.z - current_camera_pos.z
        }
        
        -- Update camera transform position using with_translation (writes to ECS immediately)
        world:call_component_method(
            camera:id(),
            "Transform",
            "with_translation",
            current_camera_pos
        )
        
        -- Update camera rotation using looking_to (writes to ECS immediately)
        world:call_component_method(
            camera:id(),
            "Transform",
            "looking_to",
            look_dir,
            {x = 0, y = 1, z = 0}
        )
    end
end

register_system("Update", function(world)
    CameraController.update(world)
end)

--------------------------------------------------------------------------------
-- Input Handling
--------------------------------------------------------------------------------

function CameraController.rotate(delta_x, delta_y)
    local sensitivity = CameraController.config.sensitivity * 0.003
    
    yaw = yaw + delta_x * sensitivity
    
    local y_mult = CameraController.config.invert_y and 1 or -1
    pitch = pitch + delta_y * sensitivity * y_mult
    
    -- Clamp pitch
    pitch = math.max(-math.pi * 0.45, math.min(math.pi * 0.45, pitch))
end

function CameraController.get_yaw()
    return yaw
end

function CameraController.get_pitch()
    return pitch
end

--------------------------------------------------------------------------------
-- Getters
--------------------------------------------------------------------------------

function CameraController.get_camera_entity()
    return camera_entity_id
end

function CameraController.get_target_entity()
    return target_entity_id
end

function CameraController.is_vr_mode()
    return is_vr
end

print("[MODULAR_CAMERA] Module loaded")

return CameraController
