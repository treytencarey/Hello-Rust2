// Network asset client for downloading scripts and assets on-demand
//
// This module implements:
// - Asset request queue for tracking pending downloads
// - Chunked transfer support for large assets
// - Encryption for asset data
// - Integration with Lua coroutine-based blocking

use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Channel ID for asset requests/responses
pub const ASSET_CHANNEL: u8 = 5;

/// Magic bytes for encrypted chunks
const ENCRYPTION_MAGIC: [u8; 4] = [0xAE, 0x53, 0x45, 0x54]; // "ASET" in hex

/// Simple XOR-based encryption key (in production, use proper key exchange)
const ENCRYPTION_KEY: [u8; 32] = [
    0x1A, 0x2B, 0x3C, 0x4D, 0x5E, 0x6F, 0x70, 0x81,
    0x92, 0xA3, 0xB4, 0xC5, 0xD6, 0xE7, 0xF8, 0x09,
    0x10, 0x21, 0x32, 0x43, 0x54, 0x65, 0x76, 0x87,
    0x98, 0xA9, 0xBA, 0xCB, 0xDC, 0xED, 0xFE, 0x0F,
];

/// Maximum chunk size for transfers (64KB)
pub const CHUNK_SIZE: usize = 64 * 1024;

/// Request timeout in seconds
pub const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Status of an asset download request
#[derive(Clone, Debug, PartialEq)]
pub enum AssetRequestStatus {
    /// Request queued but not yet sent
    Pending,
    /// Request sent, waiting for first response
    Requested,
    /// Download in progress with progress percentage
    Downloading { 
        received_bytes: usize,
        total_bytes: usize,
    },
    /// Download complete, asset available
    Complete,
    /// Asset is already up-to-date (hash matched)
    UpToDate,
    /// Download failed with error message
    Error(String),
}

impl AssetRequestStatus {
    /// Get progress as a percentage (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        match self {
            Self::Pending => 0.0,
            Self::Requested => 0.0,
            Self::Downloading { received_bytes, total_bytes } => {
                if *total_bytes == 0 {
                    0.0
                } else {
                    *received_bytes as f32 / *total_bytes as f32
                }
            }
            Self::Complete => 1.0,
            Self::UpToDate => 1.0,
            Self::Error(_) => 0.0,
        }
    }
    
    /// Check if the request is still in progress
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending | Self::Requested | Self::Downloading { .. })
    }
    
    /// Check if asset is available (complete or up-to-date)
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Complete | Self::UpToDate)
    }
}

/// Type of asset request
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AssetType {
    /// Lua script file
    Script,
    /// Image/texture asset
    Image,
    /// Other binary asset
    Binary,
}

/// Message sent from client to server to request an asset
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetRequestMessage {
    /// Request ID for matching response
    pub request_id: u64,
    /// Path to the requested asset (relative to assets/ or relative to context)
    pub path: String,
    /// Type of asset
    pub asset_type: AssetType,
    /// Optional hash of local file (for up-to-date check)
    /// If provided and matches server's hash, server responds with is_up_to_date=true
    pub local_hash: Option<String>,
    /// Optional context path (the script that's making the require)
    /// Server uses this to resolve relative paths
    /// e.g., if context is "scripts/examples/main.lua" and path is "foo.lua",
    /// server tries "scripts/examples/foo.lua" first
    pub context_path: Option<String>,
}

/// Message sent from server to client with asset data
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetResponseMessage {
    /// Request ID matching the original request
    pub request_id: u64,
    /// Path to the asset
    pub path: String,
    /// True if local hash matched - no data transfer needed
    pub is_up_to_date: bool,
    /// Hash of the server's file (for caching)
    pub server_hash: Option<String>,
    /// Current chunk index (0-based)
    pub chunk_index: u32,
    /// Total number of chunks
    pub total_chunks: u32,
    /// Total size of the complete asset in bytes
    pub total_size: usize,
    /// Encrypted chunk data
    pub data: Vec<u8>,
    /// Optional error message if request failed
    pub error: Option<String>,
}

