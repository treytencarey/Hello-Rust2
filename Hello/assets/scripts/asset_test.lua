-- Test script for generic asset creation
print("Testing generic asset creation...")

-- Create a TextureAtlasLayout
local atlas_id = create_asset("bevy_sprite::texture_atlas::TextureAtlasLayout", {
    tile_size = { x = 16, y = 16 },
    columns = 8,
    rows = 8
})

print("Created TextureAtlasLayout with ID:", atlas_id)

-- Load the tileset image
local tileset_id = load_asset("atlas_16x.png")
print("Loaded tileset with ID:", tileset_id)

-- Spawn a test sprite with the atlas
spawn({
    Transform = {
        translation = { x = 0, y = 0, z = 0 },
        scale = { x = 4, y = 4, z = 1 }
    },
    Sprite = {
        texture_atlas = atlas_id,
        index = 5,  -- Test with tile index 5
        image = tileset_id
    }
})

print("✓ Test complete!")
