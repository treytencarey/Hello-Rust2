-- NET Server Movement Authority
-- Processes PlayerInput from clients and applies authoritative movement

local NetSync = require("modules/net/init.lua")
local State = require("modules/net/state.lua")
local Movement = require("modules/shared/movement.lua")

local ServerMovement = {}

--- System that handles authoritative movement on the server
local function server_movement_system(world)
    -- Only run on server
    local state = NetSync.get_state()
    if state.mode ~= "server" then return end
    
    -- Query for players with input
    local entities = world:query({
        with = { NetSync.MARKER, "PlayerInput", "PlayerState", "Transform" },
    })
    
    for _, entity in ipairs(entities) do
        -- Skip entities with local authority (client is doing prediction in "both" mode)
        local sync = entity:get(NetSync.MARKER)
        if sync and sync.authority == "local" then
            goto continue_movement
        end

        local input = entity:get("PlayerInput")
        local player_state = entity:get("PlayerState")
        local transform = entity:get("Transform")
        
        -- Default movement speed (should ideally be in a component or resource)
        local speed = 5.0
        local dt = world:delta_time()
        
        -- Apply movement
        local move_config = { rotation_mode = input.rotation_mode or "face_movement" }
        local new_pos, new_rot = Movement.apply(world, transform, input, speed, dt, true, move_config)
        
        -- Update authoritative transform, only if changed meaningfully
        local eps = 0.0001
        local pos_changed = math.abs(new_pos.x - transform.translation.x) > eps or
                            math.abs(new_pos.y - transform.translation.y) > eps or
                            math.abs(new_pos.z - transform.translation.z) > eps
        
        local rot_changed = math.abs(new_rot.x - transform.rotation.x) > eps or
                            math.abs(new_rot.y - transform.rotation.y) > eps or
                            math.abs(new_rot.z - transform.rotation.z) > eps or
                            math.abs(new_rot.w - transform.rotation.w) > eps

        if pos_changed or rot_changed then
            -- Mark Transform as originating from the owner client (synchronous, for echo suppression)
            State.mark_inbound_source(entity:id(), "Transform", sync.owner_client)

            entity:patch({
                Transform = {
                    translation = new_pos,
                    rotation = new_rot,
                }
            })
        end
        
        -- Ack the sequence number so the client knows which input was processed
        -- Adding last_seq to PlayerState will sync it back to the client
        if input.sequence and input.sequence ~= player_state.last_seq then
            entity:patch({
                PlayerState = {
                    last_seq = input.sequence
                }
            })
        end

        ::continue_movement::
    end
end

-- Register system
register_system("Update", server_movement_system)

print("[SERVER_MOVEMENT] Authority system registered")

return ServerMovement
