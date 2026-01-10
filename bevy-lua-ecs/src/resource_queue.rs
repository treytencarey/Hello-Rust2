use bevy::prelude::*;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};

/// Resource insertion request with resource type name and data
pub struct ResourceRequest {
    pub resource_name: String,
    pub data: Arc<LuaRegistryKey>,
    pub instance_id: Option<u64>,
}

/// Resource that holds the resource insertion queue
#[derive(Resource, Clone)]
pub struct ResourceQueue {
    queue: Arc<Mutex<Vec<ResourceRequest>>>,
    /// Track which resources were inserted by which script instance
    instance_resources: Arc<Mutex<std::collections::HashMap<u64, Vec<String>>>>,
}

impl Default for ResourceQueue {
    fn default() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            instance_resources: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
}

impl ResourceQueue {
    /// Add a resource insertion request
    pub fn queue_insert(
        &self,
        resource_name: String,
        data: LuaRegistryKey,
        instance_id: Option<u64>,
    ) {
        let request = ResourceRequest {
            resource_name,
            data: Arc::new(data),
            instance_id,
        };
        self.queue.lock().unwrap().push(request);
    }

    /// Track that a resource was inserted by a script instance
    pub fn track_resource(&self, instance_id: u64, resource_name: String) {
        let mut map = self.instance_resources.lock().unwrap();
        map.entry(instance_id)
            .or_insert_with(Vec::new)
            .push(resource_name);
    }

    /// Get all resources inserted by a specific instance
    pub fn get_instance_resources(&self, instance_id: u64) -> Vec<String> {
        self.instance_resources
            .lock()
            .unwrap()
            .get(&instance_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Clear tracking for a specific instance
    pub fn clear_instance_tracking(&self, instance_id: u64) {
        self.instance_resources.lock().unwrap().remove(&instance_id);
    }

    /// Drain all pending resource requests
    pub fn drain(&self) -> Vec<ResourceRequest> {
        self.queue.lock().unwrap().drain(..).collect()
    }
}
