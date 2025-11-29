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
        .add_plugins(DefaultPlugins)
        .add_plugins(LuaSpawnPlugin)
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_lua_resources)
        .add_systems(PostStartup, load_and_run_script)
        .add_systems(Update, check_for_reload)
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

fn load_and_run_script(lua_ctx: Res<LuaScriptContext>, script_instance: Res<crate::script_entities::ScriptInstance>) {
    let script_path = "assets/scripts/hot_reload.lua";
    match fs::read_to_string(script_path) {
        Ok(script_content) => {
            info!("Loaded hot reload script: {}", script_path);
            match lua_ctx.execute_script_tracked(&script_content, "hot_reload.lua", &script_instance) {
                Ok(instance_id) => {
                    info!("Script executed with instance ID: {}", instance_id);
                }
                Err(e) => {
                    error!("Failed to execute script: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to load script {}: {}", script_path, e);
        }
    }
}

fn check_for_reload(lua_ctx: Res<LuaScriptContext>, script_instance: Res<crate::script_entities::ScriptInstance>) {
    // Check if Lua set the reload flag
    if let Ok(should_reload) = lua_ctx.lua.globals().get::<bool>("should_reload") {
        if should_reload {
            // Clear the flag
            let _ = lua_ctx.lua.globals().set("should_reload", false);
            
            // Re-execute the script
            let script_path = "assets/scripts/hot_reload.lua";
            if let Ok(script_content) = fs::read_to_string(script_path) {
                info!("Re-executing hot reload script");
                match lua_ctx.execute_script_tracked(&script_content, "hot_reload.lua", &script_instance) {
                    Ok(instance_id) => {
                        info!("Script re-executed with new instance ID: {}", instance_id);
                    }
                    Err(e) => {
                        error!("Failed to re-execute script: {}", e);
                    }
                }
            }
        }
    }
}
