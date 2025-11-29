use bevy::prelude::*;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};

/// Spawn request with component data and generic Lua components
pub struct SpawnRequest {
    pub components: Vec<(String, LuaRegistryKey)>,
    pub lua_components: Vec<(String, LuaRegistryKey)>,
}

/// Resource that holds the spawn queue
#[derive(Resource, Clone)]
pub struct SpawnQueue {
    queue: Arc<Mutex<Vec<SpawnRequest>>>,
    /// Entities that were spawned and need to be returned to Lua
    spawned_entities: Arc<Mutex<Vec<Entity>>>,
}

impl Default for SpawnQueue {
    fn default() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            spawned_entities: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl SpawnQueue {
    /// Add a spawn request
    pub fn queue_spawn(
        &self, 
        components: Vec<(String, LuaRegistryKey)>,
        lua_components: Vec<(String, LuaRegistryKey)>
    ) {
        let request = SpawnRequest {
            components,
            lua_components,
        };
        self.queue.lock().unwrap().push(request);
    }
    
    /// Drain all pending spawn requests
    pub fn drain(&self) -> Vec<SpawnRequest> {
        self.queue.lock().unwrap().drain(..).collect()
    }
    
    /// Add a spawned entity to return to Lua
    pub fn add_spawned_entity(&self, entity: Entity) {
        self.spawned_entities.lock().unwrap().push(entity);
    }
    
    /// Get the most recently spawned entity (for returning to Lua)
    pub fn take_last_spawned(&self) -> Option<Entity> {
        self.spawned_entities.lock().unwrap().pop()
    }
}
