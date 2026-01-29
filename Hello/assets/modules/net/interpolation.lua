-- NET Interpolation System
-- Smooth movement for remote entities

local Components = require("modules/net/components.lua")
local State = require("modules/net/state.lua")

local Interpolation = {}

--- Interpolation system for remote entities
--- @param world userdata
function Interpolation.system(world)
    local dt = world:delta_time()
    local entities = world:query({ Components.MARKER, Components.INTERPOLATION, "Transform", "ScriptOwned" })
    
    for _, entity in ipairs(entities) do
        local sync = entity:get(Components.MARKER)
        local target = entity:get(Components.INTERPOLATION)
        local transform = entity:get("Transform")
        
        -- Skip entities from other instances
        local script_owned = entity:get("ScriptOwned")
        if script_owned.instance_id ~= __INSTANCE_ID__ then
            goto continue_entities
        end
        
        local t = math.min(1.0, target.lerp_speed * dt)
        
        -- Calculate distance
        local dx = target.position.x - transform.translation.x
        local dy = target.position.y - transform.translation.y
        local dz = target.position.z - transform.translation.z
        local dist = math.sqrt(dx*dx + dy*dy + dz*dz)
        
        if dist > target.snap_threshold then
            -- Snap position
            entity:set({
                Transform = {
                    translation = target.position,
                    rotation = transform.rotation,
                    scale = target.scale or transform.scale,
                }
            })
        elseif dist > 0.001 then
            -- Lerp position and slerp rotation
            local current_rot = transform.rotation
            local target_rot = target.rotation or current_rot
            
            local new_rot = world:call_static_method("Quat", "slerp",
                current_rot,
                target_rot,
                t
            )
            
            -- Mark Transform as originating from the owner client (synchronous, for echo suppression)
            State.mark_inbound_source(entity:id(), "Transform", sync.owner_client)

            entity:patch({
                Transform = {
                    translation = {
                        x = transform.translation.x + dx * t,
                        y = transform.translation.y + dy * t,
                        z = transform.translation.z + dz * t,
                    },
                    rotation = new_rot,
                }
            })
        end
        
        ::continue_entities::
    end
end

return Interpolation
