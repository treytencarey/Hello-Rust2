use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

// Re-export modules from the library (lib.rs declares them)
#[cfg(feature = "physics")]
use hello::rapier;

#[cfg(feature = "tiled")]
use hello::tiled;

#[cfg(feature = "networking")]
use hello::network_asset_integration;

#[cfg(feature = "networking")]
use hello::auto_resource_bindings;

fn main() {
    let mut app = App::new();
    
    // Add Bevy's default plugins
    app.add_plugins(DefaultPlugins);
    
    // Add Rapier physics plugins if the physics feature is enabled
    #[cfg(feature = "physics")]
    app.add_plugins(rapier::RapierIntegrationPlugin);
    
    // Add Tiled map plugin if the tiled feature is enabled
    #[cfg(feature = "tiled")]
    app.add_plugins(tiled::TiledIntegrationPlugin);
    
    // Add Lua plugin (auto-initializes all resources and systems)
    app.add_plugins(LuaSpawnPlugin);
    
    // Add network asset downloading plugin (if networking enabled)
    #[cfg(feature = "networking")]
    app.add_plugins(network_asset_integration::NetworkAssetPlugin);
    
    // Register auto-generated resource method bindings (networking)
    #[cfg(feature = "networking")]
    app.add_systems(PreStartup, register_networking_bindings);
    
    app.add_systems(Startup, setup)
        .add_systems(PostStartup, load_and_run_script)
        .run();
}

#[cfg(feature = "networking")]
fn register_networking_bindings(registry: Res<LuaResourceRegistry>) {
    auto_resource_bindings::register_auto_resource_bindings(&registry);
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    info!("✓ Camera spawned");
}

fn load_and_run_script(
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<ScriptInstance>,
    script_registry: Res<ScriptRegistry>,
) {
    let script_path = std::path::PathBuf::from("assets/scripts/examples/network_client_test.lua");
    match fs::read_to_string(&script_path) {
        Ok(script_content) => {
            info!("✓ Loaded script: {:?}", script_path);
            if let Err(e) = lua_ctx.execute_script(
                &script_content, 
                "network_client_test.lua",
                script_path,
                &script_instance,
                &script_registry,
            ) {
                error!("Failed to execute script: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to load script {:?}: {}", script_path, e);
        }
    }
}

