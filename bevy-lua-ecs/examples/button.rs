use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

fn main() {
    let mut app = App::new();
    
    // Add Bevy's default plugins
    app.add_plugins(DefaultPlugins);
    
    // Create component registry AFTER plugins are added
    let component_registry = ComponentRegistry::from_type_registry(
        app.world().resource::<AppTypeRegistry>().clone()
    );
    
    app.insert_resource(component_registry)
        .init_resource::<SpawnQueue>()
        .add_plugins(LuaSpawnPlugin)
        .add_systems(Update, (
            process_spawn_queue,
            run_lua_systems,
        ))
        .add_systems(Startup, (setup, load_and_run_script).chain())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    info!("✓ Camera spawned");
}

fn load_and_run_script(lua_ctx: Res<LuaScriptContext>) {
    let script_path = "bevy-lua-ecs/assets/scripts/spawn_button.lua";
    match fs::read_to_string(script_path) {
        Ok(script_content) => {
            info!("✓ Loaded script: {}", script_path);
            if let Err(e) = lua_ctx.execute_script(&script_content, "spawn_button.lua") {
                error!("Failed to execute script: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to load script {}: {}", script_path, e);
        }
    }
}
