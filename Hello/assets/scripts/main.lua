-- Hello Game-- Main entry point for Hello
-- This is your game's main Lua script

-- -- Networking (connect to server for entity replication)
-- local NetClient = require("modules/net_client.lua")
-- NetClient.connect("127.0.0.1", 5000)

-- -- Register network update system
-- register_system("Update", function(world)
--     NetClient.update(world)
-- end)

-- -- VR UI support (auto-detects and wraps UI nodes as VR panels)
-- require("modules/vr_ui.lua")

-- require("scripts/examples/ufbx.lua")

-- require("scripts/Conflux2/main.lua")

-- local FPS = require("modules/fps.lua")
-- FPS.setup()

-- require("scripts/examples/sidebar_example.lua")

-- Test Net3 - Conflux3
require("scripts/Conflux3/main.lua")

-- -- Disable VSync for higher FPS
-- local VSync = require("modules/VSync.lua")
-- VSync.disable()
