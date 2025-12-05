-- Colors helper module
local M = {}

require_async("require_math_helpers.lua", function(math_helpers)
    spawn({
        Text2d = { text = "Color module loaded after math result: " .. math_helpers.add(1, 5) },
        TextFont = { font_size = 24 },
        TextColor = { color = {r = 0.5, g = 0.5, b = 1.0, a = 1.0} },
        Transform = { 
            translation = {x = 100, y = 100, z = 0},
            rotation = {x = 0, y = 0, z = 0, w = 1},
            scale = {x = 1, y = 1, z = 1}
        }
    })
end, { reload = true })

-- Common color presets
M.colors = {
    red = {r = 1.0, g = 0.0, b = 0.0, a = 1.0},
    green = {r = 0.0, g = 1.0, b = 0.0, a = 1.0},
    blue = {r = 0.0, g = 0.0, b = 1.0, a = 1.0},
    yellow = {r = 1.0, g = 1.0, b = 0.0, a = 1.0},
    cyan = {r = 0.0, g = 1.0, b = 1.0, a = 1.0},
    magenta = {r = 1.0, g = 0.0, b = 1.0, a = 1.0},
    white = {r = 1.0, g = 1.0, b = 1.0, a = 1.0},
    black = {r = 0.0, g = 0.0, b = 0.0, a = 1.0},
}

-- Mix two colors
function M.mix(color1, color2, ratio)
    ratio = ratio or 0.5
    ratio = ratio + .5
    return {
        r = color1.r * (1 - ratio) + color2.r * ratio,
        g = color1.g * (1 - ratio) + color2.g * ratio,
        b = color1.b * (1 - ratio) + color2.b * ratio,
        a = color1.a * (1 - ratio) + color2.a * ratio,
    }
end

-- Lighten a color
function M.lighten(color, amount)
    amount = amount or 0.2
    return M.mix(color, M.colors.white, amount)
end

-- Darken a color
function M.darken(color, amount)
    amount = amount or 0.2
    return M.mix(color, M.colors.black, amount)
end

return M
