use bevy::prelude::*;
use bevy_lua_ecs::component_updater::process_component_updates;
use bevy_lua_ecs::*;
use bevy_rapier2d::prelude::*;
use std::fs;

fn main() {
    let mut app = App::new();

    // Add Bevy's default plugins
    app.add_plugins(DefaultPlugins);

    // Add Rapier physics plugins (external plugin!)
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0));
    app.add_plugins(RapierDebugRenderPlugin::default());

    // Add Lua plugin (auto-initializes all resources and systems)
    app.add_plugins(LuaSpawnPlugin);

    // Register serde-based components (for types that don't implement Reflect)
    // This uses the SerdeComponentRegistry that was auto-initialized by LuaSpawnPlugin
    app.insert_resource(bevy_lua_ecs::serde_components![Collider,]);

    app.add_systems(Startup, setup)
        .add_systems(PostStartup, load_and_run_script)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    info!("✓ Camera spawned");
}

fn load_and_run_script(
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<crate::script_entities::ScriptInstance>,
    script_registry: Res<crate::script_registry::ScriptRegistry>,
) {
    let script_path = std::path::PathBuf::from("assets/scripts/physics_example.lua");
    match fs::read_to_string(&script_path) {
        Ok(script_content) => {
            info!("Loaded hot reload script: {:?}", script_path);
            match lua_ctx.execute_script(
                &script_content,
                "physics_example.lua",
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
