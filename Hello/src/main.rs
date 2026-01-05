//! Hello - Bevy + Lua game framework
//! 
//! Run with: cargo run -- --help
//! 
//! Examples:
//!   cargo run -- --demo physics
//!   cargo run -- --script scripts/examples/custom.lua
//!   cargo run -- --list
//!   cargo run -- --network server
//!   cargo run -- --network client --server-addr 192.168.1.100

use bevy::prelude::*;
use bevy::asset::{AssetPlugin, AssetMode};
use clap::Parser;

// Import from library crate
use hello::plugins::{HelloCorePlugin, MainScripts};

#[cfg(feature = "physics")]
use hello::plugins::HelloPhysicsPlugin;

#[cfg(feature = "networking")]
use hello::plugins::{HelloNetworkingPlugin, NetworkConfig, NetworkMode};

#[cfg(feature = "tiled")]
use hello::plugins::HelloTiledPlugin;

#[cfg(feature = "ufbx")]
use hello::plugins::HelloUfbxPlugin;

#[cfg(feature = "bevy_mod_xr")]
use hello::plugins::VrInputPlugin;

#[derive(Parser)]
#[command(name = "hello", about = "Bevy + Lua game framework")]
struct Args {
    /// Demo to run (physics, tilemap, ui_3d, rtt, networking)
    #[arg(short, long, value_name = "NAME")]
    demo: Option<String>,
    
    /// Run a specific script directly (path relative to assets/)
    #[arg(short, long, value_name = "PATH")]
    script: Option<String>,
    
    /// List available demos
    #[arg(long)]
    list: bool,
    
    /// Don't spawn the default 2D camera
    #[arg(long)]
    no_camera: bool,
    
    /// Network mode: server, client, or both (default: both when networking enabled)
    #[arg(long, value_name = "MODE")]
    network: Option<String>,
    
    /// Server address for client connections
    #[arg(long, default_value = "127.0.0.1")]
    server_addr: String,
    
    /// Server port
    #[arg(long, default_value = "5000")]
    port: u16,
    
    /// Enable VR mode (requires bevy_mod_xr feature)
    #[arg(long)]
    vr: bool,
}

/// Demo definitions: (name, script_path, description, required_feature)
const DEMOS: &[(&str, &str, &str, Option<&str>)] = &[
    ("physics", "scripts/examples/physics.lua", "2D physics with Rapier", Some("physics")),
    ("tilemap", "scripts/examples/tilemap.lua", "Tiled map loading", Some("tiled")),
    ("ui_3d", "scripts/examples/ui_3d_plane.lua", "UI rendered to 3D plane", None),
    ("rtt", "scripts/examples/render_ui_to_texture.lua", "Render UI to texture", None),
    ("networking", "scripts/main.lua", "Network peer (client mode)", Some("networking")),
    ("button", "scripts/examples/button.lua", "Simple button example", None),
    ("basic", "scripts/spawn_text.lua", "Basic text spawning", None),
    ("ufbx", "scripts/examples/ufbx.lua", "FBX model loading with ufbx", Some("ufbx")),
];

