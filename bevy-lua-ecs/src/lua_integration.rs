use crate::lua_systems::LuaSystemRegistry;
use crate::path_utils::{normalize_path, normalize_path_separators, to_forward_slash};
use crate::spawn_queue::SpawnQueue;
use bevy::prelude::*;
use mlua::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
/// Resource that holds the Lua context
#[derive(Resource, Clone)]
pub struct LuaScriptContext {
    pub lua: Arc<Lua>,
    pub script_cache: crate::script_cache::ScriptCache,
    pub script_instance: crate::script_entities::ScriptInstance,
}
impl LuaScriptContext {
    /// Create a new Lua context with component-based spawn function
    pub fn new(
        queue: SpawnQueue,
        despawn_queue: crate::despawn_queue::DespawnQueue,
        resource_queue: crate::resource_queue::ResourceQueue,
        update_queue: crate::component_update_queue::ComponentUpdateQueue,
        system_registry: LuaSystemRegistry,
        _builder_registry: crate::resource_builder::ResourceBuilderRegistry,
        script_instance: crate::script_entities::ScriptInstance,
    ) -> Result<Self, LuaError> {
        let lua = Lua::new();

        // Clone what we need for the closure
        let queue_clone = queue.clone();
        let update_queue_clone = update_queue.clone();
        let update_queue_for_parent = update_queue.clone();
        let resource_queue_clone = resource_queue.clone();
        let lua_clone = Arc::new(lua);
        let lua_for_closure = lua_clone.clone();
        let lua_for_resource = lua_clone.clone();

        // Track entity ID counter for immediate return (will be synced in process_spawn_queue)
        let entity_counter = Arc::new(std::sync::atomic::AtomicU32::new(0));

        // Create component-based spawn function that returns a chainable SpawnBuilder
        let counter_clone = entity_counter.clone();
        let spawn = lua_clone.create_function(move |lua_ctx, components: LuaTable| {
            // Capture current __INSTANCE_ID__ at queue time
            let instance_id: Option<u64> = lua_ctx.globals().get("__INSTANCE_ID__").ok();

            debug!("[SPAWN] Queuing entity with instance_id: {:?}", instance_id);

            let mut all_components = Vec::new();

            // Iterate over components table
            for pair in components.pairs::<String, LuaValue>() {
                let (component_name, component_value) = pair?;

                // Store everything as registry value
                let registry_key = lua_for_closure.create_registry_value(component_value)?;
                all_components.push((component_name, registry_key));
            }

            // Queue spawn with captured instance_id
            let temp_id = queue_clone.generate_temp_id();
            queue_clone
                .clone()
                .queue_spawn(all_components, Vec::new(), instance_id, temp_id);

            // Return SpawnBuilder for chainable API
            let builder = crate::lua_spawn_builder::LuaSpawnBuilder::new(
                temp_id,
                queue_clone.clone(),
                update_queue_clone.clone(),
                lua_for_closure.clone(),
            );
            Ok(builder)
        })?;

        // Create spawn_with_parent function (legacy - prefer spawn().with_parent())
        let queue_for_parent = queue.clone();
        let lua_for_parent = lua_clone.clone();
        let spawn_with_parent = lua_clone.create_function(
            move |lua_ctx, (parent_id, components): (u64, LuaTable)| {
                let mut all_components = Vec::new();

                // Capture current __INSTANCE_ID__ at queue time
                let instance_id: Option<u64> = lua_ctx.globals().get("__INSTANCE_ID__").ok();

                // Iterate over components table
                for pair in components.pairs::<String, LuaValue>() {
                    let (component_name, component_value) = pair?;

                    // Store everything as registry value
                    let registry_key = lua_for_parent.create_registry_value(component_value)?;
                    all_components.push((component_name, registry_key));
                }

                // Queue spawn with parent temp_id (will be resolved during spawn queue processing)
                let temp_id = queue_for_parent.generate_temp_id();
                queue_for_parent.clone().queue_spawn_with_parent(
                    parent_id,
                    all_components,
                    Vec::new(),
                    instance_id,
                    temp_id,
                );

                // Return SpawnBuilder for chainable API (can still add observers)
                let builder = crate::lua_spawn_builder::LuaSpawnBuilder::new(
                    temp_id,
                    queue_for_parent.clone(),
                    update_queue_for_parent.clone(),
                    lua_for_parent.clone(),
                );
                Ok(builder)
            },
        )?;

        // Create generic insert_resource function
        // Accepts either a table (for serde resources) or UserData (for builder-created resources)
        let insert_resource = lua_clone.create_function(
            move |lua_ctx, (resource_name, resource_data): (String, LuaValue)| {
                // Get the current instance ID from globals
                let instance_id: Option<u64> = lua_ctx.globals().get("__INSTANCE_ID__").ok();

                // Store resource data as registry value (works for both tables and UserData)
                let registry_key = lua_for_resource.create_registry_value(resource_data)?;
                resource_queue_clone.queue_insert(resource_name, registry_key, instance_id);
                Ok(())
            },
        )?;

        // Create register_system function
        let system_reg = system_registry.clone();
        let register_system = lua_clone.create_function(
            move |lua_ctx, (_schedule, func): (String, LuaFunction)| {
                // Get the current instance ID from globals
                let instance_id: u64 = lua_ctx.globals().get("__INSTANCE_ID__").unwrap_or(0);

                let registry_key = lua_ctx.create_registry_value(func)?;
                system_reg
                    .update_systems
                    .lock()
                    .unwrap()
                    .push((instance_id, Arc::new(registry_key)));
                Ok(())
            },
        )?;

        // Create copy_file function for file operations
        let copy_file = lua_clone.create_function(|_lua_ctx, (src, dest): (String, String)| {
            use std::fs;
            use std::path::Path;

            // Create destination directory if it doesn't exist
            if let Some(parent) = Path::new(&dest).parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    LuaError::RuntimeError(format!("Failed to create directory: {}", e))
                })?;
            }

            // Copy the file
            fs::copy(&src, &dest)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to copy file: {}", e)))?;