/// Subscribe/Unsubscribe message from client to server for file sync
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AssetSubscriptionMessage {
    /// Client wants updates for these paths (when require with reload=true completes)
    Subscribe { paths: Vec<String>, instance_id: u64 },
    /// Client no longer needs updates for these paths from this instance
    Unsubscribe { paths: Vec<String>, instance_id: u64 },
    /// Client instance stopped, remove all its subscriptions
    UnsubscribeAll { instance_id: u64 },
}

/// Wrapper enum for all client-to-server messages
/// This ensures proper type discrimination when deserializing with bincode
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ClientToServerMessage {
    /// Asset request
    Request(AssetRequestMessage),
    /// Subscription control
    Subscription(AssetSubscriptionMessage),
}

/// Server pushes file update to client when a subscribed file changes
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetUpdateNotification {
    /// Path that changed (relative to assets/)
    pub path: String,
    /// Hash of the new file content
    pub server_hash: String,
    /// Encrypted chunk data
    pub data: Vec<u8>,
    /// Total size of the complete file in bytes
    pub total_size: usize,
    /// Current chunk index (0-based)
    pub chunk_index: u32,
    /// Total number of chunks
    pub total_chunks: u32,
}

/// Wrapper enum for all server-to-client messages
/// This ensures proper type discrimination when deserializing with bincode
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ServerToClientMessage {
    /// Asset response (reply to request)
    Response(AssetResponseMessage),
    /// File update notification (push)
    Update(AssetUpdateNotification),
}

/// Resource to queue pending file update notifications
/// Updates are received by one system and processed by another
#[derive(Resource, Default)]
pub struct PendingAssetUpdates {
    updates: std::sync::Mutex<Vec<AssetUpdateNotification>>,
}

impl PendingAssetUpdates {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Queue an update notification for processing
    pub fn queue(&self, notification: AssetUpdateNotification) {
        self.updates.lock().unwrap().push(notification);
    }
    
    /// Take all pending updates for processing
    pub fn take_all(&self) -> Vec<AssetUpdateNotification> {
        std::mem::take(&mut *self.updates.lock().unwrap())
    }
}

/// Individual asset request tracking
#[derive(Clone, Debug)]
pub struct AssetRequest {
    /// Unique request ID
    pub request_id: u64,
    /// Path being requested
    pub path: String,
    /// Type of asset
    pub asset_type: AssetType,
    /// Current status
    pub status: AssetRequestStatus,
    /// When the request was created
    pub created_at: Instant,
    /// Accumulated data chunks (in order)
    pub chunks: Vec<Vec<u8>>,
    /// Expected total chunks
    pub total_chunks: u32,
    /// Callbacks to execute when complete (registry keys)
    pub callbacks: Vec<Arc<mlua::RegistryKey>>,
    /// Coroutine registry keys waiting for this asset
    pub waiting_coroutines: Vec<Arc<mlua::RegistryKey>>,
    /// Context path (the script making the request, for relative resolution on server)
    pub context_path: Option<String>,
}

impl AssetRequest {
    pub fn new(request_id: u64, path: String, asset_type: AssetType, context_path: Option<String>) -> Self {
        Self {
            request_id,
            path,
            asset_type,
            status: AssetRequestStatus::Pending,
            created_at: Instant::now(),
            chunks: Vec::new(),
            total_chunks: 0,
            callbacks: Vec::new(),
            waiting_coroutines: Vec::new(),
            context_path,
        }
    }
    
    /// Check if request has timed out
    pub fn is_timed_out(&self) -> bool {
        self.created_at.elapsed() > Duration::from_secs(REQUEST_TIMEOUT_SECS)
    }
    
    /// Reassemble complete asset from chunks
    pub fn reassemble_data(&self) -> Option<Vec<u8>> {
        if self.chunks.len() != self.total_chunks as usize {
            return None;
        }
        
        let mut data = Vec::new();
        for chunk in &self.chunks {
            data.extend(chunk);
        }
        Some(data)
    }
}

