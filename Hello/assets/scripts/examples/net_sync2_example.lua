-- NetSync2 Example Usage
-- Demonstrates the new ECS-based network synchronization system
--
-- Run with: --network server  (to host)
-- Run with: --network client  (to join)
-- Run with: --network both    (to host and join locally)

local NetGame2 = require("modules/net_game2.lua")
local NetSync2 = require("modules/net_sync2.lua")

--------------------------------------------------------------------------------
-- Example: Spatial Quadrant Filter
--------------------------------------------------------------------------------

--- Simple quadrant-based spatial filter
--- Only sends updates to clients in the same quadrant
local function quadrant_filter(world, client_id, entity, msg)
    if not msg.source_position then
        return true  -- No position info, allow
    end

    -- Get client's entity position (would need to look up by client_id)
    -- For this example, we'll just allow all
    -- In real usage, you'd track client positions
    return true
end

--------------------------------------------------------------------------------
-- Example: Validation Function
--------------------------------------------------------------------------------

--- Validate PlayerInput updates from clients
--- Ensures clients can only update their own entities with valid input values
local function validate_player_input(world, entity, old_value, new_value, owner_client)
    -- Check ownership
    local sync = entity:get(NetGame2.MARKER)
    if sync and sync.owner_client ~= owner_client then
        print(string.format("[VALIDATE] Rejected: client %s doesn't own entity", owner_client))
        return false, nil
    end

    -- Sanitize input values (clamp to valid range)
    local sanitized = {
        move_x = math.max(-1, math.min(1, new_value.move_x or 0)),
        move_z = math.max(-1, math.min(1, new_value.move_z or 0)),
        yaw = new_value.yaw or 0,
        jump = new_value.jump == true,
        sprint = new_value.sprint == true,
    }

    return true, sanitized
end

--------------------------------------------------------------------------------
-- Example: Spawn a Synced Player Entity
--------------------------------------------------------------------------------

local function spawn_player(client_id, spawn_pos)
    local net_id = NetGame2.next_net_id()

    local entity_id = spawn({
        -- Network sync marker with component configuration
        [NetGame2.MARKER] = {
            net_id = net_id,
            authority = "server",
            owner_client = client_id,
            sync_components = {
                Transform = {
                    rate_hz = 20,
                    authority = "server",
                    interpolate = true,  -- Remote entities interpolate
                },
                PlayerState = {
                    rate_hz = 20,
                    reliable = true,
                    authority = "server",
                },
                PlayerInput = {
                    rate_hz = 60,
                    authority = "client",  -- Client can update this
                    validation = validate_player_input,  -- Server validates
                },
            },
            -- Optional: per-entity filter
            -- filter_fn = quadrant_filter,
        },

        -- Transform component
        Transform = {
            translation = spawn_pos or { x = 0, y = 0, z = 0 },
            rotation = { x = 0, y = 0, z = 0, w = 1 },
            scale = { x = 1, y = 1, z = 1 },
        },

        -- Player state
        PlayerState = {
            velocity = { x = 0, y = 0, z = 0 },
            owner_client = client_id,
            health = 100,
        },

        -- Player input (client-authoritative)
        PlayerInput = {
            move_x = 0,
            move_z = 0,
            yaw = 0,
            jump = false,
            sprint = false,
        },
    }):id()

    -- Register ownership
    NetGame2.register_entity(net_id, entity_id)
    NetGame2.set_net_id_owner(net_id, client_id)

    -- Tell client which entity is theirs
    NetGame2.queue_your_character(client_id, net_id)

    print(string.format("[EXAMPLE] Spawned player %d for client %s at (%d,%d,%d)",
        net_id, client_id, spawn_pos.x, spawn_pos.y, spawn_pos.z))

    return entity_id, net_id
end

--------------------------------------------------------------------------------
-- Server Setup
--------------------------------------------------------------------------------

