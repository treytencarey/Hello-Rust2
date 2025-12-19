// Network asset integration module
//
// This module ties together:
// - Asset request queue (from network_asset_client)
// - Asset delivery systems (from asset_server_delivery)
// - Lua coroutine resumption (from script_cache)
// - Bevy systems to orchestrate everything

use bevy::prelude::*;
use std::sync::Arc;

#[cfg(feature = "networking")]
use bevy_lua_ecs::LuaScriptContext;

#[cfg(feature = "networking")]
use crate::network_asset_client::{
    PendingAssetRequests, PendingCoroutines, PendingScriptCoroutine,
    AssetType, AssetSubscriptionMessage, AssetUpdateNotification, ASSET_CHANNEL,
    decrypt_data,
};

/// Plugin that adds network asset downloading capabilities
pub struct NetworkAssetPlugin;

#[cfg(feature = "networking")]
impl Plugin for NetworkAssetPlugin {
    fn build(&self, app: &mut App) {
        // Initialize client resources
        app.init_resource::<PendingAssetRequests>();
        app.init_resource::<crate::network_asset_client::PendingAssetUpdates>();
        app.init_resource::<PendingCoroutines>();
        
        // Initialize server resources (only created if running as server)
        app.init_resource::<crate::subscription_registry::AssetSubscriptionRegistry>();
        app.insert_resource(crate::subscription_registry::FileWatcherResource::new());
        
        // Enable network downloads in Lua context (set global flag)
        app.add_systems(PostStartup, enable_network_downloads);
        
        // Add systems - order matters! Split into two chains to avoid tuple size limits
        app.add_systems(Update, (
            // 1. Process download requests from yielded coroutines
            process_download_requests,
            // 2. Process pending unsubscriptions from script cleanup
            process_pending_unsubscriptions,
            // 3. Send pending requests to server
            crate::asset_server_delivery::send_asset_requests_global,
            // 4. Send pending subscription messages to server
            send_subscription_messages,
            // 5. Handle incoming requests on server side
            crate::asset_server_delivery::handle_asset_requests_global,
        ).chain());
        
        app.add_systems(Update, (
            // 6. Broadcast file updates to subscribed clients (server-side)
            crate::asset_server_delivery::broadcast_file_updates,
            // 7. Receive responses from server
            crate::asset_server_delivery::receive_asset_responses_global,
            // 8. Receive file update notifications from server
            receive_asset_updates,
            // 9. Check for timeouts
            crate::asset_server_delivery::check_request_timeouts,
            // 10. Resume waiting coroutines when downloads complete
            resume_pending_coroutines,
        ).chain().after(crate::asset_server_delivery::handle_asset_requests_global));
    }
}

#[cfg(not(feature = "networking"))]
impl Plugin for NetworkAssetPlugin {
    fn build(&self, _app: &mut App) {
        // No-op when networking is disabled
    }
}

/// System to enable network downloads in Lua by setting a global flag
#[cfg(feature = "networking")]
pub fn enable_network_downloads(
    lua_ctx: Option<Res<LuaScriptContext>>,
) {
    let Some(lua_ctx) = lua_ctx else { return };
    
    if let Err(e) = lua_ctx.lua.globals().set("__NETWORK_DOWNLOAD_ENABLED__", true) {
        error!("Failed to set __NETWORK_DOWNLOAD_ENABLED__: {}", e);
    } else {
        debug!("✓ Network downloads enabled for Lua scripts");
    }
}

