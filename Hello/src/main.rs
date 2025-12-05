use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

#[cfg(feature = "physics")]
mod rapier;

#[cfg(feature = "tiled")]
mod tiled;

#[cfg(feature = "networking")]
pub mod networking;

#[cfg(feature = "networking")]
mod auto_resource_bindings;

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
    let script_path = std::path::PathBuf::from("assets/scripts/require_sync_example.lua");
    match fs::read_to_string(&script_path) {
        Ok(script_content) => {
            info!("✓ Loaded script: {:?}", script_path);
            if let Err(e) = lua_ctx.execute_script(
                &script_content, 
                "require_sync_example.lua",
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

