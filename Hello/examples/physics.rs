/// Physics Example
/// Demonstrates the Rapier physics integration with bevy-lua-ecs
/// Run with: cargo run --example physics --features physics

use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

#[path = "../src/rapier.rs"]
mod rapier;

fn main() {
    let mut app = App::new();

    app
        .add_plugins(DefaultPlugins)
        .add_plugins(rapier::RapierIntegrationPlugin) // Add Rapier physics plugins
        .add_plugins(LuaSpawnPlugin)
        .add_systems(Startup, setup)
        .add_systems(PostStartup, load_and_run_script)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    info!("Camera spawned");
}

fn load_and_run_script(
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<crate::script_entities::ScriptInstance>,
    script_registry: Res<crate::script_registry::ScriptRegistry>,
) {
    let script_path = std::path::PathBuf::from("assets/scripts/examples/physics.lua");
    match fs::read_to_string(&script_path) {
        Ok(script_content) => {
            info!("Loaded hot reload script: {:?}", script_path);
            match lua_ctx.execute_script(
                &script_content,
                "physics.lua",
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
