//! Event sending from Lua via queued pattern
//! 
//! This module provides the infrastructure for Lua scripts to send Bevy events.
//! Events are queued and dispatched to their concrete types by a generated system.

use bevy::prelude::*;
use std::sync::{Arc, Mutex};
use serde_json::Value;

/// Resource that holds pending events to be sent from Lua scripts.
/// 
/// Events are stored as JSON values with their type names, then dispatched
/// to concrete EventWriter<T> by the generated `dispatch_lua_events` system.
#[derive(Resource, Default, Clone)]
pub struct PendingLuaEvents {
    /// Events waiting to be dispatched: (type_name, json_data)
    pub events: Arc<Mutex<Vec<(String, Value)>>>,
}

impl PendingLuaEvents {
    /// Queue an event to be sent on the next frame
    pub fn queue_event(&self, type_name: String, data: Value) {
        if let Ok(mut events) = self.events.lock() {
            events.push((type_name, data));
        }
    }
    
    /// Take all pending events for dispatch
    pub fn drain_events(&self) -> Vec<(String, Value)> {
        if let Ok(mut events) = self.events.lock() {
            std::mem::take(&mut *events)
        } else {
            Vec::new()
        }
    }
}

/// Plugin that adds event sending infrastructure
pub struct LuaEventSenderPlugin;

impl Plugin for LuaEventSenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingLuaEvents>();
        // The dispatch system is added by generated code
    }
}
