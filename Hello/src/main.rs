use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

#[cfg(feature = "physics")]
mod rapier;

#[cfg(feature = "tiled")]
mod tiled;

#[cfg(feature = "networking")]
pub mod networking;

fn main() {
    let mut app = App::new();
    
    // Add Bevy's default plugins
    app.add_plugins(DefaultPlugins);
    
    // Add Rapier physics plugins if the physics feature is enabled
    #[cfg(feature = "physics")]
    app.add_plugins(rapier::RapierIntegrationPlugin);
    
    // Add Tiled map plugin if the tiled feature is enabled
    #[cfg(feature = "tiled")]
    app.add_plugins(tiled::TiledIntegrationPlugin);
    
    // Add networking plugin if the networking feature is enabled
    #[cfg(feature = "networking")]
    app.add_plugins(networking::NetworkingIntegrationPlugin);
    
    // Create component registry AFTER plugins are added
    // This automatically includes Rapier's components if enabled!
    let component_registry = ComponentRegistry::from_type_registry(
        app.world().resource::<AppTypeRegistry>().clone()
    );
    
    app.insert_resource(component_registry)
        .init_resource::<SpawnQueue>()
        .init_resource::<ComponentUpdateQueue>();
        
    app.add_plugins(LuaSpawnPlugin)
        .add_systems(Update, (
            process_spawn_queue,
            run_lua_systems,
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
    let script_path = "Hello/assets/scripts/main.lua";
    match fs::read_to_string(script_path) {
        Ok(script_content) => {
            info!("✓ Loaded script: {}", script_path);
            if let Err(e) = lua_ctx.execute_script(&script_content, "main.lua") {
                error!("Failed to execute script: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to load script {}: {}", script_path, e);
        }
    }
}