            Ok(())
        })?;

        // Create read_file_bytes function to read binary file contents
        let read_file_bytes = lua_clone.create_function(|lua_ctx, path: String| {
            use std::fs;
            let bytes = fs::read(&path)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to read file: {}", e)))?;
            lua_ctx.create_string(&bytes)
        })?;

        // Create write_file_bytes function to write binary file contents
        let write_file_bytes =
            lua_clone.create_function(|_lua_ctx, (path, data): (String, LuaString)| {
                use std::fs;
                use std::path::Path;

                // Create destination directory if it doesn't exist
                if let Some(parent) = Path::new(&path).parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        LuaError::RuntimeError(format!("Failed to create directory: {}", e))
                    })?;
                }

                fs::write(&path, data.as_bytes())
                    .map_err(|e| LuaError::RuntimeError(format!("Failed to write file: {}", e)))?;
                Ok(())
            })?;

        // Create create_directory function to create directories
        let create_directory =
            lua_clone.create_function(|_lua_ctx, path: String| {
                use std::fs;

                fs::create_dir_all(&path).map_err(|e| {
                    LuaError::RuntimeError(format!("Failed to create directory: {}", e))
                })?;
                Ok(())
            })?;

        // OS Utilities for networking and other low-level operations

        // Bind UDP socket
        let bind_udp_socket = lua_clone.create_function(|_lua_ctx, addr: String| {
            crate::os_utilities::bind_udp_socket(&addr)
                .map_err(|e| LuaError::RuntimeError(e))
                // TODO: Return socket as userdata when needed
                .map(|_socket| ())
        })?;

        // Get current time in milliseconds
        let current_time = lua_clone
            .create_function(|_lua_ctx, ()| Ok(crate::os_utilities::current_time_millis()))?;

        // Parse socket address
        let parse_socket_addr = lua_clone.create_function(|_lua_ctx, addr: String| {
            crate::os_utilities::parse_socket_addr(&addr)
                .map(|socket_addr| socket_addr.to_string())
                .map_err(|e| LuaError::RuntimeError(e))
        })?;

        // Get command-line arguments
        let get_args = lua_clone.create_function(|lua_ctx, ()| {
            let args: Vec<String> = std::env::args().collect();
            let table = lua_ctx.create_table()?;
            for (i, arg) in args.iter().enumerate() {
                table.set(i + 1, arg.as_str())?;
            }
            Ok(table)
        })?;

        // Create script cache for module loading
        let script_cache = crate::script_cache::ScriptCache::new();
        let cache_for_require = script_cache.clone();

        // Synchronous require() function
        let lua_for_require = lua_clone.clone();
        let require = lua_clone.create_function(move |lua_ctx, (path, options): (String, Option<LuaTable>)| {
            // Get reload option - must check for Nil explicitly because Lua nil converts to false
            let should_reload = if let Some(ref opts) = options {
                match opts.get::<LuaValue>("reload") {
                    Ok(LuaValue::Boolean(b)) => b,
                    Ok(LuaValue::Nil) => true, // Key doesn't exist, use default
                    _ => true, // Any error or other type, use default
                }
            } else {
                true // No options table, use default
            };

            // Resolve path: try relative to current script first, then use as canonical
            // Paths are relative to assets/ (e.g., "scripts/example.lua" or "../modules/mod.lua")
            // IMPORTANT: Always use the resolved path for downloads, even if file doesn't exist locally
            let resolved_path = if let Ok(current_script) = lua_ctx.globals().get::<String>("__SCRIPT_NAME__") {
                let current_path = std::path::Path::new(&current_script);
                if let Some(parent) = current_path.parent() {
                    // Try relative to current script's directory
                    let relative = parent.join(&path);
                    // Normalize the path (resolve .. and .)
                    let normalized = normalize_path(&relative.to_string_lossy());
                    
                    // Check if relative path exists locally
                    let full_check_path = std::path::Path::new("assets").join(&normalized);
                    if full_check_path.exists() {
                        normalized
                    } else {
                        // Check if input path (canonical) exists
                        let canonical_check = std::path::Path::new("assets").join(&path);
                        if canonical_check.exists() {
                            path.clone()
                        } else {
                            // Neither exists - prefer relative path for network download
                            // This way require("foo.lua") from scripts/examples/ will download scripts/examples/foo.lua
                            normalized
                        }
                    }
                } else {
                    path.clone()
                }
            } else {
                path.clone()
            };

            // Normalize path separators to forward slashes for consistent lookups
            let resolved_path = normalize_path_separators(&resolved_path);

            // Check if module is already cached
            if let Some(cached_key) = cache_for_require.get_module(&resolved_path) {
                debug!("📦 [REQUIRE] '{}': returning cached module (Rust cache hit)", resolved_path);
                // Update dependency tracking even if cached
                if let Ok(current_script) = lua_ctx.globals().get::<String>("__SCRIPT_NAME__") {
                    let current_script = normalize_path_separators(&current_script);
                    cache_for_require.add_dependency(resolved_path.clone(), current_script, should_reload);
                }
                
                let cached_value: LuaValue = lua_for_require.registry_value(&*cached_key)?;
                return Ok(cached_value);
            }
            
            // Check network option early - we may need to check server even if file exists
            let network_enabled = options
                .as_ref()
                .and_then(|t| t.get::<bool>("network").ok())
                .unwrap_or(false);
            
            debug!("📋 [REQUIRE] '{}': network_enabled={}, should_reload={}", resolved_path, network_enabled, should_reload);
            
            // Load module source - first try local filesystem
            let load_result = crate::script_cache::load_module_source(&resolved_path);
            
            let (source, _full_path) = match load_result {
                Ok(result) => {
                    // File exists locally
                    debug!("📋 [REQUIRE] '{}': file exists locally", resolved_path);
                    
                    // If network=true AND reload=true, queue a background server check for updates
                    // Note: We use local file immediately (no blocking) because nested requires
                    // can't yield (C-call boundary). Server check happens in background.
                    // If update is found, hot reload will apply it later.
                    if network_enabled && should_reload {
                        let has_loader: bool = lua_ctx.globals()
                            .get::<LuaValue>("__NETWORK_DOWNLOAD_ENABLED__")
                            .map(|v| v != LuaNil)
                            .unwrap_or(false);
                        
                        debug!("📋 [REQUIRE] '{}': has_loader={}", resolved_path, has_loader);
                        
                        if has_loader {
                            debug!("📥 [REQUIRE] Queuing background server check for: {}", resolved_path);
                            
                            // Queue download request - server will compare hashes
                            // This is a non-blocking check - we use local file immediately
                            // and any update will be applied via hot reload
                            cache_for_require.register_pending_download_coroutine(
                                resolved_path.clone(),
                                std::sync::Arc::new(lua_ctx.create_registry_value(LuaNil)?),
                                0,
                                false, // is_binary=false for scripts
                                lua_ctx.globals().get::<String>("__SCRIPT_NAME__").ok(), // context for server
                                true, // should_subscribe - reload=true was specified
                            );
                            
                            // Don't yield - use local file immediately
                            // Background check will trigger hot reload if update is found
                        } else {
                            debug!("📋 [REQUIRE] '{}': skipping network check - no loader", resolved_path);
                        }
                    } else {
                        debug!("📋 [REQUIRE] '{}': skipping network check - network_enabled={} should_reload={}", 
                               resolved_path, network_enabled, should_reload);
                    }
                    
                    // Use local file
                    result
                },
                Err(e) => {
                    // Local file not found - check if network download is enabled
                    if network_enabled {
                        // Check if we have a network asset loader available (via global)
                        let has_loader: bool = lua_ctx.globals()
                            .get::<LuaValue>("__NETWORK_DOWNLOAD_ENABLED__")
                            .map(|v| v != LuaNil)
                            .unwrap_or(false);
                        
                        if has_loader {
                            // Queue the download path for the script cache to track
                            // We can't yield from a Rust function due to C-call boundary,
                            // so we return a special pending table that the Lua wrapper
                            // (registered in execute_script_wrapper) will detect and yield
                            debug!("📥 [REQUIRE] File not found locally, requesting download: {}", resolved_path);
                            
                            // Store the download path for execute_script_tracked to detect
                            cache_for_require.register_pending_download_coroutine(
                                resolved_path.clone(),
                                std::sync::Arc::new(lua_ctx.create_registry_value(LuaNil)?), // Placeholder - will be replaced
                                0, // Placeholder instance ID - will be set by execute_script_tracked
                                false, // is_binary=false for scripts
                                lua_ctx.globals().get::<String>("__SCRIPT_NAME__").ok(), // context for server
                                should_reload, // should_subscribe based on reload option
                            );
                            
                            // Return a special pending table that the Lua wrapper will detect
                            let pending_table = lua_ctx.create_table()?;
                            pending_table.set("__PENDING_DOWNLOAD__", true)?;
                            pending_table.set("path", resolved_path.clone())?;
                            return Ok(LuaValue::Table(pending_table));
                        } else {
                            // No network loader, just return the original error
                            return Err(LuaError::RuntimeError(e));
                        }
                    } else {
                        // Network not enabled, return original error
                        return Err(LuaError::RuntimeError(e));
                    }
                }
            };
            
            // Instead of executing the module here (inside Rust), return the source
            // to the Lua wrapper which will execute it in Lua-land (where yields work)
            
            // Track dependency for hot reload: current script depends on resolved_path
            if let Ok(current_script) = lua_ctx.globals().get::<String>("__SCRIPT_NAME__") {
                let current_script = normalize_path_separators(&current_script);
                cache_for_require.add_dependency(resolved_path.clone(), current_script, should_reload);
            }
            
            let source_table = lua_ctx.create_table()?;
            source_table.set("__SOURCE__", true)?;
            source_table.set("source", source)?;
            source_table.set("path", resolved_path.clone())?;
            source_table.set("should_reload", should_reload)?;
            
            Ok(LuaValue::Table(source_table))
        })?;

        // Asynchronous require_async() function
        let lua_for_async = lua_clone.clone();
        let cache_for_async = script_cache.clone();
        let script_instance_for_async = script_instance.clone();
        let require_async = lua_clone.create_function(move |lua_ctx, (path, callback, options): (String, LuaFunction, Option<LuaTable>)| {
            // Get reload option - must check for Nil explicitly because Lua nil converts to false
            let should_reload = if let Some(ref opts) = options {
                match opts.get::<LuaValue>("reload") {
                    Ok(LuaValue::Boolean(b)) => b,
                    Ok(LuaValue::Nil) => false, // Key doesn't exist, use default (false for async)
                    _ => false, // Any error or other type, use default
                }
            } else {
                false // No options table, use default
            };

            // Resolve path: try relative to current script first, then use as canonical
            // Paths are relative to assets/ (e.g., "scripts/example.lua" or "../modules/mod.lua")
            let resolved_path = if let Ok(current_script) = lua_ctx.globals().get::<String>("__SCRIPT_NAME__") {
                let current_path = std::path::Path::new(&current_script);
                if let Some(parent) = current_path.parent() {
                    // Try relative to current script's directory
                    let relative = parent.join(&path);
                    // Normalize the path (resolve .. and .)
                    let normalized = normalize_path(&relative.to_string_lossy());
                    
                    // Check if relative path exists locally
                    let full_check_path = std::path::Path::new("assets").join(&normalized);
                    if full_check_path.exists() {
                        normalized
                    } else {
                        // Check if input path (canonical) exists
                        let canonical_check = std::path::Path::new("assets").join(&path);
                        if canonical_check.exists() {
                            path.clone()
                        } else {
                            // Neither exists - prefer relative path for network download
                            normalized
                        }
                    }
                } else {
                    path.clone()
                }
            } else {
                path.clone()
            };

            // Normalize path separators to forward slashes for consistent lookups
            let resolved_path = normalize_path_separators(&resolved_path);

            // Get or create module instance ID for entity tracking
            // Each (path, parent) combination gets its own instance
            let current_parent_id: u64 = lua_ctx.globals().get("__INSTANCE_ID__").unwrap_or(0);
            
            let module_instance_id = if let Some(existing_id) = cache_for_async.get_module_instance(&resolved_path, current_parent_id) {
                // Reuse existing instance for this (path, parent) combination
                debug!("require_async('{}') reusing instance {} for parent {}", resolved_path, existing_id, current_parent_id);
                existing_id
            } else {
                // First time this parent is loading this module, create new instance
                let new_id = script_instance_for_async.start(resolved_path.clone());
                cache_for_async.set_module_instance(resolved_path.clone(), current_parent_id, new_id);
                debug!("require_async('{}') created NEW instance {} for parent {}", resolved_path, new_id, current_parent_id);
                new_id
            };
            
            // Always update parent-child relationship when require_async is called
            // This ensures the module instance is linked to the current parent
            if current_parent_id != 0 {
                cache_for_async.set_module_parent(module_instance_id, current_parent_id);
            }

            // If should_reload is true, register the callback for hot reload with parent context
            if should_reload {
                let callback_key = lua_for_async.create_registry_value(callback.clone())?;
                cache_for_async.register_hot_reload_callback(resolved_path.clone(), Arc::new(callback_key), current_parent_id);
            }

            // Check for network option - must check for Nil explicitly
            let network_enabled = if let Some(ref opts) = options {
                match opts.get::<LuaValue>("network") {
                    Ok(LuaValue::Boolean(b)) => b,
                    Ok(LuaValue::Nil) => false, // Key doesn't exist, use default
                    _ => false, // Any error or other type, use default
                }
            } else {
                false
            };

            // Always execute module to run side effects, even if cached
            // This ensures top-level require_async calls in modules always execute
            let load_result = crate::script_cache::load_module_source(&resolved_path);
            
            let (source, _full_path) = match load_result {
                Ok(result) => {
                    // File exists locally
                    // If network=true AND reload=true, queue a background server check for updates
                    // This is the same behavior as sync require()
                    if network_enabled && should_reload {
                        let has_loader: bool = lua_ctx.globals()
                            .get::<LuaValue>("__NETWORK_DOWNLOAD_ENABLED__")
                            .map(|v| v != LuaNil)
                            .unwrap_or(false);
                        
                        if has_loader {
                            debug!("📥 [REQUIRE_ASYNC] Queuing background server check for: {}", resolved_path);
                            
                            // Queue download request - server will compare hashes
                            // This is a non-blocking check - we use local file immediately
                            // and any update will be applied via hot reload callback
                            cache_for_async.register_pending_download_coroutine(
                                resolved_path.clone(),
                                std::sync::Arc::new(lua_ctx.create_registry_value(LuaNil)?),
                                module_instance_id,
                                false, // is_binary=false for scripts
                                lua_ctx.globals().get::<String>("__SCRIPT_NAME__").ok(), // context for server
                                should_reload, // should_subscribe based on reload option
                            );
                        }
                    }
                    
                    result
                },
                Err(e) => {
                    // File not found - check if network download is enabled
                    if network_enabled {
                        // Check if we have a network asset loader available (via global)
                        let has_loader: bool = lua_ctx.globals()
                            .get::<LuaValue>("__NETWORK_DOWNLOAD_ENABLED__")
                            .map(|v| v != LuaNil)
                            .unwrap_or(false);
                        
                        if has_loader {
                            debug!("📥 [REQUIRE_ASYNC] File not found locally, queuing download: {}", resolved_path);
                            
                            // Store the callback to be invoked when download completes
                            // We use the hot_reload_callback mechanism which will re-execute the module
                            // and invoke all registered callbacks
                            let callback_key = lua_for_async.create_registry_value(callback.clone())?;
                            cache_for_async.register_hot_reload_callback(resolved_path.clone(), Arc::new(callback_key), current_parent_id);
                            
                            // Queue the download
                            cache_for_async.register_pending_download_coroutine(
                                resolved_path.clone(),
                                std::sync::Arc::new(lua_ctx.create_registry_value(LuaNil)?),
                                module_instance_id,
                                false, // is_binary=false for scripts
                                lua_ctx.globals().get::<String>("__SCRIPT_NAME__").ok(), // context for server
                                should_reload, // should_subscribe based on reload option
                            );
                            
                            // Return without error - download is queued, callback will fire later
                            return Ok(());
                        }
                    }
                    return Err(LuaError::RuntimeError(e));
                }
            };
            
            // Save current __INSTANCE_ID__ to restore later
            let previous_instance_id: Option<u64> = lua_ctx.globals().get("__INSTANCE_ID__").ok();
            
            // Set __INSTANCE_ID__ to module's instance BEFORE executing module
            // This ensures any require_async calls within the module use correct parent
            lua_ctx.globals().set("__INSTANCE_ID__", module_instance_id)?;
            debug!("Executing module '{}' with __INSTANCE_ID__ = {}", resolved_path, module_instance_id);
            
            // Execute module
            let module_name = format!("@{}", resolved_path);
            let result = crate::script_cache::execute_module(&lua_for_async, &source, &module_name)?;
            debug!("Module '{}' executed successfully", resolved_path);
            
            // Cache the result if not already cached
            if cache_for_async.get_module(&resolved_path).is_none() {
                let registry_key = lua_for_async.create_registry_value(result.clone())?;
                cache_for_async.cache_module(resolved_path.clone(), Arc::new(registry_key));
            }
            
            // Track dependency (but don't reload script for async require, we use callback)
            if let Ok(current_script) = lua_ctx.globals().get::<String>("__SCRIPT_NAME__") {
                let current_script = normalize_path_separators(&current_script);
                cache_for_async.add_dependency(resolved_path.clone(), current_script.clone(), false);
                // Also track as async dependency so this module gets invalidated when resolved_path changes
                cache_for_async.add_async_dependency(resolved_path.clone(), current_script);
            }
            
            // __INSTANCE_ID__ is already set to module's instance from above
            // Call the callback with the loaded module
            let callback_result = callback.call::<()>(result);
            
            // Restore previous __INSTANCE_ID__
            if let Some(prev_id) = previous_instance_id {
                lua_ctx.globals().set("__INSTANCE_ID__", prev_id)?;
            } else {
                lua_ctx.globals().set("__INSTANCE_ID__", LuaNil)?;
            }
            
            callback_result?;
            
            Ok(())
        })?;

        // Inject into globals
        lua_clone.globals().set("spawn", spawn)?;
        lua_clone
            .globals()
            .set("spawn_with_parent", spawn_with_parent)?;

        // Create despawn function
        // Entity ID may be a temp_id from spawn() or a real entity ID from query()
        // Use resolve_entity to handle both cases - it looks up temp_id->entity mapping
        // and falls back to from_bits() for real entity IDs
        let queue_for_despawn = queue.clone();
        let despawn = lua_clone.create_function(move |_lua_ctx, entity_id: u64| {
            // Resolve temp_id to real entity, or convert bits directly if it's a real ID
            let entity = queue_for_despawn.resolve_entity(entity_id);
            despawn_queue.queue_despawn(entity);
            Ok(())
        })?;
        lua_clone.globals().set("despawn", despawn)?;

        lua_clone
            .globals()
            .set("insert_resource", insert_resource)?;
        lua_clone
            .globals()
            .set("register_system", register_system)?;
        lua_clone.globals().set("copy_file", copy_file)?;
        lua_clone
            .globals()
            .set("read_file_bytes", read_file_bytes)?;
        lua_clone
            .globals()
            .set("write_file_bytes", write_file_bytes)?;
        lua_clone
            .globals()
            .set("create_directory", create_directory)?;

        // OS utilities
        lua_clone
            .globals()
            .set("bind_udp_socket", bind_udp_socket)?;
        lua_clone.globals().set("current_time", current_time)?;
        lua_clone
            .globals()
            .set("parse_socket_addr", parse_socket_addr)?;
        lua_clone.globals().set("get_args", get_args)?;

        // Script importing
        lua_clone.globals().set("require", require)?;
        lua_clone.globals().set("require_async", require_async)?;

        // Note: load_asset will be added via add_asset_loading_to_lua()
        // Note: query_resource will be added to world table in lua_systems

        Ok(Self {
            lua: lua_clone,
            script_cache,
            script_instance,
        })
    }

    /// Execute a Lua script from a string
    pub fn execute_script_untracked(
        &self,
        script_content: &str,
        script_name: &str,
    ) -> Result<(), LuaError> {
        self.lua.load(script_content).set_name(script_name).exec()?;
        Ok(())
    }

    /// Execute a script with automatic script ownership tracking
    /// Entities spawned during execution will be tagged with a unique instance ID
    /// Returns the instance ID for this execution
    ///
    /// Scripts are run inside a Lua coroutine so they can yield when waiting
    /// for network downloads (via require with {network=true}).
    pub fn execute_script_tracked(
        &self,
        script_content: &str,
        script_name: &str,
        script_instance: &crate::script_entities::ScriptInstance,
    ) -> Result<u64, LuaError> {
        let instance_id = script_instance.start(script_name.to_string());

        // Set both instance ID and script name as Lua globals
        self.lua.globals().set("__INSTANCE_ID__", instance_id)?;
        self.lua.globals().set("__SCRIPT_NAME__", script_name)?;

        // Register a wrapped require that checks for __PENDING_DOWNLOAD__ and yields
        // This wrapper is called by scripts, it calls the real _require_internal, checks result
        self.lua.load(r#"
            -- Store originals in special globals the FIRST time only
            -- This prevents re-capturing the wrapper on hot reload
            if not __RUST_REQUIRE__ then
                __RUST_REQUIRE__ = require
                __RUST_REQUIRE_ASYNC__ = require_async
                __RUST_LOAD_ASSET__ = load_asset
            end
            local _orig_require = __RUST_REQUIRE__
            local _orig_require_async = __RUST_REQUIRE_ASYNC__
            local _orig_load_asset = __RUST_LOAD_ASSET__
            
            -- Fresh module cache for this execution (NOT persisted across reloads)
            local _module_cache = {}
            
            -- Override require function to handle pending downloads AND source execution
            require = function(path, opts)
                -- print("[LUA REQUIRE] Called for: " .. tostring(path))
                
                -- Check module cache first
                local cache_key = path
                if _module_cache[cache_key] ~= nil then
                    -- print("[LUA REQUIRE] Returning from Lua cache: " .. tostring(path))
                    return _module_cache[cache_key]
                end
                
                -- print("[LUA REQUIRE] No Lua cache, calling Rust require...")
                local result = _orig_require(path, opts)
                -- print("[LUA REQUIRE] Rust returned type: " .. type(result))
                
                -- Check if the result is a pending download marker
                if type(result) == "table" and result.__PENDING_DOWNLOAD__ then
                    -- print("[LUA REQUIRE] Got __PENDING_DOWNLOAD__, yielding for: " .. tostring(result.path))
                    -- Yield the coroutine with the path - Bevy will resume after download
                    local download_path = result.path
                    coroutine.yield(download_path)
                    
                    -- When resumed, try require again (file should now be available)
                    result = _orig_require(path, opts)
                    
                    -- If still pending, something is wrong
                    if type(result) == "table" and result.__PENDING_DOWNLOAD__ then
                        error("Download failed or file still not available: " .. tostring(download_path))
                    end
                end
                
                -- Check if the result is source code to execute
                if type(result) == "table" and result.__SOURCE__ then
                    -- print("[LUA REQUIRE] Got __SOURCE__ for: " .. tostring(result.path) .. ", executing...")
                    local source_code = result.source
                    local module_path = result.path
                    
                    -- Execute the source code in Lua-land (where yields work!)
                    local chunk, err = load(source_code, "@" .. module_path)
                    if not chunk then
                        error("Failed to load module '" .. module_path .. "': " .. tostring(err))
                    end
                    
                    -- Execute the chunk directly (NOT with pcall!) so yields propagate
                    -- pcall would catch the yield as an error, breaking coroutine-based downloads
                    local module_result = chunk()
                    
                    -- print("[LUA REQUIRE] Successfully executed: " .. tostring(module_path))
                    -- Cache the result
                    _module_cache[cache_key] = module_result
                    return module_result
                end
                
                -- print("[LUA REQUIRE] Got cached value from Rust for: " .. tostring(path))
                -- Already a cached value or other result
                _module_cache[cache_key] = result
                return result
            end
            
            -- Override require_async function to handle pending downloads
            -- NOTE: require_async uses callbacks, so we don't yield - we just let
            -- the callback execute after the download completes. The coroutine
            -- mechanism is only for synchronous require().
            require_async = function(path, callback, opts)
                -- Just call the original - it handles downloads internally via callback
                return _orig_require_async(path, callback, opts)
            end
            
            -- Override load_asset function to handle pending downloads
            -- Works exactly like require() - yields if download needed
            load_asset = function(path, opts)
                local result = _orig_load_asset(path, opts)
                
                -- Check if the result is a pending download marker
                if type(result) == "table" and result.__PENDING_DOWNLOAD__ then
                    -- Yield the coroutine with the path - Bevy will resume after download
                    local download_path = result.path
                    coroutine.yield(download_path)
                    
                    -- When resumed, try load_asset again (file should now be downloaded)
                    result = _orig_load_asset(path, opts)
                    
                    -- If still pending, something is wrong
                    if type(result) == "table" and result.__PENDING_DOWNLOAD__ then
                        error("Asset download failed or file still not available: " .. tostring(download_path))
                    end
                end
                
                -- Extract asset_id from result table if present
                if type(result) == "table" and result.asset_id then
                    return result.asset_id
                end
                
                return result
            end
        "#).exec()?;

        // Load script as a function
        let script_fn = self
            .lua
            .load(script_content)
            .set_name(script_name)
            .into_function()?;

        // Create a coroutine to run the script
        let coroutine = self.lua.create_thread(script_fn)?;

        // Resume the coroutine (execute until completion or yield)
        match coroutine.resume::<mlua::Value>(()) {
            Ok(yield_value) => {
                // Check if coroutine completed or yielded
                match coroutine.status() {
                    mlua::ThreadStatus::Finished => {
                        // Script completed normally
                        debug!(
                            "Script '{}' completed with instance ID: {}",
                            script_name, instance_id
                        );
                    }
                    mlua::ThreadStatus::Resumable => {
                        // Script yielded - it needs a download
                        // The yield value should be the download path
                        if let mlua::Value::String(path_str) = yield_value {
                            let path = path_str.to_str()?.to_string();

                            debug!("📥 Script '{}' needs download: {}", script_name, path);

                            // Store the coroutine for later resumption when download completes
                            let coroutine_key =
                                std::sync::Arc::new(self.lua.create_registry_value(coroutine)?);

                            // Register for resumption after this path is downloaded
                            self.script_cache.register_pending_download_coroutine(
                                path.clone(),
                                coroutine_key,
                                instance_id,
                                false, // is_binary=false for scripts
                                self.lua.globals().get::<String>("__SCRIPT_NAME__").ok(), // context for server
                                false, // should_subscribe=false for internal resumption (subscription handled at require level)
                            );

                            debug!(
                                "📋 Registered script '{}' for resumption after download of '{}'",
                                script_name, path
                            );
                        } else {
                            warn!(
                                "⚠️ Script '{}' yielded with unexpected value: {:?}",
                                script_name, yield_value
                            );
                        }
                    }
                    mlua::ThreadStatus::Error => {
                        warn!("⚠️ Script '{}' coroutine is in error state", script_name);
                    }
                    mlua::ThreadStatus::Running => {
                        warn!(
                            "⚠️ Script '{}' coroutine unexpectedly still running",
                            script_name
                        );
                    }
                }
            }
            Err(e) => {
                // Script execution failed - propagate error
                return Err(e);
            }
        }

        // Note: We DON'T clear script_instance here so entities spawned via queues get tagged

        Ok(instance_id)
    }

    /// Execute a script with automatic script ownership tracking AND register it in ScriptRegistry
    /// This enables automatic reload on file changes
    pub fn execute_script(
        &self,
        script_content: &str,
        script_name: &str,
        script_path: std::path::PathBuf,
        script_instance: &crate::script_entities::ScriptInstance,
        script_registry: &crate::script_registry::ScriptRegistry,
    ) -> Result<u64, LuaError> {
        // Derive the proper module path from script_path by stripping "assets/" prefix
        // This ensures __SCRIPT_NAME__ has the correct path for relative require() resolution
        // e.g., "assets/scripts/examples/foo.lua" -> "scripts/examples/foo.lua"
        let module_path = to_forward_slash(&script_path);
        let module_name = module_path.strip_prefix("assets/").unwrap_or(&module_path);

        let instance_id =
            self.execute_script_tracked(script_content, module_name, script_instance)?;

        // Register in script registry for auto-reload
        script_registry.register_script(script_path.to_path_buf(), instance_id, script_content.to_string());

        Ok(instance_id)
    }

    /// Execute a script with automatic cleanup and tracking
    /// This despawns all entities from the previous instance before running the script again
    pub fn reload_script(
        &self,
        script_content: &str,
        script_name: &str,
        world: &mut bevy::prelude::World,
        instance_id: u64,
    ) -> Result<u64, LuaError> {
        // Despawn all entities from previous instance
        crate::script_entities::despawn_instance_entities(world, instance_id);

        // Get script instance resource
        let script_instance = world
            .resource::<crate::script_entities::ScriptInstance>()
            .clone();

        // Execute with tracking (creates new instance ID)
        self.execute_script_tracked(script_content, script_name, &script_instance)
    }
}

