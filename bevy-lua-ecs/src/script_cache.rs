use mlua::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use bevy::prelude::Resource;
use bevy::log::{debug, info};

/// Normalize a path by converting backslashes to forward slashes
/// This ensures consistent path handling across Windows/Unix and at all code touchpoints
fn normalize_path(path: &str) -> String {
    path.replace("\\", "/")
}

/// Resource that caches loaded Lua modules and tracks dependencies
#[derive(Clone, Resource)]
pub struct ScriptCache {
    /// Cached module exports: path -> Lua registry key (execution results)
    modules: Arc<Mutex<HashMap<String, Arc<LuaRegistryKey>>>>,
    /// Cached module source code: path -> source string
    /// This persists across cache clears to avoid re-reading unchanged files from disk
    source_cache: Arc<Mutex<HashMap<String, String>>>,
    /// Dependency tracking: imported path -> map of { importer path -> should_reload }
    dependencies: Arc<Mutex<HashMap<String, HashMap<String, bool>>>>,
    /// Async dependency tracking: imported path -> set of importer paths
    /// Tracks modules that use require_async (so they can be invalidated when dependency changes)
    async_dependencies: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    /// Pending async callbacks: path -> list of callback registry keys
    pending_callbacks: Arc<Mutex<HashMap<String, Vec<Arc<LuaRegistryKey>>>>>,
    /// Callbacks to re-trigger on hot reload: imported path -> list of (callback, parent_instance_id) tuples
    hot_reload_callbacks: Arc<Mutex<HashMap<String, Vec<(Arc<LuaRegistryKey>, u64)>>>>,
    /// Module instance IDs: (path, parent_instance_id) -> module_instance_id (for entity cleanup)
    /// Allows same module to have different instances for different parents
    module_instances: Arc<Mutex<HashMap<(String, u64), u64>>>,
    /// Track which parent instance loaded each module instance: module_instance_id -> parent_instance_id
    module_parents: Arc<Mutex<HashMap<u64, u64>>>,
    /// Pending coroutines waiting for network downloads: path -> list of (coroutine_key, instance_id) tuples
    pending_download_coroutines: Arc<Mutex<HashMap<String, Vec<(Arc<LuaRegistryKey>, u64)>>>>,
    /// Paths that are binary downloads (assets) vs text (scripts)
    binary_download_paths: Arc<Mutex<HashSet<String>>>,
    /// Context paths for downloads: requested_path -> context_script_path (for server-side relative path resolution)
    download_context_paths: Arc<Mutex<HashMap<String, String>>>,
}

