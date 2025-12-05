-- Tilemap rendering using asset loading
-- Loads map_0.tmx and renders it with the actual tileset texture

-- Map configuration
local map_width = 64
local map_height = 64
local tile_size = 16
local scale = 2.0  -- Scale for visibility

-- Load the tileset image
local tileset = load_asset("tiled/Super_Retro_World_Interior_pack_week42/atlas_16x.png")
print("Loaded tileset texture")

-- Tileset configuration (from map_0.tmx)
local tileset_columns = 48
local tileset_rows = 16
local tileset_tile_width = 16
local tileset_tile_height = 16

-- No need for TextureAtlasLayout! We can use Sprite's rect field instead
-- This is pure Lua - no Rust code needed!
print("Using Sprite rect for tile slicing (pure Lua approach)")

-- Read and parse the CSV tile data from map_0.tmx
local function read_map_data()
    local file = io.open("assets/map_0.tmx", "r")
    if not file then
        error("Failed to open map_0.tmx")
    end
    
    local content = file:read("*all")
    file:close()
    
    local csv_start = content:find("<data encoding=\"csv\">")
    local csv_end = content:find("</data>")
    
    if not csv_start or not csv_end then
        error("Failed to find CSV data in map_0.tmx")
    end
    
    local csv_data = content:sub(csv_start + 22, csv_end - 1)
    
    local tiles = {}
    for line in csv_data:gmatch("[^\r\n]+") do
        for tile_str in line:gmatch("[^,]+") do
            local tile_id = tonumber(tile_str)
            if tile_id then
                table.insert(tiles, tile_id)
            end
        end
    end
    
    return tiles
end

local tile_data = read_map_data()
print(string.format("Loaded %d tiles from map_0.tmx", #tile_data))

local tiles_spawned = 0
local sample_size = 32  -- Show a 32x32 sample from center
local start_offset = (map_width - sample_size) / 2

for y = 0, sample_size - 1 do
    for x = 0, sample_size - 1 do
        -- Get tile from center of map
        local map_x = start_offset + x
        local map_y = start_offset + y
        local index = map_y * map_width + map_x + 1
        local tile_id = tile_data[index]
        
        if tile_id and tile_id > 0 then
            -- Tile IDs in Tiled are 1-indexed, convert to 0-indexed
            local texture_index = tile_id - 1
            
            -- Calculate world position (centered on screen)
            local world_x = (x - sample_size / 2) * tile_size * scale
            local world_y = (sample_size / 2 - y) * tile_size * scale
            
            -- Calculate tile position in atlas
            local tile_x = (texture_index % tileset_columns) * tileset_tile_width
            local tile_y = math.floor(texture_index / tileset_columns) * tileset_tile_height
            
            -- Use Sprite's rect field to slice the tileset - pure Lua approach!
            spawn({
                Sprite = {
                    image = tileset,
                    color = { r = 1.0, g = 1.0, b = 1.0, a = 1.0 },
                    rect = {
                        min = { x = tile_x, y = tile_y },
                        max = { x = tile_x + tileset_tile_width, y = tile_y + tileset_tile_height }
                    }
                },
                Transform = {
                    translation = { x = world_x, y = world_y, z = 0.0 },
                    rotation = { x = 0.0, y = 0.0, z = 0.0, w = 1.0 },
                    scale = { x = scale, y = scale, z = 1.0 }
                }
            })
            
            tiles_spawned = tiles_spawned + 1
        end
    end
end

print(string.format("Spawned %d tiles from map_0.tmx", tiles_spawned))
print("")