/// Plugin that sets up Lua scripting with component-based spawn function
///
/// This plugin automatically initializes all required resources and systems:
/// - ComponentRegistry (from AppTypeRegistry)
/// - SpawnQueue, ComponentUpdateQueue, ResourceQueue
/// - ResourceBuilderRegistry, SerdeComponentRegistry
/// - All processing systems (spawn, updates, resources, assets)
///
/// Users only need to add this plugin after DefaultPlugins (or other plugins that register types).
pub struct LuaSpawnPlugin;

impl Plugin for LuaSpawnPlugin {
    fn build(&self, app: &mut App) {
        // Initialize all required resources
        // Note: ComponentRegistry needs AppTypeRegistry, so we create it in a startup system
        app.init_resource::<SpawnQueue>();
        app.init_resource::<crate::despawn_queue::DespawnQueue>();
        app.init_resource::<crate::component_update_queue::ComponentUpdateQueue>();
        app.init_resource::<crate::resource_queue::ResourceQueue>();
        app.init_resource::<crate::resource_builder::ResourceBuilderRegistry>();
        app.init_resource::<crate::serde_components::SerdeComponentRegistry>();
        app.init_resource::<crate::resource_lua_trait::LuaResourceRegistry>();
        app.init_resource::<crate::component_lua_trait::LuaComponentRegistry>();
        app.init_resource::<crate::script_entities::ScriptInstance>();
        app.init_resource::<crate::script_registry::ScriptRegistry>();
        app.init_resource::<crate::lua_observers::LuaObserverRegistry>();

        // Add file watcher plugin for auto-reload
        app.add_plugins(crate::lua_file_watcher::LuaFileWatcherPlugin);

        // Add event/message sender plugin for Lua event and message dispatch
        app.add_plugins(crate::event_sender::LuaEventSenderPlugin);

        // Register auto-generated resource method bindings
        // This must happen after LuaResourceRegistry is initialized
        app.add_systems(PreStartup, register_resource_methods);

        // Note: EventReaderRegistry is no longer needed!
        // Event reading is now fully generic via reflection in world:read_events()
        //
        // IMPORTANT: Events must be registered BEFORE Replicon plugins!
        // Users should call register_common_bevy_events() explicitly before
        // adding RepliconPlugins to ensure consistent registration order.

        // Add all required systems
        // Initialize ComponentRegistry in PreStartup to ensure it exists before setup_lua_context
        app.add_systems(PreStartup, initialize_component_registry);
        app.add_systems(Startup, setup_lua_context);
        app.add_systems(PostStartup, log_available_events);
        app.add_systems(
            Update,
            (
                // Auto-reload must run first to queue despawns/spawns before processing
                auto_reload_changed_scripts,
                // Despawn old entities first (critical for hot-reload)
                crate::despawn_queue::process_despawn_queue.after(auto_reload_changed_scripts),
                // Then create new assets
                crate::asset_loading::process_pending_assets
                    .after(crate::despawn_queue::process_despawn_queue),
                // Then spawn new entities with those assets
                crate::entity_spawner::process_spawn_queue
                    .after(crate::asset_loading::process_pending_assets),
                // Process observer registrations after entities are spawned
                crate::lua_observers::process_observer_registrations
                    .after(crate::entity_spawner::process_spawn_queue),
                // Attach observers to entities that have callbacks
                crate::lua_observers::attach_lua_observers
                    .after(crate::lua_observers::process_observer_registrations),
                // Finally apply component updates
                crate::component_updater::process_component_updates
                    .after(crate::lua_observers::attach_lua_observers),
                crate::lua_systems::run_lua_systems,
                crate::component_updater::process_component_updates,
            ),
        );
        app.add_systems(Update, (crate::resource_inserter::process_resource_queue,));
    }
}

