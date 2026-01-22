require_async("scripts/server/Conflux2/modules/net_game2.lua", function(NetGame)
    register_system("Startup", function(world)
        print("[CONFLUX2 SERVER] Hosting game server on port 5001 (NetSync2)...")
        NetGame.host(world, {
            port = 5001
        })
        register_system("Update", function(world)
            NetGame.update(world)
        end)
        return true  -- Unregister after first run
    end)
end, { reload = false, instanced = true })