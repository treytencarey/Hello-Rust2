// Asset server delivery system
//
// This module handles asset request processing using global Renet resources:
// - Server: Receives asset requests from clients, reads assets from disk, chunks/encrypts, sends responses
// - Client: Sends pending requests, receives responses, saves to disk

use bevy::prelude::*;
use bytes::Bytes;
use crate::network_asset_client::{
    AssetRequestMessage, AssetResponseMessage, AssetSubscriptionMessage, AssetUpdateNotification,
    DirectoryListingRequest, DirectoryListingResponse, FileInfo,
    AssetUploadRequest, AssetUploadResponse, UploadStatus,
    ASSET_CHANNEL, chunk_and_encrypt, decrypt_data,
};
use crate::subscription_registry::{AssetSubscriptionRegistry, FileWatcherResource};
use crate::upload_state::ServerPendingUploads;

use bevy_replicon_renet::renet::{RenetClient, RenetServer};
use bevy_lua_ecs::path_utils::normalize_path;

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
            
            debug!("📂 [SERVER] Trying relative path: {:?} (from context '{}')", full_path, context);
            
            if full_path.exists() {
                return full_path;
            }
        }
    }
    
    // Fall back to canonical path (directly from assets/)
    let canonical_path = assets_base.join(requested_path);
    debug!("📂 [SERVER] Trying canonical path: {:?}", canonical_path);
    canonical_path
}
/// System to handle incoming asset requests using global RenetServer resource
#[cfg(feature = "networking")]
pub fn handle_asset_requests_global(
    server: Option<ResMut<RenetServer>>,
    subscription_registry: Option<ResMut<AssetSubscriptionRegistry>>,
    mut file_watcher: Option<ResMut<FileWatcherResource>>,
    mut pending_uploads: Option<ResMut<ServerPendingUploads>>,
) {
    let Some(mut server) = server else { return };
    
    // Get list of connected clients
    let client_ids: Vec<u64> = server.clients_id().into_iter().collect();
    
    for client_id in client_ids {
        // Process all messages from this client on the asset channel
        while let Some(message_bytes) = server.receive_message(client_id, ASSET_CHANNEL) {
            // Deserialize as ClientToServerMessage wrapper for proper type discrimination
            match bincode::deserialize::<crate::network_asset_client::ClientToServerMessage>(&message_bytes) {
                Ok(crate::network_asset_client::ClientToServerMessage::Subscription(sub_msg)) => {
                    // Handle subscription message
                    if let Some(ref registry) = subscription_registry {
                        handle_subscription_message(client_id, sub_msg, registry, &mut file_watcher);
                    } else {
                        debug!("📡 [SERVER] Received subscription message but no registry");
                    }
                    continue;
                }
                Ok(crate::network_asset_client::ClientToServerMessage::DirectoryListing(request)) => {
                    // Handle directory listing request
                    handle_directory_listing(&mut server, client_id, request);
                    continue;
                }
                Ok(crate::network_asset_client::ClientToServerMessage::Upload(request)) => {
                    // Handle file upload request
                    if let Some(ref mut uploads) = pending_uploads {
                        handle_upload_request(&mut server, client_id, request, uploads);
                    } else {
                        warn!("📥 [SERVER] Received upload but no pending uploads resource");
                    }
                    continue;
                }
                Ok(crate::network_asset_client::ClientToServerMessage::Request(request)) => {
                    debug!(
                        "📥 [SERVER] Asset request from client {}: {} (local_hash: {:?})",
                        client_id, request.path, request.local_hash
                    );
                    
                    // Resolve the file path using context if provided
                    // If context is set, try relative to context's directory first
                    let file_path = resolve_asset_path(&request.path, request.context_path.as_deref());
                    
                    debug!("📥 [SERVER] Resolved path '{}' -> {:?}", request.path, file_path);
                    
                    // Read file from disk
                    match std::fs::read(&file_path) {
                        Ok(data) => {
                            // Compute server hash
                            let server_hash = crate::network_asset_client::compute_hash(&data);
                            
                            // Check if client's hash matches (up-to-date check)
                            if let Some(ref client_hash) = request.local_hash {
                                if client_hash == &server_hash {
                                    debug!(
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
                                    
                                    let wrapped = crate::network_asset_client::ServerToClientMessage::Response(response);
                                    if let Ok(response_bytes) = bincode::serialize(&wrapped) {
                                        server.send_message(client_id, ASSET_CHANNEL, Bytes::from(response_bytes));
                                    }
                                    continue;
                                }
                            }
                            
                            // Hash mismatch or no local hash - send full file
                            let chunks = chunk_and_encrypt(&data);
                            let total_chunks = chunks.len() as u32;
                            
                            debug!(
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
                                
                                let wrapped = crate::network_asset_client::ServerToClientMessage::Response(response);
                                if let Ok(response_bytes) = bincode::serialize(&wrapped) {
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
                            
                            let wrapped = crate::network_asset_client::ServerToClientMessage::Response(response);
                            if let Ok(response_bytes) = bincode::serialize(&wrapped) {
                                server.send_message(client_id, ASSET_CHANNEL, Bytes::from(response_bytes));
                            }
                        }
                    }
                }
                Ok(crate::network_asset_client::ClientToServerMessage::Rename(request)) => {
                    // Handle rename/move request
                    handle_rename_request(&mut server, client_id, request);
                    continue;
                }
                Ok(crate::network_asset_client::ClientToServerMessage::Delete(request)) => {
                    // Handle delete request
                    handle_delete_request(&mut server, client_id, request);
                    continue;
                }
                Ok(crate::network_asset_client::ClientToServerMessage::RequestServerFile(request)) => {
                    // Handle explicit server file request (after LocalNewer cancel)
                    handle_request_server_file(&mut server, client_id, request);
                    continue;
                }
                Err(e) => {
                    warn!("❌ [SERVER] Failed to deserialize message: {}", e);
                }
            }
        }
    }
}

/// Handle a subscription message from a client
#[cfg(feature = "networking")]
fn handle_subscription_message(
    client_id: u64,
    message: AssetSubscriptionMessage,
    registry: &AssetSubscriptionRegistry,
    file_watcher: &mut Option<ResMut<FileWatcherResource>>,
) {
    match message {
        AssetSubscriptionMessage::Subscribe { paths, instance_id } => {
            debug!("📡 [SERVER] Client {} instance {} subscribing to {} paths", 
                client_id, instance_id, paths.len());
            for path in paths {
                debug!("  - '{}'", path);
                let is_first = registry.subscribe(client_id, path.clone(), instance_id);
                if is_first {
                    // Start file watching for this path
                    if let Some(ref mut watcher) = file_watcher {
                        watcher.watch(&path);
                    }
                }
            }
        }
        AssetSubscriptionMessage::Unsubscribe { paths, instance_id } => {
            debug!("📡 [SERVER] Client {} instance {} unsubscribing from {} paths", 
                client_id, instance_id, paths.len());
            for path in paths {
                debug!("  - '{}'", path);
                let is_last = registry.unsubscribe(client_id, &path, instance_id);
                if is_last {
                    // Stop file watching for this path
                    if let Some(ref mut watcher) = file_watcher {
                        watcher.unwatch(&path);
                    }
                }
            }
        }
        AssetSubscriptionMessage::UnsubscribeAll { instance_id } => {
            debug!("📡 [SERVER] Client {} instance {} unsubscribing from all paths", 
                client_id, instance_id);
            let empty_paths = registry.unsubscribe_all(client_id, instance_id);
            for path in empty_paths {
                debug!("  - '{}'", path);
                // Stop file watching for paths with no more subscribers
                if let Some(ref mut watcher) = file_watcher {
                    watcher.unwatch(&path);
                }
            }
        }
    }
}

// ============================================================================
// Directory Listing Handler
// ============================================================================

/// Handle a directory listing request from a client
/// This includes security checks to ensure paths cannot escape the assets/ directory
#[cfg(feature = "networking")]
fn handle_directory_listing(
    server: &mut RenetServer,
    client_id: u64,
    request: DirectoryListingRequest,
) {
    debug!("📂 [SERVER] Directory listing request from client {}: '{}' (offset: {}, limit: {})",
        client_id, request.path, request.offset, request.limit);
    
    let assets_base = std::path::Path::new("assets");
    
    // Build the requested path
    let requested = if request.path.is_empty() || request.path == "." {
        assets_base.to_path_buf()
    } else {
        assets_base.join(&request.path)
    };
    
    // Security: Ensure canonicalized path is within assets/
    let (canonical, assets_canonical) = match (requested.canonicalize(), assets_base.canonicalize()) {
        (Ok(c), Ok(a)) => (c, a),
        (Err(e), _) => {
            send_directory_error(server, client_id, &request, &format!("Path not found: {}", e));
            return;
        }
        (_, Err(e)) => {
            send_directory_error(server, client_id, &request, &format!("Server error: {}", e));
            return;
        }
    };
    
    if !canonical.starts_with(&assets_canonical) {
        warn!("🚫 [SERVER] Directory traversal attempt by client {}: '{}'", client_id, request.path);
        send_directory_error(server, client_id, &request, "Access denied: path outside assets directory");
        return;
    }
    
    // Read directory entries
    let entries = match std::fs::read_dir(&canonical) {
        Ok(entries) => entries,
        Err(e) => {
            send_directory_error(server, client_id, &request, &format!("Cannot read directory: {}", e));
            return;
        }
    };
    
    // Collect all entries with metadata
    let mut files: Vec<FileInfo> = Vec::new();
    for entry_result in entries {
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };
        
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        
        let name = entry.file_name().to_string_lossy().to_string();
        
        // Compute relative path from assets/
        let entry_path = entry.path();
        let relative_path = match entry_path.strip_prefix(&assets_canonical) {
            Ok(p) => p.to_string_lossy().to_string().replace('\\', "/"),
            Err(_) => continue,
        };
        
        // Get modification time as Unix timestamp
        let modified = metadata.modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        files.push(FileInfo {
            name,
            path: relative_path,
            size: if metadata.is_file() { metadata.len() } else { 0 },
            modified,
            is_directory: metadata.is_dir(),
        });
    }
    
    // Sort: directories first, then alphabetically by name
    files.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
    
    let total_count = files.len() as u32;
    let offset = request.offset as usize;
    let limit = request.limit as usize;
    
    // Apply pagination
    let paginated_files: Vec<FileInfo> = files
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();
    
    let has_more = offset + paginated_files.len() < total_count as usize;
    
    debug!("📂 [SERVER] Sending {} files to client {} (total: {}, has_more: {})",
        paginated_files.len(), client_id, total_count, has_more);
    
    let response = DirectoryListingResponse {
        request_id: request.request_id,
        path: request.path,
        files: paginated_files,
        total_count,
        offset: request.offset,
        has_more,
        error: None,
    };
    
    let wrapped = crate::network_asset_client::ServerToClientMessage::DirectoryListing(response);
    if let Ok(response_bytes) = bincode::serialize(&wrapped) {
        server.send_message(client_id, ASSET_CHANNEL, Bytes::from(response_bytes));
    }
}

/// Send a directory listing error response
#[cfg(feature = "networking")]
fn send_directory_error(
    server: &mut RenetServer,
    client_id: u64,
    request: &DirectoryListingRequest,
    error: &str,
) {
    warn!("❌ [SERVER] Directory listing error for '{}': {}", request.path, error);
    
    let response = DirectoryListingResponse {
        request_id: request.request_id,
        path: request.path.clone(),
        files: vec![],
        total_count: 0,
        offset: 0,
        has_more: false,
        error: Some(error.to_string()),
    };
    
    let wrapped = crate::network_asset_client::ServerToClientMessage::DirectoryListing(response);
    if let Ok(response_bytes) = bincode::serialize(&wrapped) {
        server.send_message(client_id, ASSET_CHANNEL, Bytes::from(response_bytes));
    }
}

// ============================================================================
// File Upload Handler
// ============================================================================

/// Handle a file upload request from a client
/// This includes security checks and conflict detection
#[cfg(feature = "networking")]
fn handle_upload_request(
    server: &mut RenetServer,
    client_id: u64,
    request: AssetUploadRequest,
    pending_uploads: &mut ServerPendingUploads,
) {
    debug!("📥 [SERVER] Upload request from client {}: '{}' chunk {}/{} ({} bytes total)",
        client_id, request.path, request.chunk_index + 1, request.total_chunks, request.total_size);
    
    let assets_base = std::path::Path::new("assets");
    let target_path = assets_base.join(&request.path);
    
    // Security: Validate the path doesn't escape assets/ directory
    // We can't canonicalize a non-existent file, so we canonicalize the parent directory
    if let Some(parent) = target_path.parent() {
        // Create parent directories if they don't exist
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                send_upload_error(server, client_id, &request, &format!("Cannot create directory: {}", e));
                return;
            }
        }
        
        // Now canonicalize and check
        match (parent.canonicalize(), assets_base.canonicalize()) {
            (Ok(parent_canonical), Ok(assets_canonical)) => {
                if !parent_canonical.starts_with(&assets_canonical) {
                    warn!("🚫 [SERVER] Upload path traversal attempt by client {}: '{}'", client_id, request.path);
                    send_upload_error(server, client_id, &request, "Access denied: path outside assets directory");
                    return;
                }
            }
            (Err(e), _) => {
                send_upload_error(server, client_id, &request, &format!("Invalid path: {}", e));
                return;
            }
            (_, Err(e)) => {
                send_upload_error(server, client_id, &request, &format!("Server error: {}", e));
                return;
            }
        }
    }
    
    // First chunk - check for conflicts and initialize upload
    if request.chunk_index == 0 {
        // Check if file exists and if we need conflict detection
        if target_path.exists() && !request.force_overwrite {
            // Read existing file and compute hash
            if let Ok(existing_data) = std::fs::read(&target_path) {
                let server_hash = crate::network_asset_client::compute_hash(&existing_data);
                
                // If hashes differ, it's a conflict
                if server_hash != request.file_hash {
                    debug!("⚠️ [SERVER] Upload conflict for '{}': client hash {} vs server hash {}",
                        request.path, request.file_hash, server_hash);
                    
                    let response = AssetUploadResponse {
                        request_id: request.request_id,
                        path: request.path,
                        status: UploadStatus::Conflict { server_hash },
                    };
                    
                    let wrapped = crate::network_asset_client::ServerToClientMessage::UploadResponse(response);
                    if let Ok(response_bytes) = bincode::serialize(&wrapped) {
                        server.send_message(client_id, ASSET_CHANNEL, Bytes::from(response_bytes));
                    }
                    return;
                }
            }
        }
        
        // Initialize upload tracking
        pending_uploads.start_upload(
            client_id,
            request.request_id,
            request.path.clone(),
            request.total_chunks,
            request.total_size,
            request.file_hash.clone(),
            request.force_overwrite,
        );
    }
    
    // Decrypt chunk data
    let decrypted = match decrypt_data(&request.data) {
        Ok(data) => data,
        Err(e) => {
            send_upload_error(server, client_id, &request, &format!("Decryption failed: {}", e));
            pending_uploads.complete_upload(client_id, request.request_id);
            return;
        }
    };
    
    // Add chunk to assembly
    let assembly = pending_uploads.add_chunk(client_id, request.request_id, request.chunk_index, decrypted);
    
    if let Some(assembly) = assembly {
        // Send chunk acknowledgment
        let response = AssetUploadResponse {
            request_id: request.request_id,
            path: request.path.clone(),
            status: UploadStatus::ChunkReceived {
                chunk_index: request.chunk_index,
                total_chunks: request.total_chunks,
            },
        };
        
        let wrapped = crate::network_asset_client::ServerToClientMessage::UploadResponse(response);
        if let Ok(response_bytes) = bincode::serialize(&wrapped) {
            server.send_message(client_id, ASSET_CHANNEL, Bytes::from(response_bytes));
        }
        
        // Check if upload is complete
        if assembly.is_complete() {
            if let Some(data) = assembly.reassemble() {
                // Verify hash
                let computed_hash = crate::network_asset_client::compute_hash(&data);
                
                if computed_hash != assembly.file_hash {
                    warn!("❌ [SERVER] Upload hash mismatch for '{}': expected {} got {}",
                        request.path, assembly.file_hash, computed_hash);
                    send_upload_error(server, client_id, &request, "Hash verification failed");
                    pending_uploads.complete_upload(client_id, request.request_id);
                    return;
                }
                
                // Save file
                match std::fs::write(&target_path, &data) {
                    Ok(_) => {
                        debug!("✅ [SERVER] Upload complete for '{}' ({} bytes, hash: {})",
                            request.path, data.len(), computed_hash);
                        
                        let response = AssetUploadResponse {
                            request_id: request.request_id,
                            path: request.path.clone(),
                            status: UploadStatus::Complete { server_hash: computed_hash },
                        };
                        
                        let wrapped = crate::network_asset_client::ServerToClientMessage::UploadResponse(response);
                        if let Ok(response_bytes) = bincode::serialize(&wrapped) {
                            server.send_message(client_id, ASSET_CHANNEL, Bytes::from(response_bytes));
                        }
                    }
                    Err(e) => {
                        send_upload_error(server, client_id, &request, &format!("Failed to save file: {}", e));
                    }
                }
                
                // Clean up
                pending_uploads.complete_upload(client_id, request.request_id);
            }
        }
    } else {
        // Upload not found - client may have sent chunks out of order or we lost state
        send_upload_error(server, client_id, &request, "Upload session not found");
    }
}

