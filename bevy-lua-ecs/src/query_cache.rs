use bevy::ecs::component::ComponentId;
use bevy::prelude::*;
use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Cached query result with pre-serialized Lua components
#[derive(Clone)]
pub struct CachedEntityResult {
    pub entity_bits: u64,
    /// Pre-created registry keys for each component
    /// Key: component name, Value: registry key for the component data
    pub component_keys: HashMap<String, Arc<LuaRegistryKey>>,
}

/// Cached component lookup result
#[derive(Clone, Debug)]
pub enum CachedComponentInfo {
    /// Rust component with its ComponentId
    Rust(ComponentId),
    /// Lua custom component (uses LuaCustomComponents storage)
    Lua,
    /// Component not found
    NotFound,
}

/// Per-frame cache for Lua query results
/// Caches the full serialized component data, not just entity IDs
/// Cache automatically clears at the start of each frame
#[derive(Resource, Default, Clone)]
pub struct LuaQueryCache {
    inner: Arc<Mutex<QueryCacheInner>>,
}

#[derive(Default)]
struct QueryCacheInner {
    /// The frame number this cache was last used
    last_frame: u64,
    /// Cache: sorted component names -> cached entity results
    cache: HashMap<Vec<String>, Vec<CachedEntityResult>>,
    /// Permanent cache: component name -> is_rust_component (never clears)
    /// This avoids expensive type registry lookups on every query
    component_type_cache: HashMap<String, bool>,
    /// Permanent cache: component name -> ComponentId (never clears)
    /// This avoids the expensive type_registry lookups on every query
    component_id_cache: HashMap<String, CachedComponentInfo>,
}

impl LuaQueryCache {
    /// Get cached ComponentId for a component name (permanent cache)
    /// Returns None if not yet cached
    pub fn get_component_info(&self, name: &str) -> Option<CachedComponentInfo> {
        let inner = self.inner.lock().unwrap();
        inner.component_id_cache.get(name).cloned()
    }

    /// Cache ComponentId for a component name (permanent, never clears)
    pub fn cache_component_info(&self, name: &str, info: CachedComponentInfo) {
        let mut inner = self.inner.lock().unwrap();
        inner.component_id_cache.insert(name.to_string(), info);
    }

    /// Batch lookup: get cached component info for multiple names
    /// Returns (cached_results, names_needing_lookup)
    pub fn get_component_infos(&self, names: &[String]) -> (HashMap<String, CachedComponentInfo>, Vec<String>) {
        let inner = self.inner.lock().unwrap();
        let mut cached = HashMap::new();
        let mut needs_lookup = Vec::new();

        for name in names {
            if let Some(info) = inner.component_id_cache.get(name) {
                cached.insert(name.clone(), info.clone());
            } else {
                needs_lookup.push(name.clone());
            }
        }

        (cached, needs_lookup)
    }

    /// Check if a component name refers to a Rust component (cached permanently)
    /// Returns None if not yet cached (caller should do lookup and call cache_component_type)
    pub fn is_rust_component(&self, name: &str) -> Option<bool> {
        let inner = self.inner.lock().unwrap();
        inner.component_type_cache.get(name).copied()
    }

    /// Cache whether a component name is a Rust component (permanent, never clears)
    pub fn cache_component_type(&self, name: &str, is_rust: bool) {
        let mut inner = self.inner.lock().unwrap();
        inner.component_type_cache.insert(name.to_string(), is_rust);
    }

    /// Check if any components in the list are Rust components, using cached lookups
    /// Returns (has_rust_components, names_that_need_lookup)
    pub fn check_rust_components(&self, names: &[String]) -> (Option<bool>, Vec<String>) {
        let inner = self.inner.lock().unwrap();
        let mut all_cached = true;
        let mut has_rust = false;
        let mut needs_lookup = Vec::new();

        for name in names {
            if let Some(is_rust) = inner.component_type_cache.get(name) {
                if *is_rust {
                    has_rust = true;
                }
            } else {
                all_cached = false;
                needs_lookup.push(name.clone());
            }
        }

        if all_cached {
            (Some(has_rust), needs_lookup)
        } else {
            (None, needs_lookup)
        }
    }
    
    /// Get cached results for a query (with full component data)
    pub fn get_full(&self, components: &[String], current_frame: u64) -> Option<Vec<CachedEntityResult>> {
        let mut inner = self.inner.lock().unwrap();
        
        // Auto-clear if this is a new frame
        if inner.last_frame != current_frame {
            inner.cache.clear();
            inner.last_frame = current_frame;
            return None;
        }
        
        let key = Self::make_key(components);
        inner.cache.get(&key).cloned()
    }
    
    /// Store query result with full component data
    pub fn insert_full(&self, components: &[String], results: Vec<CachedEntityResult>, current_frame: u64) {
        let mut inner = self.inner.lock().unwrap();
        
        // Auto-clear if this is a new frame
        if inner.last_frame != current_frame {
            inner.cache.clear();
            inner.last_frame = current_frame;
        }
        
        let key = Self::make_key(components);
        inner.cache.insert(key, results);
    }
    
    /// Legacy: Get cached entity IDs for a query (for compatibility)
    pub fn get(&self, components: &[String], current_frame: u64) -> Option<Vec<u64>> {
        self.get_full(components, current_frame)
            .map(|results| results.iter().map(|r| r.entity_bits).collect())
    }
    
    /// Legacy: Store query result in cache (just entity IDs - not recommended)
    pub fn insert(&self, components: &[String], entities: Vec<u64>, current_frame: u64) {
        let results: Vec<CachedEntityResult> = entities
            .into_iter()
            .map(|entity_bits| CachedEntityResult {
                entity_bits,
                component_keys: HashMap::new(),
            })
            .collect();
        self.insert_full(components, results, current_frame);
    }
    
    fn make_key(components: &[String]) -> Vec<String> {
        let mut key = components.to_vec();
        key.sort();
        key
    }
}
