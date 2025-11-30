/// Hot Reload Example
/// Demonstrates moving sprites that "hot reload" every 5 seconds
/// Run with: cargo run --example hot_reload

use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

#[derive(Resource, serde::Serialize, serde::Deserialize)]
struct TestResource {
    reload_number: u32,
    message: String,
}

fn main() {
    let mut app = App::new();

    app
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Enable asset hot-reloading for file watching
            watch_for_changes_override: Some(true),
            ..default()
        }))
        .add_plugins(LuaSpawnPlugin)
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_lua_resources)
        .add_systems(PostStartup, load_and_run_script)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    info!("Camera spawned");
}

fn setup_lua_resources(world: &mut World) {
    // Register TestResource so it can be inserted and removed from Lua
    let mut serde_registry = world.resource_mut::<crate::serde_components::SerdeComponentRegistry>();
    serde_registry.register_resource::<TestResource>("TestResource");
    info!("Registered TestResource");
}

fn load_and_run_script(
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<crate::script_entities::ScriptInstance>,
    script_registry: Res<crate::script_registry::ScriptRegistry>,
) {
    let script_path = std::path::PathBuf::from("assets/scripts/hot_reload.lua");
    match fs::read_to_string(&script_path) {
        Ok(script_content) => {
            info!("Loaded hot reload script: {:?}", script_path);
            match lua_ctx.execute_script(
                &script_content,
                "hot_reload.lua",
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
