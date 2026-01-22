_G.registration_count = 0
require_async("scripts/reload_true_child.lua", function(Child)
    _G.registration_count = _G.registration_count + 1
    register_system("Update", function(world)
        _G.system_called = true
    end)
end, { reload = true })
