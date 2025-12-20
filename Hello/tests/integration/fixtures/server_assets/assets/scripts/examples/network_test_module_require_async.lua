-- Network test module require async script for integration tests
-- This module is loaded via require_async and reloads independently

print("=== [TEST] network_test_module_require_async: Loaded ===")
print("Instance ID: " .. tostring(__INSTANCE_ID__))

return {
    name = "Network Async Require Module",
    version = "1.0",
    message = "This module was loaded asynchronously"
}
