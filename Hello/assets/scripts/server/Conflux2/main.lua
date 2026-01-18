require_async("scripts/server/Conflux2/modules/net_game.lua", function(NetGame)
    register_system("Startup", function(world)
        print("[CONFLUX SERVER] Hosting game server on port 5001...")
        NetGame.host(world, {
            port = 5001
        })
        register_system("Update", function(world)
            NetGame.update(world)
        end)
        return true -- Unregister after first run
    end)
end, { reload = false, instanced = true }) -- Do not recreate the server on module reload
