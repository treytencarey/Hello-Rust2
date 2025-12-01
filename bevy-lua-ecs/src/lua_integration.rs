use bevy::prelude::*;
use mlua::prelude::*;
use crate::spawn_queue::SpawnQueue;
use crate::lua_systems::LuaSystemRegistry;
use std::sync::Arc;

/// Resource that holds the Lua context
#[derive(Resource, Clone)]
pub struct LuaScriptContext {
    pub lua: Arc<Lua>,
}
impl LuaScriptContext {
    /// Create a new Lua context with component-based spawn function
    pub fn new(
        queue: SpawnQueue,
        despawn_queue: crate::despawn_queue::DespawnQueue,
        resource_queue: crate::resource_queue::ResourceQueue,
        system_registry: LuaSystemRegistry,
        _builder_registry: crate::resource_builder::ResourceBuilderRegistry,
    ) -> Result<Self, LuaError> {
        let lua = Lua::new();
        
        // Clone what we need for the closure
        let queue_clone = queue.clone();
        let resource_queue_clone = resource_queue.clone();
        let lua_clone = Arc::new(lua);
        let lua_for_closure = lua_clone.clone();
        let lua_for_resource = lua_clone.clone();
        
        // Track entity ID counter for immediate return (will be synced in process_spawn_queue)
        let entity_counter = Arc::new(std::sync::atomic::AtomicU32::new(0));
        
        // Create component-based spawn function that returns an entity ID
        let counter_clone = entity_counter.clone();
        let spawn = lua_clone.create_function(move |_lua_ctx, components: LuaTable| {
            let mut all_components = Vec::new();
            
            // Iterate over components table
            for pair in components.pairs::<String, LuaValue>() {
                let (component_name, component_value) = pair?;
                
                // Store everything as registry value
                let registry_key = lua_for_closure.create_registry_value(component_value)?;
                all_components.push((component_name, registry_key));
            }
            
            // Queue spawn
            queue_clone.clone().queue_spawn(all_components, Vec::new());
            
            // Generate a temporary entity ID (will be replaced with real ID in spawner)
            // For now we use a counter, but this won't match real entity IDs
            let temp_id = counter_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok(temp_id as u64)
        })?;

        // Create spawn_with_parent function
        let queue_for_parent = queue.clone();
        let lua_for_parent = lua_clone.clone();
        let counter_for_parent = entity_counter.clone();
        let spawn_with_parent = lua_clone.create_function(move |_lua_ctx, (parent_id, components): (u64, LuaTable)| {
            let mut all_components = Vec::new();
            
            // Iterate over components table
            for pair in components.pairs::<String, LuaValue>() {
                let (component_name, component_value) = pair?;
                
                // Store everything as registry value
                let registry_key = lua_for_parent.create_registry_value(component_value)?;
                all_components.push((component_name, registry_key));
            }
            
            // Convert u64 to Entity
            let parent = Entity::from_raw_u32(parent_id as u32)
                .ok_or_else(|| LuaError::RuntimeError("Invalid entity ID".to_string()))?;
            
            // Queue spawn with parent
            queue_for_parent.clone().queue_spawn_with_parent(parent, all_components, Vec::new());
            
            // Return temporary ID
            let temp_id = counter_for_parent.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok(temp_id as u64)
        })?;


        
        // Create generic insert_resource function
        // Accepts either a table (for serde resources) or UserData (for builder-created resources)
        let insert_resource = lua_clone.create_function(move |lua_ctx, (resource_name, resource_data): (String, LuaValue)| {
            // Get the current instance ID from globals
            let instance_id: Option<u64> = lua_ctx.globals().get("__INSTANCE_ID__").ok();
            
            // Store resource data as registry value (works for both tables and UserData)
            let registry_key = lua_for_resource.create_registry_value(resource_data)?;
            resource_queue_clone.queue_insert(resource_name, registry_key, instance_id);
            Ok(())
        })?;
        
        // Create register_system function
        let system_reg = system_registry.clone();
        let register_system = lua_clone.create_function(move |lua_ctx, (_schedule, func): (String, LuaFunction)| {
            // Get the current instance ID from globals
            let instance_id: u64 = lua_ctx.globals().get("__INSTANCE_ID__")
                .unwrap_or(0);
            
            let registry_key = lua_ctx.create_registry_value(func)?;
            system_reg.update_systems.lock().unwrap().push((instance_id, Arc::new(registry_key)));
            Ok(())
        })?;
        
        // Create copy_file function for file operations
        let copy_file = lua_clone.create_function(|_lua_ctx, (src, dest): (String, String)| {
            use std::fs;
            use std::path::Path;
            
            // Create destination directory if it doesn't exist
            if let Some(parent) = Path::new(&dest).parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| LuaError::RuntimeError(format!("Failed to create directory: {}", e)))?;
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
        let write_file_bytes = lua_clone.create_function(|_lua_ctx, (path, data): (String, LuaString)| {
            use std::fs;
            use std::path::Path;
            
            // Create destination directory if it doesn't exist
            if let Some(parent) = Path::new(&path).parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| LuaError::RuntimeError(format!("Failed to create directory: {}", e)))?;
            }
            
            fs::write(&path, data.as_bytes())
                .map_err(|e| LuaError::RuntimeError(format!("Failed to write file: {}", e)))?;
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
        let current_time = lua_clone.create_function(|_lua_ctx, ()| {
            Ok(crate::os_utilities::current_time_millis())
        })?;
        
        // Parse socket address
        let parse_socket_addr = lua_clone.create_function(|_lua_ctx, addr: String| {
            crate::os_utilities::parse_socket_addr(&addr)
                .map(|socket_addr| socket_addr.to_string())
                .map_err(|e| LuaError::RuntimeError(e))
        })?;
        
        // Inject into globals
        lua_clone.globals().set("spawn", spawn)?;
        lua_clone.globals().set("spawn_with_parent", spawn_with_parent)?;
        
        // Create despawn function
        let despawn = lua_clone.create_function(move |_lua_ctx, entity_id: u64| {
            let entity = Entity::from_raw_u32(entity_id as u32)
                .ok_or_else(|| LuaError::RuntimeError("Invalid entity ID".to_string()))?;
            despawn_queue.queue_despawn(entity);
            Ok(())
        })?;
        lua_clone.globals().set("despawn", despawn)?;
        
        lua_clone.globals().set("insert_resource", insert_resource)?;
        lua_clone.globals().set("register_system", register_system)?;
        lua_clone.globals().set("copy_file", copy_file)?;
        lua_clone.globals().set("read_file_bytes", read_file_bytes)?;
        lua_clone.globals().set("write_file_bytes", write_file_bytes)?;
        
        // OS utilities
        lua_clone.globals().set("bind_udp_socket", bind_udp_socket)?;
        lua_clone.globals().set("current_time", current_time)?;
        lua_clone.globals().set("parse_socket_addr", parse_socket_addr)?;
        
        // Note: load_asset will be added via add_asset_loading_to_lua()
        // Note: query_resource will be added to world table in lua_systems
        
        Ok(Self {
            lua: lua_clone,
        })
    }
    
    /// Execute a Lua script from a string
    pub fn execute_script_untracked(&self, script_content: &str, script_name: &str) -> Result<(), LuaError> {
        self.lua.load(script_content).set_name(script_name).exec()?;
        Ok(())
    }
    
    /// Execute a script with automatic script ownership tracking
    /// Entities spawned during execution will be tagged with a unique instance ID
    /// Returns the instance ID for this execution
    pub fn execute_script_tracked(&self, script_content: &str, script_name: &str, script_instance: &crate::script_entities::ScriptInstance) -> Result<u64, LuaError> {
        let instance_id = script_instance.start(script_name.to_string());
        
        // Set both instance ID and script name as Lua globals
        self.lua.globals().set("__INSTANCE_ID__", instance_id)?;
        self.lua.globals().set("__SCRIPT_NAME__", script_name)?;
        
        self.lua.load(script_content).set_name(script_name).exec()?;
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
        let instance_id = self.execute_script_tracked(script_content, script_name, script_instance)?;
        
        // Register in script registry for auto-reload
        script_registry.register_script(script_path, instance_id, script_content.to_string());
        
        Ok(instance_id)
    }
    
    /// Execute a script with automatic cleanup and tracking
    /// This despawns all entities from the previous instance before running the script again
    pub fn reload_script(&self, script_content: &str, script_name: &str, world: &mut bevy::prelude::World, instance_id: u64) -> Result<u64, LuaError> {
        // Despawn all entities from previous instance
        crate::script_entities::despawn_instance_entities(world, instance_id);
        
        // Get script instance resource
        let script_instance = world.resource::<crate::script_entities::ScriptInstance>().clone();
        
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
        
        // Add file watcher plugin for auto-reload
        app.add_plugins(crate::lua_file_watcher::LuaFileWatcherPlugin);
        
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
        app.add_systems(Update, (
            crate::entity_spawner::process_spawn_queue,
            crate::component_updater::process_component_updates,
            crate::lua_systems::run_lua_systems,
            crate::component_updater::process_component_updates,
            crate::despawn_queue::process_despawn_queue,
            crate::asset_loading::process_pending_assets,
        ));
        app.add_systems(Update, (
            crate::resource_inserter::process_resource_queue,
            auto_reload_changed_scripts,
        ));
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
fn initialize_component_registry(
    mut commands: Commands,
    type_registry: Res<AppTypeRegistry>,
) {
    let component_registry = crate::components::ComponentRegistry::from_type_registry(
        type_registry.clone()
    );
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
                        info!("Events available in Lua via world:read_events():");
                    }
                    info!("  ✓ {}", event_type);
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
    builder_registry: Res<crate::resource_builder::ResourceBuilderRegistry>,
    asset_server: Res<AssetServer>,
    mut component_registry: ResMut<crate::components::ComponentRegistry>,
    type_registry: Res<AppTypeRegistry>,
) {
    let system_registry = LuaSystemRegistry::default();
    
    // Create AssetRegistry with handle setters for all asset types
    let asset_registry = crate::asset_loading::AssetRegistry::from_type_registry(&type_registry);
    
    // Update ComponentRegistry with AssetRegistry reference
    component_registry.set_asset_registry(asset_registry.clone());
    
    match LuaScriptContext::new(
        queue.clone(),
        despawn_queue.clone(),
        resource_queue.clone(),
        system_registry.clone(),
        builder_registry.clone(),
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
        info!("File change detected: {:?}", event.path);
        
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
        
        let script_name = event.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.lua");
        
        info!("Reloading {} active instance(s) of '{}'", instances.len(), script_name);
        
        for (instance_id, _old_content) in instances {
            // Cleanup the instance
            cleanup_script_instance(instance_id, world);
            
            // Re-execute the script with the same instance tracking
            match lua_ctx.execute_script_tracked(&script_content, script_name, &script_instance) {
                Ok(new_instance_id) => {
                    info!("✓ Reloaded instance {} -> {} for '{}'", instance_id, new_instance_id, script_name);
                    
                    // Register the new instance in the registry
                    script_registry.register_script(
                        event.path.clone(),
                        new_instance_id,
                        script_content.clone()
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
fn cleanup_script_instance(instance_id: u64, world: &World) {
    // SAFETY: We need mutable access to cleanup. This is safe because we're in an exclusive system.
    #[allow(invalid_reference_casting)]
    let world_mut = unsafe { &mut *(world as *const World as *mut World) };
    
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
        let component_update_queue = world.resource::<crate::component_update_queue::ComponentUpdateQueue>().clone();
        let cleared_keys = component_update_queue.clear_for_entities(&entities_to_despawn);
        
        let num_cleared = cleared_keys.len();
        
        // Clean up the Lua registry keys to prevent memory leaks
        if let Some(lua_ctx) = world.get_resource::<LuaScriptContext>() {
            for key in cleared_keys {
                let _ = lua_ctx.lua.remove_registry_value(key);
            }
        }
        
        debug!("Cleared {} pending component updates for {} entities", 
               num_cleared, entities_to_despawn.len());
    }
    
    // 3. Clear all systems registered by this instance
    let system_registry = world.resource::<LuaSystemRegistry>().clone();
    system_registry.clear_instance_systems(instance_id);
    
    // 4. Remove all resources inserted by this instance
    let resource_queue = world.resource::<crate::resource_queue::ResourceQueue>().clone();
    let resources_to_clear = resource_queue.get_instance_resources(instance_id);
    
    if !resources_to_clear.is_empty() {
        let serde_registry = world.resource::<crate::serde_components::SerdeComponentRegistry>().clone();
        let builder_registry = world.resource::<crate::resource_builder::ResourceBuilderRegistry>().clone();
        
        for resource_name in &resources_to_clear {
            if !builder_registry.try_remove(resource_name, world_mut) {
                serde_registry.try_remove_resource(resource_name, world_mut);
            }
        }
    }
    
    resource_queue.clear_instance_tracking(instance_id);
    
    // 5. Despawn all entities owned by this instance
    crate::script_entities::despawn_instance_entities(world_mut, instance_id);
}