/// System to process download requests from yielded coroutines
/// This looks for paths in script_cache that need downloads and queues them
#[cfg(feature = "networking")]
pub fn process_download_requests(
    pending_requests: Res<PendingAssetRequests>,
    lua_ctx: Option<Res<LuaScriptContext>>,
) {
    let Some(lua_ctx) = lua_ctx else { return };
    
    // Get all paths that have pending coroutines waiting for downloads
    let pending_paths = lua_ctx.script_cache.get_all_pending_download_paths();
    
    if !pending_paths.is_empty() {
        debug!("📋 [PROCESS] Found {} paths with pending download coroutines: {:?}", pending_paths.len(), pending_paths);
    }
    
    for path in pending_paths {
        // Check if we already have a pending request for this path
        if pending_requests.has_pending_request(&path) {
            debug!("⏭️ [PROCESS] Skipping '{}' - already has pending request", path);
            continue;
        }
        
        // Check if already completed
        if pending_requests.is_completed(&path) {
            debug!("⏭️ [PROCESS] Skipping '{}' - already completed (in completed_assets)", path);
            continue;
        }
        
        // Check if already marked up-to-date
        if pending_requests.is_up_to_date(&path) {
            debug!("⏭️ [PROCESS] Skipping '{}' - already marked up-to-date", path);
            continue;
        }
        
        // Get the context path (the script that requested this download)
        let context_path = lua_ctx.script_cache.get_download_context(&path);
        
        // Queue a new download request with context for server-side path resolution
        let request_id = pending_requests.queue_request(path.clone(), AssetType::Script, context_path.clone());
        debug!("📤 [PROCESS] Queued download request for '{}' (request_id: {}, context: {:?})", path, request_id, context_path);
    }
}

/// System to process pending unsubscriptions from script cleanup
/// This picks up unsubscription events queued by cleanup_script_instance and sends them to server
#[cfg(feature = "networking")]
pub fn process_pending_unsubscriptions(
    pending_requests: Res<PendingAssetRequests>,
    lua_ctx: Option<Res<LuaScriptContext>>,
) {
    let Some(lua_ctx) = lua_ctx else { return };
    
    // Get pending unsubscription events from script cache
    let unsubs = lua_ctx.script_cache.take_pending_unsubscriptions();
    
    for (instance_id, empty_paths) in unsubs {
        // Queue UnsubscribeAll message for this instance
        // This tells the server this instance no longer needs updates
        info!("📤 [UNSUBSCRIBE] Instance {} cleaned up, sending UnsubscribeAll", instance_id);
        pending_requests.queue_subscription(AssetSubscriptionMessage::UnsubscribeAll {
            instance_id,
        });
        
        // If there are paths that became empty (no more subscribers), log them
        if !empty_paths.is_empty() {
            debug!("📝 [UNSUBSCRIBE] Paths with no more subscribers: {:?}", empty_paths);
        }
    }
}

