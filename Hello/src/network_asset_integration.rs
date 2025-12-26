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
    decrypt_data, DirectoryListingRequest,
};

#[cfg(feature = "networking")]
use crate::asset_events::{
    AssetEventsPlugin, AssetDirectoryListingEvent, AssetUploadProgressEvent, 
    AssetLocalNewerEvent, AssetFileInfo,
};

#[cfg(feature = "networking")]
use crate::asset_server_delivery::{PendingDirectoryListings, PendingUploadResponses};

#[cfg(feature = "networking")]
use crate::upload_state::PendingUploads;

#[cfg(feature = "networking")]
use crate::server_hash_tracker::ServerFileHashes;

/// Plugin that adds network asset downloading capabilities
pub struct NetworkAssetPlugin;

#[cfg(feature = "networking")]
impl Plugin for NetworkAssetPlugin {
    fn build(&self, app: &mut App) {
        // Add asset events plugin (registers Message types for directory listing, upload, localnewer)
        app.add_plugins(AssetEventsPlugin);
        
        // Initialize client resources
        app.init_resource::<PendingAssetRequests>();
        app.init_resource::<crate::network_asset_client::PendingAssetUpdates>();
        app.init_resource::<PendingCoroutines>();
        
        // Initialize asset browser resources
        app.init_resource::<PendingDirectoryListings>();
        app.init_resource::<PendingUploadResponses>();
        app.init_resource::<PendingUploads>();
        app.init_resource::<ServerFileHashes>();
        
        // Initialize server resources (only created if running as server)
        app.init_resource::<crate::subscription_registry::AssetSubscriptionRegistry>();
        app.insert_resource(crate::subscription_registry::FileWatcherResource::new());
        app.init_resource::<crate::upload_state::ServerPendingUploads>();
        
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
        
        // Initialize reload debounce resource
        app.init_resource::<ReloadDebounce>();
        
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
            // 11. Clean up expired debounce entries
            cleanup_reload_debounce,
            // 12. Emit directory listing events from pending responses
            emit_directory_listing_events,
            // 13. Emit upload progress events from pending responses  
            emit_upload_progress_events,
            // 14. Detect local files newer than server (for upload prompts)
            detect_local_newer_files,
        ).chain().after(crate::asset_server_delivery::handle_asset_requests_global));
    }
}

#[cfg(not(feature = "networking"))]
impl Plugin for NetworkAssetPlugin {
    fn build(&self, _app: &mut App) {
        // No-op when networking is disabled
    }
}

/// Resource to debounce rapid reload events
/// Prevents the same script from reloading multiple times in quick succession
#[cfg(feature = "networking")]
#[derive(Resource, Default)]
pub struct ReloadDebounce {
    /// Map of script path -> last reload timestamp
    recent_reloads: std::collections::HashMap<String, std::time::Instant>,
}

#[cfg(feature = "networking")]
impl ReloadDebounce {
    /// Check if a reload should be skipped (already reloaded recently)
    /// Returns true if should skip, false if should proceed
    pub fn should_skip(&self, path: &str) -> bool {
        if let Some(last_reload) = self.recent_reloads.get(path) {
            // Skip if reloaded within last 500ms
            if last_reload.elapsed() < std::time::Duration::from_millis(500) {
                debug!("⏭️ [DEBOUNCE] Skipping duplicate reload for '{}' (reloaded {:?} ago)", 
                    path, last_reload.elapsed());
                return true;
            }
        }
        false
    }
    
    /// Mark a path as recently reloaded
    pub fn mark_reloaded(&mut self, path: &str) {
        self.recent_reloads.insert(path.to_string(), std::time::Instant::now());
    }
    
    /// Clean up expired entries (older than 1 second)
    pub fn cleanup(&mut self) {
        let cutoff = std::time::Duration::from_secs(1);
        self.recent_reloads.retain(|_, instant| instant.elapsed() < cutoff);
    }
}

/// System to clean up expired reload debounce entries
#[cfg(feature = "networking")]
fn cleanup_reload_debounce(mut debounce: ResMut<ReloadDebounce>) {
    debounce.cleanup();
}

