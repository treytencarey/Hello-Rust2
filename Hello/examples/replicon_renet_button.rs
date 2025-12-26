// Hybrid Renet + Replicon Button Example
// Demonstrates: Renet for Lua messaging + Replicon ready for component replication

use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

#[cfg(feature = "networking")]
use bevy_replicon::prelude::*;
#[cfg(feature = "networking")]
use bevy_replicon_renet::RepliconRenetPlugins;

#[cfg(feature = "networking")]
mod renet_lua {
    include!("../src/renet_lua.rs");
}

#[cfg(feature = "networking")]
mod networking {
    include!("../src/networking.rs");
}

fn main() {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins);
    
    // Add both Replicon (for future component replication) and Renet (for Lua messaging)
    #[cfg(feature = "networking")]
    {
        // Replicon plugins - ready for component replication when needed
        app.add_plugins(RepliconPlugins)
            .add_plugins(RepliconRenetPlugins);
        
        // Initialize Renet event resources for Lua messaging
        app.init_resource::<renet_lua::LuaEventQueue>();
        app.init_resource::<renet_lua::ReceivedLuaEvents>();
        
        // Add Renet messaging systems (for Lua events)
        // IMPORTANT: Must run before run_lua_systems so events are available in Lua
        app.add_systems(Update, (
            renet_lua::sync_lua_events_to_queue,
            renet_lua::send_lua_events_renet,
            renet_lua::receive_lua_events_renet,
            renet_lua::sync_received_events_to_lua,
        ).chain().before(run_lua_systems));
    }
    
    app.add_plugins(LuaSpawnPlugin);
    
    #[cfg(feature = "networking")]
    app.add_systems(PreStartup, register_networking_bindings);
    
    #[cfg(feature = "networking")]
    app.add_systems(PostStartup, renet_lua::setup_renet_lua_bindings.before(load_and_run_script));
    
    app.add_systems(PostStartup, load_and_run_script)
        .run();
}

#[cfg(feature = "networking")]
fn register_networking_bindings(
    builder_registry: Res<ResourceBuilderRegistry>,
    method_registry: Res<LuaResourceRegistry>,
) {
    // Register networking constructors from library (generic infrastructure)
    networking::register_networking_constructors(&builder_registry);
    networking::register_networking_methods(&method_registry);
}

fn load_and_run_script(
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<ScriptInstance>,
    script_registry: Res<ScriptRegistry>,
) {
    let script_path = std::path::PathBuf::from("assets/scripts/examples/hybrid_button.lua");
    match fs::read_to_string(&script_path) {
        Ok(script_content) => {
            info!("Loaded script: {:?}", script_path);
            match lua_ctx.execute_script(
                &script_content,
                "hybrid_button.lua",
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
