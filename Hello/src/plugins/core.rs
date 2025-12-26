//! Core Hello plugin - always required
//! Wraps LuaBindingsPlugin (which includes LuaSpawnPlugin) and provides script execution

use bevy::prelude::*;
use bevy_lua_ecs::{LuaScriptContext, ScriptInstance, ScriptRegistry};

/// Resource to specify which script to run on startup
#[derive(Resource, Clone)]
pub struct MainScript(pub String);

/// Core Hello plugin - always required
/// Wraps LuaBindingsPlugin (which includes LuaSpawnPlugin)
pub struct HelloCorePlugin {}

impl Default for HelloCorePlugin {
    fn default() -> Self {
        Self {}
    }
}

impl Plugin for HelloCorePlugin {
    fn build(&self, app: &mut App) {
        // Use auto-generated LuaBindingsPlugin (includes LuaSpawnPlugin + all bindings)
        app.add_plugins(crate::auto_resource_bindings::LuaBindingsPlugin);
        
        app.add_systems(PostStartup, run_initial_script.run_if(resource_exists::<MainScript>));
    }
}

fn run_initial_script(
    script: Res<MainScript>,
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<ScriptInstance>,
    script_registry: Res<ScriptRegistry>,
) {
    let script_path = std::path::PathBuf::from(format!("assets/{}", script.0));
    match std::fs::read_to_string(&script_path) {
        Ok(content) => {
            info!("🚀 Running initial script: {}", script.0);
            if let Err(e) = lua_ctx.execute_script(
                &content, 
                &script.0, 
                script_path, 
                &script_instance, 
                &script_registry
            ) {
                error!("Failed to execute script: {}", e);
            }
        }
        Err(e) => error!("Failed to load script {}: {}", script.0, e),
    }
}