/// Send an upload error response
#[cfg(feature = "networking")]
fn send_upload_error(
    server: &mut RenetServer,
    client_id: u64,
    request: &AssetUploadRequest,
    error: &str,
) {
    warn!("❌ [SERVER] Upload error for '{}': {}", request.path, error);
    
    let response = AssetUploadResponse {
        request_id: request.request_id,
        path: request.path.clone(),
        status: UploadStatus::Error(error.to_string()),
    };
    
    let wrapped = crate::network_asset_client::ServerToClientMessage::UploadResponse(response);
    if let Ok(response_bytes) = bincode::serialize(&wrapped) {
        server.send_message(client_id, ASSET_CHANNEL, Bytes::from(response_bytes));
    }
}

/// Handle a rename/move request
#[cfg(feature = "networking")]
fn handle_rename_request(
    server: &mut RenetServer,
    client_id: u64,
    request: crate::network_asset_client::AssetRenameRequest,
) {
    debug!("📝 [SERVER] Rename request from client {}: '{}' -> '{}'", 
        client_id, request.old_path, request.new_path);
    
    let old_file_path = std::path::Path::new("assets").join(&request.old_path);
    let new_file_path = std::path::Path::new("assets").join(&request.new_path);
    
    // Security check: ensure paths stay within assets/
    let canonical_old = match old_file_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            send_rename_response(server, client_id, &request, false, Some(format!("Old path not found: {}", e)));
            return;
        }
    };
    
    let assets_dir = match std::path::Path::new("assets").canonicalize() {
        Ok(p) => p,
        Err(e) => {
            send_rename_response(server, client_id, &request, false, Some(format!("Assets dir error: {}", e)));
            return;
        }
    };
    
    if !canonical_old.starts_with(&assets_dir) {
        send_rename_response(server, client_id, &request, false, Some("Path traversal detected".to_string()));
        return;
    }
    
    // Create parent directories for new path if needed
    if let Some(parent) = new_file_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            send_rename_response(server, client_id, &request, false, Some(format!("Failed to create directory: {}", e)));
            return;
        }
    }
    
    // Perform rename
    match std::fs::rename(&old_file_path, &new_file_path) {
        Ok(_) => {
            debug!("✅ [SERVER] Renamed '{}' to '{}'", request.old_path, request.new_path);
            send_rename_response(server, client_id, &request, true, None);
        }
        Err(e) => {
            send_rename_response(server, client_id, &request, false, Some(format!("Rename failed: {}", e)));
        }
    }
}

