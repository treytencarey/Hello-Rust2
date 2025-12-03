/// Physics 3D Example
/// Demonstrates the Rapier 3D physics integration with bevy-lua-ecs
/// Run with: cargo run --example physics_3d --features physics3d

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_lua_ecs::*;
use std::fs;


fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(
            0xF9 as f32 / 255.0,
            0xF9 as f32 / 255.0,
            0xFF as f32 / 255.0,
        )))
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(LuaSpawnPlugin)
        // Register Collider component for Lua (doesn't implement Reflect)
        .insert_resource(bevy_lua_ecs::serde_components![
            Collider,
        ])
        .add_systems(PostStartup, load_and_run_script)
        .run();
}

fn load_and_run_script(
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<crate::script_entities::ScriptInstance>,
    script_registry: Res<crate::script_registry::ScriptRegistry>,
) {
    let script_path = std::path::PathBuf::from("assets/scripts/physics_3d.lua");
    match fs::read_to_string(&script_path) {
        Ok(script_content) => {
            info!("Loaded hot reload script: {:?}", script_path);
            match lua_ctx.execute_script(
                &script_content,
                "physics_3d.lua",
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
