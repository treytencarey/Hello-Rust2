// Server-side subscription registry for file sync
//
// Tracks which clients are subscribed to which file paths
// and manages file watching to detect changes and broadcast updates

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver};
use std::path::PathBuf;

use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event};
use bevy_lua_ecs::path_utils::to_forward_slash;

/// Resource that holds the file watcher and receives change events
#[derive(Resource)]
pub struct FileWatcherResource {
    /// The file watcher
    watcher: Option<RecommendedWatcher>,
    /// Receiver for file change events - wrapped in Arc<Mutex> for thread safety
    rx: Arc<Mutex<Option<Receiver<notify::Result<Event>>>>>,
    /// Currently watched paths (to avoid re-watching)
    watched_paths: HashSet<PathBuf>,
}

impl Default for FileWatcherResource {
    fn default() -> Self {
        Self::new()
    }
}

impl FileWatcherResource {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        
        let watcher = notify::recommended_watcher(tx).ok();
        
        if watcher.is_some() {
            info!("📁 [FILE WATCHER] Initialized successfully");
        } else {
            warn!("📁 [FILE WATCHER] Failed to initialize");
        }
        
        Self {
            watcher,
            rx: Arc::new(Mutex::new(Some(rx))),
            watched_paths: HashSet::new(),
        }
    }
    
    /// Start watching a path (relative to assets/)
    pub fn watch(&mut self, relative_path: &str) {
        // Build the path and canonicalize it for consistent comparison
        let raw_path = PathBuf::from("assets").join(relative_path);
        
        // Try to canonicalize for absolute, normalized path
        // Fall back to raw path if file doesn't exist yet
        let full_path = raw_path.canonicalize().unwrap_or(raw_path);
        
        // Skip if already watching
        if self.watched_paths.contains(&full_path) {
            debug!("📁 [FILE WATCHER] Already watching: {:?}", full_path);
            return;
        }
        
        if let Some(ref mut watcher) = self.watcher {
            // Watch the specific file, not recursively
            match watcher.watch(&full_path, RecursiveMode::NonRecursive) {
                Ok(()) => {
                    info!("📁 [FILE WATCHER] Started watching: {:?}", full_path);
                    self.watched_paths.insert(full_path);
                }
                Err(e) => {
                    warn!("📁 [FILE WATCHER] Failed to watch {:?}: {}", full_path, e);
                }
            }
        }
    }
    
    /// Stop watching a path
    pub fn unwatch(&mut self, relative_path: &str) {
        // Build the path and canonicalize it for consistent comparison
        let raw_path = PathBuf::from("assets").join(relative_path);
        let full_path = raw_path.canonicalize().unwrap_or(raw_path);
        
        if !self.watched_paths.contains(&full_path) {
            return;
        }
        
        if let Some(ref mut watcher) = self.watcher {
            match watcher.unwatch(&full_path) {
                Ok(()) => {
                    info!("📁 [FILE WATCHER] Stopped watching: {:?}", full_path);
                    self.watched_paths.remove(&full_path);
                }
                Err(e) => {
                    warn!("📁 [FILE WATCHER] Failed to unwatch {:?}: {}", full_path, e);
                }
            }
        }
    }
    
    /// Poll for file change events, returns list of changed paths (relative to assets/)
    pub fn poll_changes(&self) -> Vec<String> {
        let mut changes = Vec::new();
        
        // Get the canonical path to assets/ directory for prefix stripping
        let assets_prefix = std::path::Path::new("assets")
            .canonicalize()
            .ok();
        
        let rx_guard = self.rx.lock().unwrap();
        if let Some(ref rx) = *rx_guard {
            // Non-blocking receive all pending events
            while let Ok(result) = rx.try_recv() {
                match result {
                    Ok(event) => {
                        // We're interested in Modify events
                        if matches!(event.kind, notify::EventKind::Modify(_)) {
                            for path in event.paths {
                                // Convert to relative path by stripping the canonicalized assets prefix
                                let relative_result = if let Some(ref prefix) = assets_prefix {
                                    path.strip_prefix(prefix).ok()
                                } else {
                                    // Fall back to trying with just "assets"
                                    path.strip_prefix("assets").ok()
                                };
                                
                                if let Some(relative) = relative_result {
                                    let relative_str = to_forward_slash(relative);
                                    info!("📁 [FILE WATCHER] File changed: {}", relative_str);
                                    changes.push(relative_str);
                                } else {
                                    warn!("📁 [FILE WATCHER] Could not strip assets prefix from: {:?}", path);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("📁 [FILE WATCHER] Error: {}", e);
                    }
                }
            }
        }
        
        changes
    }
}

/// Tracks client subscriptions for file sync
/// Used by the asset server to know which clients should receive file updates
#[derive(Resource, Clone, Default)]
pub struct AssetSubscriptionRegistry {
    /// Subscriptions: path -> (client_id -> set of instance_ids)
    /// Multiple instances on the same client can subscribe to the same path
    subscriptions: Arc<Mutex<HashMap<String, HashMap<u64, HashSet<u64>>>>>,
    /// Files currently being watched
    watched_paths: Arc<Mutex<HashSet<String>>>,
    /// Pending file change notifications: path -> list of client_ids to notify
    pending_notifications: Arc<Mutex<Vec<(String, Vec<u64>)>>>,
}

impl AssetSubscriptionRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Subscribe a client instance to updates for a path
    /// Returns true if this is the first subscriber for this path (should start watching)
    pub fn subscribe(&self, client_id: u64, path: String, instance_id: u64) -> bool {
        let mut subs = self.subscriptions.lock().unwrap();
        
        // Check if this is the first subscriber BEFORE modifying
        let was_empty = !subs.contains_key(&path);
        
        let path_subs = subs.entry(path.clone()).or_insert_with(HashMap::new);
        let client_instances = path_subs.entry(client_id).or_insert_with(HashSet::new);
        client_instances.insert(instance_id);
        
        // Return true if path was previously empty (first subscriber)
        if was_empty {
            info!("📡 [SERVER] First subscription for '{}', should start watching", path);
        } else {
            debug!("📡 [SERVER] Subscription for '{}' (client {}, instance {})", path, client_id, instance_id);
        }
        was_empty
    }
    
    /// Unsubscribe a client instance from a specific path
    /// Returns true if this was the last subscriber for this path (should stop watching)
    pub fn unsubscribe(&self, client_id: u64, path: &str, instance_id: u64) -> bool {
        let mut subs = self.subscriptions.lock().unwrap();
        
        let should_remove_path = if let Some(path_subs) = subs.get_mut(path) {
            if let Some(client_instances) = path_subs.get_mut(&client_id) {
                client_instances.remove(&instance_id);
                
                // Clean up empty client entry
                if client_instances.is_empty() {
                    path_subs.remove(&client_id);
                }
            }
            
            // Check if path is now empty
            path_subs.is_empty()
        } else {
            false
        };
        
        if should_remove_path {
            subs.remove(path);
            info!("📡 [SERVER] Last subscriber left '{}', should stop watching", path);
            return true;
        }
        false
    }
    
    /// Unsubscribe all paths for a specific client instance
    /// Returns list of paths that now have no subscribers
    pub fn unsubscribe_all(&self, client_id: u64, instance_id: u64) -> Vec<String> {
        let mut subs = self.subscriptions.lock().unwrap();
        
        // Collect paths to modify
        let paths: Vec<_> = subs.keys().cloned().collect();
        
        // First pass: modify entries and collect paths to remove
        let mut paths_to_remove = Vec::new();
        
        for path in paths {
            if let Some(path_subs) = subs.get_mut(&path) {
                if let Some(client_instances) = path_subs.get_mut(&client_id) {
                    client_instances.remove(&instance_id);
                    
                    if client_instances.is_empty() {
                        path_subs.remove(&client_id);
                    }
                }
                
                if path_subs.is_empty() {
                    paths_to_remove.push(path);
                }
            }
        }
        
        // Second pass: remove empty paths
        for path in &paths_to_remove {
            subs.remove(path);
        }
        
        if !paths_to_remove.is_empty() {
            info!("📡 [SERVER] Client {} instance {} unsubscribed, {} paths now empty", 
                client_id, instance_id, paths_to_remove.len());
        }
        
        paths_to_remove
    }
    
    /// Unsubscribe all paths for a client (when client disconnects)
    pub fn unsubscribe_client(&self, client_id: u64) -> Vec<String> {
        let mut subs = self.subscriptions.lock().unwrap();
        
        // Collect paths to modify
        let paths: Vec<_> = subs.keys().cloned().collect();
        
        // First pass: remove client and collect empty paths
        let mut paths_to_remove = Vec::new();
        
        for path in paths {
            if let Some(path_subs) = subs.get_mut(&path) {
                path_subs.remove(&client_id);
                
                if path_subs.is_empty() {
                    paths_to_remove.push(path);
                }
            }
        }
        
        // Second pass: remove empty paths
        for path in &paths_to_remove {
            subs.remove(path);
        }
        
        if !paths_to_remove.is_empty() {
            info!("📡 [SERVER] Client {} disconnected, {} paths now empty", client_id, paths_to_remove.len());
        }
        
        paths_to_remove
    }
    
    /// Get all client IDs that are subscribed to a path
    pub fn get_subscribers(&self, path: &str) -> Vec<u64> {
        self.subscriptions.lock().unwrap()
            .get(path)
            .map(|path_subs| path_subs.keys().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Check if a path has any subscribers
    pub fn has_subscribers(&self, path: &str) -> bool {
        self.subscriptions.lock().unwrap()
            .get(path)
            .map(|path_subs| !path_subs.is_empty())
            .unwrap_or(false)
    }
    
    /// Get all subscribed paths
    pub fn get_all_paths(&self) -> Vec<String> {
        self.subscriptions.lock().unwrap().keys().cloned().collect()
    }
    
    /// Queue a notification for a file change
    pub fn queue_notification(&self, path: String, client_ids: Vec<u64>) {
        if !client_ids.is_empty() {
            self.pending_notifications.lock().unwrap().push((path, client_ids));
        }
    }
    
    /// Take all pending notifications
    pub fn take_pending_notifications(&self) -> Vec<(String, Vec<u64>)> {
        std::mem::take(&mut *self.pending_notifications.lock().unwrap())
    }
}
