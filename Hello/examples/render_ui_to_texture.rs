use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

// Include auto-generated bindings
#[path = "../src/auto_resource_bindings.rs"]
mod auto_resource_bindings;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    
    app.add_plugins(crate::auto_resource_bindings::LuaBindingsPlugin);
    app.add_systems(Startup, setup);
    app.add_systems(Update, load_and_run_script.run_if(run_once));
    app.run();
}

fn setup() {
    info!("Lua RTT UI example starting - Zero Rust picking enabled...");
}

fn load_and_run_script(
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<ScriptInstance>,
    script_registry: Res<ScriptRegistry>,
) {
    let script_path = std::path::PathBuf::from("assets/scripts/examples/render_ui_to_texture.lua");
    match fs::read_to_string(&script_path) {
        Ok(script_content) => {
            info!("Loaded RTT UI script: {:?}", script_path);
            match lua_ctx.execute_script(
                &script_content,
                "render_ui_to_texture.lua",
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
