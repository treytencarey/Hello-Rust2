-- Conflux Spawn Points Configuration
-- Server-only: defines spawn locations for player spawning
--
-- Usage:
--   require("scripts/Conflux/spawn_points.lua")
--   -- SpawnSystem is automatically initialized

local SpawnSystem = require("modules/spawn_system.lua")

-- 4 spawn locations arranged in a square pattern
local SPAWN_LOCATIONS = {
    {x = -5, y = 1, z = -5},
    {x = 5, y = 1, z = -5},
    {x = -5, y = 1, z = 5},
    {x = 5, y = 1, z = 5},
}

SpawnSystem.init({
    locations = SPAWN_LOCATIONS,
    show_debug = false  -- Set to true to show spawn markers on clients
})

print("[CONFLUX] Spawn points initialized")
