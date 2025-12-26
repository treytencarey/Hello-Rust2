// Upload state tracking for progress reporting
//
// This module tracks pending file uploads so we can:
// 1. Report upload progress to Lua via events
// 2. Handle chunked upload reassembly on the server
// 3. Resume or retry failed uploads

use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Information about a pending upload
#[derive(Clone, Debug)]
pub struct UploadInfo {
    /// Request ID for this upload
    pub request_id: u64,
    /// Destination path on server (relative to assets/)
    pub path: String,
    /// Total number of chunks
    pub total_chunks: u32,
    /// Number of chunks sent to server
    pub sent_chunks: u32,
    /// Number of chunks acknowledged by server
    pub acked_chunks: u32,
    /// Total file size in bytes
    pub total_size: usize,
    /// Hash of the complete file
    pub file_hash: String,
    /// When the upload started
    pub started_at: Instant,
    /// Whether we're waiting for conflict resolution
    pub has_conflict: bool,
    /// Server's hash if there's a conflict
    pub server_hash: Option<String>,
}

impl UploadInfo {
    /// Calculate upload progress as 0.0 - 1.0
    pub fn progress(&self) -> f32 {
        if self.total_chunks == 0 {
            0.0
        } else {
            self.acked_chunks as f32 / self.total_chunks as f32
        }
    }
    
    /// Check if all chunks have been acknowledged
    pub fn is_complete(&self) -> bool {
        self.acked_chunks >= self.total_chunks
    }
}

/// Resource tracking pending uploads (client-side)
/// 
/// This resource is thread-safe to allow background upload threads
#[derive(Resource, Clone, Default)]
pub struct PendingUploads {
    /// Map from request_id to upload info
    uploads: Arc<Mutex<HashMap<u64, UploadInfo>>>,
    /// Counter for generating unique request IDs
    next_request_id: Arc<std::sync::atomic::AtomicU64>,
}

impl PendingUploads {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Generate a new unique request ID
    pub fn next_request_id(&self) -> u64 {
        self.next_request_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
    
    /// Start tracking a new upload
    pub fn start_upload(&self, request_id: u64, path: String, total_chunks: u32, total_size: usize, file_hash: String) {
        debug!("📤 [UPLOAD_STATE] Starting upload {} for '{}' ({} chunks, {} bytes)", 
            request_id, path, total_chunks, total_size);
        
        self.uploads.lock().unwrap().insert(request_id, UploadInfo {
            request_id,
            path,
            total_chunks,
            sent_chunks: 0,
            acked_chunks: 0,
            total_size,
            file_hash,
            started_at: Instant::now(),
            has_conflict: false,
            server_hash: None,
        });
    }
    
    /// Mark a chunk as sent
    pub fn mark_chunk_sent(&self, request_id: u64) {
        if let Some(info) = self.uploads.lock().unwrap().get_mut(&request_id) {
            info.sent_chunks += 1;
        }
    }
    
    /// Mark a chunk as acknowledged by server
    pub fn mark_chunk_acked(&self, request_id: u64, chunk_index: u32) -> Option<UploadInfo> {
        let mut uploads = self.uploads.lock().unwrap();
        if let Some(info) = uploads.get_mut(&request_id) {
            info.acked_chunks = info.acked_chunks.max(chunk_index + 1);
            Some(info.clone())
        } else {
            None
        }
    }
    
    /// Mark upload as having a conflict
    pub fn mark_conflict(&self, request_id: u64, server_hash: String) -> Option<UploadInfo> {
        let mut uploads = self.uploads.lock().unwrap();
        if let Some(info) = uploads.get_mut(&request_id) {
            info.has_conflict = true;
            info.server_hash = Some(server_hash);
            Some(info.clone())
        } else {
            None
        }
    }
    
    /// Mark upload as complete and remove from tracking
    pub fn complete_upload(&self, request_id: u64) -> Option<UploadInfo> {
        self.uploads.lock().unwrap().remove(&request_id)
    }
    
    /// Mark upload as failed and remove from tracking
    pub fn fail_upload(&self, request_id: u64) -> Option<UploadInfo> {
        self.uploads.lock().unwrap().remove(&request_id)
    }
    
    /// Get upload info by request ID
    pub fn get_upload(&self, request_id: u64) -> Option<UploadInfo> {
        self.uploads.lock().unwrap().get(&request_id).cloned()
    }
    
    /// Get upload info by path
    pub fn get_upload_by_path(&self, path: &str) -> Option<UploadInfo> {
        self.uploads.lock().unwrap().values()
            .find(|u| u.path == path)
            .cloned()
    }
    
    /// Get all pending uploads
    pub fn get_all_uploads(&self) -> Vec<UploadInfo> {
        self.uploads.lock().unwrap().values().cloned().collect()
    }
    
    /// Check if any uploads are pending
    pub fn has_pending_uploads(&self) -> bool {
        !self.uploads.lock().unwrap().is_empty()
    }
}

// ============================================================================
// Server-side upload assembly
// ============================================================================

/// Information about a pending upload being assembled on the server
#[derive(Clone, Debug)]
pub struct ServerUploadAssembly {
    /// Request ID
    pub request_id: u64,
    /// Client ID that's uploading
    pub client_id: u64,
    /// Destination path
    pub path: String,
    /// Total chunks expected
    pub total_chunks: u32,
    /// Total file size expected
    pub total_size: usize,
    /// Received chunks (indexed)
    pub chunks: Vec<Option<Vec<u8>>>,
    /// Client's file hash (for verification)
    pub file_hash: String,
    /// Whether to skip conflict check
    pub force_overwrite: bool,
    /// When the upload started
    pub started_at: Instant,
}

impl ServerUploadAssembly {
    /// Check if all chunks have been received
    pub fn is_complete(&self) -> bool {
        self.chunks.iter().all(|c| c.is_some())
    }
    
