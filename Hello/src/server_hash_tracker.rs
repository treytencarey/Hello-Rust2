// Server hash tracker for LocalNewer detection
//
// This module tracks the last-known server hash for files that the client
// has downloaded or received updates for. When a local file changes, we can
// compare its hash against the stored server hash to detect if the local
// version is newer (modified offline).

use bevy::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

/// Information about a file's last-known server state
#[derive(Clone, Debug)]
pub struct ServerHashInfo {
    /// Hash of the file on the server
    pub hash: String,
    /// When we last synced with the server
    pub last_sync: Instant,
}

/// Resource tracking last-known server hashes for files
/// 
/// This is used for LocalNewer detection:
/// 1. When we download a file or receive an update, we store the server's hash
/// 2. When a local file changes, we compare its hash against the stored server hash
/// 3. If they differ, we know the local version is newer and emit AssetLocalNewerEvent
#[derive(Resource, Default)]
pub struct ServerFileHashes {
    hashes: HashMap<String, ServerHashInfo>,
}

impl ServerFileHashes {
    /// Create a new empty hash tracker
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Update the server hash for a file path
    /// Call this when a file is downloaded or an update is received
    pub fn update(&mut self, path: &str, hash: String) {
        debug!("📊 [HASH_TRACKER] Updating server hash for '{}': {}", path, hash);
        self.hashes.insert(path.to_string(), ServerHashInfo {
            hash,
            last_sync: Instant::now(),
        });
    }
    
    /// Get the last-known server hash for a path
    pub fn get_hash(&self, path: &str) -> Option<&str> {
        self.hashes.get(path).map(|i| i.hash.as_str())
    }
    
    /// Get full info for a path
    pub fn get_info(&self, path: &str) -> Option<&ServerHashInfo> {
        self.hashes.get(path)
    }
    
    /// Check if we have a hash for this path
    pub fn has_hash(&self, path: &str) -> bool {
        self.hashes.contains_key(path)
    }
    
    /// Remove a path from tracking
    pub fn remove(&mut self, path: &str) {
        self.hashes.remove(path);
    }
    
    /// Get all tracked paths
    pub fn paths(&self) -> impl Iterator<Item = &String> {
        self.hashes.keys()
    }
    
    /// Clear all tracked hashes
    pub fn clear(&mut self) {
        self.hashes.clear();
    }
}
