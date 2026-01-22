-- Spatial Filter: Quadrants (NetSync2 compatible)
-- Divides the play area into 4 quadrants, only syncing entities in the same quadrant
--
-- Uses ECS queries to find client positions (no external state required)

local NetSync2 = require("modules/net_sync2.lua")

local SpatialQuadrants = {}

--- Initialize (no config needed - uses ECS queries)
function SpatialQuadrants.init(config)
    print("[SPATIAL_QUADRANTS2] Initialized (ECS-based)")
end

--- Get quadrant from position
local function get_quadrant(position)
    if position.x < 0 and position.z < 0 then
        return "NW"
    elseif position.x >= 0 and position.z < 0 then
        return "NE"
    elseif position.x < 0 and position.z >= 0 then
        return "SW"
    else
        return "SE"
    end
end

--- Find the entity owned by a specific client
--- @param world userdata
--- @param client_id number
--- @return userdata|nil entity
local function find_client_entity(world, client_id)
    -- Query all entities with NetworkSync and Transform
    local entities = world:query({NetSync2.MARKER, "Transform"})

    for _, entity in ipairs(entities) do
        local sync = entity:get(NetSync2.MARKER)
        if sync and sync.owner_client == client_id then
            return entity
        end
    end

    return nil
end

--- Filter function: only sync entities in the same quadrant
--- @param world userdata
--- @param client_id number
--- @param net_id number
--- @return boolean
function SpatialQuadrants.filter_fn(world, client_id, net_id)
    -- Find client's entity via ECS query
    local client_entity = find_client_entity(world, client_id)
    if not client_entity then
        -- Client has no entity yet, allow all updates
        return true
    end

    -- Find target entity by net_id
    local target_entity_id = NetSync2.get_entity(net_id)
    if not target_entity_id then
        return false
    end

    local target_entity = world:get_entity(target_entity_id)
    if not target_entity then
        return false
    end

    local client_transform = client_entity:get("Transform")
    local target_transform = target_entity:get("Transform")

    if not client_transform or not target_transform then
        return false
    end

    local client_quad = get_quadrant(client_transform.translation)
    local target_quad = get_quadrant(target_transform.translation)

    print(string.format("[SPATIAL_QUADRANTS2] Client %d quadrant: %s, Target quadrant: %s", client_id, client_quad, target_quad))
    return client_quad == target_quad
end

return SpatialQuadrants
