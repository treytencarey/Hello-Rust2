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
        
        // Create component-based spawn function
        let spawn = lua_clone.create_function(move |_lua_ctx, components: LuaTable| {
            let mut all_components = Vec::new();
            
            // Iterate over components table
            for pair in components.pairs::<String, LuaValue>() {
                let (component_name, component_value) = pair?;
                
                // Store everything as registry value
                let registry_key = lua_for_closure.create_registry_value(component_value)?;
                all_components.push((component_name, registry_key));
            }
            
            // Pass empty list for lua_components, we'll sort it out in the spawner
            queue_clone.clone().queue_spawn(all_components, Vec::new());
            Ok(())
        })?;
        
        // Create generic insert_resource function
        // Accepts either a table (for serde resources) or UserData (for builder-created resources)
        let insert_resource = lua_clone.create_function(move |_lua_ctx, (resource_name, resource_data): (String, LuaValue)| {
            // Store resource data as registry value (works for both tables and UserData)
            let registry_key = lua_for_resource.create_registry_value(resource_data)?;
            resource_queue_clone.queue_insert(resource_name, registry_key);
            Ok(())
        })?;
        
        // Create register_system function
        let system_reg = system_registry.clone();
        let register_system = lua_clone.create_function(move |lua_ctx, (_schedule, func): (String, LuaFunction)| {
            let registry_key = lua_ctx.create_registry_value(func)?;
            system_reg.update_systems.lock().unwrap().push(Arc::new(registry_key));
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
    pub fn execute_script(&self, script_content: &str, script_name: &str) -> Result<(), LuaError> {
        self.lua.load(script_content).set_name(script_name).exec()?;
        Ok(())
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
        app.init_resource::<crate::component_update_queue::ComponentUpdateQueue>();
        app.init_resource::<crate::resource_queue::ResourceQueue>();
        app.init_resource::<crate::resource_builder::ResourceBuilderRegistry>();
        app.init_resource::<crate::serde_components::SerdeComponentRegistry>();
        app.init_resource::<crate::resource_lua_trait::LuaResourceRegistry>();
        
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
            crate::lua_systems::run_lua_systems,
            crate::component_updater::process_component_updates,
            crate::asset_loading::process_pending_assets,
            crate::resource_inserter::process_resource_queue,
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
