use bevy::prelude::*;
use bevy_lua_ecs::*;
use mlua::prelude::*;

#[cfg(feature = "networking")]
use renet::{RenetClient, RenetServer};

// Renet channel IDs for Lua events
#[cfg(feature = "networking")]
pub const LUA_EVENTS_CHANNEL: u8 = 4;

/// Queue for Lua events to be sent over Renet
#[cfg(feature = "networking")]
#[derive(Resource, Default)]
pub struct LuaEventQueue {
    pub events: Vec<serde_json::Value>,
}

/// Resource to store received Lua events
#[cfg(feature = "networking")]
#[derive(Resource, Default)]
pub struct ReceivedLuaEvents {
    pub events: Vec<serde_json::Value>,
}

/// Set up Lua bindings for Renet messaging
#[cfg(feature = "networking")]
pub fn setup_renet_lua_bindings(
    lua_ctx: Res<LuaScriptContext>,
) {
    let lua = &lua_ctx.lua;
    
    // Create JSON encoder and Lua messaging functions
    if let Err(e) = lua.load(r#"
        -- Simple JSON decoder for basic structures
        function _json_decode(json_str)
            -- Remove outer braces
            local content = json_str:match("^%s*{(.+)}%s*$")
            if not content then return {} end
            
            local result = {}
            -- Parse key-value pairs
            for key, value in content:gmatch('"([^"]+)"%s*:%s*([^,}]+)') do
                -- Trim whitespace
                value = value:match("^%s*(.-)%s*$")
                
                -- Parse value type
                if value:match('^".*"$') then
                    -- String value
                    result[key] = value:sub(2, -2)
                elseif value == "true" then
                    result[key] = true
                elseif value == "false" then
                    result[key] = false
                elseif tonumber(value) then
                    result[key] = tonumber(value)
                else
                    result[key] = value
                end
            end
            return result
        end
        
        -- Simple JSON encoder for Lua tables
        function _json_encode(tbl)
            local function encode_value(val)
                local t = type(val)
                if t == "table" then
                    local result = "{"
                    local first = true
                    for k, v in pairs(val) do
                        if not first then result = result .. "," end
                        first = false
                        result = result .. '"' .. tostring(k) .. '":' .. encode_value(v)
                    end
                    return result .. "}"
                elseif t == "string" then
                    return '"' .. val:gsub('"', '\\"') .. '"'
                elseif t == "number" or t == "boolean" then
                    return tostring(val)
                else
                    return "null"
                end
            end
            return encode_value(tbl)
        end
        
        -- Initialize global event queue
        _G._lua_event_queue = _G._lua_event_queue or {}
        _G._received_lua_events = _G._received_lua_events or {}
        
        -- Extend world metatable with Renet messaging methods
        if not _G._world_mt then
            _G._world_mt = {}
            _G._world_mt.__index = _G._world_mt
        end
        
        -- Send event to server (or broadcast if on server)
        _G._world_mt.send_network_event = function(self, data)
            local json_str = _json_encode(data)
            table.insert(_G._lua_event_queue, json_str)
        end
        
        -- Alias for clarity
        _G._world_mt.broadcast_event = function(self, data)
            return self:send_network_event(data)
        end
        
        -- Read received network events
        _G._world_mt.read_network_events = function(self)
            local events = _G._received_lua_events
            _G._received_lua_events = {}
            
            -- Parse JSON strings back to Lua tables
            local result = {}
            for i, json_str in ipairs(events) do
                local parsed = _json_decode(json_str)
                table.insert(result, parsed)
            end
            
            return result
        end
    "#).exec() {
        error!("Failed to set up Renet Lua bindings: {}", e);
        return;
    }
    
    info!("✓ Registered Renet Lua bindings");
}

/// System to sync Lua event queue to Bevy resource
#[cfg(feature = "networking")]
pub fn sync_lua_events_to_queue(
    lua_ctx: Res<LuaScriptContext>,
    mut event_queue: ResMut<LuaEventQueue>,
) {
    let lua = &lua_ctx.lua;
    
    if let Ok(lua_queue) = lua.globals().get::<LuaTable>("_lua_event_queue") {
        for json_str in lua_queue.sequence_values::<String>() {
            if let Ok(s) = json_str {
                if let Ok(json_value) = serde_json::from_str(&s) {
                    event_queue.events.push(json_value);
                }
            }
        }
        // Clear Lua queue
        let _ = lua.globals().set("_lua_event_queue", lua.create_table().ok());
    }
}