local function setup_server(world)
    local spawn_points = {
        { x = 0, y = 0, z = 0 },
        { x = 5, y = 0, z = 0 },
        { x = -5, y = 0, z = 0 },
        { x = 0, y = 0, z = 5 },
    }
    local next_spawn = 1
    local player_entities = {}  -- client_id -> entity_id

    NetGame2.host(world, {
        port = 5001,
        max_players = 4,

        -- Optional: global filter for all entities
        -- filter_fn = quadrant_filter,

        on_player_join = function(client_id)
            -- Spawn player at next spawn point
            local pos = spawn_points[next_spawn]
            next_spawn = (next_spawn % #spawn_points) + 1

            local entity_id, net_id = spawn_player(client_id, pos)
            player_entities[client_id] = entity_id
        end,

        on_player_leave = function(client_id)
            -- Entity despawn is handled automatically by NetSync2
            -- when the entity is despawned (which triggers despawn messages)
            local entity_id = player_entities[client_id]
            if entity_id then
                despawn(entity_id)
                player_entities[client_id] = nil
            end
        end,
    })

    print("[EXAMPLE] Server setup complete")
end

--------------------------------------------------------------------------------
-- Client Setup
--------------------------------------------------------------------------------

local function setup_client(world)
    NetGame2.join(world, {
        server_addr = "127.0.0.1",
        port = 5001,

        on_connected = function()
            print("[EXAMPLE] Connected to server!")
        end,

        on_disconnected = function()
            print("[EXAMPLE] Disconnected from server")
        end,
    })

    print("[EXAMPLE] Client setup complete")
end

--------------------------------------------------------------------------------
-- Main Entry Point
--------------------------------------------------------------------------------

-- Check command line args for network mode
local args = get_args and get_args() or {}
local network_mode = "offline"

for i, arg in ipairs(args) do
    if arg == "--network" and args[i + 1] then
        network_mode = args[i + 1]
    end
end

print(string.format("[EXAMPLE] Network mode: %s", network_mode))

-- Startup system - runs once
register_system("Startup", function(world)
    if network_mode == "server" or network_mode == "both" then
        setup_server(world)
    end

    if network_mode == "client" then
        setup_client(world)
    end

    -- In "both" mode, client setup would be in a separate instanced script
    -- See scripts/Conflux2/ for example of both mode setup

    return true  -- One-shot system
end)

-- Update system - runs every frame
register_system("Update", function(world)
    NetGame2.update(world)

    -- Example: Update player input from keyboard (client-side)
    if NetGame2.is_client() then
        local my_net_id = NetGame2.get_my_net_id()
        if my_net_id then
            local entity_id = NetGame2.get_entity(my_net_id)
            if entity_id then
                local entity = world:get_entity(entity_id)
                if entity then
                    -- Read keyboard input (example)
                    local input = entity:get("PlayerInput") or {}

                    -- Update input based on keyboard state
                    -- (In real code, use InputManager or ButtonInput resource)
                    -- input.move_x = ...
                    -- input.move_z = ...

                    -- Only patch if changed
                    -- entity:patch({ PlayerInput = input })
                end
            end
        end
    end

    -- Example: Server-side movement processing
    if NetGame2.is_server() then
        -- Query all entities with PlayerInput and Transform
        local players = world:query({NetGame2.MARKER, "PlayerInput", "Transform"})

        for _, entity in ipairs(players) do
            local sync = entity:get(NetGame2.MARKER)
            if sync.authority ~= "remote" then
                local input = entity:get("PlayerInput")
                local transform = entity:get("Transform")
                local dt = world:delta_time()

                -- Simple movement
                local speed = 5.0
                local new_x = transform.translation.x + input.move_x * speed * dt
                local new_z = transform.translation.z + input.move_z * speed * dt

                entity:set({
                    Transform = {
                        translation = { x = new_x, y = transform.translation.y, z = new_z },
                        rotation = transform.rotation,
                        scale = transform.scale,
                    }
                })
            end
        end
    end
end)

print("[EXAMPLE] NetSync2 example loaded")
