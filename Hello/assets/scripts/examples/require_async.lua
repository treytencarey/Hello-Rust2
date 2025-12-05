local order = 0

spawn({
    Camera2d = {} --
})

spawn({
    Text2d = { text = order .. ": " ..__INSTANCE_ID__ },
    Transform = { 
        translation = {x = 0, y = 24 * order, z = 0},
        rotation = {x = 0, y = 0, z = 0, w = 1},
        scale = {x = 1, y = 1, z = 1}
    }
})

order = 1

require_async("require_async_2.lua", function()
    spawn({
        Text2d = { text = order .. ": " ..__INSTANCE_ID__ },
        Transform = { 
            translation = {x = 0, y = 24 * order, z = 0},
            rotation = {x = 0, y = 0, z = 0, w = 1},
            scale = {x = 1, y = 1, z = 1}
        }
    })
end, { reload = true })