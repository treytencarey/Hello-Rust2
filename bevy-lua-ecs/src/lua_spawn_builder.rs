//! Chainable spawn builder for Lua entities
//!
//! Provides a builder pattern for spawning entities from Lua:
//! ```lua
//! spawn({ Transform = {...} })
//!     :with_parent(parent_id)
//!     :observe("Pointer<Over>", function(entity, event) ... end)
//!     :id()
//! ```

use crate::component_update_queue::ComponentUpdateQueue;
use crate::spawn_queue::SpawnQueue;
use mlua::prelude::*;
use std::sync::Arc;

/// Lua userdata for chainable entity spawning
/// Returned by spawn() function, allows chaining :with_parent(), :observe(), :id(), :set()
#[derive(Clone)]
pub struct LuaSpawnBuilder {
    pub temp_id: u64,
    pub spawn_queue: SpawnQueue,
    pub update_queue: ComponentUpdateQueue,
    pub lua: Arc<Lua>,
}

impl LuaSpawnBuilder {
    pub fn new(temp_id: u64, spawn_queue: SpawnQueue, update_queue: ComponentUpdateQueue, lua: Arc<Lua>) -> Self {
        Self {
            temp_id,
            spawn_queue,
            update_queue,
            lua,
        }
    }
}

impl LuaUserData for LuaSpawnBuilder {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Get the temp_id for referencing this entity
        methods.add_method("id", |_, this, ()| Ok(this.temp_id));

        // Set parent - chainable, returns self
        methods.add_method("with_parent", |_, this, parent_id: u64| {
            // Update the spawn request to have a parent
            this.spawn_queue.set_parent(this.temp_id, parent_id);
            Ok(this.clone())
        });

        // Add observer - chainable, returns self
        // Usage: :observe("Pointer<Over>", function(entity, event) ... end)
        methods.add_method(
            "observe",
            |lua, this, (event_type, callback): (String, LuaFunction)| {
                // Store the callback as a registry key
                let registry_key = lua.create_registry_value(callback)?;

                // Register the callback in the spawn queue so it can be transferred to LuaObserverRegistry
                this.spawn_queue
                    .register_observer_callback(this.temp_id, event_type, registry_key);

                Ok(this.clone())
            },
        );

        // Set/update components using spawn-style syntax - chainable, returns self
        // Usage: :set({ Camera = { target = {...} }, Transform = {...} })
        // Works both during spawn (before entity exists) and after spawn (runtime updates)
        methods.add_method(
            "set",
            |lua, this, components: LuaTable| {
                // Check if the entity has been spawned yet by trying to resolve it
                let maybe_entity = this.spawn_queue.get_entity(this.temp_id);
                
                // Iterate through the table - keys are component names, values are component data
                for pair in components.pairs::<String, LuaValue>() {
                    let (component_name, component_value) = pair?;
                    
                    // Convert the value to a table if possible, or create a wrapper for scalar values
                    let component_data = match component_value {
                        LuaValue::Table(table) => table,
                        _ => {
                            // For non-table values, create a wrapper table with _0 (tuple struct style)
                            let wrapper = lua.create_table()?;
                            wrapper.set("_0", component_value)?;
                            wrapper
                        }
                    };
                    
                    // Create a registry key for the component data
                    let registry_key = lua.create_registry_value(component_data)?;

                    if let Some(entity) = maybe_entity {
                        // Entity has been spawned - use update queue for runtime updates
                        bevy::log::info!("[SPAWN_BUILDER] Queueing update for component '{}' on entity {:?} (temp_id: {})", component_name, entity, this.temp_id);
                        this.update_queue
                            .queue_update(entity, component_name, registry_key);
                    } else {
                        // Entity not yet spawned - use spawn queue for spawn-time components
                        bevy::log::warn!("[SPAWN_BUILDER] Entity not found for temp_id: {}, using spawn queue for '{}'", this.temp_id, component_name);
                        this.spawn_queue
                            .add_pending_component(this.temp_id, component_name, registry_key);
                    }
                }

                Ok(this.clone())
            },
        );
    }
}
