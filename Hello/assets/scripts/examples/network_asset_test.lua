-- Network Asset Test Script
-- Tests load_asset_async with network=true and reload=true
-- Spawns text showing the script instance ID
-- When the image is updated on the server, the callback re-runs with a new instance ID

print("=== Network Asset Test: Starting ===")

local instance_id = __INSTANCE_ID__ or 0
print("Script Instance ID: " .. tostring(instance_id))

-- Spawn a camera so we can see the UI
spawn({
    Camera2d = {}
})

-- Spawn a test sprite with the atlas
local img_id = load_asset("images/test_image.png")
spawn({
    Sprite = {
        image = img_id,
        custom_size = {x = 200, y = 200}
    },
    Transform = {
        translation = {x = 00, y = 0, z = 0},
        rotation = {x = 0.0, y = 0.0, z = 0.0, w = 1.0},
        scale = {x = 1.0, y = 1.0, z = 1.0}
    }
})

-- -- Load an asset with network=true and reload=true
-- -- When the server has an updated version, the callback will re-run
-- load_asset_async("images/test_image.png", function(asset_id)
--     local new_instance_id = __INSTANCE_ID__ or 0
--     print("Asset loaded! Asset ID: " .. tostring(asset_id) .. ", Instance ID: " .. tostring(new_instance_id))
    
--     -- Update the text to show the current instance ID
--     local text_value = "Instance ID: " .. tostring(new_instance_id) .. "\nAsset ID: " .. tostring(asset_id)
    
--     -- Spawn new text with the updated instance ID
--     spawn({
--         Text = {
--             sections = {
--                 {
--                     value = text_value,
--                     style = {
--                         font_size = 64.0,
--                         color = { LinearRgba = { red = 0.0, green = 1.0, blue = 0.5, alpha = 1.0 } }
--                     }
--                 }
--             }
--         },
--         Node = {
--             position_type = "Absolute",
--             left = { Px = 100.0 },
--             top = { Px = 150.0 }
--         }
--     })
    
--     print("Text spawned with Instance ID: " .. tostring(new_instance_id))
-- end, { network = true, reload = true })

print("=== Network Asset Test: Async load queued ===")

return {
    name = "Network Asset Test",
    instance_id = instance_id
}
