// Asset server delivery system
//
// This module handles asset request processing using global Renet resources:
// - Server: Receives asset requests from clients, reads assets from disk, chunks/encrypts, sends responses
// - Client: Sends pending requests, receives responses, saves to disk

use bevy::prelude::*;
use bytes::Bytes;
use crate::network_asset_client::{
    AssetRequestMessage, AssetResponseMessage,
    ASSET_CHANNEL, chunk_and_encrypt, decrypt_data,
};

use bevy_replicon_renet::renet::{RenetClient, RenetServer};

/// Normalize a path by resolving .. and . components and converting to forward slashes
fn normalize_path(path: &str) -> String {
    let path = path.replace("\\", "/");
    let mut parts: Vec<&str> = Vec::new();
    
    for part in path.split('/') {
        match part {
            "" | "." => {} // Skip empty and current directory
            ".." => {
                parts.pop(); // Go up one directory
            }
            _ => parts.push(part),
        }
    }
    
    parts.join("/")
}

/// Resolve asset path using context for relative paths
/// If context_path is provided, tries relative to context's directory first
/// Falls back to canonical path from assets/
fn resolve_asset_path(requested_path: &str, context_path: Option<&str>) -> std::path::PathBuf {
    let assets_base = std::path::Path::new("assets");
    
    // If we have a context, try resolving relative to it first
    if let Some(context) = context_path {
        let context_dir = std::path::Path::new(context).parent();
        if let Some(parent) = context_dir {
            // Build relative path: parent directory of context + requested path
            let relative = parent.join(requested_path);
            let normalized = normalize_path(&relative.to_string_lossy());
            let full_path = assets_base.join(&normalized);
            
            info!("📂 [SERVER] Trying relative path: {:?} (from context '{}')", full_path, context);
            
            if full_path.exists() {
                return full_path;
            }
        }
    }
    
    // Fall back to canonical path (directly from assets/)
    let canonical_path = assets_base.join(requested_path);
    info!("📂 [SERVER] Trying canonical path: {:?}", canonical_path);
    canonical_path
}
/// System to handle incoming asset requests using global RenetServer resource
#[cfg(feature = "networking")]
pub fn handle_asset_requests_global(
    server: Option<ResMut<RenetServer>>,
) {
    let Some(mut server) = server else { return };
    
    // Get list of connected clients
    let client_ids: Vec<u64> = server.clients_id().into_iter().collect();
    
    for client_id in client_ids {
        // Process all asset request messages from this client
        while let Some(message_bytes) = server.receive_message(client_id, ASSET_CHANNEL) {
            match bincode::deserialize::<AssetRequestMessage>(&message_bytes) {
                Ok(request) => {
                    info!(
                        "📥 [SERVER] Asset request from client {}: {} (local_hash: {:?})",
                        client_id, request.path, request.local_hash
                    );
                    
                    // Resolve the file path using context if provided
                    // If context is set, try relative to context's directory first
                    let file_path = resolve_asset_path(&request.path, request.context_path.as_deref());
                    
                    info!("📥 [SERVER] Resolved path '{}' -> {:?}", request.path, file_path);
                    
                    // Read file from disk
                    match std::fs::read(&file_path) {
                        Ok(data) => {
                            // Compute server hash
                            let server_hash = crate::network_asset_client::compute_hash(&data);
                            
                            // Check if client's hash matches (up-to-date check)
                            if let Some(ref client_hash) = request.local_hash {
                                if client_hash == &server_hash {
                                    info!(
                                        "✓ [SERVER] Asset '{}' is up-to-date for client {} (hash: {})",
                                        request.path, client_id, server_hash
                                    );
                                    
                                    // Send up-to-date response (no data transfer needed)
                                    let response = AssetResponseMessage {
                                        request_id: request.request_id,
                                        path: request.path.clone(),
                                        is_up_to_date: true,
                                        server_hash: Some(server_hash),
                                        chunk_index: 0,
                                        total_chunks: 0,
                                        total_size: 0,
                                        data: vec![],
                                        error: None,
                                    };
                                    
                                    if let Ok(response_bytes) = bincode::serialize(&response) {
                                        server.send_message(client_id, ASSET_CHANNEL, Bytes::from(response_bytes));
                                    }
                                    continue;
                                }
                            }
                            
                            // Hash mismatch or no local hash - send full file
                            let chunks = chunk_and_encrypt(&data);
                            let total_chunks = chunks.len() as u32;
                            
                            info!(
                                "📤 [SERVER] Sending asset '{}' to client {} ({} bytes, {} chunks, hash: {})",
                                request.path, client_id, data.len(), total_chunks, server_hash
                            );
                            
                            // Send each chunk as a separate message
                            for (index, chunk_data) in chunks.into_iter().enumerate() {
                                let response = AssetResponseMessage {
                                    request_id: request.request_id,
                                    path: request.path.clone(),
                                    is_up_to_date: false,
                                    server_hash: if index == 0 { Some(server_hash.clone()) } else { None },
                                    chunk_index: index as u32,
                                    total_chunks,
                                    total_size: data.len(),
                                    data: chunk_data,
                                    error: None,
                                };
                                
                                if let Ok(response_bytes) = bincode::serialize(&response) {
                                    server.send_message(client_id, ASSET_CHANNEL, Bytes::from(response_bytes));
                                }
                            }
                        }
                        Err(e) => {
                            // Send error response
                            warn!(
                                "❌ [SERVER] Asset not found: {} (error: {})",
                                request.path, e
                            );
                            
                            let response = AssetResponseMessage {
                                request_id: request.request_id,
                                path: request.path.clone(),
                                is_up_to_date: false,
                                server_hash: None,
                                chunk_index: 0,
                                total_chunks: 0,
                                total_size: 0,
                                data: vec![],
                                error: Some(format!("Asset not found: {}", e)),
                            };
                            
                            if let Ok(response_bytes) = bincode::serialize(&response) {
                                server.send_message(client_id, ASSET_CHANNEL, Bytes::from(response_bytes));
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("❌ [SERVER] Failed to deserialize asset request: {}", e);
                }
            }
        }
    }
}

/// System to send pending asset requests using global RenetClient resource
#[cfg(feature = "networking")]
pub fn send_asset_requests_global(
    pending_requests: Res<crate::network_asset_client::PendingAssetRequests>,
    client: Option<ResMut<RenetClient>>,
) {
    let Some(mut client) = client else { return };
    
    // Get pending requests that need to be sent
    let requests_to_send = pending_requests.drain_pending_requests();
    
    for request in requests_to_send {
        if let Ok(message_bytes) = bincode::serialize(&request) {
            info!(
                "📤 [CLIENT] Sending asset request: {} (id: {})",
                request.path, request.request_id
            );
            client.send_message(ASSET_CHANNEL, Bytes::from(message_bytes));
        }
    }
}

/// System to receive asset responses using global RenetClient resource
#[cfg(feature = "networking")]
pub fn receive_asset_responses_global(
    pending_requests: Res<crate::network_asset_client::PendingAssetRequests>,
    client: Option<ResMut<RenetClient>>,
) {
    let Some(mut client) = client else { return };
    
    while let Some(message_bytes) = client.receive_message(ASSET_CHANNEL) {
        match bincode::deserialize::<AssetResponseMessage>(&message_bytes) {
            Ok(response) => {
                process_asset_response(&pending_requests, response);
            }
            Err(e) => {
                warn!("❌ [CLIENT] Failed to deserialize asset response: {}", e);
            }
        }
    }
}

/// Process an asset response message
#[cfg(feature = "networking")]
fn process_asset_response(
    pending_requests: &crate::network_asset_client::PendingAssetRequests,
    response: AssetResponseMessage,
) {
    // Check for error response
    if let Some(error) = &response.error {
        warn!(
            "❌ [CLIENT] Asset request failed for '{}': {}",
            response.path, error
        );
        pending_requests.mark_failed(&response.path, error.clone());
        return;
    }
    
    // Check if asset is up-to-date (hash matched, no transfer needed)
    if response.is_up_to_date {
        info!(
            "✓ [CLIENT] Asset '{}' is up-to-date (no download needed)",
            response.path
        );
        pending_requests.update_status(
            &response.path, 
            crate::network_asset_client::AssetRequestStatus::UpToDate
        );
        return;
    }
    
    debug!(
        "📥 [CLIENT] Received chunk {}/{} for '{}' ({} bytes)",
        response.chunk_index + 1,
        response.total_chunks,
        response.path,
        response.data.len()
    );
    
    // Decrypt chunk
    let decrypted = match decrypt_data(&response.data) {
        Ok(data) => data,
        Err(e) => {
            warn!("❌ [CLIENT] Failed to decrypt chunk: {}", e);
            pending_requests.mark_failed(&response.path, e.to_string());
            return;
        }
    };
    
    // Add chunk to request
    if pending_requests.add_chunk(
        response.request_id,
        response.chunk_index,
        response.total_chunks,
        response.total_size,
        decrypted,
    ).is_some() {
        // Check if request is complete
        if let Some(request) = pending_requests.get_request_by_id(response.request_id) {
            if matches!(request.status, crate::network_asset_client::AssetRequestStatus::Complete) {
                // Reassemble data
                if let Some(data) = request.reassemble_data() {
                    info!(
                        "✅ [CLIENT] Asset download complete: '{}' ({} bytes)",
                        response.path, data.len()
                    );
                    
                    // Determine save path - all paths are relative to assets/
                    // Paths already include their subfolder (e.g., "scripts/examples/foo.lua" or "images/test.png")
                    let save_path = std::path::Path::new("assets").join(&request.path);
                    
                    // Create parent directories if needed
                    if let Some(parent) = save_path.parent() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            warn!("❌ [CLIENT] Failed to create directory {:?}: {}", parent, e);
                        }
                    }
                    
                    // Save to disk
                    match std::fs::write(&save_path, &data) {
                        Ok(_) => {
                            info!("💾 [CLIENT] Saved asset to {:?}", save_path);
                            // Mark as complete and store for retrieval by scripts
                            pending_requests.complete_request(&request.path, data);
                        }
                        Err(e) => {
                            warn!("❌ [CLIENT] Failed to save asset: {}", e);
                            // Still store in memory even if disk save fails
                            pending_requests.complete_request(&request.path, data);
                        }
                    }
                }
            }
        }
    }
}

/// System to check for timed out requests
#[cfg(feature = "networking")]
pub fn check_request_timeouts(
    pending_requests: Res<crate::network_asset_client::PendingAssetRequests>,
) {
    let timed_out = pending_requests.get_timed_out_requests();
    
    for path in timed_out {
        warn!("⏱️ [CLIENT] Asset request timed out: '{}'", path);
        pending_requests.mark_failed(&path, "Request timed out".to_string());
    }
}
