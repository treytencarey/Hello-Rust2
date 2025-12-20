// Asset server delivery system
//
// This module handles asset request processing using global Renet resources:
// - Server: Receives asset requests from clients, reads assets from disk, chunks/encrypts, sends responses
// - Client: Sends pending requests, receives responses, saves to disk

use bevy::prelude::*;
use bytes::Bytes;
use crate::network_asset_client::{
    AssetRequestMessage, AssetResponseMessage, AssetSubscriptionMessage, AssetUpdateNotification,
    ASSET_CHANNEL, chunk_and_encrypt, decrypt_data,
};
use crate::subscription_registry::{AssetSubscriptionRegistry, FileWatcherResource};

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
}

/// System to receive asset responses using global RenetClient resource
/// Handles both AssetResponseMessage and AssetUpdateNotification via the wrapper enum
#[cfg(feature = "networking")]
pub fn receive_asset_responses_global(
    pending_requests: Res<crate::network_asset_client::PendingAssetRequests>,
    mut pending_updates: ResMut<crate::network_asset_client::PendingAssetUpdates>,
    client: Option<ResMut<RenetClient>>,
) {
    let Some(mut client) = client else { return };
    
    while let Some(message_bytes) = client.receive_message(ASSET_CHANNEL) {
        match bincode::deserialize::<crate::network_asset_client::ServerToClientMessage>(&message_bytes) {
            Ok(crate::network_asset_client::ServerToClientMessage::Response(response)) => {
                process_asset_response(&pending_requests, response);
            }
            Ok(crate::network_asset_client::ServerToClientMessage::Update(notification)) => {
                // Queue the update for processing by receive_asset_updates system
                pending_updates.queue(notification);
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
