use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LuaSpawnPlugin)
        .add_systems(Startup, setup)
        .add_systems(PostStartup, load_and_run_script)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    info!("Camera spawned");
}

fn load_and_run_script(lua_ctx: Res<LuaScriptContext>) {
    let script_path = "Hello/assets/scripts/asset_test.lua";
    match fs::read_to_string(script_path) {
        Ok(script_content) => {
            info!("Loaded test script: {}", script_path);
            if let Err(e) = lua_ctx.execute_script(&script_content, "asset_test.lua") {
                error!("Failed to execute script: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to load script {}: {}", script_path, e);
        }
    }
}