/// Send rename response to client
#[cfg(feature = "networking")]
fn send_rename_response(
    server: &mut RenetServer,
    client_id: u64,
    request: &crate::network_asset_client::AssetRenameRequest,
    success: bool,
    error: Option<String>,
) {
    let response = crate::network_asset_client::AssetRenameResponse {
        request_id: request.request_id,
        old_path: request.old_path.clone(),
        new_path: request.new_path.clone(),
        success,
        error,
    };
    
    let wrapped = crate::network_asset_client::ServerToClientMessage::RenameResponse(response);
    if let Ok(response_bytes) = bincode::serialize(&wrapped) {
        server.send_message(client_id, ASSET_CHANNEL, Bytes::from(response_bytes));
    }
}

/// Handle a delete request
#[cfg(feature = "networking")]
fn handle_delete_request(
    server: &mut RenetServer,
    client_id: u64,
    request: crate::network_asset_client::AssetDeleteRequest,
) {
    debug!("🗑️ [SERVER] Delete request from client {}: '{}'", client_id, request.path);
    
    let file_path = std::path::Path::new("assets").join(&request.path);
    
    // Security check: ensure path stays within assets/
    let canonical = match file_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            send_delete_response(server, client_id, &request, false, Some(format!("Path not found: {}", e)));
            return;
        }
    };
    
    let assets_dir = match std::path::Path::new("assets").canonicalize() {
        Ok(p) => p,
        Err(e) => {
            send_delete_response(server, client_id, &request, false, Some(format!("Assets dir error: {}", e)));
            return;
        }
    };
    
    if !canonical.starts_with(&assets_dir) {
        send_delete_response(server, client_id, &request, false, Some("Path traversal detected".to_string()));
        return;
    }
    
    // Delete file or directory
    let result = if file_path.is_dir() {
        std::fs::remove_dir_all(&file_path)
    } else {
        std::fs::remove_file(&file_path)
    };
    
    match result {
        Ok(_) => {
            debug!("✅ [SERVER] Deleted '{}'", request.path);
            send_delete_response(server, client_id, &request, true, None);
        }
        Err(e) => {
            send_delete_response(server, client_id, &request, false, Some(format!("Delete failed: {}", e)));
        }
    }
}