/// System to enable network downloads in Lua by setting a global flag and registering API functions
#[cfg(feature = "networking")]
pub fn enable_network_downloads(
    lua_ctx: Option<Res<LuaScriptContext>>,
    pending_requests: Option<Res<PendingAssetRequests>>,
) {
    let Some(lua_ctx) = lua_ctx else { return };
    let Some(pending_requests) = pending_requests else { return };
    
    if let Err(e) = lua_ctx.lua.globals().set("__NETWORK_DOWNLOAD_ENABLED__", true) {
        error!("Failed to set __NETWORK_DOWNLOAD_ENABLED__: {}", e);
    } else {
        debug!("✓ Network downloads enabled for Lua scripts");
    }
    
    // Register list_server_directory function
    let pending_for_list = Arc::new(pending_requests.clone());
    let list_fn = lua_ctx.lua.create_function(move |_lua, (path, offset, limit): (String, Option<u32>, Option<u32>)| {
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(100);
        
        debug!("📂 [LUA] list_server_directory('{}', offset={}, limit={})", path, offset, limit);
        
        // Queue the directory listing request
        let request_id = pending_for_list.queue_directory_listing(path.clone(), offset, limit);
        
        // Return request_id so Lua can correlate with events if needed
        Ok(request_id)
    });
    
    match list_fn {
        Ok(func) => {
            if let Err(e) = lua_ctx.lua.globals().set("list_server_directory", func) {
                error!("Failed to register list_server_directory: {}", e);
            } else {
                debug!("✓ Registered Lua function: list_server_directory");
            }
        }
        Err(e) => error!("Failed to create list_server_directory function: {}", e),
    }
    
    // Register upload_asset function
    let pending_for_upload = pending_requests.clone();
    let upload_fn = lua_ctx.lua.create_function(move |_lua, (local_path, server_path, force_overwrite): (String, Option<String>, Option<bool>)| {
        use std::path::Path;
        
        let server_path = server_path.unwrap_or_else(|| local_path.clone());
        let force_overwrite = force_overwrite.unwrap_or(false);
        
        debug!("📤 [LUA] upload_asset('{}', '{}', force={})", local_path, server_path, force_overwrite);
        
        // Determine if it's a file or directory
        let full_local_path = Path::new("assets").join(&local_path);
        
        if !full_local_path.exists() {
            return Err(mlua::Error::RuntimeError(format!(
                "Path does not exist: {}", local_path
            )));
        }
        
        if full_local_path.is_dir() {
            // Queue directory upload (all files recursively)
            pending_for_upload.queue_directory_upload(
                local_path.clone(),
                server_path.clone(),
                force_overwrite,
            );
        } else {
            // Queue single file upload
            pending_for_upload.queue_file_upload(
                local_path.clone(),
                server_path.clone(),
                force_overwrite,
            );
        }
        
        Ok(mlua::Value::Boolean(true))
    });
    
    match upload_fn {
        Ok(func) => {
            if let Err(e) = lua_ctx.lua.globals().set("upload_asset", func) {
                error!("Failed to register upload_asset: {}", e);
            } else {
                debug!("✓ Registered Lua function: upload_asset");
            }
        }
        Err(e) => error!("Failed to create upload_asset function: {}", e),
    }
    
    // Register rename_asset(old_path, new_path) -> request_id
    let pending_for_rename = pending_requests.clone();
    let rename_fn = lua_ctx.lua.create_function(move |_lua, (old_path, new_path): (String, String)| {
        debug!("📝 [LUA] rename_asset('{}' -> '{}')", old_path, new_path);
        let request_id = pending_for_rename.queue_rename(old_path, new_path);
        Ok(request_id)
    });
    
    match rename_fn {
        Ok(func) => {
            if let Err(e) = lua_ctx.lua.globals().set("rename_asset", func) {
                error!("Failed to register rename_asset: {}", e);
            } else {
                debug!("✓ Registered Lua function: rename_asset");
            }
        }
        Err(e) => error!("Failed to create rename_asset function: {}", e),
    }
    
    // Register delete_asset(path) -> request_id
    let pending_for_delete = pending_requests.clone();
    let delete_fn = lua_ctx.lua.create_function(move |_lua, path: String| {
        debug!("🗑️ [LUA] delete_asset('{}')", path);
        let request_id = pending_for_delete.queue_delete(path);
        Ok(request_id)
    });
    
    match delete_fn {
        Ok(func) => {
            if let Err(e) = lua_ctx.lua.globals().set("delete_asset", func) {
                error!("Failed to register delete_asset: {}", e);
            } else {
                debug!("✓ Registered Lua function: delete_asset");
            }
        }
        Err(e) => error!("Failed to create delete_asset function: {}", e),
    }
    
    // Register request_server_file(path) -> request_id
    // Used after LocalNewer cancel to explicitly download server version
    let pending_for_server = pending_requests.clone();
    let request_server_fn = lua_ctx.lua.create_function(move |_lua, path: String| {
        debug!("📥 [LUA] request_server_file('{}')", path);
        let request_id = pending_for_server.queue_request_server_file(path);
        Ok(request_id)
    });
    
    match request_server_fn {
        Ok(func) => {
            if let Err(e) = lua_ctx.lua.globals().set("request_server_file", func) {
                error!("Failed to register request_server_file: {}", e);
            } else {
                debug!("✓ Registered Lua function: request_server_file");
            }
        }
        Err(e) => error!("Failed to create request_server_file function: {}", e),
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
        debug!("📤 [UNSUBSCRIBE] Instance {} cleaned up, sending UnsubscribeAll", instance_id);
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
                                    debug!("📥 [RESUME] Coroutine yielded again for new download: {}", new_path);
                                    
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
                debug!("📤 [CLIENT] Sending Subscribe for {} paths (instance {})", paths.len(), instance_id);
            }
            AssetSubscriptionMessage::Unsubscribe { paths, instance_id } => {
                debug!("📤 [CLIENT] Sending Unsubscribe for {} paths (instance {})", paths.len(), instance_id);
            }
            AssetSubscriptionMessage::UnsubscribeAll { instance_id } => {
                debug!("📤 [CLIENT] Sending UnsubscribeAll for instance {}", instance_id);
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
    mut debounce: ResMut<ReloadDebounce>,
    mut received_files: Option<ResMut<crate::plugins::NetworkReceivedFiles>>,
) {
    let Some(ref lua_ctx) = lua_ctx else { return };
    let Some(ref pending_updates) = pending_updates else { return };
    
    // Process all queued updates
    for notification in pending_updates.take_all() {
        debug!("📥 [CLIENT] Received file update notification for '{}' ({} bytes, chunk {}/{})", 
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
            
            // Mark file as received from network to prevent broadcast loops in peer mode
            if let Some(ref mut received) = received_files {
                received.mark_received(&notification.path);
                debug!("📋 [PEER] Marked '{}' as received from network (broadcast loop prevention)", notification.path);
            }
            
            debug!("✅ [CLIENT] Updated file: '{}' ({} bytes)", notification.path, decrypted.len());
            
            // If it's a Lua script, update source cache and trigger hot reload
            if let Ok(source) = String::from_utf8(decrypted.clone()) {
                lua_ctx.script_cache.update_source(&notification.path, source);
                
                // Trigger hot reload via LuaFileChangeEvent
                file_events.write(bevy_lua_ecs::lua_file_watcher::LuaFileChangeEvent {
                    path: std::path::PathBuf::from(&notification.path),
                });
                
                debug!("🔄 [CLIENT] Triggered hot reload for '{}'", notification.path);
            } else {
                // Binary asset (image, etc.) - check for dependent scripts that should reload
                let dependent_scripts = lua_ctx.script_cache.get_dependent_scripts(&notification.path);
                
                if !dependent_scripts.is_empty() {
                    debug!("📷 [CLIENT] Asset '{}' updated, triggering reload for {} dependent script(s)", 
                        notification.path, dependent_scripts.len());
                    
                    // Trigger reload for each dependent script
                    // We collect unique script paths since multiple instances may use same script
                    let mut unique_scripts: std::collections::HashSet<String> = std::collections::HashSet::new();
                    for (script_path, _instance_id) in dependent_scripts {
                        unique_scripts.insert(script_path);
                    }
                    
                    for script_path in unique_scripts {
                        // Check debounce - skip if reloaded recently
                        if debounce.should_skip(&script_path) {
                            continue;
                        }
                        
                        // Mark as reloaded before triggering
                        debounce.mark_reloaded(&script_path);
                        
                        debug!("🔄 [CLIENT] Triggering reload for script '{}' (depends on '{}')", script_path, notification.path);
                        file_events.write(bevy_lua_ecs::lua_file_watcher::LuaFileChangeEvent {
                            path: std::path::PathBuf::from(format!("assets/{}", script_path)),
                        });
                    }
                } else {
                    debug!("📷 [CLIENT] Binary asset '{}' updated, no dependent scripts registered", notification.path);
                }
            }
        } else {
            // Multi-chunk file - would need assembly logic
            // For now, log a warning
            debug!("📥 [CLIENT] Multi-chunk update for '{}' (chunk {}/{}) - assembly not yet implemented", 
                notification.path, notification.chunk_index + 1, notification.total_chunks);
        }
    }
}

// ============================================================================
// Event Emission Systems
// ============================================================================

/// System to emit directory listing events from pending responses
#[cfg(feature = "networking")]
pub fn emit_directory_listing_events(
    pending: Res<PendingDirectoryListings>,
    mut events: MessageWriter<AssetDirectoryListingEvent>,
) {
    for response in pending.take_all() {
        debug!("📂 [EVENT] Emitting directory listing event for path '{}'", response.path);
        
        events.write(AssetDirectoryListingEvent {
            path: response.path,
            files: response.files.iter().map(AssetFileInfo::from).collect(),
            total_count: response.total_count,
            offset: response.offset,
            has_more: response.has_more,
            error: response.error,
        });
    }
}

/// System to emit upload progress events from pending responses
#[cfg(feature = "networking")]
pub fn emit_upload_progress_events(
    pending: Res<PendingUploadResponses>,
    pending_uploads: Res<PendingUploads>,
    mut events: MessageWriter<AssetUploadProgressEvent>,
) {
    use crate::network_asset_client::UploadStatus;
    
    for response in pending.take_all() {
        // Get current upload info
        let upload_info = pending_uploads.get_upload(response.request_id);
        
        let (status, chunks_sent, total_chunks, progress, error, server_hash) = match &response.status {
            UploadStatus::ChunkReceived { chunk_index, total_chunks: total } => {
                let acked = chunk_index + 1;
                let prog = acked as f32 / *total as f32;
                ("uploading".to_string(), acked, *total, prog, None, None)
            }
            UploadStatus::Complete { server_hash } => {
                let total = upload_info.map(|i| i.total_chunks).unwrap_or(1);
                ("complete".to_string(), total, total, 1.0, None, Some(server_hash.clone()))
            }
            UploadStatus::Error(msg) => {
                let (sent, total) = upload_info.map(|i| (i.sent_chunks, i.total_chunks)).unwrap_or((0, 1));
                let prog = sent as f32 / total as f32;
                ("error".to_string(), sent, total, prog, Some(msg.clone()), None)
            }
            UploadStatus::Conflict { server_hash } => {
                let (sent, total) = upload_info.map(|i| (i.sent_chunks, i.total_chunks)).unwrap_or((0, 1));
                ("conflict".to_string(), sent, total, 0.0, None, Some(server_hash.clone()))
            }
        };
        
        debug!("📤 [EVENT] Emitting upload progress event for '{}': {} ({:.0}%)", 
            response.path, status, progress * 100.0);
        
        events.write(AssetUploadProgressEvent {
            path: response.path,
            chunks_sent,
            total_chunks,
            progress,
            status,
            error,
            server_hash,
        });
    }
}

/// System to detect when local files are newer than server versions
/// 
/// Monitors file change events and compares local hashes against stored server hashes.
/// Emits AssetLocalNewerEvent when a local file differs from the server version.
#[cfg(feature = "networking")]
pub fn detect_local_newer_files(
    mut file_events: bevy::prelude::MessageReader<bevy_lua_ecs::lua_file_watcher::LuaFileChangeEvent>,
    server_hashes: Res<ServerFileHashes>,
    mut local_newer_events: bevy::prelude::MessageWriter<AssetLocalNewerEvent>,
) {
    for event in file_events.read() {
        // Convert file path to asset-relative path
        let path_str = event.path.to_string_lossy().replace('\\', "/");
        let asset_path = if path_str.starts_with("assets/") {
            path_str.strip_prefix("assets/").unwrap_or(&path_str).to_string()
        } else {
            path_str.to_string()
        };
        
        // Check if we have a server hash for this file
        if let Some(server_hash) = server_hashes.get_hash(&asset_path) {
            // Read local file and compute hash
            let full_path = std::path::Path::new("assets").join(&asset_path);
            
            if let Ok(data) = std::fs::read(&full_path) {
                let local_hash = crate::network_asset_client::compute_hash(&data);
                
                // Compare hashes
                if local_hash != server_hash {
                    // Get modification time
                    let local_modified = std::fs::metadata(&full_path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    
                    debug!(
                        "🔄 [LOCAL_NEWER] Detected local change for '{}': local={} != server={}",
                        asset_path, local_hash, server_hash
                    );
                    
                    local_newer_events.write(AssetLocalNewerEvent {
                        path: asset_path.clone(),
                        local_hash,
                        server_hash: server_hash.to_string(),
                        local_modified,
                    });
                }
            }
        }
    }
}
