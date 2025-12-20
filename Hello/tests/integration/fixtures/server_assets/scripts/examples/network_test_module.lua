-- Network test module for integration tests
-- This is the "root" script that loads other scripts via require

print("=== [TEST] network_test_module: Starting ===")
print("Instance ID: " .. tostring(__INSTANCE_ID__))

-- Sync require - these should reload when this script reloads
local asset_test = require("network_asset_test.lua", { network = true, reload = true })
local require_test = require("network_test_module_require.lua", { network = true, reload = true })

print("=== [TEST] network_test_module: All requires complete ===")
print("asset_test.name = " .. tostring(asset_test and asset_test.name or "nil"))
print("require_test.name = " .. tostring(require_test and require_test.name or "nil"))

return {
    name = "Test Root Module",
    version = "1.0",
    asset_test = asset_test,
    require_test = require_test
}
