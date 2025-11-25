use bevy::prelude::*;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};

/// Resource insertion request with resource type name and data
pub struct ResourceRequest {
    pub resource_name: String,
    pub data: LuaRegistryKey,
}

/// Resource that holds the resource insertion queue
#[derive(Resource, Clone)]
pub struct ResourceQueue {
    queue: Arc<Mutex<Vec<ResourceRequest>>>,
}

impl Default for ResourceQueue {
    fn default() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl ResourceQueue {
    /// Add a resource insertion request
    pub fn queue_insert(&self, resource_name: String, data: LuaRegistryKey) {
        let request = ResourceRequest {
            resource_name,
            data,
        };
        self.queue.lock().unwrap().push(request);
    }
    
    /// Drain all pending resource requests
    pub fn drain(&self) -> Vec<ResourceRequest> {
        self.queue.lock().unwrap().drain(..).collect()
    }
}
