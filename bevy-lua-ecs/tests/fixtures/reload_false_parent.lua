_G.system_called = false
require_async("scripts/reload_false_child.lua", function(Child)
    register_system("Update", function(world)
        _G.system_called = true
    end)
end, { reload = false })