/// System that resumes coroutines when their awaited assets are downloaded
#[cfg(feature = "networking")]
pub fn resume_pending_coroutines(
    pending_requests: Res<PendingAssetRequests>,
    lua_ctx: Option<Res<LuaScriptContext>>,
    mut file_events: bevy::prelude::MessageWriter<bevy_lua_ecs::lua_file_watcher::LuaFileChangeEvent>,
) {
    let Some(lua_ctx) = lua_ctx else { return };
    
    // Get list of paths that have pending coroutines
    let pending_paths = lua_ctx.script_cache.get_all_pending_download_paths();
    
    for path in pending_paths {
        // Check if this asset has been downloaded
        if pending_requests.is_completed(&path) {
            // Take the downloaded data
            if let Some(data) = pending_requests.take_completed(&path) {
                debug!("✅ [RESUME] Downloaded '{}' ({} bytes)", path, data.len());
                
                // Always write to disk as binary (works for both scripts and assets)
                let asset_path = std::path::Path::new("assets").join(&path);
                if let Some(parent) = asset_path.parent() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        error!("❌ [RESUME] Failed to create directory for '{}': {}", path, e);
                        continue;
                    }
                }
                
                if let Err(e) = std::fs::write(&asset_path, &data) {
                    error!("❌ [RESUME] Failed to write file '{}': {}", path, e);
                    continue;
                }
                
                debug!("✅ [RESUME] Written to: {:?}", asset_path);
                
                // Check if any instances should subscribe to this path (reload=true)
                let subscription_instances = lua_ctx.script_cache.take_pending_subscription_instances(&path);
                if !subscription_instances.is_empty() {
                    debug!("📝 [SUBSCRIPTION] {} instance(s) subscribing to '{}'", subscription_instances.len(), path);
                    
                    // Subscribe each instance locally
                    for instance_id in &subscription_instances {
                        lua_ctx.script_cache.subscribe(path.clone(), *instance_id);
                    }
                    
                    // Queue a Subscribe message to the server (batch all instances for this path)
                    // The server just needs to know which paths, not which instances (that's client-side)
                    let first_instance = subscription_instances[0];
                    pending_requests.queue_subscription(AssetSubscriptionMessage::Subscribe {
                        paths: vec![path.clone()],
                        instance_id: first_instance,
                    });
                }
                
                // For scripts (text files), also update source cache and trigger hot reload
                // Try to parse as UTF-8 - if it works, it's a script; if not, it's binary
                if let Ok(source) = String::from_utf8(data) {
                    // It's a valid UTF-8 file (script) - update source cache
                    lua_ctx.script_cache.update_source(&path, source.clone());
                    
                    // Use resume_coroutines_with_source which also invokes hot reload callbacks
                    resume_coroutines_with_source(&lua_ctx, &pending_requests, &path, source);
                } else {
                    // Binary asset - just resume waiting coroutines
                    resume_coroutines(&lua_ctx, &path);
                }
            }
        } else if pending_requests.is_up_to_date(&path) {
            // Asset is up-to-date, just resume coroutines
            debug!("✅ [RESUME] Asset '{}' confirmed up-to-date", path);
            
            // Also check for subscriptions on up-to-date (file existed locally but was verified)
            let subscription_instances = lua_ctx.script_cache.take_pending_subscription_instances(&path);
            if !subscription_instances.is_empty() {
                debug!("📝 [SUBSCRIPTION] {} instance(s) subscribing to '{}' (up-to-date)", subscription_instances.len(), path);
                
                for instance_id in &subscription_instances {
                    lua_ctx.script_cache.subscribe(path.clone(), *instance_id);
                }
                
                let first_instance = subscription_instances[0];
                pending_requests.queue_subscription(AssetSubscriptionMessage::Subscribe {
                    paths: vec![path.clone()],
                    instance_id: first_instance,
                });
            }
            
            pending_requests.mark_up_to_date(&path); // Clear the status
            resume_coroutines(&lua_ctx, &path);
        }
    }
}

