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
}