/// Resource tracking all pending asset requests
#[derive(Resource, Clone, Default)]
pub struct PendingAssetRequests {
    /// Map from path to request
    requests: Arc<Mutex<HashMap<String, AssetRequest>>>,
    /// Map from request ID to path (for quick lookup)
    request_id_to_path: Arc<Mutex<HashMap<u64, String>>>,
    /// Counter for generating unique request IDs
    next_request_id: Arc<std::sync::atomic::AtomicU64>,
    /// Completed assets ready for use (path -> data)
    completed_assets: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    /// Pending subscription messages to send to server
    pending_subscriptions: Arc<Mutex<Vec<AssetSubscriptionMessage>>>,
}

impl PendingAssetRequests {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Queue a new asset request, returns the request ID
    /// If a request for this path is already pending, returns existing request ID
    /// context_path is the script making the request (for relative path resolution on server)
    pub fn queue_request(&self, path: String, asset_type: AssetType, context_path: Option<String>) -> u64 {
        let mut requests = self.requests.lock().unwrap();
        
        // Check if already pending
        if let Some(existing) = requests.get(&path) {
            return existing.request_id;
        }
        
        // Generate new request ID
        let request_id = self.next_request_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // Create and store request
        let request = AssetRequest::new(request_id, path.clone(), asset_type, context_path);
        requests.insert(path.clone(), request);
        self.request_id_to_path.lock().unwrap().insert(request_id, path);
        
        request_id
    }
    
    /// Get a pending request by path
    pub fn get_request(&self, path: &str) -> Option<AssetRequest> {
        self.requests.lock().unwrap().get(path).cloned()
    }
    
    /// Get a pending request by ID
    pub fn get_request_by_id(&self, request_id: u64) -> Option<AssetRequest> {
        let path = self.request_id_to_path.lock().unwrap().get(&request_id)?.clone();
        self.get_request(&path)
    }
    
    /// Check if a path has a pending request
    pub fn has_pending_request(&self, path: &str) -> bool {
        if let Some(req) = self.requests.lock().unwrap().get(path) {
            req.status.is_pending()
        } else {
            false
        }
    }
    
    /// Update request status
    pub fn update_status(&self, path: &str, status: AssetRequestStatus) {
        if let Some(req) = self.requests.lock().unwrap().get_mut(path) {
            req.status = status;
        }
    }
    
    /// Add a received chunk to a request
    pub fn add_chunk(&self, request_id: u64, chunk_index: u32, total_chunks: u32, total_size: usize, data: Vec<u8>) -> Option<String> {
        let path = self.request_id_to_path.lock().unwrap().get(&request_id)?.clone();
        let mut requests = self.requests.lock().unwrap();
        
        if let Some(req) = requests.get_mut(&path) {
            // Initialize chunks vector if needed
            if req.chunks.is_empty() {
                req.chunks.resize(total_chunks as usize, Vec::new());
                req.total_chunks = total_chunks;
            }
            
            // Store chunk
            if (chunk_index as usize) < req.chunks.len() {
                req.chunks[chunk_index as usize] = data;
            }
            
            // Update status
            let received_bytes: usize = req.chunks.iter().map(|c| c.len()).sum();
            req.status = AssetRequestStatus::Downloading { received_bytes, total_bytes: total_size };
            
            // Check if complete
            if req.chunks.iter().all(|c| !c.is_empty()) {
                req.status = AssetRequestStatus::Complete;
            }
            
            return Some(path);
        }
        
        None
    }
    
    /// Mark a request as complete and move data to completed storage
    pub fn complete_request(&self, path: &str, data: Vec<u8>) {
        self.completed_assets.lock().unwrap().insert(path.to_string(), data);
        self.requests.lock().unwrap().remove(path);
        // Note: We keep the request_id_to_path mapping for a while for late chunks
    }
    
    /// Take completed asset data (removes from storage)
    pub fn take_completed(&self, path: &str) -> Option<Vec<u8>> {
        self.completed_assets.lock().unwrap().remove(path)
    }
    
    /// Check if completed asset is available
    pub fn is_completed(&self, path: &str) -> bool {
        self.completed_assets.lock().unwrap().contains_key(path)
    }
    
    /// Check if asset is marked as up-to-date (no download needed)
    pub fn is_up_to_date(&self, path: &str) -> bool {
        if let Some(req) = self.requests.lock().unwrap().get(path) {
            matches!(req.status, AssetRequestStatus::UpToDate)
        } else {
            false
        }
    }
    
