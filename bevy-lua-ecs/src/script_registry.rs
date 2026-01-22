use bevy::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Information about a script instance
#[derive(Clone, Debug)]
pub struct ScriptInstanceInfo {
    pub instance_id: u64,
    pub script_content: String,
    pub stopped: bool,
}

/// Resource that tracks all loaded script instances for automatic reload
#[derive(Resource, Clone)]
pub struct ScriptRegistry {
    // Map: script file path -> list of script instances
    scripts: Arc<Mutex<HashMap<PathBuf, Vec<ScriptInstanceInfo>>>>,
}

impl Default for ScriptRegistry {
    fn default() -> Self {
        Self {
            scripts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl ScriptRegistry {
    /// Register a new script instance
    /// If an instance with the same ID already exists for this path, it is updated.
    pub fn register_script(&self, path: PathBuf, instance_id: u64, content: String) {
        let mut scripts = self.scripts.lock().unwrap();

        let list = scripts.entry(path.clone()).or_insert_with(Vec::new);

        // Check for existing instance with same ID
        if let Some(existing) = list.iter_mut().find(|info| info.instance_id == instance_id) {
            // Update existing
            existing.script_content = content;
            existing.stopped = false;
            debug!(
                "Updated script instance {} for path {:?}",
                instance_id, path
            );
        } else {
            // Add new
            let info = ScriptInstanceInfo {
                instance_id,
                script_content: content,
                stopped: false,
            };
            list.push(info);
            debug!(
                "Registered script instance {} for path {:?}",
                instance_id, path
            );
        }
    }

    /// Get all active (non-stopped) instance IDs for a given script path
    pub fn get_active_instances(&self, path: &PathBuf) -> Vec<(u64, String)> {
        let scripts = self.scripts.lock().unwrap();

        scripts
            .get(path)
            .map(|instances| {
                instances
                    .iter()
                    .filter(|info| !info.stopped)
                    .map(|info| (info.instance_id, info.script_content.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all instance IDs (including stopped ones) for a given script path
    pub fn get_all_instances(&self, path: &PathBuf) -> Vec<u64> {
        let scripts = self.scripts.lock().unwrap();

        scripts
            .get(path)
            .map(|instances| instances.iter().map(|info| info.instance_id).collect())
            .unwrap_or_default()
    }

    /// Mark a specific instance as stopped (prevents auto-reload)
    pub fn mark_stopped(&self, instance_id: u64) {
        let mut scripts = self.scripts.lock().unwrap();

        for instances in scripts.values_mut() {
            for info in instances.iter_mut() {
                if info.instance_id == instance_id {
                    info.stopped = true;
                    debug!("Marked script instance {} as stopped", instance_id);
                    return;
                }
            }
        }

        warn!(
            "Could not find script instance {} to mark as stopped",
            instance_id
        );
    }

    /// Check if an instance is stopped
    pub fn is_stopped(&self, instance_id: u64) -> bool {
        let scripts = self.scripts.lock().unwrap();

        for instances in scripts.values() {
            for info in instances {
                if info.instance_id == instance_id {
                    return info.stopped;
                }
            }
        }

        false
    }

    /// Remove a specific instance from the registry completely
    pub fn remove_instance(&self, instance_id: u64) {
        let mut scripts = self.scripts.lock().unwrap();

        for instances in scripts.values_mut() {
            instances.retain(|info| info.instance_id != instance_id);
        }

        // Clean up empty entries
        scripts.retain(|_, instances| !instances.is_empty());

        debug!("Removed script instance {} from registry", instance_id);
    }

    /// Get the script content for a specific instance
    pub fn get_instance_content(&self, instance_id: u64) -> Option<String> {
        let scripts = self.scripts.lock().unwrap();

        for instances in scripts.values() {
            for info in instances {
                if info.instance_id == instance_id {
                    return Some(info.script_content.clone());
                }
            }
        }

        None
    }

    /// Get the script path for a specific instance
    pub fn get_instance_path(&self, instance_id: u64) -> Option<PathBuf> {
        let scripts = self.scripts.lock().unwrap();

        for (path, instances) in scripts.iter() {
            for info in instances {
                if info.instance_id == instance_id {
                    return Some(path.clone());
                }
            }
        }

        None
    }

    /// Get all active (non-stopped) instance IDs across all scripts
    /// Used by event accumulator to distribute events to all active scripts
    pub fn all_active_instance_ids(&self) -> Vec<u64> {
        let scripts = self.scripts.lock().unwrap();
        scripts
            .values()
            .flat_map(|instances| {
                instances
                    .iter()
                    .filter(|info| !info.stopped)
                    .map(|info| info.instance_id)
            })
            .collect()
    }
}