/// Send delete response to client
#[cfg(feature = "networking")]
fn send_delete_response(
    server: &mut RenetServer,
    client_id: u64,
    request: &crate::network_asset_client::AssetDeleteRequest,
    success: bool,
    error: Option<String>,
) {
    let response = crate::network_asset_client::AssetDeleteResponse {
        request_id: request.request_id,
        path: request.path.clone(),
        success,
        error,
    };
    
    let wrapped = crate::network_asset_client::ServerToClientMessage::DeleteResponse(response);
    if let Ok(response_bytes) = bincode::serialize(&wrapped) {
        server.send_message(client_id, ASSET_CHANNEL, Bytes::from(response_bytes));
    }
}

/// Handle explicit server file request (after LocalNewer cancel)
#[cfg(feature = "networking")]
fn handle_request_server_file(
    server: &mut RenetServer,
    client_id: u64,
    request: crate::network_asset_client::RequestServerFile,
) {
    debug!("📥 [SERVER] RequestServerFile from client {}: '{}'", client_id, request.path);
    
    let file_path = std::path::Path::new("assets").join(&request.path);
    
    // Read and send file (same as regular asset request but without hash check)
    match std::fs::read(&file_path) {
        Ok(data) => {
            let server_hash = crate::network_asset_client::compute_hash(&data);
            let chunks = chunk_and_encrypt(&data);
            let total_chunks = chunks.len() as u32;
            
            debug!("📤 [SERVER] Sending requested file '{}' to client {} ({} bytes, {} chunks)",
                request.path, client_id, data.len(), total_chunks);
            
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
                
                let wrapped = crate::network_asset_client::ServerToClientMessage::Response(response);
                if let Ok(response_bytes) = bincode::serialize(&wrapped) {
                    server.send_message(client_id, ASSET_CHANNEL, Bytes::from(response_bytes));
                }
            }
        }
        Err(e) => {
            let response = AssetResponseMessage {
                request_id: request.request_id,
                path: request.path.clone(),
                is_up_to_date: false,
                server_hash: None,
                chunk_index: 0,
                total_chunks: 0,
                total_size: 0,
                data: vec![],
                error: Some(format!("File not found: {}", e)),
            };
            
            let wrapped = crate::network_asset_client::ServerToClientMessage::Response(response);
            if let Ok(response_bytes) = bincode::serialize(&wrapped) {
                server.send_message(client_id, ASSET_CHANNEL, Bytes::from(response_bytes));
            }
        }
    }
}