    /// Mark request as up-to-date and remove from pending
    pub fn mark_up_to_date(&self, path: &str) {
        self.requests.lock().unwrap().remove(path);
    }
    
    /// Add a callback to be called when request completes
    pub fn add_callback(&self, path: &str, callback: Arc<mlua::RegistryKey>) {
        if let Some(req) = self.requests.lock().unwrap().get_mut(path) {
            req.callbacks.push(callback);
        }
    }
    
    /// Add a waiting coroutine for this request
    pub fn add_waiting_coroutine(&self, path: &str, coroutine: Arc<mlua::RegistryKey>) {
        if let Some(req) = self.requests.lock().unwrap().get_mut(path) {
            req.waiting_coroutines.push(coroutine);
        }
    }
    
    /// Take callbacks for a completed request
    pub fn take_callbacks(&self, path: &str) -> Vec<Arc<mlua::RegistryKey>> {
        if let Some(req) = self.requests.lock().unwrap().get_mut(path) {
            std::mem::take(&mut req.callbacks)
        } else {
            Vec::new()
        }
    }
    
    /// Take waiting coroutines for a completed request
    pub fn take_waiting_coroutines(&self, path: &str) -> Vec<Arc<mlua::RegistryKey>> {
        if let Some(req) = self.requests.lock().unwrap().get_mut(path) {
            std::mem::take(&mut req.waiting_coroutines)
        } else {
            Vec::new()
        }
    }
    
    /// Get all requests to send (status = Pending)
    pub fn drain_pending_requests(&self) -> Vec<AssetRequestMessage> {
        let mut requests = self.requests.lock().unwrap();
        let mut to_send = Vec::new();
        
        for req in requests.values_mut() {
            if req.status == AssetRequestStatus::Pending {
                // Compute local hash if file exists
                let local_hash = compute_local_file_hash(&req.path, &req.asset_type);
                
                to_send.push(AssetRequestMessage {
                    request_id: req.request_id,
                    path: req.path.clone(),
                    asset_type: req.asset_type.clone(),
                    local_hash,
                    context_path: req.context_path.clone(),
                });
                req.status = AssetRequestStatus::Requested;
            }
        }
        
        to_send
    }
    
    /// Get timed out requests
    pub fn get_timed_out_requests(&self) -> Vec<String> {
        self.requests.lock().unwrap()
            .iter()
            .filter(|(_, req)| req.is_timed_out() && req.status.is_pending())
            .map(|(path, _)| path.clone())
            .collect()
    }
    
    /// Mark request as failed
    pub fn mark_failed(&self, path: &str, error: String) {
        if let Some(req) = self.requests.lock().unwrap().get_mut(path) {
            req.status = AssetRequestStatus::Error(error);
        }
    }
    
    /// Queue a subscription message to be sent to server
    pub fn queue_subscription(&self, message: AssetSubscriptionMessage) {
        self.pending_subscriptions.lock().unwrap().push(message);
    }
    
    /// Drain pending subscription messages for sending
    pub fn drain_pending_subscriptions(&self) -> Vec<AssetSubscriptionMessage> {
        std::mem::take(&mut *self.pending_subscriptions.lock().unwrap())
    }
}

/// Simple XOR encryption/decryption for asset data
pub fn encrypt_data(data: &[u8]) -> Vec<u8> {
    let mut encrypted = Vec::with_capacity(ENCRYPTION_MAGIC.len() + data.len());
    encrypted.extend_from_slice(&ENCRYPTION_MAGIC);
    
    for (i, byte) in data.iter().enumerate() {
        encrypted.push(byte ^ ENCRYPTION_KEY[i % ENCRYPTION_KEY.len()]);
    }
    
    encrypted
}

