
require_async("scripts/Conflux2/modules/net_game2.lua", function(NetGame)
    local NetClient2 = require("modules/net_client2.lua", { reload = false })
    
    local retry_timer = 8.0
    local current_timer = -1.0

    register_system("Update", function(world)
        local dt = world:delta_time()
        if current_timer < 0 or current_timer >= retry_timer then
            current_timer = 0
            print("[CONFLUX2] Joining game server on port 5001 (NetSync2)...")
            NetGame.join(world, {
                server_addr = "127.0.0.1",
                port = 5001
            })
        end
        current_timer = current_timer + dt

        if NetClient2.is_connected(world) then
            print("[CONFLUX2] Connected to game server on port 5001")
            return true  -- Success, unregister system
        end
    end)

    register_system("Update", function(world)
        NetGame.update(world)
    end)
end, { reload = false, instanced = true })
