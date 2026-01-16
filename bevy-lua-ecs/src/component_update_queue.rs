use bevy::prelude::*;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

/// Update request for a component on an entity
pub struct ComponentUpdateRequest {
    pub entity: Entity,
    pub component_name: String,
    pub data: Arc<LuaRegistryKey>,
}

/// Resource that holds the component update queue
#[derive(Resource, Clone)]
pub struct ComponentUpdateQueue {
    queue: Arc<Mutex<Vec<ComponentUpdateRequest>>>,
    /// Lock-free flag for fast-path empty check (optimization)
    has_updates: Arc<AtomicBool>,
}

impl Default for ComponentUpdateQueue {
    fn default() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            has_updates: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl ComponentUpdateQueue {
    /// Add a component update request
    pub fn queue_update(&self, entity: Entity, component_name: String, data: LuaRegistryKey) {
        let request = ComponentUpdateRequest {
            entity,
            component_name,
            data: Arc::new(data),
        };
        self.queue.lock().unwrap().push(request);
        self.has_updates.store(true, Ordering::Relaxed);
    }

    /// Drain all pending update requests
    pub fn drain(&self) -> Vec<ComponentUpdateRequest> {
        let drained = self.queue.lock().unwrap().drain(..).collect();
        self.has_updates.store(false, Ordering::Relaxed);
        drained
    }

    /// Fast check if queue has any pending updates (lock-free for performance)
    /// Used by component method dispatcher to avoid overhead when queue is empty
    pub fn has_pending(&self) -> bool {
        self.has_updates.load(Ordering::Relaxed)
    }

    /// Remove all pending updates for specific entities (e.g., when they're despawned)
    pub fn clear_for_entities(&self, entities: &[Entity]) -> Vec<Arc<LuaRegistryKey>> {
        let mut queue = self.queue.lock().unwrap();
        let mut removed_requests = Vec::new();

        // Separate updates: keep those NOT for the specified entities, collect the rest
        let mut remaining = Vec::new();
        for request in queue.drain(..) {
            if entities.contains(&request.entity) {
                removed_requests.push(request);
            } else {
                remaining.push(request);
            }
        }

        // Put back the remaining requests
        *queue = remaining;

        // Return the registry keys that need to be cleaned up
        removed_requests.into_iter().map(|r| r.data).collect()
    }

    /// Peek at pending update for a specific entity+component (for read-through cache)
    /// Returns a reference to the most recent pending update's data if one exists
    pub fn peek_pending(&self, entity: Entity, component_name:&str) -> Option<Arc<LuaRegistryKey>> {
        let queue = self.queue.lock().unwrap();
        
        // Find the LAST (most recent) update for this entity+component and clone the Arc
        queue.iter()
            .rev()
            .find(|req| req.entity == entity && req.component_name == component_name)
            .map(|req| Arc::clone(&req.data))
    }
}

/// Get a component from the ECS, merging any queued updates if present
/// This ensures component methods see queued data instead of stale ECS state
/// 
/// Returns None if the component doesn't exist on the entity
pub fn get_component_with_queue<T: Component + Clone + bevy::reflect::Reflect>(
    world: &World,
    lua: &Lua,
    entity: Entity,
    component_name: &str,
    update_queue: &ComponentUpdateQueue,
    type_registry: &bevy::ecs::reflect::AppTypeRegistry,
) -> mlua::Result<Option<T>> {
    // Fast-path: if queue is empty, just use ECS state
    if !update_queue.has_pending() {
        return Ok(world.get::<T>(entity).cloned());
    }
    
    debug!("[QUEUE_MERGE] Queue has pending updates, checking for entity {:?} component {}", entity, component_name);
    
    // Get ECS component
    let Some(mut component) = world.get::<T>(entity).cloned() else {
        return Ok(None);
    };
    
    // Check for pending update
    let Some(pending_key) = update_queue.peek_pending(entity, component_name) else {
        // No pending update for this specific component, use ECS state
        debug!("[QUEUE_MERGE] No pending update for entity {:?} component {}", entity, component_name);
        return Ok(Some(component));
    };
    
    // Merge pending update into component
    debug!("[QUEUE_MERGE] ✓ Found pending update for entity {:?} component {}, merging!", entity, component_name);
    let pending_table: LuaTable = lua.registry_value(&pending_key)?;

    
    // Use reflection to merge the pending Lua table into the component
    // This handles partial updates (e.g., only translation queued, keep existing rotation)
    let registry = type_registry.read();
    let type_info = component.get_represented_type_info().ok_or_else(|| {
        mlua::Error::RuntimeError(format!("Component {} has no type info", component_name))
    })?;
    
    match type_info {
        bevy::reflect::TypeInfo::Struct(struct_info) => {
            use bevy::reflect::ReflectMut;
            
            // Get mutable reflection of the component
            if let ReflectMut::Struct(struct_mut) = component.reflect_mut() {
                // Iterate through fields in the pending table
                for pair in pending_table.pairs::<String, LuaValue>() {
                    let (field_name, lua_value) = pair?;
                    
                    // Find and update the field
                    if let Some(field) = struct_mut.field_mut(&field_name) {
                        crate::components::set_field_from_lua(
                            field,
                            &lua_value,
                            None, // No asset registry needed for component methods
                            type_registry,
                            Some(&field_name),
                        )?;
                    }
                }
            }
        }
        _ => {
            // For non-struct components, we can't do partial updates
            // This shouldn't happen for Transform and similar components
            return Err(mlua::Error::RuntimeError(format!(
                "Component {} is not a struct, cannot merge partial updates",
                component_name
            )));
        }
    }
    
    Ok(Some(component))
}