/// System to register auto-generated resource method bindings
/// This runs in PreStartup to ensure methods are available before Lua scripts execute
fn register_resource_methods(
    lua_resource_registry: Res<crate::resource_lua_trait::LuaResourceRegistry>,
) {
    crate::auto_bindings::register_auto_bindings(&lua_resource_registry);
}

/// System to initialize ComponentRegistry from AppTypeRegistry
/// This runs before setup_lua_context so the registry is available
fn initialize_component_registry(mut commands: Commands, type_registry: Res<AppTypeRegistry>) {
    let component_registry =
        crate::components::ComponentRegistry::from_type_registry(type_registry.clone());
    commands.insert_resource(component_registry);
}

/// System to log which events are available for Lua
fn log_available_events(type_registry: Res<AppTypeRegistry>) {
    let registry = type_registry.read();
    let mut count = 0;

    for registration in registry.iter() {
        let type_path = registration.type_info().type_path();
        if type_path.starts_with("bevy_ecs::event::collections::Events<") {
            if let Some(inner) = type_path.strip_prefix("bevy_ecs::event::collections::Events<") {
                if let Some(event_type) = inner.strip_suffix(">") {
                    count += 1;
                    if count == 1 {
                        debug!("Events available in Lua via world:read_events():");
                    }
                    debug!("  ✓ {}", event_type);
                }
            }
        }
    }

    if count == 0 {
        warn!("No Events<T> registered. Use register_common_bevy_events() or register_lua_events! macro to enable event reading.");
    }
}

