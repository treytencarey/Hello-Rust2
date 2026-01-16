-- Parent module that requires child with reload=false
local child = require("./child.lua", { reload = false })

return {
    parent = true,
    child_data = child
}
