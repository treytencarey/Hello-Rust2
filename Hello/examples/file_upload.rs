// Pure Lua File Upload Example
// Demonstrates "Zero Rust" networking - all logic in Lua
//
// This Rust file only:
// 1. Sets up Bevy app
// 2. Loads Lua script
// 3. Registers networking constructors (generic utilities)
//
// ALL file upload logic is in assets/scripts/file_upload.lua
//
// Run as server: cargo run --example file_upload --features networking
// Run as client: cargo run --example file_upload --features networking client

use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

#[cfg(feature = "networking")]
use bevy_renet::{RenetServerPlugin, RenetClientPlugin};
#[cfg(feature = "networking")]
use bevy_renet::netcode::{NetcodeServerPlugin, NetcodeClientPlugin};

#[cfg(feature = "networking")]
mod networking {
    include!("../src/networking.rs");
}

#[derive(Resource)]
struct IsClient(pub bool);

fn main() {
    let mut app = App::new();

    // Determine if running as client or server
    let args: Vec<String> = std::env::args().collect();
    let is_client = args.len() > 1 && args[1] == "client";

    // Add Bevy plugins
    app.add_plugins(DefaultPlugins);

    // Add networking plugins
    #[cfg(feature = "networking")]
    {
        if is_client {
            app.add_plugins(RenetClientPlugin);
            app.add_plugins(NetcodeClientPlugin);
        } else {
            app.add_plugins(RenetServerPlugin);
            app.add_plugins(NetcodeServerPlugin);
        }
    }

    // Add Lua plugin (auto-initializes all resources and systems)
    app.add_plugins(LuaSpawnPlugin);

    // Register networking components and methods (must be after LuaSpawnPlugin)
    #[cfg(feature = "networking")]
    app.add_systems(PreStartup, setup_networking);

    // Setup camera and register networking methods
    app.add_systems(Startup, setup);
    app.insert_resource(IsClient(is_client));
    app.add_systems(PostStartup, load_and_run_script);

    app.run();
}

#[cfg(feature = "networking")]
fn setup_networking(
    builder_registry: Res<ResourceBuilderRegistry>,
    lua_resource_registry: Res<LuaResourceRegistry>,
) {
    // Register networking constructors (for insert_resource in Lua)
    networking::register_networking_constructors(&builder_registry);
    
    // Register networking method bindings (for world:call_resource_method in Lua)
    networking::register_networking_methods(&lua_resource_registry);
    info!("✓ Networking components and methods registered");
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    info!("✓ Camera spawned");
}

fn load_and_run_script(
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<crate::script_entities::ScriptInstance>,
    script_registry: Res<crate::script_registry::ScriptRegistry>,
    is_client: Res<IsClient>,
) {
    // Set client mode flag
    lua_ctx.lua.globals().set("IS_CLIENT_MODE", is_client.0).unwrap();

    let mode = if is_client.0 { "CLIENT" } else { "SERVER" };
    info!("✓ Starting in {} mode", mode);

    // Load the pure Lua file upload script
    let script_path = std::path::PathBuf::from("assets/scripts/examples/file_upload.lua");
    match fs::read_to_string(&script_path) {
        Ok(script_content) => {
            info!("Loaded hot reload script: {:?}", script_path);
            match lua_ctx.execute_script(
                &script_content,
                "file_upload.lua",
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
