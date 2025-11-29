use bevy::prelude::*;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};

/// Update request for a component on an entity
pub struct ComponentUpdateRequest {
    pub entity: Entity,
    pub component_name: String,
    pub data: LuaRegistryKey,
}

/// Resource that holds the component update queue
#[derive(Resource, Clone)]
pub struct ComponentUpdateQueue {
    queue: Arc<Mutex<Vec<ComponentUpdateRequest>>>,
}

impl Default for ComponentUpdateQueue {
    fn default() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl ComponentUpdateQueue {
    /// Add a component update request
    pub fn queue_update(
        &self,
        entity: Entity,
        component_name: String,
        data: LuaRegistryKey,
    ) {
        let request = ComponentUpdateRequest {
            entity,
            component_name,
            data,
        };
        self.queue.lock().unwrap().push(request);
    }
    
    /// Drain all pending update requests
    pub fn drain(&self) -> Vec<ComponentUpdateRequest> {
        self.queue.lock().unwrap().drain(..).collect()
    }
    
    /// Remove all pending updates for specific entities (e.g., when they're despawned)
    pub fn clear_for_entities(&self, entities: &[Entity]) -> Vec<LuaRegistryKey> {
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
}