/// Decrypt asset data
pub fn decrypt_data(encrypted: &[u8]) -> Result<Vec<u8>, &'static str> {
    if encrypted.len() < ENCRYPTION_MAGIC.len() {
        return Err("Data too short");
    }
    
    // Verify magic bytes
    if &encrypted[..4] != &ENCRYPTION_MAGIC {
        return Err("Invalid magic bytes - data may not be encrypted");
    }
    
    let data = &encrypted[4..];
    let mut decrypted = Vec::with_capacity(data.len());
    
    for (i, byte) in data.iter().enumerate() {
        decrypted.push(byte ^ ENCRYPTION_KEY[i % ENCRYPTION_KEY.len()]);
    }
    
    Ok(decrypted)
}

/// Split data into encrypted chunks for transfer
pub fn chunk_and_encrypt(data: &[u8]) -> Vec<Vec<u8>> {
    data.chunks(CHUNK_SIZE)
        .map(|chunk| encrypt_data(chunk))
        .collect()
}

/// Compute a simple hash of data (FNV-1a 64-bit)
/// This is NOT cryptographically secure but is fast and good for change detection
pub fn compute_hash(data: &[u8]) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    
    let mut hash = FNV_OFFSET;
    for byte in data {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    
    format!("{:016x}", hash)
}

/// Compute hash of a local file if it exists
/// Returns None if file doesn't exist
pub fn compute_local_file_hash(path: &str, asset_type: &AssetType) -> Option<String> {
    let file_path = match asset_type {
        AssetType::Script => std::path::Path::new("assets").join(path),
        _ => std::path::Path::new("assets").join(path),
    };
    
    match std::fs::read(&file_path) {
        Ok(data) => {
            let hash = compute_hash(&data);
            bevy::log::debug!("📊 Local file hash for '{}': {}", path, hash);
            Some(hash)
        }
        Err(_) => {
            bevy::log::debug!("📊 No local file for '{}' (hash check skipped)", path);
            None
        }
    }
}

/// Component marker for asset server connection
#[derive(Component, Default)]
pub struct AssetServerMarker;

/// Component marker for asset client connection
#[derive(Component, Default)]
pub struct AssetClientMarker;

/// Pending coroutine waiting for asset download
#[derive(Clone)]
pub struct PendingScriptCoroutine {
    /// The Lua coroutine registry key
    pub coroutine_key: Arc<mlua::RegistryKey>,
    /// Path of the script being downloaded
    pub awaiting_path: String,
    /// Script instance ID for entity tracking
    pub instance_id: u64,
}

/// Resource tracking pending coroutines
#[derive(Resource, Clone, Default)]
pub struct PendingCoroutines {
    coroutines: Arc<Mutex<Vec<PendingScriptCoroutine>>>,
}

impl PendingCoroutines {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a pending coroutine
    pub fn add(&self, coroutine: PendingScriptCoroutine) {
        self.coroutines.lock().unwrap().push(coroutine);
    }
    
    /// Take coroutines waiting for a specific path
    pub fn take_waiting_for(&self, path: &str) -> Vec<PendingScriptCoroutine> {
        let mut coroutines = self.coroutines.lock().unwrap();
        let mut waiting = Vec::new();
        let mut remaining = Vec::new();
        
        for co in coroutines.drain(..) {
            if co.awaiting_path == path {
                waiting.push(co);
            } else {
                remaining.push(co);
            }
        }
        
        *coroutines = remaining;
        waiting
    }
    
    /// Get count of pending coroutines
    pub fn count(&self) -> usize {
        self.coroutines.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_roundtrip() {
        let original = b"Hello, World! This is test data.";
        let encrypted = encrypt_data(original);
        let decrypted = decrypt_data(&encrypted).unwrap();
        assert_eq!(original.as_slice(), decrypted.as_slice());
    }
    
    #[test]
    fn test_chunk_and_encrypt() {
        let data = vec![0u8; CHUNK_SIZE * 2 + 100]; // 2 full chunks + partial
        let chunks = chunk_and_encrypt(&data);
        assert_eq!(chunks.len(), 3);
    }
    
    #[test]
    fn test_request_status_progress() {
        assert_eq!(AssetRequestStatus::Pending.progress(), 0.0);
        assert_eq!(AssetRequestStatus::Complete.progress(), 1.0);
        assert_eq!(
            AssetRequestStatus::Downloading { received_bytes: 50, total_bytes: 100 }.progress(),
            0.5
        );
    }
}
