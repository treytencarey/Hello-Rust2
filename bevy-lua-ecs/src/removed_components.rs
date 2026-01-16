use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Tracks removed Lua custom components for `world:query_removed()` API
/// 
/// Each frame, we compare current LuaCustomComponents state against previous state
/// to detect which component keys were removed (either explicitly or via entity despawn).
#[derive(Resource, Clone, Default)]
pub struct RemovedComponentsTracker {
    inner: Arc<Mutex<RemovedComponentsInner>>,
}

#[derive(Default)]
struct RemovedComponentsInner {
    /// Previous frame's snapshot: entity -> set of component names
    previous_state: HashMap<Entity, HashSet<String>>,
    
    /// Components removed this frame: component_name -> list of entity bits
    removed_this_frame: HashMap<String, Vec<u64>>,
    
    /// Frame number when removals were last computed
    last_computed_frame: u64,
}

impl RemovedComponentsTracker {
    /// Get entities that had a specific component removed this frame
    /// Returns entity bits (for Lua compatibility)
    pub fn get_removed(&self, component_name: &str) -> Vec<u64> {
        let inner = self.inner.lock().unwrap();
        inner.removed_this_frame
            .get(component_name)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Get all removed component names this frame (for debugging)
    pub fn get_all_removed_names(&self) -> Vec<String> {
        let inner = self.inner.lock().unwrap();
        inner.removed_this_frame.keys().cloned().collect()
    }
    
    /// Update tracking state - call this each frame before Lua systems run
    pub fn update(
        &self,
        world: &World,
        current_frame: u64,
    ) {
        let mut inner = self.inner.lock().unwrap();
        
        // Only compute once per frame
        if inner.last_computed_frame == current_frame {
            return;
        }
        inner.last_computed_frame = current_frame;
        
        // Clear previous frame's removals
        inner.removed_this_frame.clear();
        
        // Build current state snapshot
        let mut current_state: HashMap<Entity, HashSet<String>> = HashMap::new();
        
        // Query all entities with LuaCustomComponents
        for entity_ref in world.iter_entities() {
            let entity = entity_ref.id();
            if let Some(lua_comps) = entity_ref.get::<crate::components::LuaCustomComponents>() {
                let comp_names: HashSet<String> = lua_comps.components.keys().cloned().collect();
                if !comp_names.is_empty() {
                    current_state.insert(entity, comp_names);
                }
            }
        }
        
        // Compare previous state to current state to find removals
        // Clone previous_state to avoid borrow conflict with removed_this_frame
        let previous_state = inner.previous_state.clone();
        for (entity, prev_components) in &previous_state {
            let entity_bits = entity.to_bits();
            
            match current_state.get(entity) {
                Some(curr_components) => {
                    // Entity still exists - check which components were removed
                    for comp_name in prev_components {
                        if !curr_components.contains(comp_name) {
                            // Component was removed from this entity
                            inner.removed_this_frame
                                .entry(comp_name.clone())
                                .or_insert_with(Vec::new)
                                .push(entity_bits);
                            debug!(
                                "[REMOVED_COMPONENTS] Component '{}' removed from entity {:?}",
                                comp_name, entity
                            );
                        }
                    }
                }
                None => {
                    // Entity was despawned - all its components are "removed"
                    for comp_name in prev_components {
                        inner.removed_this_frame
                            .entry(comp_name.clone())
                            .or_insert_with(Vec::new)
                            .push(entity_bits);
                        debug!(
                            "[REMOVED_COMPONENTS] Component '{}' removed (entity {:?} despawned)",
                            comp_name, entity
                        );
                    }
                }
            }
        }
        
        // Update previous state for next frame
        inner.previous_state = current_state;
        
        // Log summary if any removals
        if !inner.removed_this_frame.is_empty() {
            let total: usize = inner.removed_this_frame.values().map(|v| v.len()).sum();
            debug!(
                "[REMOVED_COMPONENTS] Frame {}: {} removals across {} component types",
                current_frame,
                total,
                inner.removed_this_frame.len()
            );
        }
    }
}

/// System to update the removed components tracker
/// Runs before Lua systems to ensure removals are available for query
pub fn update_removed_components_tracker(world: &mut World) {
    let tracker = world.resource::<RemovedComponentsTracker>().clone();
    let current_frame = world
        .get_resource::<bevy::diagnostic::FrameCount>()
        .map(|f| f.0 as u64)
        .unwrap_or(0);
    
    tracker.update(world, current_frame);
}
