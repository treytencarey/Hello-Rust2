-- Spawn System Module
-- Manages SpawnLocation entities for player spawning
--
-- Usage (Server):
--   local SpawnSystem = require("modules/spawn_system.lua")
--   SpawnSystem.init({
--       locations = {{x=0, y=1, z=0}, {x=5, y=1, z=0}},
--       show_debug = false
--   })
--   local pos = SpawnSystem.claim_spawn(client_id)
--   SpawnSystem.release_spawn(client_id)

local NetRole = require("modules/net_role.lua")

local SpawnSystem = {}

--------------------------------------------------------------------------------
-- State
--------------------------------------------------------------------------------

local spawn_locations = {}  -- index -> { position = {x,y,z}, occupied_by = client_id or nil }
local client_to_spawn = {}  -- client_id -> spawn_index
local initialized = false
local debug_markers = {}    -- spawn_index -> entity_id (if show_debug)

--------------------------------------------------------------------------------
-- Initialization
--------------------------------------------------------------------------------

--- Initialize spawn system with locations
--- @param config table { locations = {{x,y,z}, ...}, show_debug = false }
function SpawnSystem.init(config)
    if initialized then
        print("[SPAWN_SYSTEM] Already initialized, skipping")
        return
    end
    
    local locations = config.locations or {}
    local show_debug = config.show_debug or false
    
    for i, pos in ipairs(locations) do
        spawn_locations[i] = {
            position = pos,
            occupied_by = nil
        }
        
        -- Spawn debug markers if enabled (client-only visuals)
        if show_debug and (NetRole.is_client() or NetRole.is_offline()) then
            local marker_mesh = create_asset("bevy_mesh::mesh::Mesh", {
                primitive = { Cylinder = { radius = 0.5, half_height = 0.1 } }
            })
            local marker_material = create_asset("bevy_pbr::pbr_material::StandardMaterial", {
                base_color = { r = 0.2, g = 0.6, b = 1.0, a = 0.5 },
                alpha_mode = "Blend"
            })
            
            debug_markers[i] = spawn({
                Mesh3d = { _0 = marker_mesh },
                ["MeshMaterial3d<StandardMaterial>"] = { _0 = marker_material },
                Transform = {
                    translation = { x = pos.x, y = pos.y + 0.1, z = pos.z },
                    rotation = { x = 0, y = 0, z = 0, w = 1 },
                    scale = { x = 1, y = 1, z = 1 }
                }
            })
        end
    end
    
    initialized = true
    print(string.format("[SPAWN_SYSTEM] Initialized with %d spawn locations (debug=%s)",
        #locations, tostring(show_debug)))
end

--------------------------------------------------------------------------------
-- Spawn Management (Server)
--------------------------------------------------------------------------------

--- Claim a spawn location for a client
--- @param client_id number
--- @return table|nil position {x,y,z} or nil if all occupied
function SpawnSystem.claim_spawn(client_id)
    -- Check if client already has a spawn
    if client_to_spawn[client_id] then
        local idx = client_to_spawn[client_id]
        local loc = spawn_locations[idx]
        if loc then
            return loc.position
        end
    end
    
    -- Find first available spawn
    for i, loc in ipairs(spawn_locations) do
        if not loc.occupied_by then
            loc.occupied_by = client_id
            client_to_spawn[client_id] = i
            print(string.format("[SPAWN_SYSTEM] Client %d claimed spawn %d at (%.1f, %.1f, %.1f)",
                client_id, i, loc.position.x, loc.position.y, loc.position.z))
            return loc.position
        end
    end
    
    print(string.format("[SPAWN_SYSTEM] No available spawns for client %d", client_id))
    return nil
end

--- Release spawn location for a client
--- @param client_id number
function SpawnSystem.release_spawn(client_id)
    local idx = client_to_spawn[client_id]
    if idx and spawn_locations[idx] then
        spawn_locations[idx].occupied_by = nil
        print(string.format("[SPAWN_SYSTEM] Client %d released spawn %d", client_id, idx))
    end
    client_to_spawn[client_id] = nil
end

--- Get spawn position for a client (if they have one)
--- @param client_id number
--- @return table|nil position {x,y,z}
function SpawnSystem.get_spawn_for(client_id)
    local idx = client_to_spawn[client_id]
    if idx and spawn_locations[idx] then
        return spawn_locations[idx].position
    end
    return nil
end

--- Get total number of spawn locations
function SpawnSystem.get_count()
    return #spawn_locations
end

--- Get number of available spawn locations
function SpawnSystem.get_available_count()
    local count = 0
    for _, loc in ipairs(spawn_locations) do
        if not loc.occupied_by then
            count = count + 1
        end
    end
    return count
end

--- Check if system is initialized
function SpawnSystem.is_initialized()
    return initialized
end

print("[SPAWN_SYSTEM] Module loaded")

return SpawnSystem