fn main() {
    let args = Args::parse();
    
    // Handle --list
    if args.list {
        println!("Available demos:\n");
        for (name, _script, desc, feature) in DEMOS {
            let available = match feature {
                None => true,
                Some("physics") => cfg!(feature = "physics"),
                Some("tiled") => cfg!(feature = "tiled"),
                Some("networking") => cfg!(feature = "networking"),
                Some("ufbx") => cfg!(feature = "ufbx"),
                _ => false,
            };
            let status = if available { "✓" } else { "✗" };
            println!("  {} {:12} - {}", status, name, desc);
        }
        println!("\nNetwork modes:");
        println!("  cargo run -- --network server   Start as server only");
        println!("  cargo run -- --network client   Start as client only");
        println!("  cargo run -- --network both     Start as full peer (default)");
        println!("\nRun with: cargo run -- --demo <NAME>");
        return;
    }
    
    // Show help if no args
    if args.demo.is_none() && args.script.is_none() && args.network.is_none() {
        println!("Hello - Bevy + Lua game framework\n");
        println!("Usage:");
        println!("  cargo run -- --demo <NAME>       Run a demo");
        println!("  cargo run -- --script <PATH>     Run a specific script");
        println!("  cargo run -- --network <MODE>    Network mode (server/client/both)");
        println!("  cargo run -- --list              List available demos");
        println!("\nRun 'cargo run -- --help' for more options.");
        return;
    }
    
    let mut app = App::new();
    
    // Add Bevy default plugins, configure AssetPlugin to allow absolute paths
    // (FBX files often embed absolute texture paths from export environment)
    // When VR mode is enabled, use XR plugins instead of standard DefaultPlugins
    #[cfg(feature = "bevy_mod_xr")]
    if args.vr {
        app.add_plugins(bevy_mod_openxr::add_xr_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    mode: AssetMode::Unprocessed,
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
                    ..default()
                })
                .set(bevy::pbr::PbrPlugin {
                    prepass_enabled: false,
                    ..default()
                })
        ));
        app.add_plugins(bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin);
        app.add_plugins(VrInputPlugin);
    } else {
        app.add_plugins(DefaultPlugins.set(AssetPlugin {
            mode: AssetMode::Unprocessed,
            meta_check: bevy::asset::AssetMetaCheck::Never,
            unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
            ..default()
        }));
    }
    
    #[cfg(not(feature = "bevy_mod_xr"))]
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
            mode: AssetMode::Unprocessed,
            meta_check: bevy::asset::AssetMetaCheck::Never,
            // Allow absolute paths embedded in FBX files to be loaded
            // Security note: Be cautious with this in production - only enable for trusted content
            unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
            ..default()
    }));
    
    // Add text input support (editable text fields with clipboard, selection)
    app.add_plugins(bevy_ui_text_input::TextInputPlugin);
    // Add Lua wrapper for text input spawning
    app.add_plugins(hello::text_input::LuaTextInputPlugin);
    
    // Add core plugin
    app.add_plugins(HelloCorePlugin {});

    // Add ufbx plugin if feature enabled
    #[cfg(feature = "ufbx")]
    app.add_plugins(HelloUfbxPlugin);
    
    // Parse network mode
    #[cfg(feature = "networking")]
    let network_mode = match args.network.as_deref() {
        Some("server") => Some(NetworkMode::ServerOnly),
        Some("client") => Some(NetworkMode::ClientOnly),
        Some("both") => Some(NetworkMode::Both),
        None if args.demo.as_deref() == Some("networking") => Some(NetworkMode::Both),
        _ => None,
    };
    
    // Determine which script(s) to run based on network mode
    let script_paths: Vec<String> = if let Some(ref script) = args.script {
        vec![script.clone()]
    } else if let Some(ref demo_name) = args.demo {
        // Find the demo
        DEMOS.iter()
            .find(|(name, _, _, _)| *name == demo_name.as_str())
            .map(|(_, script, _, _)| vec![script.to_string()])
            .unwrap_or_default()
    } else {
        // Network mode specific scripts
        #[cfg(feature = "networking")]
        match network_mode {
            Some(NetworkMode::ServerOnly) => vec!["scripts/server/main.lua".to_string()],
            Some(NetworkMode::ClientOnly) => vec!["scripts/main.lua".to_string()],
            Some(NetworkMode::Both) => vec![
                "scripts/server/main.lua".to_string(),
                "scripts/main.lua".to_string(),
            ],
            None => vec![],
        }
        #[cfg(not(feature = "networking"))]
        vec![]
    };
    
    // Add feature-specific plugins based on demo or network mode
    #[cfg(feature = "networking")]
    if let Some(mode) = network_mode {
        let config = NetworkConfig {
            mode,
            server_addr: args.server_addr.clone(),
            server_port: args.port,
            ..Default::default()
        };
        app.add_plugins(HelloNetworkingPlugin::with_config(config));
    }
    
    if let Some(ref demo_name) = args.demo {
        match demo_name.as_str() {
            "physics" => {
                #[cfg(feature = "physics")]
                app.add_plugins(HelloPhysicsPlugin);
                #[cfg(not(feature = "physics"))]
                error!("Physics demo requires 'physics' feature");
            }
            "tilemap" => {
                #[cfg(feature = "tiled")]
                app.add_plugins(HelloTiledPlugin);
                #[cfg(not(feature = "tiled"))]
                error!("Tilemap demo requires 'tiled' feature");
            }
            "networking" => {
                // Already handled above via network_mode
            }
            _ => {}
        }
    }
    
    // Set initial scripts if we have any
    if !script_paths.is_empty() {
        app.insert_resource(MainScripts(script_paths));
    }
    
    app.run();
}
