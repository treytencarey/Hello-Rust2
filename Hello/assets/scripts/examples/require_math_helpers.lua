-- Helper module with math utilities
local M = {}

-- Add two numbers
function M.add(a, b)
    return a + b
end

-- Multiply two numbers
function M.multiply(a, b)
    return a * b
end

-- Calculate average of a table of numbers
function M.average(numbers)
    if #numbers == 0 then return 0 end
    local sum = 0
    for _, n in ipairs(numbers) do
        sum = sum + n
    end
    return sum / #numbers
end

-- Clamp a value between min and max
function M.clamp(value, min, max)
    if value < min then return min end
    if value > max then return max end
    return value
end

-- Return the module table
return M
