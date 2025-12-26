use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

#[cfg(feature = "networking")]
use bevy_replicon::prelude::*;
#[cfg(feature = "networking")]
use bevy_replicon_renet::RepliconRenetPlugins;

#[cfg(feature = "networking")]
mod networking {
    include!("../src/networking.rs");
}

fn main() {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins);
    
    // Register events BEFORE Replicon plugins for consistent protocol
    #[cfg(feature = "networking")]
    register_common_bevy_events(&mut app);
    
    // Add replicon plugins (third-party plugins, not game logic)
    #[cfg(feature = "networking")]
    {
        app.add_plugins(RepliconPlugins)
            .add_plugins(RepliconRenetPlugins);
        
        // Register components for replication (configuration)
        // Only replicate Transform (Replicated is just a marker)
        app.replicate::<Transform>();
    }
    
    app.add_plugins(LuaSpawnPlugin)
        .add_systems(PreStartup, setup_networking_registries)
        .add_systems(PostStartup, load_and_run_script)
        .run();
}

/// Register networking-specific components and resource constructors
/// Runs in PreStartup (after LuaSpawnPlugin's PreStartup but before Startup)
fn setup_networking_registries(
    mut serde_registry: ResMut<SerdeComponentRegistry>,
    builder_registry: Res<ResourceBuilderRegistry>,
    method_registry: Res<LuaResourceRegistry>,
) {
    // Register Replicated marker component for this example
    #[cfg(feature = "networking")]
    serde_registry.register_marker::<Replicated>("Replicated");
    
    // Register networking constructors from library (generic infrastructure)
    #[cfg(feature = "networking")]
    networking::register_networking_constructors(&builder_registry);
    
    // Register networking method bindings (send_message, receive_message, etc.)
    #[cfg(feature = "networking")]
    networking::register_networking_methods(&method_registry);
}

fn load_and_run_script(
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<crate::script_entities::ScriptInstance>,
    script_registry: Res<crate::script_registry::ScriptRegistry>,
) {
    let script_path = std::path::PathBuf::from("assets/scripts/examples/networking_example.lua");
    match fs::read_to_string(&script_path) {
        Ok(script_content) => {
            info!("Loaded hot reload script: {:?}", script_path);
            match lua_ctx.execute_script(
                &script_content,
                "networking_example.lua",
                script_path,
                &script_instance,
                &script_registry,
            ) {
                Ok(instance_id) => {
                    info!("Script executed with instance ID: {}", instance_id);
                }
                Err(e) => {
                    error!("Failed to execute script: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to load script {:?}: {}", script_path, e);
        }
    }
}
