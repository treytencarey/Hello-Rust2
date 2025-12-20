-- Modified version for test
print("=== [TEST] network_test_module_require: Starting (MODIFIED) ===")
print("Instance ID: " .. tostring(__INSTANCE_ID__))

require_async("network_test_module_require_async.lua", function(async_mod)
    print("[TEST] require_async callback: loaded (MODIFIED)")
end, { network = true, reload = true })

print("=== [TEST] network_test_module_require: Done (MODIFIED) ===")

return {
    name = "Network Require Test (MODIFIED)",
    version = "2.0"
}
