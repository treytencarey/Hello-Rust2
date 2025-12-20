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
use clap::Parser;

// Import from library crate
use hello::plugins::{HelloCorePlugin, MainScript};

#[cfg(feature = "physics")]
use hello::plugins::HelloPhysicsPlugin;

#[cfg(feature = "networking")]
use hello::plugins::{HelloNetworkingPlugin, NetworkConfig, NetworkMode};

#[cfg(feature = "tiled")]
use hello::plugins::HelloTiledPlugin;

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
    
    // Add Bevy default plugins
    app.add_plugins(DefaultPlugins);
    
    // Add core plugin
    app.add_plugins(HelloCorePlugin {
        spawn_camera_2d: !args.no_camera,
    });
    
    // Parse network mode
    #[cfg(feature = "networking")]
    let network_mode = match args.network.as_deref() {
        Some("server") => Some(NetworkMode::ServerOnly),
        Some("client") => Some(NetworkMode::ClientOnly),
        Some("both") => Some(NetworkMode::Both),
        None if args.demo.as_deref() == Some("networking") => Some(NetworkMode::Both),
        _ => None,
    };
    
    // Determine which script to run based on network mode
    let script_path = if let Some(ref script) = args.script {
        Some(script.clone())
    } else if let Some(ref demo_name) = args.demo {
        // Find the demo
        DEMOS.iter()
            .find(|(name, _, _, _)| *name == demo_name.as_str())
            .map(|(_, script, _, _)| script.to_string())
    } else {
        // Network mode specific scripts
        #[cfg(feature = "networking")]
        match network_mode {
            Some(NetworkMode::ServerOnly) => Some("scripts/server/main.lua".to_string()),
            Some(NetworkMode::ClientOnly) | Some(NetworkMode::Both) => Some("scripts/main.lua".to_string()),
            None => None,
        }
        #[cfg(not(feature = "networking"))]
        None
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
    
    // Set initial script if we have one
    if let Some(path) = script_path {
        app.insert_resource(MainScript(path));
    }
    
    app.run();
}
