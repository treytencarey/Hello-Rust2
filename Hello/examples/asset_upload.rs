use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

#[cfg(feature = "networking")]
use bevy_replicon::prelude::*;
#[cfg(feature = "networking")]
use bevy_replicon_renet::RepliconRenetPlugins;

fn main() {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins);
    
    // Add replicon plugins (third-party plugins, not game logic)
    #[cfg(feature = "networking")]
    {
        app.add_plugins(RepliconPlugins)
            .add_plugins(RepliconRenetPlugins);
        
        // Register components for replication
        // Only replicate Transform - Sprite will be added by client locally
        // (Sprite doesn't implement Serialize, and we don't need to replicate the image data)
        app.replicate::<Transform>();
    }
    
    app.add_plugins(LuaSpawnPlugin)
        .add_systems(PreStartup, setup_event_readers)
        .add_systems(Startup, setup)
        .add_systems(PostStartup, load_and_run_script)
        .run();
}

/// Register networking-specific components and resource constructors
/// Runs in PreStartup (after LuaSpawnPlugin's PreStartup but before Startup)
fn setup_event_readers(
    mut serde_registry: ResMut<SerdeComponentRegistry>,
    builder_registry: Res<ResourceBuilderRegistry>,
) {
    // Register Replicated marker component for this example
    #[cfg(feature = "networking")]
    serde_registry.register_marker::<Replicated>("Replicated");
    
    // Register networking constructors from library (generic infrastructure)
    #[cfg(feature = "networking")]
    register_networking_constructors(&builder_registry);
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn load_and_run_script(lua_ctx: Res<LuaScriptContext>) {
    let script_path = "Hello/assets/scripts/asset_upload_example.lua";
    match fs::read_to_string(script_path) {
        Ok(script_content) => {
            if let Err(e) = lua_ctx.execute_script(&script_content, "asset_upload_example.lua") {
                error!("Failed to execute script: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to load script {}: {}", script_path, e);
        }
    }
}
