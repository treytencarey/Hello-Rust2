-- Net3 Prediction System
-- Client-side prediction reconciliation for owned entities

local Components = require("modules/net3/components.lua")

local Prediction = {}

--- Prediction reconciliation system for own entity
--- @param world userdata
function Prediction.system(world)
    local entities = world:query({ Components.PREDICTION, "Transform", "ScriptOwned" })
    
    for _, entity in ipairs(entities) do
        local pred = entity:get(Components.PREDICTION)
        
        -- Skip entities from other instances
        local script_owned = entity:get("ScriptOwned")
        if script_owned.instance_id ~= __INSTANCE_ID__ then
            goto continue_pred
        end
        
        -- Only process if we have server state to reconcile
        if not pred.server_state then
            goto continue_pred
        end
        
        local transform = entity:get("Transform")
        local entity_id = entity:id()
        
        -- Calculate error
        local dx = pred.server_state.position.x - transform.translation.x
        local dy = pred.server_state.position.y - transform.translation.y
        local dz = pred.server_state.position.z - transform.translation.z
        local error_dist = math.sqrt(dx*dx + dy*dy + dz*dz)
        
        if error_dist > pred.snap_threshold then
            -- Large error: snap to server position
            entity:set({
                Transform = {
                    translation = pred.server_state.position,
                    rotation = pred.server_state.rotation or transform.rotation,
                    scale = transform.scale,
                }
            })
        elseif error_dist > 0.01 then
            -- Small error: smooth correction
            local lerp_factor = math.min(0.3, error_dist * 0.8)
            entity:set({
                Transform = {
                    translation = {
                        x = transform.translation.x + dx * lerp_factor,
                        y = transform.translation.y + dy * lerp_factor,
                        z = transform.translation.z + dz * lerp_factor,
                    },
                    rotation = transform.rotation,
                    scale = transform.scale,
                }
            })
        end
        
        -- Clear server state after processing
        entity:patch({ [Components.PREDICTION] = { server_state = nil } })
        
        -- Prune old predictions
        if pred.last_acked_sequence then
            local new_predictions = {}
            for seq, data in pairs(pred.predictions or {}) do
                if seq > pred.last_acked_sequence then
                    new_predictions[seq] = data
                end
            end
            entity:patch({ [Components.PREDICTION] = { predictions = new_predictions } })
        end
        
        ::continue_pred::
    end
end

return Prediction