/// System to send queued Lua events via Renet
#[cfg(feature = "networking")]
pub fn send_lua_events_renet(
    mut event_queue: ResMut<LuaEventQueue>,
    mut server: Option<ResMut<RenetServer>>,
    mut client: Option<ResMut<RenetClient>>,
) {
    if event_queue.events.is_empty() {
        return;
    }

    debug!("📤 SEND: {} events in queue", event_queue.events.len());

    for event_data in event_queue.events.drain(..) {
        // Serialize to JSON string instead of bincode
        let json_str = serde_json::to_string(&event_data).unwrap_or_default();
        let message = json_str.as_bytes().to_vec();
        let msg_len = message.len();
        
        if let Some(ref mut server) = server {
            // Server: broadcast to all clients
            server.broadcast_message(LUA_EVENTS_CHANNEL, message);
            debug!("  → Server broadcast {} bytes (JSON)", msg_len);
        } else if let Some(ref mut client) = client {
            // Client: send to server
            client.send_message(LUA_EVENTS_CHANNEL, message);
            debug!("  → Client sent {} bytes to server (JSON)", msg_len);
        }
    }
}

/// System to receive Lua events via Renet
#[cfg(feature = "networking")]
pub fn receive_lua_events_renet(
    mut server: Option<ResMut<RenetServer>>,
    mut client: Option<ResMut<RenetClient>>,
    mut received: ResMut<ReceivedLuaEvents>,
) {
    if let Some(ref mut server) = server {
        // Server: receive from clients
        for client_id in server.clients_id() {
            while let Some(message) = server.receive_message(client_id, LUA_EVENTS_CHANNEL) {
                debug!("📥 SERVER RECEIVED: {} bytes from Client {}", message.len(), client_id);
                // Deserialize JSON string
                if let Ok(json_str) = String::from_utf8(message.to_vec()) {
                    if let Ok(event_data) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        debug!("  ✓ Deserialized: {:?}", event_data);
                        received.events.push(event_data.clone());
                        debug!("  ✓ Added to queue (total: {})", received.events.len());
                        // Echo back to all clients
                        let response = json_str.as_bytes().to_vec();
                        server.broadcast_message(LUA_EVENTS_CHANNEL, response);
                    } else {
                        error!("  ✗ JSON PARSE FAILED: {}", json_str);
                    }
                } else {
                    error!("  ✗ UTF-8 DECODE FAILED");
                }
            }
        }
    } else if let Some(ref mut client) = client {
        // Client: receive from server
        while let Some(message) = client.receive_message(LUA_EVENTS_CHANNEL) {
            debug!("📥 CLIENT RECEIVED: {} bytes from Server", message.len());
            // Deserialize JSON string
            if let Ok(json_str) = String::from_utf8(message.to_vec()) {
                if let Ok(event_data) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    debug!("  ✓ Deserialized: {:?}", event_data);
                    received.events.push(event_data);
                    debug!("  ✓ Added to queue (total: {})", received.events.len());
                } else {
                    error!("  ✗ JSON PARSE FAILED: {}", json_str);
                }
            } else {
                error!("  ✗ UTF-8 DECODE FAILED");
            }
        }
    }
}

/// System to sync received events from Bevy to Lua
#[cfg(feature = "networking")]
pub fn sync_received_events_to_lua(
    lua_ctx: Res<LuaScriptContext>,
    mut received: ResMut<ReceivedLuaEvents>,
) {
    debug!("🔄 sync_received_events_to_lua CALLED (events: {})", received.events.len());
    
    if received.events.is_empty() {
        return;
    }
    
    debug!("🔄 SYNC TO LUA: {} events to sync", received.events.len());
    let lua = &lua_ctx.lua;
    
    if let Ok(lua_received) = lua.globals().get::<LuaTable>("_received_lua_events") {
        debug!("  ✓ Found existing _received_lua_events table");
        for event in received.events.drain(..) {
            if let Ok(json_str) = serde_json::to_string(&event) {
                debug!("  → Appending event: {}", json_str);
                let _ = lua_received.push(json_str);
            }
        }
        // Verify
        let count = lua_received.len().unwrap_or(0);
        debug!("  ✓ _received_lua_events now has {} events", count);
    } else {
        debug!("  ⚠ _received_lua_events NOT FOUND, creating new table");
        // Initialize if doesn't exist
        if let Ok(new_table) = lua.create_table() {
            for event in received.events.drain(..) {
                if let Ok(json_str) = serde_json::to_string(&event) {
                    debug!("  → Adding to new table: {}", json_str);
                    let _ = new_table.push(json_str);
                }
            }
            let _ = lua.globals().set("_received_lua_events", new_table);
            debug!("  ✓ Created and set _received_lua_events");
        }
    }
}