/// Helper function to resume coroutines waiting for a path
#[cfg(feature = "networking")]
fn resume_coroutines(
    lua_ctx: &LuaScriptContext,
    path: &str,
) {
    // Take waiting coroutines
    let waiting_coroutines = lua_ctx.script_cache.take_pending_download_coroutines(&path);
    
    debug!("📋 Resuming {} coroutines for '{}'", waiting_coroutines.len(), path);
    
    for (coroutine_key, instance_id) in waiting_coroutines {
        debug!("Resuming coroutine for '{}' (instance {})", path, instance_id);
        
        // First check if this is a nil placeholder (used for background server checks)
        // Background checks register with nil - they're not actual coroutines to resume
        let registry_value: mlua::Value = match lua_ctx.lua.registry_value(&*coroutine_key) {
            Ok(v) => v,
            Err(e) => {
                error!("❌ [RESUME] Failed to get registry value: {}", e);
                continue;
            }
        };
        
        // Skip nil placeholders (from background server checks)
        if matches!(registry_value, mlua::Value::Nil) {
            debug!("⏭️ [RESUME] Skipping nil placeholder for '{}' (background check)", path);
            continue;
        }
        
        // Convert to thread
        let coroutine: mlua::Thread = match registry_value {
            mlua::Value::Thread(t) => t,
            _ => {
                warn!("⚠️ [RESUME] Expected thread but got {:?} for '{}'", registry_value, path);
                continue;
            }
        };
        
        // Resume the coroutine - it will retry require/load_asset and find the file locally
        match coroutine.resume::<mlua::Value>(()) {
            Ok(yield_value) => {
                debug!("✅ [RESUME] Coroutine resumed for '{}'", path);
                
                // Check status
                match coroutine.status() {
                    mlua::ThreadStatus::Finished => {
                        debug!("✅ [RESUME] Coroutine completed for '{}'", path);
                    }
                    mlua::ThreadStatus::Resumable => {
                        // Coroutine yielded again - needs another download
                        // The yield value should be the new download path
                        if let mlua::Value::String(path_str) = yield_value {
                            if let Ok(new_path) = path_str.to_str() {
                                let new_path = new_path.to_string();
                                debug!("📥 [RESUME] Coroutine for '{}' yielded for new download: {}", path, new_path);
                                
                                // Re-register the coroutine for the new path
                                let coroutine_key = match lua_ctx.lua.create_registry_value(coroutine) {
                                    Ok(key) => std::sync::Arc::new(key),
                                    Err(e) => {
                                        error!("❌ [RESUME] Failed to store coroutine in registry: {}", e);
                                        continue;
                                    }
                                };
                                
                                // Note: is_binary is just metadata - actual binary/text detection
                                // uses UTF-8 parsing at resume time
                                lua_ctx.script_cache.register_pending_download_coroutine(
                                    new_path.clone(),
                                    coroutine_key,
                                    instance_id,
                                    false, // Detection happens at resume via UTF-8 parsing
                                    Some(path.to_string()), // Use the current path as context
                                    false, // should_subscribe handled at original require level
                                );
                                
                                debug!("📋 [RESUME] Re-registered coroutine for '{}'", new_path);
                            }
                        } else {
                            debug!("⏳ [RESUME] Coroutine yielded again (may need another download)");
                        }
                    }
                    _ => {}
                }
            }
            Err(e) => {
                error!("❌ [RESUME] Failed to resume coroutine: {}", e);
            }
        }
    }
}

