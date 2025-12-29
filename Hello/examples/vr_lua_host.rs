//! VR Lua Host Example
//! 
//! A minimal Rust bootstrap for VR Lua applications.
//! Uses VrInputPlugin for controller tracking, button state polling, and OpenXR setup.
//! Just runs a Lua script that can access all VR state via world:get_resource().
//!
//! Usage: cargo run --example vr_lua_host --features bevy_mod_xr
//! Optional: cargo run --example vr_lua_host --features bevy_mod_xr -- path/to/script.lua

use bevy::prelude::*;
use bevy_lua_ecs::*;
use hello::plugins::{HelloCorePlugin, VrInputPlugin};
use std::fs;

use bevy_mod_openxr::add_xr_plugins;

// Default script path - can be overridden via command line
const DEFAULT_SCRIPT: &str = "assets/scripts/examples/vr_sidebar.lua";

fn main() -> AppExit {
    let mut app = App::new();
    
    // Get script path from args or use default
    let script_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_SCRIPT.to_string());
    
    // Store script path as resource
    app.insert_resource(ScriptPath(script_path));
    
    // Configure default plugins with XR
    app.add_plugins(add_xr_plugins(DefaultPlugins.set(bevy::pbr::PbrPlugin {
        prepass_enabled: false,
        ..default()
    })));
    app.add_plugins(bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin);
    
    // VR input plugin - handles all controller tracking, buttons, and OpenXR actions
    app.add_plugins(VrInputPlugin);
    
    // Hello core plugin for Lua globals (pick_files_dialog, etc)
    app.add_plugins(HelloCorePlugin {});
    
    // Network asset plugin for directory listing and upload functions
    app.add_plugins(hello::network_asset_integration::NetworkAssetPlugin);
    
    // Script loading
    app.add_systems(PostStartup, load_and_run_script);

    app.run()
}

#[derive(Resource)]
struct ScriptPath(String);

fn load_and_run_script(
    script_path: Res<ScriptPath>,
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<crate::script_entities::ScriptInstance>,
    script_registry: Res<crate::script_registry::ScriptRegistry>,
) {
    let path = std::path::PathBuf::from(&script_path.0);
    match fs::read_to_string(&path) {
        Ok(script_content) => {
            info!("Loading Lua script: {:?}", path);
            match lua_ctx.execute_script(
                &script_content,
                path.file_name().unwrap().to_str().unwrap(),
                path.clone(),
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
            error!("Failed to load script {:?}: {}", path, e);
        }
    }
}
