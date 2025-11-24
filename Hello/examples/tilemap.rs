use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

fn main() {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins);
    
    // Create component registry
    let component_registry = ComponentRegistry::from_type_registry(
        app.world().resource::<AppTypeRegistry>().clone()
    );
    
    app.insert_resource(component_registry)
        .init_resource::<SpawnQueue>()
        .init_resource::<ComponentUpdateQueue>()
        .init_resource::<SerdeComponentRegistry>();
        
    app.add_plugins(LuaSpawnPlugin)
        .add_systems(Update, (
            process_spawn_queue,
            run_lua_systems,
        ))
        .add_systems(PostStartup, load_tilemap_script)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    info!("✓ Camera spawned");
}

fn load_tilemap_script(lua_ctx: Res<LuaScriptContext>) {
    let script_path = "Hello/assets/scripts/tilemap.lua";
    match fs::read_to_string(script_path) {
        Ok(script_content) => {
            info!("✓ Loading tilemap from Lua script");
            if let Err(e) = lua_ctx.execute_script(&script_content, "tilemap.lua") {
                error!("Failed to execute tilemap script: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to load script {}: {}", script_path, e);
        }
    }
}
