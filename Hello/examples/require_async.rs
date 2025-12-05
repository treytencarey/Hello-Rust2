use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LuaSpawnPlugin)
        .add_systems(PostStartup, load_and_run_script)
        .run();
}

fn load_and_run_script(
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<ScriptInstance>,
    script_registry: Res<ScriptRegistry>,
) {
    let script_path = std::path::PathBuf::from("assets/scripts/examples/require_async.lua");
    match fs::read_to_string(&script_path) {
        Ok(script_content) => {
            info!("✓ Loaded script: {:?}", script_path);
            if let Err(e) = lua_ctx.execute_script(
                &script_content, 
                "examples/require_async.lua",
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