/// System to send pending asset requests using global RenetClient resource
#[cfg(feature = "networking")]
pub fn send_asset_requests_global(
    pending_requests: Res<crate::network_asset_client::PendingAssetRequests>,
    mut pending_uploads: ResMut<crate::upload_state::PendingUploads>,
    client: Option<ResMut<RenetClient>>,
) {
    let Some(mut client) = client else { return };
    
    // Get pending requests that need to be sent
    let requests_to_send = pending_requests.drain_pending_requests();
    
    for request in requests_to_send {
        // Wrap in ClientToServerMessage for proper type discrimination
        let wrapped = crate::network_asset_client::ClientToServerMessage::Request(request.clone());
        if let Ok(message_bytes) = bincode::serialize(&wrapped) {
            debug!(
                "📤 [CLIENT] Sending asset request: {} (id: {})",
                request.path, request.request_id
            );
            client.send_message(ASSET_CHANNEL, Bytes::from(message_bytes));
        }
    }
    
    // Send directory listing requests
    let dir_listings = pending_requests.drain_pending_directory_listings();
    for request in dir_listings {
        let wrapped = crate::network_asset_client::ClientToServerMessage::DirectoryListing(request.clone());
        if let Ok(message_bytes) = bincode::serialize(&wrapped) {
            debug!(
                "📂 [CLIENT] Sending directory listing request: {} (id: {})",
                request.path, request.request_id
            );
            client.send_message(ASSET_CHANNEL, Bytes::from(message_bytes));
        }
    }
    
    // Process file upload requests
    let file_uploads = pending_requests.drain_pending_file_uploads();
    for (local_path, server_path, force_overwrite) in file_uploads {
        debug!("📤 [CLIENT] Processing file upload: {} -> {}", local_path, server_path);
        
        // Read file content
        let full_path = std::path::Path::new("assets").join(&local_path);
        let data = match std::fs::read(&full_path) {
            Ok(d) => d,
            Err(e) => {
                error!("❌ [UPLOAD] Failed to read file '{}': {}", local_path, e);
                continue;
            }
        };
        
        // Compute hash
        let hash = crate::network_asset_client::compute_hash(&data);
        
        // Chunk and encrypt the data
        let chunks = crate::network_asset_client::chunk_and_encrypt(&data);
        let total_chunks = chunks.len() as u32;
        
        // Generate request ID and track upload
        let request_id = pending_uploads.next_request_id();
        pending_uploads.start_upload(request_id, server_path.clone(), total_chunks, data.len(), hash.clone());
        
        // Send each chunk
        for (chunk_index, chunk_data) in chunks.into_iter().enumerate() {
            let upload_request = crate::network_asset_client::AssetUploadRequest {
                request_id,
                path: server_path.clone(),
                chunk_index: chunk_index as u32,
                total_chunks,
                total_size: data.len(),
                data: chunk_data,
                file_hash: hash.clone(),
                force_overwrite,
            };
            
            let wrapped = crate::network_asset_client::ClientToServerMessage::Upload(upload_request);
            if let Ok(message_bytes) = bincode::serialize(&wrapped) {
                debug!(
                    "📤 [CLIENT] Sending upload chunk {}/{} for '{}' (id: {})",
                    chunk_index + 1, total_chunks, server_path, request_id
                );
                client.send_message(ASSET_CHANNEL, Bytes::from(message_bytes));
            }
            
            // Update sent count
            pending_uploads.mark_chunk_sent(request_id);
        }
    }
    
    // Send rename requests
    let renames = pending_requests.drain_pending_renames();
    for request in renames {
        let wrapped = crate::network_asset_client::ClientToServerMessage::Rename(request.clone());
        if let Ok(message_bytes) = bincode::serialize(&wrapped) {
            debug!(
                "📝 [CLIENT] Sending rename request: '{}' -> '{}' (id: {})",
                request.old_path, request.new_path, request.request_id
            );
            client.send_message(ASSET_CHANNEL, Bytes::from(message_bytes));
        }
    }
    
    // Send delete requests
    let deletes = pending_requests.drain_pending_deletes();
    for request in deletes {
        let wrapped = crate::network_asset_client::ClientToServerMessage::Delete(request.clone());
        if let Ok(message_bytes) = bincode::serialize(&wrapped) {
            debug!(
                "🗑️ [CLIENT] Sending delete request: '{}' (id: {})",
                request.path, request.request_id
            );
            client.send_message(ASSET_CHANNEL, Bytes::from(message_bytes));
        }
    }
    
    // Send request_server_file requests (for LocalNewer cancel)
    let server_file_requests = pending_requests.drain_pending_server_file_requests();
    for request in server_file_requests {
        let wrapped = crate::network_asset_client::ClientToServerMessage::RequestServerFile(request.clone());
        if let Ok(message_bytes) = bincode::serialize(&wrapped) {
            debug!(
                "📥 [CLIENT] Sending request_server_file: '{}' (id: {})",
                request.path, request.request_id
            );
            client.send_message(ASSET_CHANNEL, Bytes::from(message_bytes));
        }
    }
}

