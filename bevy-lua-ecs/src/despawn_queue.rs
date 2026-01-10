use bevy::prelude::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Queue for despawning entities from Lua
#[derive(Resource, Clone)]
pub struct DespawnQueue {
    queue: Arc<Mutex<HashSet<Entity>>>,
}

impl Default for DespawnQueue {
    fn default() -> Self {
        Self {
            queue: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

impl DespawnQueue {
    /// Queue an entity for despawning (duplicates are automatically ignored)
    pub fn queue_despawn(&self, entity: Entity) {
        self.queue.lock().unwrap().insert(entity);
    }
}

/// System to process the despawn queue
pub fn process_despawn_queue(
    mut commands: Commands,
    despawn_queue: Res<DespawnQueue>,
    component_update_queue: Res<crate::component_update_queue::ComponentUpdateQueue>,
    lua_ctx: Res<crate::lua_integration::LuaScriptContext>,
) {
    let mut queue = despawn_queue.queue.lock().unwrap();
    let entities_to_despawn: Vec<Entity> = queue.drain().collect();
    drop(queue);

    if entities_to_despawn.is_empty() {
        return;
    }

    // Clear any pending component updates for these entities
    let removed_keys = component_update_queue.clear_for_entities(&entities_to_despawn);

    // Clean up Lua registry values
    // Arc ensures we only clean up once all references are dropped
    for key_arc in removed_keys {
        // Try to unwrap Arc - if this is the last reference, clean up
        if let Ok(key) = Arc::try_unwrap(key_arc) {
            if let Err(e) = lua_ctx.lua.remove_registry_value(key) {
                warn!(
                    "Failed to remove registry value for despawned entity: {}",
                    e
                );
            }
        }
        // If Arc::try_unwrap fails, there are other references (e.g., in peek_pending)
        // so we shouldn't clean up yet
    }

    // Despawn the entities (Bevy 0.16+ despawn() handles children automatically)
    for entity in entities_to_despawn {
        commands.entity(entity).despawn();
        debug!("Despawned entity: {:?}", entity);
    }
}
