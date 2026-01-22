use bevy::prelude::*;
use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::script_entities::SpawnPhase;

/// Spawn request with component data and generic Lua components
pub struct SpawnRequest {
    pub components: Vec<(String, LuaRegistryKey)>,
    pub lua_components: Vec<(String, LuaRegistryKey)>,
    /// Parent temp_id (will be resolved to Entity during spawn processing)
    pub parent_temp_id: Option<u64>,
    pub instance_id: Option<u64>,
    /// When the entity should be spawned relative to script lifecycle
    pub spawn_phase: SpawnPhase,
    /// Temporary ID returned to Lua before actual entity is spawned
    pub temp_id: u64,
}

/// Observer registration request
pub struct ObserverRequest {
    pub temp_id: u64,
    pub event_type: String,
    pub callback_index: usize,
}

/// Resource that holds the spawn queue
#[derive(Resource, Clone)]
pub struct SpawnQueue {
    queue: Arc<Mutex<Vec<SpawnRequest>>>,
    /// Entities that were spawned and need to be returned to Lua
    spawned_entities: Arc<Mutex<Vec<Entity>>>,
    /// Mapping from temp_id (returned to Lua) to actual Entity (created during spawn)
    /// This allows entity references like UiTargetCamera to resolve temp IDs
    temp_id_to_entity: Arc<Mutex<HashMap<u64, Entity>>>,
    /// Counter for generating temp IDs
    next_temp_id: Arc<std::sync::atomic::AtomicU64>,
    /// Queue of observer registrations (temp_id, event_type, callback_index)
    observer_queue: Arc<Mutex<Vec<ObserverRequest>>>,
    /// Mapping from temp_id to registered Lua callback registry keys
    /// Structure: temp_id -> Vec<(event_type, LuaRegistryKey)>
    observer_callbacks: Arc<Mutex<HashMap<u64, Vec<(String, LuaRegistryKey)>>>>,
}

