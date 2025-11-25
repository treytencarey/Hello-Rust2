use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LuaSpawnPlugin)
        .add_systems(Startup, (setup, load_and_run_script).chain())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    info!("Camera spawned");
}

fn load_and_run_script(lua_ctx: Res<LuaScriptContext>) {
    let script_path = "bevy-lua-ecs/assets/scripts/spawn_text.lua";
    match fs::read_to_string(script_path) {
        Ok(script_content) => {
            info!("Loaded script: {}", script_path);
            if let Err(e) = lua_ctx.execute_script(&script_content, "spawn_text.lua") {
                error!("Failed to execute script: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to load script {}: {}", script_path, e);
        }
    }
}
