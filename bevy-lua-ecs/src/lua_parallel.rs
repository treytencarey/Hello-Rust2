//! Parallel execution of Lua systems grouped by state_id
//!
//! When the `parallel-systems` feature is enabled, systems from different
//! Lua state IDs can execute in parallel using rayon. Systems within the
//! same state execute sequentially to maintain Lua's single-threaded model.
//!
//! On WASM targets, rayon automatically falls back to sequential execution.

use bevy::prelude::*;

#[cfg(feature = "parallel-systems")]
use rayon::prelude::*;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::lua_systems::LuaSystemEntry;

/// Configuration for parallel Lua system execution
#[derive(Resource, Clone)]
pub struct LuaParallelConfig {
    /// Enable parallel execution at runtime (default: true)
    /// Set to false to force sequential execution for debugging
    pub enabled: bool,
    /// Minimum number of state groups before parallelizing (default: 2)
    /// If there's only one state group, parallel overhead isn't worth it
    pub min_groups_for_parallel: usize,
}

impl Default for LuaParallelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_groups_for_parallel: 2,
        }
    }
}

/// A group of systems that share the same Lua state
#[derive(Debug)]
pub struct StateSystemGroup {
    pub state_id: usize,
    pub system_indices: Vec<usize>,
}

/// Result of executing a single system
pub struct SystemExecutionResult {
    pub index: usize,
    pub new_tick: u32,
    pub new_real_time: Instant,
    pub should_remove: bool,
    pub elapsed: Duration,
}

/// Group systems by their state_id
pub fn group_systems_by_state(systems: &[LuaSystemEntry]) -> Vec<StateSystemGroup> {
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    
    for (idx, entry) in systems.iter().enumerate() {
        groups.entry(entry.state_id).or_default().push(idx);
    }
    
    // Convert to Vec and sort by state_id for deterministic ordering
    let mut result: Vec<StateSystemGroup> = groups
        .into_iter()
        .map(|(state_id, system_indices)| StateSystemGroup { state_id, system_indices })
        .collect();
    
    result.sort_by_key(|g| g.state_id);
    result
}

/// Check if parallel execution should be used
pub fn should_use_parallel(config: &LuaParallelConfig, num_groups: usize) -> bool {
    #[cfg(feature = "parallel-systems")]
    {
        config.enabled && num_groups >= config.min_groups_for_parallel
    }
    
    #[cfg(not(feature = "parallel-systems"))]
    {
        let _ = (config, num_groups);
        false
    }
}

/// Execute state groups in parallel (when feature enabled and conditions met)
/// 
/// # Arguments
/// * `groups` - State groups to execute
/// * `execute_group` - Closure that executes all systems in a group and returns results
/// 
/// # Returns
/// Vector of execution results from all systems
#[cfg(feature = "parallel-systems")]
pub fn execute_groups_parallel<F>(
    groups: Vec<StateSystemGroup>,
    execute_group: F,
) -> Vec<SystemExecutionResult>
where
    F: Fn(&StateSystemGroup) -> Vec<SystemExecutionResult> + Sync,
{
    groups
        .par_iter()
        .flat_map(|group| execute_group(group))
        .collect()
}

/// Sequential fallback when parallel feature is disabled
#[cfg(not(feature = "parallel-systems"))]
pub fn execute_groups_parallel<F>(
    groups: Vec<StateSystemGroup>,
    execute_group: F,
) -> Vec<SystemExecutionResult>
where
    F: Fn(&StateSystemGroup) -> Vec<SystemExecutionResult>,
{
    groups
        .iter()
        .flat_map(|group| execute_group(group))
        .collect()
}

/// Sequential execution of groups (used when parallel is disabled or only one group)
pub fn execute_groups_sequential<F>(
    groups: Vec<StateSystemGroup>,
    execute_group: F,
) -> Vec<SystemExecutionResult>
where
    F: Fn(&StateSystemGroup) -> Vec<SystemExecutionResult>,
{
    groups
        .iter()
        .flat_map(|group| execute_group(group))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    #[test]
    fn test_group_systems_by_state() {
        // Create mock entries with a simple registry key
        let lua = Arc::new(Lua::new());
        // Use mlua::Value::Nil which implements IntoLua
        let key = Arc::new(lua.create_registry_value(mlua::Value::Nil).unwrap());
        
        let entries = vec![
            LuaSystemEntry {
                instance_id: 1,
                system_key: key.clone(),
                last_run: 0,
                last_run_real_time: Instant::now(),
                state_id: 0,
                system_name: "system_a".to_string(),
            },
            LuaSystemEntry {
                instance_id: 2,
                system_key: key.clone(),
                last_run: 0,
                last_run_real_time: Instant::now(),
                state_id: 1,
                system_name: "system_b".to_string(),
            },
            LuaSystemEntry {
                instance_id: 3,
                system_key: key.clone(),
                last_run: 0,
                last_run_real_time: Instant::now(),
                state_id: 0,
                system_name: "system_c".to_string(),
            },
        ];
        
        let groups = group_systems_by_state(&entries);
        
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].state_id, 0);
        assert_eq!(groups[0].system_indices, vec![0, 2]);
        assert_eq!(groups[1].state_id, 1);
        assert_eq!(groups[1].system_indices, vec![1]);
    }
    
    #[test]
    fn test_should_use_parallel() {
        let config = LuaParallelConfig::default();
        
        assert!(!should_use_parallel(&config, 0));
        assert!(!should_use_parallel(&config, 1));
        assert!(should_use_parallel(&config, 2) == cfg!(feature = "parallel-systems"));
        assert!(should_use_parallel(&config, 3) == cfg!(feature = "parallel-systems"));
        
        let disabled_config = LuaParallelConfig { enabled: false, ..Default::default() };
        assert!(!should_use_parallel(&disabled_config, 5));
    }
}
