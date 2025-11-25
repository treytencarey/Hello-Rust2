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
        .init_resource::<ComponentUpdateQueue>()
        .init_resource::<ResourceQueue>()
        .init_resource::<ResourceBuilderRegistry>()
        .init_resource::<SerdeComponentRegistry>()
        .add_plugins(LuaSpawnPlugin)
        .add_systems(Update, (
            process_spawn_queue,
            run_lua_systems,
            process_component_updates,
        ))
        .add_systems(PostStartup, load_and_run_script)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    info!("✓ Camera spawned");
}

fn load_and_run_script(lua_ctx: Res<LuaScriptContext>) {
    let script_path = "bevy-lua-ecs/assets/scripts/spawn_sprites.lua";
    match fs::read_to_string(script_path) {
        Ok(script_content) => {
            info!("✓ Loaded script: {}", script_path);
            if let Err(e) = lua_ctx.execute_script(&script_content, "spawn_sprites.lua") {
                error!("Failed to execute script: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to load script {}: {}", script_path, e);
        }
    }
}
