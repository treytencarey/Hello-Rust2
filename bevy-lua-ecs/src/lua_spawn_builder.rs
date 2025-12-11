//! Chainable spawn builder for Lua entities
//! 
//! Provides a builder pattern for spawning entities from Lua:
//! ```lua
//! spawn({ Transform = {...} })
//!     :with_parent(parent_id)
//!     :observe("Pointer<Over>", function(entity, event) ... end)
//!     :id()
//! ```

use mlua::prelude::*;
use std::sync::Arc;
use crate::spawn_queue::SpawnQueue;

/// Lua userdata for chainable entity spawning
/// Returned by spawn() function, allows chaining :with_parent(), :observe(), :id()
#[derive(Clone)]
pub struct LuaSpawnBuilder {
    pub temp_id: u64,
    pub spawn_queue: SpawnQueue,
    pub lua: Arc<Lua>,
}

impl LuaSpawnBuilder {
    pub fn new(temp_id: u64, spawn_queue: SpawnQueue, lua: Arc<Lua>) -> Self {
        Self {
            temp_id,
            spawn_queue,
            lua,
        }
    }
}

impl LuaUserData for LuaSpawnBuilder {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Get the temp_id for referencing this entity
        methods.add_method("id", |_, this, ()| {
            Ok(this.temp_id)
        });
        
        // Set parent - chainable, returns self
        methods.add_method("with_parent", |_, this, parent_id: u64| {
            // Update the spawn request to have a parent
            this.spawn_queue.set_parent(this.temp_id, parent_id);
            Ok(this.clone())
        });
        
        // Add observer - chainable, returns self
        // Usage: :observe("Pointer<Over>", function(entity, event) ... end)
        methods.add_method("observe", |lua, this, (event_type, callback): (String, LuaFunction)| {
            // Store the callback as a registry key
            let registry_key = lua.create_registry_value(callback)?;
            
            // Register the callback in the spawn queue so it can be transferred to LuaObserverRegistry
            this.spawn_queue.register_observer_callback(this.temp_id, event_type, registry_key);
            
            Ok(this.clone())
        });
        
        // Set/add component - chainable, returns self
        // Usage: :set("Camera", { target = {...}, order = -1 })
        // This queues the component to be added during spawn processing
        methods.add_method("set", |lua, this, (component_name, component_data): (String, LuaTable)| {
            // Create a registry key for the component data
            let registry_key = lua.create_registry_value(component_data)?;
            
            // Queue the component to be added during spawn
            this.spawn_queue.add_pending_component(this.temp_id, component_name, registry_key);
            
            Ok(this.clone())
        });
    }
}