/// System to initialize Lua context
fn setup_lua_context(
    mut commands: Commands,
    queue: Res<SpawnQueue>,
    despawn_queue: Res<crate::despawn_queue::DespawnQueue>,
    resource_queue: Res<crate::resource_queue::ResourceQueue>,
    update_queue: Res<crate::component_update_queue::ComponentUpdateQueue>,
    builder_registry: Res<crate::resource_builder::ResourceBuilderRegistry>,
    asset_server: Res<AssetServer>,
    mut component_registry: ResMut<crate::components::ComponentRegistry>,
    type_registry: Res<AppTypeRegistry>,
    script_instance: Res<crate::script_entities::ScriptInstance>,
) {
    let system_registry = LuaSystemRegistry::default();

    // Create AssetRegistry with handle setters for all asset types
    let mut asset_registry = crate::asset_loading::AssetRegistry::from_type_registry(&type_registry);

    // Set AssetServer for typed path loading in set_field_from_lua
    asset_registry.set_asset_server(asset_server.clone());

    // Auto-discover asset types from TypeRegistry (supplements build-time config)
    asset_registry.discover_and_register_handle_creators(&type_registry);
    // NOTE: typed_path_loaders are registered via register_auto_typed_path_loaders() in LuaBindingsPlugin
    // which uses compile-time discovered types for proper Handle<T> TypeId matching

    // Update ComponentRegistry with AssetRegistry reference
    component_registry.set_asset_registry(asset_registry.clone());

    match LuaScriptContext::new(
        queue.clone(),
        despawn_queue.clone(),
        resource_queue.clone(),
        update_queue.clone(),
        system_registry.clone(),
        builder_registry.clone(),
        script_instance.clone(),
    ) {
        Ok(ctx) => {
            // Add asset loading to Lua
            if let Err(e) = crate::asset_loading::add_asset_loading_to_lua(
                &ctx,
                asset_server.clone(),
                asset_registry.clone(),
            ) {
                error!("Failed to add asset loading to Lua: {}", e);
            }

            commands.insert_resource(ctx);
            commands.insert_resource(system_registry);
            commands.insert_resource(asset_registry);
        }
        Err(e) => {
            error!("Failed to initialize Lua context: {}", e);
        }
    }
}

