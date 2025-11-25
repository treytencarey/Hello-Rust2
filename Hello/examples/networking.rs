use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

#[cfg(feature = "networking")]
use bevy_replicon::prelude::*;
#[cfg(feature = "networking")]
use bevy_replicon_renet::RepliconRenetPlugins;

fn main() {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins);
    
    // Add replicon plugins (third-party plugins, not game logic)
    #[cfg(feature = "networking")]
    {
        app.add_plugins(RepliconPlugins)
            .add_plugins(RepliconRenetPlugins);
        
        // Register components for replication (configuration)
        // Only replicate Transform - Sprite will be added by client locally
        app.replicate::<Transform>();
    }
    
    // Create component registry
    let component_registry = ComponentRegistry::from_type_registry(
        app.world().resource::<AppTypeRegistry>().clone()
    );
    
    let mut serde_registry = SerdeComponentRegistry::default();
    
    // Register Replicated marker component for this example
    #[cfg(feature = "networking")]
    serde_registry.register_marker::<Replicated>("Replicated");
    
    // Create builder registry and register networking constructors from library
    // These are GENERIC library infrastructure, not game-specific code!
    let builder_registry = ResourceBuilderRegistry::default();
    #[cfg(feature = "networking")]
    register_networking_constructors(&builder_registry);
    
    app.insert_resource(component_registry)
        .init_resource::<SpawnQueue>()
        .init_resource::<ResourceQueue>()
        .init_resource::<ComponentUpdateQueue>()
        .insert_resource(serde_registry)
        .insert_resource(builder_registry)
        .init_resource::<ResourceConstructorRegistry>();  // Keep for future Reflect-based resources
        
    app.add_plugins(LuaSpawnPlugin)
        .add_systems(Update, (
            process_spawn_queue,
            run_lua_systems,
            bevy_lua_ecs::component_updater::process_component_updates,
        ))
        .add_systems(PostStartup, load_and_run_script)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn load_and_run_script(lua_ctx: Res<LuaScriptContext>) {
    let script_path = "assets/scripts/networking_example.lua";
    match fs::read_to_string(script_path) {
        Ok(script_content) => {
            if let Err(e) = lua_ctx.execute_script(&script_content, "networking_example.lua") {
                error!("Failed to execute script: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to load script {}: {}", script_path, e);
        }
    }
}