/// System to receive asset responses using global RenetClient resource
/// Handles both AssetResponseMessage and AssetUpdateNotification via the wrapper enum
#[cfg(feature = "networking")]
pub fn receive_asset_responses_global(
    pending_requests: Res<crate::network_asset_client::PendingAssetRequests>,
    pending_updates: ResMut<crate::network_asset_client::PendingAssetUpdates>,
    pending_directory_listings: ResMut<PendingDirectoryListings>,
    pending_upload_responses: ResMut<PendingUploadResponses>,
    mut server_hashes: ResMut<crate::server_hash_tracker::ServerFileHashes>,
    client: Option<ResMut<RenetClient>>,
) {
    let Some(mut client) = client else { return };
    
    while let Some(message_bytes) = client.receive_message(ASSET_CHANNEL) {
        match bincode::deserialize::<crate::network_asset_client::ServerToClientMessage>(&message_bytes) {
            Ok(crate::network_asset_client::ServerToClientMessage::Response(response)) => {
                process_asset_response(&pending_requests, &mut server_hashes, response);
            }
            Ok(crate::network_asset_client::ServerToClientMessage::Update(notification)) => {
                // Store hash from update notification
                server_hashes.update(&notification.path, notification.server_hash.clone());
                // Queue the update for processing by receive_asset_updates system
                pending_updates.queue(notification);
            }
            Ok(crate::network_asset_client::ServerToClientMessage::DirectoryListing(response)) => {
                // Queue for event emission in a separate system
                pending_directory_listings.queue(response);
            }
            Ok(crate::network_asset_client::ServerToClientMessage::UploadResponse(response)) => {
                // Queue for event emission and upload state updates
                pending_upload_responses.queue(response);
            }
            Ok(crate::network_asset_client::ServerToClientMessage::RenameResponse(response)) => {
                // Log and ignore - events are emitted by a separate system if needed
                if response.success {
                    debug!("✅ [CLIENT] Rename succeeded: '{}' -> '{}'", response.old_path, response.new_path);
                } else {
                    warn!("❌ [CLIENT] Rename failed: '{}' -> '{}': {:?}", response.old_path, response.new_path, response.error);
                }
            }
            Ok(crate::network_asset_client::ServerToClientMessage::DeleteResponse(response)) => {
                // Log and ignore - events are emitted by a separate system if needed
                if response.success {
                    debug!("✅ [CLIENT] Delete succeeded: '{}'", response.path);
                } else {
                    warn!("❌ [CLIENT] Delete failed: '{}': {:?}", response.path, response.error);
                }
            }
            Err(e) => {
                warn!("❌ [CLIENT] Failed to deserialize server message: {}", e);
            }
        }
    }
}