    /// Reassemble the complete file data
    pub fn reassemble(&self) -> Option<Vec<u8>> {
        if !self.is_complete() {
            return None;
        }
        
        let mut data = Vec::with_capacity(self.total_size);
        for chunk in &self.chunks {
            if let Some(bytes) = chunk {
                data.extend(bytes);
            } else {
                return None;
            }
        }
        Some(data)
    }
    
    /// Count received chunks
    pub fn received_count(&self) -> u32 {
        self.chunks.iter().filter(|c| c.is_some()).count() as u32
    }
}

/// Resource tracking pending uploads on the server
#[derive(Resource, Default)]
pub struct ServerPendingUploads {
    /// Map from (client_id, request_id) to assembly state
    uploads: HashMap<(u64, u64), ServerUploadAssembly>,
}

impl ServerPendingUploads {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Start tracking a new upload from a client
    pub fn start_upload(
        &mut self,
        client_id: u64,
        request_id: u64,
        path: String,
        total_chunks: u32,
        total_size: usize,
        file_hash: String,
        force_overwrite: bool,
    ) {
        debug!("📥 [SERVER_UPLOAD] Starting upload {} from client {} for '{}' ({} chunks)", 
            request_id, client_id, path, total_chunks);
        
        self.uploads.insert((client_id, request_id), ServerUploadAssembly {
            request_id,
            client_id,
            path,
            total_chunks,
            total_size,
            chunks: vec![None; total_chunks as usize],
            file_hash,
            force_overwrite,
            started_at: Instant::now(),
        });
    }
    
    /// Add a received chunk
    pub fn add_chunk(&mut self, client_id: u64, request_id: u64, chunk_index: u32, data: Vec<u8>) -> Option<&ServerUploadAssembly> {
        if let Some(assembly) = self.uploads.get_mut(&(client_id, request_id)) {
            if (chunk_index as usize) < assembly.chunks.len() {
                assembly.chunks[chunk_index as usize] = Some(data);
            }
            Some(assembly)
        } else {
            None
        }
    }
    
    /// Get upload assembly
    pub fn get_upload(&self, client_id: u64, request_id: u64) -> Option<&ServerUploadAssembly> {
        self.uploads.get(&(client_id, request_id))
    }
    
    /// Remove completed upload
    pub fn complete_upload(&mut self, client_id: u64, request_id: u64) -> Option<ServerUploadAssembly> {
        self.uploads.remove(&(client_id, request_id))
    }
    
    /// Clean up uploads from a disconnected client
    pub fn cleanup_client(&mut self, client_id: u64) {
        self.uploads.retain(|(cid, _), _| *cid != client_id);
    }
}
