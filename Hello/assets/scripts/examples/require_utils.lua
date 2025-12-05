-- Utils module that imports other modules (nested requires)
local math_helpers = require("require_math_helpers.lua")
local color_helpers = require("require_color_helpers.lua")

local M = {}

-- Expose sub-modules
M.math = math_helpers
M.color = color_helpers

-- Combined utility: create a colored square at a position
function M.create_square(x, y, size, color_name)
    local color = color_helpers.colors[color_name] or color_helpers.colors.white
    
    spawn({
        Sprite = { 
            color = color,
            custom_size = {x = size, y = size}
        },
        Transform = { 
            translation = {x = x, y = y, z = 0},
            rotation = {x = 0, y = 0, z = 0, w = 1},
            scale = {x = 1, y = 1, z = 1}
        }
    })
end

-- Combined utility: create a grid of squares
function M.create_grid(rows, cols, spacing, base_color_name)
    local base_color = color_helpers.colors[base_color_name] or color_helpers.colors.blue
    local start_x = -(cols - 1) * spacing / 2
    local start_y = (rows - 1) * spacing / 2
    
    for row = 0, rows - 1 do
        for col = 0, cols - 1 do
            local x = start_x + col * spacing
            local y = start_y - row * spacing
            
            -- Vary the color slightly for each cell
            local ratio = (row + col) / (rows + cols)
            local cell_color = color_helpers.mix(base_color, color_helpers.colors.white, ratio * 0.5)
            
            spawn({
                Sprite = { 
                    color = cell_color,
                    custom_size = {x = spacing * 0.8, y = spacing * 0.8}
                },
                Transform = { 
                    translation = {x = x, y = y, z = 0},
                    rotation = {x = 0, y = 0, z = 0, w = 1},
                    scale = {x = 1, y = 1, z = 1}
                }
            })
        end
    end
end

return M