/// Process an asset response message
#[cfg(feature = "networking")]
fn process_asset_response(
    pending_requests: &crate::network_asset_client::PendingAssetRequests,
    server_hashes: &mut crate::server_hash_tracker::ServerFileHashes,
    response: AssetResponseMessage,
) {
    // Store server hash if provided (for LocalNewer detection)
    if let Some(ref hash) = response.server_hash {
        server_hashes.update(&response.path, hash.clone());
    }
    
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
        debug!(
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
                    debug!(
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
                            debug!("💾 [CLIENT] Saved asset to {:?}", save_path);
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

/// System to poll file watcher for changes and broadcast updates to subscribed clients
#[cfg(feature = "networking")]
pub fn broadcast_file_updates(
    server: Option<ResMut<RenetServer>>,
    subscription_registry: Option<Res<AssetSubscriptionRegistry>>,
    mut file_watcher: Option<ResMut<FileWatcherResource>>,
    received_files: Option<Res<crate::plugins::NetworkReceivedFiles>>,
) {
    let Some(mut server) = server else { return };
    let Some(registry) = subscription_registry else { return };
    let Some(mut watcher) = file_watcher else { return };
    
    // Poll for file changes
    let changed_paths = watcher.poll_changes();
    
    for path in changed_paths {
        // In peer mode, skip files that were recently received from network
        // This prevents the broadcast loop: server→client→save→watcher→broadcast→server
        if let Some(ref received) = received_files {
            if received.was_received_recently(&path, std::time::Duration::from_secs(2)) {
                debug!("⏭️ [BROADCAST] Skipping '{}' - recently received from network (loop prevention)", path);
                continue;
            }
        }
        
        // Get subscribers for this path
        let subscribers = registry.get_subscribers(&path);
        
        if subscribers.is_empty() {
            debug!("📡 [BROADCAST] No subscribers for changed file: {}", path);
            continue;
        }
        
        debug!("📡 [BROADCAST] File '{}' changed, notifying {} clients", path, subscribers.len());
        
        // Read the updated file
        let file_path = std::path::Path::new("assets").join(&path);
        match std::fs::read(&file_path) {
            Ok(data) => {
                // Compute hash
                let server_hash = crate::network_asset_client::compute_hash(&data);
                
                // Chunk and encrypt for sending
                let chunks = chunk_and_encrypt(&data);
                let total_chunks = chunks.len() as u32;
                
                // Send to each subscriber
                for client_id in subscribers {
                    debug!("📡 [BROADCAST] Sending update for '{}' to client {} ({} chunks)", 
                        path, client_id, total_chunks);
                    
                    for (index, chunk) in chunks.iter().enumerate() {
                        let notification = AssetUpdateNotification {
                            path: path.clone(),
                            server_hash: server_hash.clone(),
                            data: chunk.clone(),
                            total_size: data.len(),
                            chunk_index: index as u32,
                            total_chunks,
                        };
                        
                        let wrapped = crate::network_asset_client::ServerToClientMessage::Update(notification);
                        if let Ok(msg_bytes) = bincode::serialize(&wrapped) {
                            server.send_message(client_id, ASSET_CHANNEL, Bytes::from(msg_bytes));
                        }
                    }
                }
            }
            Err(e) => {
                warn!("📡 [BROADCAST] Failed to read changed file '{}': {}", path, e);
            }
        }
    }
}

/// Resource to track connected clients for disconnect detection
#[cfg(feature = "networking")]
#[derive(Resource, Default)]
pub struct ConnectedClients {
    clients: std::collections::HashSet<u64>,
}

/// System to clean up subscriptions for disconnected clients
#[cfg(feature = "networking")]
pub fn cleanup_disconnected_clients(
    server: Option<Res<RenetServer>>,
    mut connected_clients: ResMut<ConnectedClients>,
    subscription_registry: Option<Res<AssetSubscriptionRegistry>>,
    mut file_watcher: Option<ResMut<FileWatcherResource>>,
) {
    let Some(server) = server else { return };
    let Some(registry) = subscription_registry else { return };
    
    // Get current connected clients
    let current_clients: std::collections::HashSet<u64> = server.clients_id().into_iter().collect();
    
    // Find disconnected clients (were connected, now not)
    let disconnected: Vec<u64> = connected_clients.clients
        .difference(&current_clients)
        .cloned()
        .collect();
    
    // Clean up subscriptions for disconnected clients
    for client_id in disconnected {
        debug!("🧹 [SERVER] Cleaning up subscriptions for disconnected client {}", client_id);
        let empty_paths = registry.unsubscribe_client(client_id);
        
        // Unwatch paths that have no more subscribers
        if let Some(ref mut watcher) = file_watcher {
            for path in empty_paths {
                watcher.unwatch(&path);
            }
        }
    }
    
    // Update connected clients set
    connected_clients.clients = current_clients;
}

// ============================================================================
// Pending Response Resources (for client-side event emission)
// ============================================================================

/// Resource to queue pending directory listing responses for event emission
#[derive(Resource, Default)]
pub struct PendingDirectoryListings {
    responses: std::sync::Mutex<Vec<DirectoryListingResponse>>,
}

impl PendingDirectoryListings {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Queue a response for event emission
    pub fn queue(&self, response: DirectoryListingResponse) {
        self.responses.lock().unwrap().push(response);
    }
    
    /// Take all pending responses for processing
    pub fn take_all(&self) -> Vec<DirectoryListingResponse> {
        std::mem::take(&mut *self.responses.lock().unwrap())
    }
}

/// Resource to queue pending upload responses for event emission
#[derive(Resource, Default)]
pub struct PendingUploadResponses {
    responses: std::sync::Mutex<Vec<AssetUploadResponse>>,
}

impl PendingUploadResponses {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Queue a response for event emission
    pub fn queue(&self, response: AssetUploadResponse) {
        self.responses.lock().unwrap().push(response);
    }
    
    /// Take all pending responses for processing
    pub fn take_all(&self) -> Vec<AssetUploadResponse> {
        std::mem::take(&mut *self.responses.lock().unwrap())
    }
}
