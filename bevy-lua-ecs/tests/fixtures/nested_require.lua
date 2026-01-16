-- Module that requires another module using relative path
local inner = require("./inner.lua")

return {
    outer = true,
    inner_data = inner
}
