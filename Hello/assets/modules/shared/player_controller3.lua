-- Player Controller 3 Module
-- ECS-pure player controller with resource-based state
-- Uses query-based camera attachment (no repeated system registration)
--
-- Usage:
--   local PlayerController3 = require("modules/shared/player_controller3.lua")
--   PlayerController3.register_systems()

local NetSync3 = require("modules/net3/init.lua")
local CameraController = require("modules/camera3/controller.lua")

local PlayerController3 = {}

--------------------------------------------------------------------------------
-- State (using resources for hot-reload safety)
--------------------------------------------------------------------------------

local function get_state()
    return define_resource("PlayerController3State", {
        attached_entities = {},   -- entity_id -> true (entities with camera attached)
        my_player_entity = nil,   -- Our controlled player entity ID
        movement_speed = 5.0,
        jump_force = 8.0,
        enabled = false,
    })
end

--------------------------------------------------------------------------------
-- Camera Attachment System
--------------------------------------------------------------------------------

--- System that attaches camera to owned player entities
--- Uses ECS query instead of callbacks - survives hot-reload
local function camera_attachment_system(world)
    local state = get_state()

    if not state.enabled then
        return
    end
    
    local my_client_id = NetSync3.get_my_client_id()
    if not my_client_id then
        return
    end
    
    -- Query for entities with NetworkSync and PlayerState
    local entities = world:query({ NetSync3.MARKER, "PlayerState", "ScriptOwned" })
    
    for _, entity in ipairs(entities) do
        local entity_id = entity:id()
        
        -- Skip if already processed
        if state.attached_entities[entity_id] then
            goto continue_attachment
        end
        
        -- Check if owned by us
        local sync = entity:get(NetSync3.MARKER)
        if sync.owner_client == my_client_id then
            print(string.format("[PLAYER3] Found my player entity: %d", entity_id))
            
            -- Attach camera
            CameraController.attach(world, entity_id)
            state.my_player_entity = entity_id
            state.attached_entities[entity_id] = true
        end
        
        ::continue_attachment::
    end
end

--------------------------------------------------------------------------------
-- Movement System
--------------------------------------------------------------------------------

--- System that handles player movement input
local function movement_system(world)
    local state = get_state()
    
    if not state.enabled or not state.my_player_entity then
        return
    end
    
    local entity = world:get_entity(state.my_player_entity)
    if not entity then
        -- Entity despawned, reset state
        state.my_player_entity = nil
        state.attached_entities = {}
        return
    end
    
    local dt = world:delta_time()
    local transform = entity:get("Transform")
    if not transform then return end
    
    -- Get input (this would normally come from an InputResource)
    -- For now, we'll leave this as a placeholder for the actual input system
    local input = get_lua_resource("PlayerInput") or {}
    local move_x = input.move_x or 0
    local move_z = input.move_z or 0
    local jump = input.jump or false
    
    if move_x == 0 and move_z == 0 and not jump then
        return
    end
    
    -- Get camera yaw for movement direction
    local yaw = CameraController.get_yaw()
    
    -- Calculate movement in camera-relative direction
    local sin_yaw = math.sin(yaw)
    local cos_yaw = math.cos(yaw)
    
    local dx = (move_x * cos_yaw - move_z * sin_yaw) * state.movement_speed * dt
    local dz = (move_x * sin_yaw + move_z * cos_yaw) * state.movement_speed * dt
    
    local new_pos = {
        x = transform.translation.x + dx,
        y = transform.translation.y,
        z = transform.translation.z + dz,
    }
    
    -- Apply jump if grounded (simplified - no actual ground check)
    if jump then
        -- Would need physics integration for proper jumping
    end
    
    entity:set({
        Transform = {
            translation = new_pos,
            rotation = transform.rotation,
            scale = transform.scale,
        }
    })
end

--------------------------------------------------------------------------------
-- Initialization (self-registering at module load)
--------------------------------------------------------------------------------

-- Register systems at module load
register_system("Update", camera_attachment_system)
register_system("Update", movement_system)

-- Systems are registered but disabled by default - call enable() when ready
print("[PLAYER3] Systems registered (disabled by default, call enable())")

--- Enable player controller (call this when player entity is ready)
function PlayerController3.enable()
    local state = get_state()
    state.enabled = true
    print("[PLAYER3] Enabled")
end

--- Disable player controller
function PlayerController3.disable()
    local state = get_state()
    state.enabled = false
    print("[PLAYER3] Disabled")
end

--- Get the controlled player entity ID
--- @return number|nil
function PlayerController3.get_my_player()
    return get_state().my_player_entity
end

--- Set movement speed
--- @param speed number
function PlayerController3.set_movement_speed(speed)
    get_state().movement_speed = speed
end

return PlayerController3
