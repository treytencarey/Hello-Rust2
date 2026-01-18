-- Conflux Server Networking Module
-- Game-specific network filtering configuration
--
-- NetworkModule Interface:
--   M.init()
--   M.filter_fn(world, client_id, net_id) → boolean

--------------------------------------------------------------------------------
-- Module
--------------------------------------------------------------------------------

local M = {}

-- State
local spatial_filter = nil
local initialized = false
local loading = false

function M.init()
    if initialized or loading then
        return
    end
    
    loading = true
    
    -- Load spatial filter via require_async
    require_async("modules/spatial_filters/quadrants.lua", function(module)
        if not module then
            print("[CONFLUX_NETWORKING] ERROR: Failed to load quadrants filter")
            loading = false
            return
        end
        
        -- Note: We don't pass floor_size or player_entities here
        -- The filter should query ECS directly for positions
        -- This keeps the module decoupled from game-specific state
        spatial_filter = module
        initialized = true
        loading = false
        print("[CONFLUX_NETWORKING] Quadrants filter loaded")
    end)
end

--- Filter function for area-of-interest
--- @param world userdata The ECS world
--- @param client_id number The client to check
--- @param net_id number The entity's network ID
--- @return boolean True if client should receive updates for this entity
function M.filter_fn(world, client_id, net_id)
    -- Ensure initialization started
    if not loading and not initialized then
        M.init()
    end
    
    -- If filter not ready, allow all (safe default)
    if not initialized or not spatial_filter then
        return true
    end
    
    -- Delegate to spatial filter
    if spatial_filter.filter_fn then
        return spatial_filter.filter_fn(world, client_id, net_id)
    end
    
    return true
end

--- Check if networking is ready
function M.is_ready()
    return initialized and spatial_filter ~= nil
end

print("[CONFLUX_NETWORKING] Module loaded")

return M