impl Default for SpawnQueue {
    fn default() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            spawned_entities: Arc::new(Mutex::new(Vec::new())),
            temp_id_to_entity: Arc::new(Mutex::new(HashMap::new())),
            next_temp_id: Arc::new(std::sync::atomic::AtomicU64::new(1)), // Start at 1, 0 is reserved
            observer_queue: Arc::new(Mutex::new(Vec::new())),
            observer_callbacks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl SpawnQueue {
    /// Generate a new temporary ID for a spawn request
    pub fn generate_temp_id(&self) -> u64 {
        self.next_temp_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Add a spawn request with a temp_id
    pub fn queue_spawn(
        &self,
        components: Vec<(String, LuaRegistryKey)>,
        lua_components: Vec<(String, LuaRegistryKey)>,
        instance_id: Option<u64>,
        spawn_phase: SpawnPhase,
        temp_id: u64,
    ) {
        let request = SpawnRequest {
            components,
            lua_components,
            parent_temp_id: None,
            instance_id,
            spawn_phase,
            temp_id,
        };
        self.queue.lock().unwrap().push(request);
    }

    /// Add a spawn request with a parent temp_id (resolved during spawn processing)
    pub fn queue_spawn_with_parent(
        &self,
        parent_temp_id: u64,
        components: Vec<(String, LuaRegistryKey)>,
        lua_components: Vec<(String, LuaRegistryKey)>,
        instance_id: Option<u64>,
        spawn_phase: SpawnPhase,
        temp_id: u64,
    ) {
        let request = SpawnRequest {
            components,
            lua_components,
            parent_temp_id: Some(parent_temp_id),
            instance_id,
            spawn_phase,
            temp_id,
        };
        self.queue.lock().unwrap().push(request);
    }

    /// Drain all pending spawn requests
    pub fn drain(&self) -> Vec<SpawnRequest> {
        self.queue.lock().unwrap().drain(..).collect()
    }

    /// Register a temp_id -> Entity mapping (called when entity is actually spawned)
    pub fn register_entity(&self, temp_id: u64, entity: Entity) {
        self.temp_id_to_entity
            .lock()
            .unwrap()
            .insert(temp_id, entity);
    }

    /// Resolve an entity ID that could be either:
    /// - A temp_id from spawn() (small number, needs lookup)
    /// - Real entity bits from world:query() (large number, use directly)
    ///
    /// This makes the API work seamlessly regardless of where the ID came from.
    pub fn resolve_entity(&self, id: u64) -> Entity {
        // First, try temp_id lookup (handles spawn() return values)
        if let Some(entity) = self.temp_id_to_entity.lock().unwrap().get(&id).copied() {
            return entity;
        }

        // Fall back to treating as raw entity bits (handles query() return values)
        // Entity::from_bits reconstructs the full Entity (index + generation)
        Entity::from_bits(id)
    }

    /// Look up actual Entity from temp_id only (legacy method)
    pub fn get_entity(&self, temp_id: u64) -> Option<Entity> {
        self.temp_id_to_entity
            .lock()
            .unwrap()
            .get(&temp_id)
            .copied()
    }

    /// Add a spawned entity to return to Lua
    pub fn add_spawned_entity(&self, entity: Entity) {
        self.spawned_entities.lock().unwrap().push(entity);
    }

    /// Get the most recently spawned entity (for returning to Lua)
    pub fn take_last_spawned(&self) -> Option<Entity> {
        self.spawned_entities.lock().unwrap().pop()
    }

    /// Set parent for an existing spawn request (for chainable :with_parent())
    pub fn set_parent(&self, temp_id: u64, parent_temp_id: u64) {
        let mut queue = self.queue.lock().unwrap();
        for request in queue.iter_mut() {
            if request.temp_id == temp_id {
                request.parent_temp_id = Some(parent_temp_id);
                debug!(
                    "[SPAWN_QUEUE] Set parent {} for temp_id {}",
                    parent_temp_id, temp_id
                );
                return;
            }
        }
    }

    /// Add a pending component to an existing spawn request (for chainable :set())
    /// This allows modifying Camera settings after Camera2d is spawned, etc.
    pub fn add_pending_component(
        &self,
        temp_id: u64,
        component_name: String,
        data: LuaRegistryKey,
    ) {
        let mut queue = self.queue.lock().unwrap();
        for request in queue.iter_mut() {
            if request.temp_id == temp_id {
                // Add to existing spawn request's components
                // Note: This will be processed after the original components, so it can override/modify them
                request.components.push((component_name.clone(), data));
                debug!(
                    "[SPAWN_QUEUE] Added pending component {} for temp_id {}",
                    component_name, temp_id
                );
                return;
            }
        }
        // If the spawn request isn't found, log a warning (might have already been processed)
        warn!(
            "[SPAWN_QUEUE] Could not find spawn request for temp_id {} to add component {}",
            temp_id, component_name
        );
    }

    /// Queue an observer registration (for chainable :observe())
    pub fn queue_observer(&self, temp_id: u64, event_type: String, callback_index: usize) {
        self.observer_queue.lock().unwrap().push(ObserverRequest {
            temp_id,
            event_type,
            callback_index,
        });
    }

    /// Register a Lua callback for an observer
    pub fn register_observer_callback(
        &self,
        temp_id: u64,
        event_type: String,
        callback: LuaRegistryKey,
    ) {
        let mut callbacks = self.observer_callbacks.lock().unwrap();
        callbacks
            .entry(temp_id)
            .or_default()
            .push((event_type, callback));
    }

    /// Drain observer queue
    pub fn drain_observer_queue(&self) -> Vec<ObserverRequest> {
        self.observer_queue.lock().unwrap().drain(..).collect()
    }

    /// Take observer callbacks for an entity by temp_id (removes them from storage)
    /// LuaRegistryKey doesn't implement Clone, so we must take ownership
    pub fn take_observer_callbacks(&self, temp_id: u64) -> Vec<(String, LuaRegistryKey)> {
        self.observer_callbacks
            .lock()
            .unwrap()
            .remove(&temp_id)
            .unwrap_or_default()
    }

    /// Take all observer callbacks (for processing during spawn)
    pub fn take_all_observer_callbacks(&self) -> HashMap<u64, Vec<(String, LuaRegistryKey)>> {
        std::mem::take(&mut *self.observer_callbacks.lock().unwrap())
    }
}
