-- Network asset test script for integration tests
-- Tests load_asset with network=true, reload=true

print("=== [TEST] network_asset_test: Starting ===")
print("Instance ID: " .. tostring(__INSTANCE_ID__))

-- Load an image asset with reload=true - this should trigger script reload when image updates
local img_id = load_asset("images/test_image.png", { network = true, reload = true })
print("[TEST] Loaded image with ID: " .. tostring(img_id))

-- Spawn a camera so we can see the UI
spawn({ Camera2d = {} })

-- Spawn a sprite with the loaded image
spawn({
    Sprite = {
        image = img_id,
        custom_size = {x = 200, y = 200}
    },
    Transform = {
        translation = {x = 0, y = 0, z = 0},
        rotation = {x = 0.0, y = 0.0, z = 0.0, w = 1.0},
        scale = {x = 1.0, y = 1.0, z = 1.0}
    }
})

print("=== [TEST] network_asset_test: Done ===")

return {
    name = "Network Asset Test",
    image_id = img_id
}
