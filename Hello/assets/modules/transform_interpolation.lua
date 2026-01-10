-- Transform Interpolation Module
-- Smoothly interpolates remote entity transforms toward network targets
--
-- Usage:
--   local Interp = require("modules/transform_interpolation.lua")
--   register_system("Update", Interp.create_system())

local NetSync = require("modules/net_sync.lua")

local TransformInterpolation = {}

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

TransformInterpolation.config = {
    lerp_speed = 15.0,      -- How fast to interpolate (higher = snappier)
    snap_threshold = 5.0,   -- Snap if distance exceeds this
}

--------------------------------------------------------------------------------
-- Interpolation System
--------------------------------------------------------------------------------

--- Create the interpolation system
--- @return function System function for register_system
function TransformInterpolation.create_system()
    return function(world)
        local dt = world:delta_time()
        local targets = NetSync.get_interpolation_targets()
        
        for entity_id, target in pairs(targets) do
            local entity = world:get_entity(entity_id)
            if not entity then
                -- Clean up stale target
                targets[entity_id] = nil
                goto continue
            end
            
            local transform = entity:get("Transform")
            if not transform then goto continue end
            
            local current = transform.translation
            local target_pos = target.position
            
            -- Calculate distance
            local dx = target_pos.x - current.x
            local dy = target_pos.y - current.y
            local dz = target_pos.z - current.z
            local dist = math.sqrt(dx*dx + dy*dy + dz*dz)
            
            -- Snap if too far, otherwise lerp
            if dist > TransformInterpolation.config.snap_threshold then
                -- Even when snapping position, slerp rotation for smoother visuals
                local snap_rotation = transform.rotation
                if target.rotation then
                    snap_rotation = world:call_static_method("Quat", "slerp",
                        transform.rotation,
                        target.rotation,
                        0.5  -- Faster rotation slerp when snapping
                    )
                end
                entity:set({ Transform = {
                    translation = target_pos,
                    rotation = snap_rotation,
                    scale = target.scale or transform.scale
                }})
            elseif dist > 0.001 then
                -- Lerp toward target
                local t = math.min(1, TransformInterpolation.config.lerp_speed * dt)
                
                -- Interpolate rotation using slerp if target rotation provided
                local new_rotation = transform.rotation
                if target.rotation then
                    new_rotation = world:call_static_method("Quat", "slerp",
                        transform.rotation,
                        target.rotation,
                        t
                    )
                end
                
                entity:set({ Transform = {
                    translation = {
                        x = current.x + dx * t,
                        y = current.y + dy * t,
                        z = current.z + dz * t,
                    },
                    rotation = new_rotation,
                    scale = target.scale or transform.scale
                }})
            end
            
            ::continue::
        end
    end
end

print("[TRANSFORM_INTERPOLATION] Module loaded")

return TransformInterpolation