/// System that automatically reloads scripts when file changes are detected
fn auto_reload_changed_scripts(
    mut events: MessageReader<crate::lua_file_watcher::LuaFileChangeEvent>,
    script_registry: Res<crate::script_registry::ScriptRegistry>,
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<crate::script_entities::ScriptInstance>,
    world: &World,
) {
    for event in events.read() {
        // Normalize path to forward slashes for consistent comparison
        let event_path_str = to_forward_slash(&event.path);
        debug!(
            "🔄 [HOT_RELOAD] File change event received: {} (normalized: {})",
            event.path.display(),
            event_path_str
        );

        let mut reloaded_paths = std::collections::HashSet::new();

        // Check if this is a module file (.lua file in assets/)
        // Invalidate module cache if it's a module
        // Support scripts in any location under assets/, not just assets/scripts/
        debug!(
            "🔍 [HOT_RELOAD] Checking path: starts_with('assets/')={}, ends_with('.lua')={}",
            event_path_str.starts_with("assets/"),
            event_path_str.ends_with(".lua")
        );
        if event_path_str.starts_with("assets/") && event_path_str.ends_with(".lua") {
            // Get relative path from assets/ (keeps full subpath like "scripts/examples/foo.lua")
            if let Some(module_path_str) = event_path_str.strip_prefix("assets/") {
                // Convert to owned String for use in cache operations
                let module_path = module_path_str.to_string();

                // Invalidate the module cache and get all dependent scripts
                debug!(
                    "🔄 [HOT_RELOAD] Module path: '{}', invalidating cache...",
                    module_path
                );
                let invalidated = lua_ctx.script_cache.invalidate_module(&module_path);
                debug!("🔄 [HOT_RELOAD] Invalidated modules: {:?}", invalidated);

                if !invalidated.is_empty() {
                    debug!("Invalidated module cache for: {:?}", invalidated);

                    // Update source cache for the changed module
                    // This ensures hot reload uses the new source
                    if let Ok((new_source, _)) =
                        lua_ctx.script_cache.load_module_source(&module_path)
                    {
                        // load_module_source already updated the cache, but we force it here to be explicit
                        lua_ctx.script_cache.update_source(&module_path, new_source);
                    }

                    // For each invalidated module, reload scripts that depend on it
                    for invalidated_path in &invalidated {
                        // Find and reload any main scripts that imported this module
                        // We need to construct the full path for the script registry lookup
                        // invalidated_path includes the scripts/ prefix (e.g., "scripts/examples/foo.lua")
                        let full_path = std::path::Path::new("assets").join(invalidated_path);

                        // Get active instances for this dependent script
                        let instances = script_registry.get_active_instances(&full_path);

                        if !instances.is_empty() {
                            debug!(
                                "Reloading {} dependent instance(s) of '{}'",
                                instances.len(),
                                invalidated_path
                            );

                            // Read script content
                            let script_content = match std::fs::read_to_string(&full_path) {
                                Ok(c) => c,
                                Err(e) => {
                                    error!(
                                        "Failed to read dependent script {:?}: {}",
                                        full_path, e
                                    );
                                    continue;
                                }
                            };

                            for (instance_id, _old_content) in instances {
                                cleanup_script_instance(instance_id, world, true); // Recursive: main script reload

                                // Clear the module cache for all dependencies of this script
                                // This ensures nested require() calls see fresh versions
                                lua_ctx
                                    .script_cache
                                    .clear_dependency_caches(invalidated_path);

                                match lua_ctx.execute_script_tracked(
                                    &script_content,
                                    invalidated_path,
                                    &script_instance,
                                ) {
                                    Ok(new_id) => {
                                        debug!(
                                            "✓ Reloaded dependent instance {} -> {} for '{}'",
                                            instance_id, new_id, invalidated_path
                                        );
                                        script_registry.register_script(
                                            full_path.clone(),
                                            new_id,
                                            script_content.clone(),
                                        );
                                        script_registry.remove_instance(instance_id);
                                    }
                                    Err(e) => error!(
                                        "Failed to reload dependent script '{}': {}",
                                        invalidated_path, e
                                    ),
                                }
                            }
                            reloaded_paths.insert(full_path);
                        }
                    }
                }

                // Trigger hot reload callbacks
                let callbacks = lua_ctx.script_cache.get_hot_reload_callbacks(&module_path);
                if !callbacks.is_empty() {
                    debug!(
                        "Triggering {} hot reload callbacks for '{}'",
                        callbacks.len(),
                        module_path
                    );

                    // Get ALL instance IDs for this module (one per parent that required it)
                    let old_instance_ids =
                        lua_ctx.script_cache.get_all_module_instances(&module_path);

                    // For each module instance, also get all its descendants (nested requires)
                    // This ensures we clean up entities spawned in nested callbacks
                    let mut all_instances_to_cleanup = HashSet::new();
                    for old_id in &old_instance_ids {
                        all_instances_to_cleanup.insert(*old_id);
                        // Add all descendants (modules loaded within this instance's callbacks)
                        let descendants =
                            lua_ctx.script_cache.get_all_descendant_instances(*old_id);
                        all_instances_to_cleanup.extend(descendants);
                    }

                    // Clean up entities from ALL previous module instances and their descendants
                    if !all_instances_to_cleanup.is_empty() {
                        debug!("Hot reload: Cleaning up {} total instance(s) for '{}' (including descendants): {:?}", 
                            all_instances_to_cleanup.len(), module_path, all_instances_to_cleanup);
                        for old_id in &all_instances_to_cleanup {
                            debug!(
                                "Hot reload: Cleaning up instance {} for '{}' or its descendants",
                                old_id, module_path
                            );
                            cleanup_script_instance(*old_id, world, false); // Non-recursive: we collected all descendants
                        }
                    } else {
                        warn!(
                            "Hot reload: No module instances found for '{}'",
                            module_path
                        );
                    }

                    // Clear all instance mappings for this module (they'll be recreated when we re-execute)
                    lua_ctx.script_cache.clear_module_instances(&module_path);

                    // Load the module source (uses cache if unchanged, disk if new)
                    if let Ok((source, _)) = lua_ctx.script_cache.load_module_source(&module_path) {
                        // Note: source cache was already updated by load_module_source if it read from disk
                        let module_name = format!("@{}", module_path);

                        // Execute module and call callbacks - each with its parent's __INSTANCE_ID__ context
                        // We need to execute the module separately for each parent because module execution
                        // has side effects (nested require_async calls) that depend on __INSTANCE_ID__
                        for (callback_key, parent_instance_id) in callbacks {
                            // Check if parent instance was cleaned up (it might be in all_instances_to_cleanup)
                            // If so, skip this callback to avoid creating instances with stale parents
                            if all_instances_to_cleanup.contains(&parent_instance_id) {
                                debug!(
                                    "Hot reload: Skipping callback for parent {} (was cleaned up)",
                                    parent_instance_id
                                );
                                continue;
                            }

                            if let Ok(callback) =
                                lua_ctx.lua.registry_value::<LuaFunction>(&*callback_key)
                            {
                                // Save current __INSTANCE_ID__
                                let previous_instance_id: Option<u64> =
                                    lua_ctx.lua.globals().get("__INSTANCE_ID__").ok();

                                // Create NEW module instance for this (module, parent) combination
                                let new_module_instance_id =
                                    script_instance.start(module_path.clone());
                                lua_ctx.script_cache.set_module_instance(
                                    module_path.clone(),
                                    parent_instance_id,
                                    new_module_instance_id,
                                );
                                lua_ctx
                                    .script_cache
                                    .set_module_parent(new_module_instance_id, parent_instance_id);
                                debug!(
                                    "Hot reload: Created new instance {} for '{}' with parent {}",
                                    new_module_instance_id, module_path, parent_instance_id
                                );

                                // Set __INSTANCE_ID__ to the MODULE's instance before executing
                                // This ensures nested require_async calls use correct parent
                                if let Err(e) = lua_ctx
                                    .lua
                                    .globals()
                                    .set("__INSTANCE_ID__", new_module_instance_id)
                                {
                                    error!("Failed to set module __INSTANCE_ID__: {}", e);
                                    continue;
                                }

                                // Execute the module with this instance context
                                match crate::script_cache::execute_module(
                                    &lua_ctx.lua,
                                    &source,
                                    &module_name,
                                ) {
                                    Ok(result) => {
                                        // Cache the result (will overwrite previous, but they should be equivalent)
                                        if let Ok(registry_key) =
                                            lua_ctx.lua.create_registry_value(result.clone())
                                        {
                                            lua_ctx.script_cache.cache_module(
                                                module_path.clone(),
                                                Arc::new(registry_key),
                                            );
                                        }

                                        // Execute callback - entities spawned will be tagged with module's instance
                                        if let Err(e) = callback.call::<()>(result) {
                                            error!(
                                                "Error in hot reload callback for '{}': {}",
                                                module_path, e
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        error!(
                                            "Failed to execute module '{}' during hot reload: {}",
                                            module_path, e
                                        );
                                    }
                                }

                                // Restore previous __INSTANCE_ID__
                                if let Some(prev_id) = previous_instance_id {
                                    let _ = lua_ctx.lua.globals().set("__INSTANCE_ID__", prev_id);
                                } else {
                                    let _ = lua_ctx.lua.globals().set("__INSTANCE_ID__", mlua::Nil);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check if we already reloaded this file as a dependent
        if reloaded_paths.contains(&event.path) {
            continue;
        }

        // Get all active instances of this script
        let instances = script_registry.get_active_instances(&event.path);

        if instances.is_empty() {
            debug!("No active instances found for {:?}", event.path);
            continue;
        }

        // Read the new script content
        let script_content = match std::fs::read_to_string(&event.path) {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read script file {:?}: {}", event.path, e);
                continue;
            }
        };

        let script_name = event
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.lua");

        debug!(
            "Reloading {} active instance(s) of '{}'",
            instances.len(),
            script_name
        );

        for (instance_id, _old_content) in instances {
            // Cleanup the instance
            cleanup_script_instance(instance_id, world, true); // Recursive: main script reload

            // Clear the module cache for all dependencies of this script
            // Use normalized path for module cache operations
            if let Some(module_path) = event_path_str.strip_prefix("assets/") {
                lua_ctx.script_cache.clear_dependency_caches(module_path);
            }

            // Re-execute the script with the same instance tracking
            match lua_ctx.execute_script_tracked(&script_content, script_name, &script_instance) {
                Ok(new_instance_id) => {
                    debug!(
                        "✓ Reloaded instance {} -> {} for '{}'",
                        instance_id, new_instance_id, script_name
                    );

                    // Register the new instance in the registry
                    script_registry.register_script(
                        event.path.clone(),
                        new_instance_id,
                        script_content.clone(),
                    );

                    // Remove the old instance from the registry
                    script_registry.remove_instance(instance_id);
                }
                Err(e) => {
                    error!("Failed to reload script '{}': {}", script_name, e);
                }
            }
        }
    }
}

/// Helper function to clean up a script instance (despawn entities, remove resources, clear systems)
/// If recursive is true, also cleans up all child module instances (for main script reload)
/// If recursive is false, only cleans up this specific instance (caller handles tree traversal)
fn cleanup_script_instance(instance_id: u64, world: &World, recursive: bool) {
    // SAFETY: We need mutable access to cleanup. This is safe because we're in an exclusive system.
    #[allow(invalid_reference_casting)]
    let world_mut = unsafe { &mut *(world as *const World as *mut World) };

    // If recursive, clean up all child module instances first
    if recursive {
        let child_instances = if let Some(lua_ctx) = world.get_resource::<LuaScriptContext>() {
            lua_ctx.script_cache.get_child_instances(instance_id)
        } else {
            Vec::new()
        };

        for child_id in child_instances {
            cleanup_script_instance(child_id, world, true);
        }
    }

    // 1. Get list of entities to be despawned BEFORE despawning them
    let entities_to_despawn = {
        let mut entities = Vec::new();
        // Query using world_mut (which is already created)
        let mut query_state = world_mut.query::<(Entity, &crate::script_entities::ScriptOwned)>();
        for (entity, script_owned) in query_state.iter(world_mut) {
            if script_owned.instance_id == instance_id {
                entities.push(entity);
            }
        }
        entities
    };

    // 2. Clear pending component updates for these entities
    if !entities_to_despawn.is_empty() {
        let component_update_queue = world
            .resource::<crate::component_update_queue::ComponentUpdateQueue>()
            .clone();
        let cleared_keys = component_update_queue.clear_for_entities(&entities_to_despawn);

        let num_cleared = cleared_keys.len();

        // Clean up the Lua registry keys to prevent memory leaks
        if let Some(lua_ctx) = world.get_resource::<LuaScriptContext>() {
            for key in cleared_keys {
                let _ = lua_ctx.lua.remove_registry_value(key);
            }
        }

        debug!(
            "Cleared {} pending component updates for {} entities",
            num_cleared,
            entities_to_despawn.len()
        );
    }

    // 3. Clear all systems registered by this instance
    let system_registry = world.resource::<LuaSystemRegistry>().clone();
    system_registry.clear_instance_systems(instance_id);

    // 4. Remove all resources inserted by this instance
    let resource_queue = world
        .resource::<crate::resource_queue::ResourceQueue>()
        .clone();
    let resources_to_clear = resource_queue.get_instance_resources(instance_id);

    if !resources_to_clear.is_empty() {
        let serde_registry = world
            .resource::<crate::serde_components::SerdeComponentRegistry>()
            .clone();
        let builder_registry = world
            .resource::<crate::resource_builder::ResourceBuilderRegistry>()
            .clone();

        for resource_name in &resources_to_clear {
            if !builder_registry.try_remove(resource_name, world_mut) {
                serde_registry.try_remove_resource(resource_name, world_mut);
            }
        }
    }

    resource_queue.clear_instance_tracking(instance_id);

    // 5. Remove hot reload callbacks registered by this instance
    // This prevents callback accumulation when scripts are reloaded
    if let Some(lua_ctx) = world.get_resource::<LuaScriptContext>() {
        lua_ctx
            .script_cache
            .remove_callbacks_for_instance(instance_id);
    }

    // 5b. Clear asset dependencies for this instance
    // This prevents stale references when reloading scripts that use assets
    if let Some(lua_ctx) = world.get_resource::<LuaScriptContext>() {
        lua_ctx.script_cache.clear_asset_dependencies(instance_id);
    }

    // 6. Unsubscribe from file sync for this instance
    // NOTE: Commenting out automatic unsubscribe - subscriptions should persist for hot reload
    // even after the script finishes executing. The subscription is for the file path, not the instance.
    // When the file changes, we want to trigger a hot reload which will re-execute the script.
    // Unsubscribing should only happen when the app exits or explicitly unsubscribes.
    //
    // if let Some(lua_ctx) = world.get_resource::<LuaScriptContext>() {
    //     let empty_paths = lua_ctx.script_cache.unsubscribe_all(instance_id);
    //     // Queue the unsubscription event for the networking layer to send to server
    //     lua_ctx.script_cache.queue_unsubscription(instance_id, empty_paths);
    // }

    // 7. Despawn all entities owned by this instance
    crate::script_entities::despawn_instance_entities(world_mut, instance_id);
}
