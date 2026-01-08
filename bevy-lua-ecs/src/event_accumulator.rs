//! Per-script-instance event accumulator for Lua
//!
//! Solves the problem of Lua systems missing Bevy events when deferred by frame budgets.
//! Bevy clears events after 2 frames, but Lua systems may not run every frame.
//!
//! Architecture:
//! 1. Rust accumulator system runs EVERY frame, reading from Bevy's EventReader
//! 2. Events are copied to ALL active script instance buffers
//! 3. When Lua calls read_events(), it drains from its own script's buffer
//! 4. Each script instance sees every event exactly once

use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Per-script-instance event buffers
/// 
/// Each script instance has its own buffer for each event type.
/// Events are copied to all active instances, then each instance drains its own buffer.
#[derive(Resource, Default, Clone)]
pub struct LuaEventAccumulator {
    /// Map: instance_id -> event_type -> Vec<event_as_json>
    buffers: Arc<Mutex<HashMap<u64, HashMap<String, Vec<serde_json::Value>>>>>,
}

impl LuaEventAccumulator {
    /// Push an event to ALL specified script instances
    pub fn push_to_instances(&self, instance_ids: &[u64], event_type: &str, event: serde_json::Value) {
        let mut buffers = self.buffers.lock().unwrap();
        for &instance_id in instance_ids {
            buffers
                .entry(instance_id)
                .or_default()
                .entry(event_type.to_string())
                .or_default()
                .push(event.clone());
        }
    }
    
    /// Drain all events of a specific type for a specific script instance
    /// 
    /// Returns the accumulated events and clears the buffer for that event type.
    pub fn drain(&self, instance_id: u64, event_type: &str) -> Vec<serde_json::Value> {
        let mut buffers = self.buffers.lock().unwrap();
        buffers
            .get_mut(&instance_id)
            .and_then(|m| m.remove(event_type))
            .unwrap_or_default()
    }
    
    /// Clear all buffers for a script instance (on reload/stop)
    pub fn clear_instance(&self, instance_id: u64) {
        let mut buffers = self.buffers.lock().unwrap();
        buffers.remove(&instance_id);
    }
    
    /// Get debug info about buffer sizes
    pub fn debug_info(&self) -> String {
        let buffers = self.buffers.lock().unwrap();
        let mut info = String::new();
        for (instance_id, event_map) in buffers.iter() {
            for (event_type, events) in event_map.iter() {
                if !events.is_empty() {
                    info.push_str(&format!(
                        "  instance={} type={} count={}\n",
                        instance_id, event_type, events.len()
                    ));
                }
            }
        }
        info
    }
}

/// Convert serde_json::Value to mlua::Value
/// 
/// This is a public utility for generated code to convert accumulated JSON events to Lua.
pub fn json_to_lua_value(lua: &mlua::Lua, value: &serde_json::Value) -> mlua::Result<mlua::Value> {
    use mlua::Value as LuaValue;
    
    match value {
        serde_json::Value::Null => Ok(LuaValue::Nil),
        serde_json::Value::Bool(b) => Ok(LuaValue::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(LuaValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(LuaValue::Number(f))
            } else {
                Ok(LuaValue::Nil)
            }
        }
        serde_json::Value::String(s) => Ok(LuaValue::String(lua.create_string(s)?)),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                table.set(i + 1, json_to_lua_value(lua, v)?)?;
            }
            Ok(LuaValue::Table(table))
        }
        serde_json::Value::Object(map) => {
            let table = lua.create_table()?;
            for (k, v) in map.iter() {
                table.set(k.as_str(), json_to_lua_value(lua, v)?)?;
            }
            Ok(LuaValue::Table(table))
        }
    }
}
