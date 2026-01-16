-- Module with mutable state for testing caching behavior
local state = { count = 0 }

return {
    increment = function()
        state.count = state.count + 1
    end,
    get = function()
        return state.count
    end
}