impl Default for ScriptCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptCache {
    pub fn new() -> Self {
        Self {
            modules: Arc::new(Mutex::new(HashMap::new())),
            source_cache: Arc::new(Mutex::new(HashMap::new())),
            dependencies: Arc::new(Mutex::new(HashMap::new())),
            async_dependencies: Arc::new(Mutex::new(HashMap::new())),
            pending_callbacks: Arc::new(Mutex::new(HashMap::new())),
            hot_reload_callbacks: Arc::new(Mutex::new(HashMap::new())),
            module_instances: Arc::new(Mutex::new(HashMap::new())),
            module_parents: Arc::new(Mutex::new(HashMap::new())),
            pending_download_coroutines: Arc::new(Mutex::new(HashMap::new())),
            binary_download_paths: Arc::new(Mutex::new(HashSet::new())),
            download_context_paths: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Get a cached module if it exists
    pub fn get_module(&self, path: &str) -> Option<Arc<LuaRegistryKey>> {
        self.modules.lock().unwrap().get(path).cloned()
    }
    
    /// Cache a loaded module
    pub fn cache_module(&self, path: String, registry_key: Arc<LuaRegistryKey>) {
        self.modules.lock().unwrap().insert(path, registry_key);
    }
    
    /// Track a dependency relationship
    /// importer_path imports imported_path
    pub fn add_dependency(&self, imported_path: String, importer_path: String, should_reload: bool) {
        self.dependencies.lock().unwrap()
            .entry(imported_path)
            .or_insert_with(HashMap::new)
            .insert(importer_path, should_reload);
    }
    
    /// Track an async dependency relationship (module uses require_async)
    /// importer_path calls require_async(imported_path)
    pub fn add_async_dependency(&self, imported_path: String, importer_path: String) {
        self.async_dependencies.lock().unwrap()
            .entry(imported_path)
            .or_insert_with(HashSet::new)
            .insert(importer_path);
    }
    
    /// Get all scripts that import a given module path
    /// Returns a list of (importer_path, should_reload) tuples
    pub fn get_importers(&self, module_path: &str) -> Vec<(String, bool)> {
        let path = normalize_path(module_path);
        let deps = self.dependencies.lock().unwrap();
        if let Some(importers) = deps.get(&path) {
            importers.iter()
                .map(|(importer, should_reload)| (importer.clone(), *should_reload))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Clear module cache for all dependencies of a given module path (recursively)
    /// This ensures that when the module re-executes, it sees fresh versions of its dependencies
    /// and their dependencies (transitive closure)
    pub fn clear_dependency_caches(&self, module_path: &str) {
        debug!("clear_dependency_caches called for '{}'", module_path);
        
        let mut to_clear = Vec::new();
        let mut to_visit = vec![module_path.to_string()];
        let mut visited = HashSet::new();
        
        let deps = self.dependencies.lock().unwrap();
        
        while let Some(current) = to_visit.pop() {
            if !visited.insert(current.clone()) {
                continue; // Already processed
            }
            
            // Find all modules that current depends on
            for (imported_path, importers) in deps.iter() {
                if importers.contains_key(&current) {
                    to_clear.push(imported_path.clone());
                    to_visit.push(imported_path.clone()); // Process transitive deps
                }
            }
        }
        drop(deps);
        
        // Clear their caches
        let mut modules = self.modules.lock().unwrap();
        for path in &to_clear {
            modules.remove(path);
        }
        
        if !to_clear.is_empty() {
            debug!("Cleared {} module cache(s) for '{}': {:?}", to_clear.len(), module_path, to_clear);
        } else {
            debug!("No caches to clear for '{}'", module_path);
        }
    }
    
    /// Invalidate a module and all modules that depend on it (for hot reload)
    pub fn invalidate_module(&self, path: &str) -> Vec<String> {
        let mut invalidated = Vec::new();
        let mut to_invalidate = vec![path.to_string()];
        let mut visited = HashSet::new();
        
        while let Some(current) = to_invalidate.pop() {
            if !visited.insert(current.clone()) {
                continue; // Already processed
            }
            
            // Remove from cache
            self.modules.lock().unwrap().remove(&current);
            invalidated.push(current.clone());
            
            // Invalidate all scripts that imported this one AND want to reload
            if let Some(dependents) = self.dependencies.lock().unwrap().get(&current) {
                for (dependent, should_reload) in dependents {
                    if *should_reload {
                        to_invalidate.push(dependent.clone());
                    }
                }
            }
        }
        
        // After invalidating the dependency chain, also clear the cache for modules
        // that have async dependencies on ANY of the invalidated modules.
        // This ensures that if a parent uses sync require() on a module that uses async require(),
        // it will see the fresh version with updated callbacks, not the stale cached version.
        // BUT we don't add them to `to_invalidate` - we just clear their cache.
        // This way, they'll re-execute when required, but won't trigger a full script reload.
        let mut modules_lock = self.modules.lock().unwrap();
        let async_deps = self.async_dependencies.lock().unwrap();
        for invalidated_path in &invalidated {
            if let Some(async_dependents) = async_deps.get(invalidated_path) {
                for dependent in async_dependents {
                    // Clear cache but don't add to invalidation list
                    if let Some(removed) = modules_lock.remove(dependent) {
                        drop(removed); // Explicitly drop the registry key
                    }
                }
            }
        }
        drop(modules_lock);
        drop(async_deps);
        
        invalidated
    }
    
    /// Register a callback for async loading
    pub fn register_callback(&self, path: String, callback: Arc<LuaRegistryKey>) {
        self.pending_callbacks.lock().unwrap()
            .entry(path)
            .or_insert_with(Vec::new)
            .push(callback);
    }
    
    /// Register a callback to be re-triggered on hot reload with parent context
    pub fn register_hot_reload_callback(&self, path: String, callback: Arc<LuaRegistryKey>, parent_instance_id: u64) {
        let normalized_path = normalize_path(&path);
        self.hot_reload_callbacks.lock().unwrap()
            .entry(normalized_path)
            .or_insert_with(Vec::new)
            .push((callback, parent_instance_id));
    }
    
    /// Get hot reload callbacks for a path with their parent instance IDs
    pub fn get_hot_reload_callbacks(&self, path: &str) -> Vec<(Arc<LuaRegistryKey>, u64)> {
        let normalized_path = normalize_path(path);
        let callbacks_map = self.hot_reload_callbacks.lock().unwrap();
        let result = callbacks_map.get(&normalized_path).cloned().unwrap_or_default();
        
        // Debug: show what we're looking for and what we have
        if !callbacks_map.is_empty() {
            let registered_paths: Vec<_> = callbacks_map.keys().collect();
            debug!("🔍 [HOT_RELOAD] Looking for callbacks for '{}', registered paths: {:?}", path, registered_paths);
        }
        
        if result.is_empty() {
            debug!("🔍 [HOT_RELOAD] No callbacks found for '{}'", path);
        } else {
            debug!("🔍 [HOT_RELOAD] Found {} callbacks for '{}'", result.len(), path);
        }
        
        result
    }
    
    /// Clear all hot reload callbacks for a path
    pub fn clear_hot_reload_callbacks(&self, path: &str) {
        let normalized_path = normalize_path(path);
        self.hot_reload_callbacks.lock().unwrap().remove(&normalized_path);
    }
    
    /// Remove hot reload callbacks that were registered by a specific parent instance
    /// This is called when cleaning up an instance to prevent callback accumulation
    pub fn remove_callbacks_for_instance(&self, instance_id: u64) {
        let mut callbacks = self.hot_reload_callbacks.lock().unwrap();
        
        // For each module path, filter out callbacks with matching parent_instance_id
        for (_path, callback_list) in callbacks.iter_mut() {
            callback_list.retain(|(_, parent_id)| *parent_id != instance_id);
        }
        
        // Remove empty entries
        callbacks.retain(|_, callback_list| !callback_list.is_empty());
    }
    
    /// Set the instance ID for a module with a specific parent
    pub fn set_module_instance(&self, path: String, parent_instance_id: u64, instance_id: u64) {
        self.module_instances.lock().unwrap().insert((path, parent_instance_id), instance_id);
    }
    
    /// Get the instance ID for a module with a specific parent
    pub fn get_module_instance(&self, path: &str, parent_instance_id: u64) -> Option<u64> {
        self.module_instances.lock().unwrap().get(&(path.to_string(), parent_instance_id)).copied()
    }
    
    /// Get all instance IDs for a module (across all parents)
    pub fn get_all_module_instances(&self, path: &str) -> Vec<u64> {
        self.module_instances.lock().unwrap()
            .iter()
            .filter_map(|((p, _parent), instance_id)| {
                if p == path {
                    Some(*instance_id)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Clear all instance mappings for a module (used during hot reload)
    pub fn clear_module_instances(&self, path: &str) {
        let mut instances = self.module_instances.lock().unwrap();
        instances.retain(|(p, _parent), _instance| p != path);
    }
    
    /// Set the parent instance ID for a module instance
    pub fn set_module_parent(&self, module_instance_id: u64, parent_instance_id: u64) {
        self.module_parents.lock().unwrap().insert(module_instance_id, parent_instance_id);
    }
    
    /// Get all module instance IDs that are children of a parent instance (recursively)
    pub fn get_child_instances(&self, parent_instance_id: u64) -> Vec<u64> {
        let mut children = Vec::new();
        let parents = self.module_parents.lock().unwrap();
        
        // Find direct children
        for (module_id, parent_id) in parents.iter() {
            if *parent_id == parent_instance_id {
                children.push(*module_id);
            }
        }
        
        // Recursively find grandchildren
        let direct_children = children.clone();
        for child_id in direct_children {
            children.extend(self.get_child_instances_internal(child_id, &parents));
        }
        
        children
    }
    
    /// Get all descendant instances recursively (children, grandchildren, etc.)
    /// This is crucial for proper cleanup during hot reload of nested modules
    pub fn get_all_descendant_instances(&self, instance_id: u64) -> Vec<u64> {
        let module_instances = self.module_instances.lock().unwrap();
        let mut descendants = Vec::new();
        let mut to_visit = vec![instance_id];
        let mut visited = HashSet::new();
        
        while let Some(current) = to_visit.pop() {
            if !visited.insert(current) {
                continue; // Avoid cycles
            }
            
            // Find all instances where current is the parent
            for ((_, parent_id), child_id) in module_instances.iter() {
                if *parent_id == current {
                    descendants.push(*child_id);
                    to_visit.push(*child_id);
                }
            }
        }
        
        descendants
    }
    
    /// Find all parent instances that have a specific module instance as a child
    pub fn get_parent_instances(&self, module_instance_id: u64) -> Vec<u64> {
        let parents = self.module_parents.lock().unwrap();
        if let Some(parent_id) = parents.get(&module_instance_id) {
            vec![*parent_id]
        } else {
            Vec::new()
        }
    }
    
    fn get_child_instances_internal(&self, parent_instance_id: u64, parents: &HashMap<u64, u64>) -> Vec<u64> {
        let mut children = Vec::new();
        
        for (module_id, parent_id) in parents.iter() {
            if *parent_id == parent_instance_id {
                children.push(*module_id);
                children.extend(self.get_child_instances_internal(*module_id, parents));
            }
        }
        
        children
    }
    
    /// Get and clear pending callbacks for a path
    pub fn take_callbacks(&self, path: &str) -> Vec<Arc<LuaRegistryKey>> {
        self.pending_callbacks.lock().unwrap()
            .remove(path)
            .unwrap_or_default()
    }
    
    /// Load module source from cache or filesystem
    /// Checks source cache first to avoid unnecessary disk I/O for unchanged files
    /// Path should be relative to assets/ (e.g., "scripts/example.lua")
    pub fn load_module_source(&self, relative_path: &str) -> Result<(String, PathBuf), String> {
        // Check source cache first
        {
            let cache = self.source_cache.lock().unwrap();
            if let Some(cached_source) = cache.get(relative_path) {
                let base_path = PathBuf::from("assets");
                let full_path = base_path.join(relative_path);
                return Ok((cached_source.clone(), full_path));
            }
        }
        
        // Not in cache, load from disk
        let base_path = PathBuf::from("assets");
        let full_path = base_path.join(relative_path);
        
        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| format!("Failed to load module '{}': {}", relative_path, e))?;
        
        // Cache the source
        self.source_cache.lock().unwrap().insert(relative_path.to_string(), content.clone());
       
        Ok((content, full_path))
    }
    
    /// Update source cache for a specific module (called when file changes)
    pub fn update_source(&self, relative_path: &str, source: String) {
        let normalized_path = normalize_path(relative_path);
        self.source_cache.lock().unwrap().insert(normalized_path, source);
    }
    
    /// Register a coroutine waiting for a download
    /// Called when require() or load_asset() encounters a missing local file and needs to download
    /// is_binary: true for binary assets (images, etc), false for text (scripts)
    /// context_path: the script making the request (for server-side relative path resolution)
    pub fn register_pending_download_coroutine(&self, path: String, coroutine_key: Arc<LuaRegistryKey>, instance_id: u64, is_binary: bool, context_path: Option<String>) {
        let normalized_path = normalize_path(&path);
        debug!("📥 Registering pending download for '{}' (instance {}, binary: {}, context: {:?})", normalized_path, instance_id, is_binary, context_path);
        self.pending_download_coroutines.lock().unwrap()
            .entry(normalized_path.clone())
            .or_insert_with(Vec::new)
            .push((coroutine_key, instance_id));
        
        // Track if this is a binary download
        if is_binary {
            self.binary_download_paths.lock().unwrap().insert(normalized_path.clone());
        }
        
        // Track the context path for this download
        if let Some(ctx) = context_path {
            self.download_context_paths.lock().unwrap().insert(normalized_path, ctx);
        }
    }
    
    /// Get the context path for a pending download
    pub fn get_download_context(&self, path: &str) -> Option<String> {
        let normalized_path = normalize_path(path);
        self.download_context_paths.lock().unwrap().get(&normalized_path).cloned()
    }
    
    /// Clear context tracking for a path (after download completes)
    pub fn clear_download_context(&self, path: &str) {
        let normalized_path = normalize_path(path);
        self.download_context_paths.lock().unwrap().remove(&normalized_path);
    }
    
    /// Check if a pending download is for a binary asset
    pub fn is_binary_download(&self, path: &str) -> bool {
        let normalized_path = normalize_path(path);
        self.binary_download_paths.lock().unwrap().contains(&normalized_path)
    }
    
    /// Clear binary download tracking for a path (after download completes)
    pub fn clear_binary_download(&self, path: &str) {
        let normalized_path = normalize_path(path);
        self.binary_download_paths.lock().unwrap().remove(&normalized_path);
    }
    
    /// Take all coroutines waiting for a specific path
    /// Called when download completes and we need to resume coroutines
    pub fn take_pending_download_coroutines(&self, path: &str) -> Vec<(Arc<LuaRegistryKey>, u64)> {
        let normalized_path = normalize_path(path);
        self.pending_download_coroutines.lock().unwrap()
            .remove(&normalized_path)
            .unwrap_or_default()
    }
    
    /// Check if any coroutines are waiting for a path
    pub fn has_pending_download_coroutines(&self, path: &str) -> bool {
        let normalized_path = normalize_path(path);
        self.pending_download_coroutines.lock().unwrap()
            .get(&normalized_path)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }
    
    /// Get all paths with pending download coroutines
    pub fn get_all_pending_download_paths(&self) -> Vec<String> {
        self.pending_download_coroutines.lock().unwrap()
            .keys()
            .cloned()
            .collect()
    }
}

/// Load a Lua module from the filesystem (standalone function for backwards compat)
/// Path should be relative to assets/ (e.g., "scripts/example.lua")
pub fn load_module_source(relative_path: &str) -> Result<(String, PathBuf), String> {
    // Resolve path relative to assets/
    let base_path = PathBuf::from("assets");
    let full_path = base_path.join(relative_path);
    
    // Read the file
    std::fs::read_to_string(&full_path)
        .map(|content| (content, full_path))
        .map_err(|e| format!("Failed to load module '{}': {}", relative_path, e))
}

/// Execute a module and return its exported value
pub fn execute_module(lua: &Lua, source: &str, module_name: &str) -> LuaResult<LuaValue> {
    // Save the previous __SCRIPT_NAME__ to restore it after execution (for nested requires)
    let previous_script_name: Option<String> = lua.globals().get("__SCRIPT_NAME__").ok();
    
    // Set __SCRIPT_NAME__ to the current module path (without the @ prefix)
    let script_path = module_name.strip_prefix('@').unwrap_or(module_name);
    lua.globals().set("__SCRIPT_NAME__", script_path)?;
    
    // Execute the module script
    // The return value of the script becomes the module export
    let result = lua.load(source).set_name(module_name).eval();
    
    // Restore the previous __SCRIPT_NAME__
    if let Some(prev) = previous_script_name {
        lua.globals().set("__SCRIPT_NAME__", prev)?;
    } else {
        lua.globals().set("__SCRIPT_NAME__", LuaNil)?;
    }
    
    result
}
