use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;
use std::env;

#[cfg(feature = "networking")]
use bevy_replicon::prelude::*;
#[cfg(feature = "networking")]
use bevy_replicon_renet::RepliconRenetPlugins;

#[cfg(feature = "networking")]
mod networking {
    include!("../src/networking.rs");
}

fn main() {
    // Check if we should run as client
    let args: Vec<String> = env::args().collect();
    let is_client = args.len() > 1 && (args[1] == "client" || args[1] == "--client");
    
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins);
    
    // Register events BEFORE Replicon plugins for consistent protocol
    #[cfg(feature = "networking")]
    register_common_bevy_events(&mut app);
    
    // Add replicon plugins (third-party plugins, not game logic)
    // Networking resources will be created from Lua script
    #[cfg(feature = "networking")]
    {
        app.add_plugins(RepliconPlugins)
            .add_plugins(RepliconRenetPlugins);
        
        // Register components for replication
        app.replicate::<Transform>();
    }
    
    app.add_plugins(LuaSpawnPlugin)
        .add_systems(PreStartup, setup_event_readers)
        .add_systems(Startup, setup);
    
    app.add_systems(PostStartup, load_and_run_script);
    
    // Insert client mode resource if needed
    if is_client {
        app.insert_resource(ClientMode(true));
    }
    
    app.run();
}

/// Register networking-specific components and resource constructors
/// Runs in PreStartup (after LuaSpawnPlugin's PreStartup but before Startup)
fn setup_event_readers(
    mut serde_registry: ResMut<SerdeComponentRegistry>,
    builder_registry: Res<ResourceBuilderRegistry>,
    lua_resource_registry: Res<bevy_lua_ecs::LuaResourceRegistry>,
) {
    // Register Replicated marker component for this example
    #[cfg(feature = "networking")]
    serde_registry.register_marker::<Replicated>("Replicated");
    
    // Register networking constructors (for insert_resource in Lua)
    #[cfg(feature = "networking")]
    networking::register_networking_constructors(&builder_registry);
    
    // Register networking method bindings (for world:call_resource_method in Lua)
    #[cfg(feature = "networking")]
    networking::register_networking_methods(&lua_resource_registry);
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Resource)]
struct ClientMode(bool);

fn load_and_run_script(
    lua_ctx: Res<LuaScriptContext>,
    client_mode: Option<Res<ClientMode>>,
) {
    // Set client mode global in Lua
    let is_client = client_mode.map(|c| c.0).unwrap_or(false);
    if let Err(e) = lua_ctx.lua.globals().set("IS_CLIENT_MODE", is_client) {
        error!("Failed to set client mode: {}", e);
    }
    
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
