//! Core Hello plugin - always required
//! Wraps LuaBindingsPlugin (which includes LuaSpawnPlugin) and provides script execution

use bevy::prelude::*;
use bevy_lua_ecs::{LuaScriptContext, ScriptInstance, ScriptRegistry};
use mlua::prelude::*;

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
        
        // Register application-specific Lua functions (run in PostStartup before initial script)
        app.add_systems(PostStartup, (
            register_hello_lua_functions,
            run_initial_script.run_if(resource_exists::<MainScript>),
        ).chain());
    }
}

/// Register Hello-specific Lua functions (like file picker)
fn register_hello_lua_functions(lua_ctx: Res<LuaScriptContext>) {
    let lua = &lua_ctx.lua;
    
    // Create file picker dialog function (uses rfd crate for native dialogs)
    let pick_files_dialog = match lua.create_function(|lua_ctx, ()| {
        use rfd::FileDialog;
        
        let files = FileDialog::new()
            .set_title("Select files to upload")
            .pick_files();
        
        if let Some(paths) = files {
            // Convert paths to Lua table of strings
            let table = lua_ctx.create_table()?;
            for (i, path) in paths.iter().enumerate() {
                table.set(i + 1, path.to_string_lossy().to_string())?;
            }
            Ok(Some(table))
        } else {
            Ok(None)
        }
    }) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to create pick_files_dialog function: {:?}", e);
            return;
        }
    };
    
    if let Err(e) = lua.globals().set("pick_files_dialog", pick_files_dialog) {
        error!("Failed to register pick_files_dialog: {:?}", e);
    } else {
        info!("✓ Registered pick_files_dialog Lua function");
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
