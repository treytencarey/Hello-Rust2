-- Stateful module for testing instanced isolation
local state = { value = 0 }

return {
    set = function(v)
        state.value = v
    end,
    get = function()
        return state.value
    end
}
