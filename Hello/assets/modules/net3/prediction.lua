-- Net3 Prediction System
-- Client-side prediction reconciliation for owned entities

local NetSync3 = require("modules/net3/init.lua")
local Movement = require("modules/shared/movement.lua")

local Prediction = {}

--- Prediction reconciliation system for own entity
--- @param world userdata
function Prediction.system(world)
    -- We need Transform for the state, PlayerState to get the last_seq acked by server,
    -- and PredictionState for our local buffer.
    -- Query for predicted entities
    local entities = world:query({
        with = { NetSync3.MARKER, NetSync3.PREDICTION, "Transform", "PlayerState", "ScriptOwned" }
    })
    for _, entity in ipairs(entities) do
        local sync = entity:get(NetSync3.MARKER)
        local pred = entity:get(NetSync3.PREDICTION)
        local player_state = entity:get("PlayerState")
        local transform = entity:get("Transform")
        
        -- Skip entities from other instances
        local script_owned = entity:get("ScriptOwned")
        if script_owned.instance_id ~= __INSTANCE_ID__ then
            goto continue_pred
        end
        
        -- Reconciliation happens when the server acknowledges a new sequence number
        local last_acked = player_state.last_seq or 0
        if last_acked > (pred.last_acked_sequence or 0) then
            -- 1. Check if the prediction for this sequence matches the server result
            local historical = pred.predictions[last_acked]
            
            if historical then
                local dx = transform.translation.x - historical.position.x
                local dy = transform.translation.y - historical.position.y
                local dz = transform.translation.z - historical.position.z
                local error_dist = math.sqrt(dx*dx + dy*dy + dz*dz)
                
                -- If error is above threshold, we need to re-simulate
                if error_dist > (pred.snap_threshold or 0.1) then
                    print(string.format("[PREDICTION] Recon error: %.3f at seq %d. Re-simulating...", error_dist, last_acked))
                    
                    -- Reset to server authoritative position
                    local current_pos = transform.translation
                    local current_rot = transform.rotation
                    
                    -- Start re-simulation from the server's acked state
                    -- We use the current server's transform as the basis
                    local sim_pos = current_pos
                    
                    -- 2. Re-apply all inputs that occurred AFTER the acked sequence
                    -- We sort the sequences to ensure they are applied in order
                    local pending_seqs = {}
                    for seq, _ in pairs(pred.predictions) do
                        if seq > last_acked then
                            table.insert(pending_seqs, seq)
                        end
                    end
                    table.sort(pending_seqs)
                    
                    -- Speed used for simulation (should be consistent)
                    local speed = 5.0 
                    
                    for _, seq in ipairs(pending_seqs) do
                        local data = pred.predictions[seq]
                        -- Note: This is an approximation since dt might be different
                        -- In a perfect system, we'd store the dt of each frame
                        local sim_dt = 1/60 -- Approximation or stored dt
                        
                        local temp_transform = { translation = sim_pos, rotation = current_rot }
                        sim_pos, current_rot = Movement.apply(world, temp_transform, data.input, speed, sim_dt, true)
                        
                        -- Update the pos and rot in the prediction buffer too
                        data.position = sim_pos
                        data.rotation = current_rot
                    end
                    
                    -- 3. Apply the final re-simulated position and rotation, only if changed meaningfully
                    local eps = 0.0001
                    local pos_changed = math.abs(sim_pos.x - transform.translation.x) > eps or
                                        math.abs(sim_pos.y - transform.translation.y) > eps or
                                        math.abs(sim_pos.z - transform.translation.z) > eps
                    
                    local rot_changed = math.abs(current_rot.x - transform.rotation.x) > eps or
                                        math.abs(current_rot.y - transform.rotation.y) > eps or
                                        math.abs(current_rot.z - transform.rotation.z) > eps or
                                        math.abs(current_rot.w - transform.rotation.w) > eps

                    if pos_changed or rot_changed then
                        -- Mark Transform as originating from the owner client (synchronous, for echo suppression)
                        State.mark_inbound_source(entity:id(), "Transform", sync.owner_client)

                        entity:patch({
                            Transform = {
                                translation = sim_pos,
                                rotation = current_rot
                            }
                        })
                    end
                end
            end
            
            -- Update local ack state
            pred.last_acked_sequence = last_acked
            
            -- Prune old predictions (keep predictions AFTER last_acked for reconciliation)
            local new_predictions = {}
            for seq, data in pairs(pred.predictions or {}) do
                if seq > last_acked then
                    new_predictions[seq] = data
                end
            end
            pred.predictions = new_predictions
            
            entity:patch({ [NetSync3.PREDICTION] = pred })
        end
        
        ::continue_pred::
    end
end

return Prediction
