use bevy::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

static NEXT_INSTANCE_ID: AtomicU64 = AtomicU64::new(1);

/// When the entity was spawned relative to script lifecycle
/// Used to determine cleanup behavior during hot-reload
#[derive(Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub enum SpawnPhase {
    #[default]
    Script,   // Spawned during script/module execution → respawn on reload
    Runtime,  // Spawned by system/callback at runtime → preserve on reload
}

/// Component that tracks which script instance spawned an entity
#[derive(Component, Clone, Debug, bevy::prelude::Reflect)]
#[reflect(Component)]
pub struct ScriptOwned {
    pub instance_id: u64,
    pub spawn_phase: SpawnPhase,
}

/// Resource that tracks the currently executing script instance
/// Each script execution gets a unique instance ID, allowing the same script to run multiple times
#[derive(Resource, Clone, Default)]
pub struct ScriptInstance {
    current: Arc<Mutex<Option<(u64, String)>>>,
}

impl ScriptInstance {
    /// Start a new script instance and return its unique ID
    pub fn start(&self, name: String) -> u64 {
        let instance_id = NEXT_INSTANCE_ID.fetch_add(1, Ordering::SeqCst);
        *self.current.lock().unwrap() = Some((instance_id, name));
        instance_id
    }

    /// Get the current script instance ID if one is executing
    pub fn get_id(&self) -> Option<u64> {
        self.current.lock().unwrap().as_ref().map(|(id, _)| *id)
    }

    /// Get the current script instance info (id and name)
    pub fn get(&self) -> Option<(u64, String)> {
        self.current.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        *self.current.lock().unwrap() = None;
    }
}

/// Helper function to despawn all entities owned by a specific script instance
/// Only despawns entities with SpawnPhase::Script (spawned during script execution)
/// Entities with SpawnPhase::Runtime (spawned by systems at runtime) are preserved
/// Returns the list of entities that will be despawned
pub fn despawn_instance_entities(world: &mut World, instance_id: u64) -> Vec<Entity> {
    let mut entities_to_despawn = Vec::new();

    // Query for all entities with ScriptOwned component matching this instance
    // Only despawn Script phase entities - Runtime phase entities persist across hot-reload
    for (entity, script_owned) in world.query::<(Entity, &ScriptOwned)>().iter(world) {
        if script_owned.instance_id == instance_id 
           && script_owned.spawn_phase == SpawnPhase::Script {
            entities_to_despawn.push(entity);
        }
    }

    let count = entities_to_despawn.len();

    // Queue them for despawn (this also clears component updates)
    let despawn_queue = world
        .resource::<crate::despawn_queue::DespawnQueue>()
        .clone();
    for entity in &entities_to_despawn {
        despawn_queue.queue_despawn(*entity);
        debug!(
            "Queued despawn for entity {:?} owned by instance {} (Script phase)",
            entity, instance_id
        );
    }

    debug!(
        "Queued {} Script-phase entities from instance {} for despawn (Runtime entities preserved)",
        count, instance_id
    );

    entities_to_despawn
}
