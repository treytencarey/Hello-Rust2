// Lightyear integration for bevy-lua-ecs
// Provides generic JSON-based messaging that works with any Lua table

use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear::prelude::client::*;
use lightyear::prelude::server::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Generic message type that can carry any JSON data from Lua
/// This allows Lua to send structured data without defining Rust types
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LuaMessage {
    /// Message type identifier (e.g., "chat", "player_move", "game_event")
    pub message_type: String,
    /// The actual message data as JSON (converted from Lua tables)
    pub data: JsonValue,
}

/// Protocol definition - registers all message types and channels
#[derive(Clone)]
pub struct LuaProtocol;

impl Protocol for LuaProtocol {
    fn components(&self) -> Vec<lightyear::prelude::ComponentProtocol> {
        vec![]
    }

    fn messages(&self) -> Vec<lightyear::prelude::MessageProtocol> {
        vec![LuaMessage::protocol()]
    }

    fn channels(&self) -> Vec<lightyear::prelude::ChannelProtocol> {
        vec![
            Channel1::protocol(),
        ]
    }
}

// Define channel for reliable ordered messages (good for chat, events)
#[derive(Channel)]
pub struct Channel1;

/// Register Lua API for Lightyear messaging
pub fn register_lightyear_lua_api(_registry: &bevy_lua_ecs::LuaResourceRegistry) {
    info!("🚀 Registering Lightyear Lua API...");
    
    // TODO: Add Lua bindings for:
    // - send_message(message_type, data_table)
    // - receive_messages() -> array of {type, data}
    // - is_connected()
    
    info!("✅ Lightyear Lua API registered");
}
