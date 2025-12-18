-- A simple test module that will be served by the server
-- This file should exist on the SERVER but NOT on the client

print("=== Network Test Module: Starting.... ===")

require("network_asset_test.lua", { network = true })
require("network_test_module_require.lua", { network = true })

print("=== Network Test Module: Done! ===")

return {
    name = "Remote Test Module",
    version = "1.0",
    
    greet = function(name)
        return "Hello, " .. tostring(name) .. "! This module was downloaded from the asset server."
    end,
    
    get_message = function()
        return "You successfully downloaded this module via Renet!"
    end
}