/// Helper function to resume coroutines and invoke callbacks with source
#[cfg(feature = "networking")]
fn resume_coroutines_with_source(
    lua_ctx: &LuaScriptContext,
    _pending_requests: &PendingAssetRequests,
    path: &str,
    source: String,
) {
    // Take waiting coroutines
    let waiting_coroutines = lua_ctx.script_cache.take_pending_download_coroutines(&path);
    
    for (coroutine_key, instance_id) in waiting_coroutines {
        debug!("Resuming coroutine for '{}' (instance {})", path, instance_id);
        
        // Resume the coroutine with the downloaded source
        if let Ok(coroutine) = lua_ctx.lua.registry_value::<mlua::Thread>(&*coroutine_key) {
            // Set instance ID before resuming
            if let Err(e) = lua_ctx.lua.globals().set("__INSTANCE_ID__", instance_id) {
                error!("Failed to set __INSTANCE_ID__: {}", e);
            }
            
            // Resume with the source code
            match coroutine.resume::<mlua::Value>(source.clone()) {
                Ok(yield_value) => {
                    debug!("✓ Coroutine resumed for '{}'", path);
                    
                    // Check if the coroutine yielded again (needs another download)
                    match coroutine.status() {
                        mlua::ThreadStatus::Finished => {
                            debug!("✓ Coroutine completed for '{}'", path);
                        }
                        mlua::ThreadStatus::Resumable => {
                            // Coroutine yielded again - needs another download
                            if let mlua::Value::String(path_str) = yield_value {
                                if let Ok(new_path) = path_str.to_str() {
                                    let new_path = new_path.to_string();
                                    info!("📥 [RESUME] Coroutine yielded again for new download: {}", new_path);
                                    
                                    // Re-register the coroutine for the new path
                                    let coroutine_key = match lua_ctx.lua.create_registry_value(coroutine) {
                                        Ok(key) => std::sync::Arc::new(key),
                                        Err(e) => {
                                            error!("❌ [RESUME] Failed to store coroutine in registry: {}", e);
                                            continue;
                                        }
                                    };
                                    
                                    // Note: is_binary is just metadata - actual binary/text detection
                                    // uses UTF-8 parsing at resume time
                                    lua_ctx.script_cache.register_pending_download_coroutine(
                                        new_path.clone(),
                                        coroutine_key,
                                        instance_id,
                                        false, // Detection happens at resume via UTF-8 parsing
                                        Some(path.to_string()),
                                        false,
                                    );
                                    
                                    debug!("📋 [RESUME] Re-registered coroutine for '{}'", new_path);
                                }
                            } else {
                                debug!("⏳ [RESUME] Coroutine yielded with non-string value (may need another download)");
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    error!("❌ Failed to resume coroutine for '{}': {}", path, e);
                }
            }
        }
    }
    
    // Also invoke hot_reload callbacks (used by require_async for initial downloads)
    let callbacks = lua_ctx.script_cache.get_hot_reload_callbacks(&path);
    if !callbacks.is_empty() {
        debug!("Invoking {} require_async callbacks for '{}'", callbacks.len(), path);
        
        // Load and execute the module first
        let module_name = format!("@{}", path);
        match bevy_lua_ecs::script_cache::execute_module(&lua_ctx.lua, &source, &module_name) {
            Ok(module_result) => {
                // Invoke each callback with the module result
                for (callback_key, parent_instance_id) in callbacks {
                    if let Ok(callback) = lua_ctx.lua.registry_value::<mlua::Function>(&*callback_key) {
                        // Set parent instance ID
                        if let Err(e) = lua_ctx.lua.globals().set("__INSTANCE_ID__", parent_instance_id) {
                            error!("Failed to set __INSTANCE_ID__: {}", e);
                        }
                        
                        // Call the callback with the module result
                        if let Err(e) = callback.call::<()>(module_result.clone()) {
                            error!("❌ Failed to invoke callback for '{}': {}", path, e);
                        } else {
                            debug!("✓ Callback invoked for '{}'", path);
                        }
                    }
                }
                
                // Clear callbacks after invoking (they're one-time for initial load)
                lua_ctx.script_cache.clear_hot_reload_callbacks(&path);
            }
            Err(e) => {
                error!("❌ Failed to execute module '{}': {}", path, e);
            }
        }
    }
}


/// Helper function to queue a script download request
/// Called from the require() function when a script isn't available locally
#[cfg(feature = "networking")]
pub fn queue_script_download(
    pending_requests: &PendingAssetRequests,
    path: &str,
    context_path: Option<String>,
) -> u64 {
    pending_requests.queue_request(path.to_string(), AssetType::Script, context_path)
}

/// Check if a script is available (either locally or download completed)
#[cfg(feature = "networking")]
pub fn is_script_available(
    pending_requests: &PendingAssetRequests,
    path: &str,
) -> bool {
    // Check local filesystem first (paths include scripts/ prefix, e.g., "scripts/examples/foo.lua")
    let local_path = std::path::Path::new("assets").join(path);
    if local_path.exists() {
        return true;
    }
    
    // Check if download completed
    pending_requests.is_completed(path)
}

/// Get the status of a script download
#[cfg(feature = "networking")]
pub fn get_download_status(
    pending_requests: &PendingAssetRequests,
    path: &str,
) -> Option<crate::network_asset_client::AssetRequestStatus> {
    pending_requests.get_request(path).map(|r| r.status)
}

/// System to send pending subscription messages to server
#[cfg(feature = "networking")]
pub fn send_subscription_messages(
    pending_requests: Res<PendingAssetRequests>,
    client: Option<ResMut<bevy_replicon_renet::renet::RenetClient>>,
) {
    let Some(mut client) = client else { return };
    
    // Get pending subscription messages
    let subscriptions = pending_requests.drain_pending_subscriptions();
    
    for sub_msg in subscriptions {
        match &sub_msg {
            AssetSubscriptionMessage::Subscribe { paths, instance_id } => {
                info!("📤 [CLIENT] Sending Subscribe for {} paths (instance {})", paths.len(), instance_id);
            }
            AssetSubscriptionMessage::Unsubscribe { paths, instance_id } => {
                info!("📤 [CLIENT] Sending Unsubscribe for {} paths (instance {})", paths.len(), instance_id);
            }
            AssetSubscriptionMessage::UnsubscribeAll { instance_id } => {
                info!("📤 [CLIENT] Sending UnsubscribeAll for instance {}", instance_id);
            }
        }
        
        // Wrap in ClientToServerMessage for proper type discrimination
        let wrapped = crate::network_asset_client::ClientToServerMessage::Subscription(sub_msg);
        if let Ok(message_bytes) = bincode::serialize(&wrapped) {
            client.send_message(ASSET_CHANNEL, bytes::Bytes::from(message_bytes));
        }
    }
}

/// System to receive file update notifications from server (push updates)
/// Processes updates that were queued by receive_asset_responses_global
#[cfg(feature = "networking")]
pub fn receive_asset_updates(
    lua_ctx: Option<Res<LuaScriptContext>>,
    pending_updates: Option<Res<crate::network_asset_client::PendingAssetUpdates>>,
    mut file_events: bevy::prelude::MessageWriter<bevy_lua_ecs::lua_file_watcher::LuaFileChangeEvent>,
) {
    let Some(ref lua_ctx) = lua_ctx else { return };
    let Some(ref pending_updates) = pending_updates else { return };
    
    // Process all queued updates
    for notification in pending_updates.take_all() {
        info!("📥 [CLIENT] Received file update notification for '{}' ({} bytes, chunk {}/{})", 
            notification.path, notification.total_size, 
            notification.chunk_index + 1, notification.total_chunks);
        
        // For single-chunk files, process immediately
        // TODO: For multi-chunk files, we'd need to assemble them (similar to asset_server_delivery)
        if notification.total_chunks == 1 {
            // Decrypt the data
            let decrypted = match decrypt_data(&notification.data) {
                Ok(data) => data,
                Err(e) => {
                    error!("❌ [CLIENT] Failed to decrypt update for '{}': {}", notification.path, e);
                    continue;
                }
            };
            
            // Write to disk
            let asset_path = std::path::Path::new("assets").join(&notification.path);
            if let Some(parent) = asset_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    error!("❌ [CLIENT] Failed to create directory for '{}': {}", notification.path, e);
                    continue;
                }
            }
            
            if let Err(e) = std::fs::write(&asset_path, &decrypted) {
                error!("❌ [CLIENT] Failed to write updated file '{}': {}", notification.path, e);
                continue;
            }
            
            info!("✅ [CLIENT] Updated file: '{}' ({} bytes)", notification.path, decrypted.len());
            
            // If it's a Lua script, update source cache and trigger hot reload
            if let Ok(source) = String::from_utf8(decrypted.clone()) {
                lua_ctx.script_cache.update_source(&notification.path, source);
                
                // Trigger hot reload via LuaFileChangeEvent
                file_events.write(bevy_lua_ecs::lua_file_watcher::LuaFileChangeEvent {
                    path: std::path::PathBuf::from(&notification.path),
                });
                
                info!("🔄 [CLIENT] Triggered hot reload for '{}'", notification.path);
            }
        } else {
            // Multi-chunk file - would need assembly logic
            // For now, log a warning
            debug!("📥 [CLIENT] Multi-chunk update for '{}' (chunk {}/{}) - assembly not yet implemented", 
                notification.path, notification.chunk_index + 1, notification.total_chunks);
        }
    }
}

