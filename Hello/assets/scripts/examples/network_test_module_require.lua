-- A simple test module that will be served by the server
-- This file should exist on the SERVER but NOT on the client

print("=== Network Test Module - Require: Starting.... ===")

require_async("network_test_module_require_async.lua", function()
    print("network_test_module_require_async loaded")
end, { network = true, reload = true })

print("=== Network Test Module - Require: Done! ===")

return {
    name = "Remote Test Module - Require",
    version = "1.0",
    
    greet = function(name)
        return "Hello, " .. tostring(name) .. "! This module was downloaded from the asset server."
    end,
    
    get_message = function()
        return "You successfully downloaded this module via Renet!"
    end
}
