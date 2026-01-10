-- Character Sync Module
-- Character model loading with animation probing
--
-- Usage:
--   local CharacterSync = require("modules/character_sync.lua")
--   local entity = CharacterSync.create_character({
--       model_path = "Conflux/Placeholder-Character.glb",
--       position = {x = 0, y = 0, z = 0},
--       owner_client = client_id,  -- Server only
--       net_id = 12345
--   })

local NetRole = require("modules/net_role.lua")

local CharacterSync = {}

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

CharacterSync.config = {
    default_scale = {x = 1, y = 1, z = 1},
}

--------------------------------------------------------------------------------
-- Animation State
--------------------------------------------------------------------------------

--- Derive animation state from velocity
--- @param velocity table {x, y, z}
--- @return string "idle", "walk", or "run"
function CharacterSync.animation_from_velocity(velocity)
    local speed = math.sqrt((velocity.x or 0)^2 + (velocity.z or 0)^2)
    if speed < 0.1 then
        return "idle"
    elseif speed < 6 then
        return "walk"
    else
        return "run"
    end
end

--------------------------------------------------------------------------------
-- Character Creation
--------------------------------------------------------------------------------

--- Create a networked character entity
--- @param config table { model_path, position, rotation, scale, owner_client, net_id }
--- @return number entity_id
function CharacterSync.create_character(config)
    local model_path = config.model_path
    local position = config.position or {x = 0, y = 0, z = 0}
    local rotation = config.rotation or {x = 0, y = 0, z = 0, w = 1}
    local scale = config.scale or CharacterSync.config.default_scale
    local net_id = config.net_id
    local owner_client = config.owner_client
    
    local spawn_data = {
        Transform = {
            translation = position,
            rotation = rotation,
            scale = scale
        },
        -- Network sync component with per-component authority
        NetworkSync = {
            net_id = net_id,
            authority = owner_client and "server" or "owner",
            sync_components = {
                Transform = { rate_hz = 20, authority = "server", interpolate = true },
                PlayerState = { rate_hz = 20, reliable = true, authority = "server" },
                AnimationState = { rate_hz = 10, authority = "server" },
                PlayerInput = { rate_hz = 60, authority = "client" }  -- Client controls input
            }
        },
        -- Player state for server-authoritative movement
        -- Includes model_path so clients can load visuals
        PlayerState = {
            velocity = {x = 0, y = 0, z = 0},
            last_acked_seq = 0,
            owner_client = owner_client,
            model_path = model_path  -- Synced so clients know what model to load
        },
        -- Animation state (derived from velocity)
        AnimationState = {
            current = "idle"
        }
    }
    
    -- Add PlayerInput component for receiving client input (server needs this)
    if NetRole.is_server() then
        spawn_data.PlayerInput = {
            sequence = 0,
            forward = 0,
            right = 0,
            jump = false,
            sprint = false
        }
    end
    
    -- Add visuals if we're a client (or offline for testing)
    -- Note: In system callbacks, load_asset may return nil if download is needed
    -- In that case, the character spawns without visuals initially
    if NetRole.is_client() or NetRole.is_offline() then
        if model_path then
            local handle = load_asset(model_path)
            if handle then
                spawn_data.SceneRoot = { id = handle }
            else
                print(string.format("[CHARACTER_SYNC] Warning: Could not load model '%s' (async load may be in progress)", model_path))
            end
        end
    end
    
    local entity_id = spawn(spawn_data):id()
    
    print(string.format("[CHARACTER_SYNC] Created character at (%.1f, %.1f, %.1f) net_id=%s visuals=%s",
        position.x, position.y, position.z,
        tostring(net_id),
        tostring(NetRole.is_client() or NetRole.is_offline())))
    
    return entity_id
end

--------------------------------------------------------------------------------
-- Animation Updates
--------------------------------------------------------------------------------

--- Update animation state based on velocity
--- @param entity userdata
function CharacterSync.update_animation(entity)
    local player_state = entity:get("PlayerState")
    local anim_state = entity:get("AnimationState")
    
    if not player_state or not anim_state then return end
    
    local new_anim = CharacterSync.animation_from_velocity(player_state.velocity)
    
    if anim_state.current ~= new_anim then
        entity:set({
            AnimationState = { current = new_anim }
        })
    end
end

--- Create system function for animation updates
function CharacterSync.create_animation_system()
    return function(world)
        local characters = world:query({"PlayerState", "AnimationState"}, nil)
        for _, entity in ipairs(characters) do
            CharacterSync.update_animation(entity)
        end
    end
end

print("[CHARACTER_SYNC] Module loaded")

return CharacterSync
