//! Hello - Bevy + Lua game framework
//! 
//! Run with: cargo run -- --help
//! 
//! Examples:
//!   cargo run -- --demo physics
//!   cargo run -- --script scripts/examples/custom.lua
//!   cargo run -- --list

use bevy::prelude::*;
use clap::Parser;

// Import from library crate
use hello::plugins::{HelloCorePlugin, InitialScript};

#[cfg(feature = "physics")]
use hello::plugins::HelloPhysicsPlugin;

#[cfg(feature = "networking")]
use hello::plugins::HelloNetworkingPlugin;

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
}

/// Demo definitions: (name, script_path, description, required_feature)
const DEMOS: &[(&str, &str, &str, Option<&str>)] = &[
    ("physics", "scripts/examples/physics.lua", "2D physics with Rapier", Some("physics")),
    ("tilemap", "scripts/examples/tilemap.lua", "Tiled map loading", Some("tiled")),
    ("ui_3d", "scripts/examples/ui_3d_plane.lua", "UI rendered to 3D plane", None),
    ("rtt", "scripts/examples/render_ui_to_texture.lua", "Render UI to texture", None),
    ("networking", "scripts/examples/network_client_test.lua", "Network asset client", Some("networking")),
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
        println!("\nRun with: cargo run -- --demo <NAME>");
        return;
    }
    
    // Show help if no args
    if args.demo.is_none() && args.script.is_none() {
        println!("Hello - Bevy + Lua game framework\n");
        println!("Usage:");
        println!("  cargo run -- --demo <NAME>     Run a demo");
        println!("  cargo run -- --script <PATH>   Run a specific script");
        println!("  cargo run -- --list            List available demos");
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
    
    // Determine which script to run
    let script_path = if let Some(ref script) = args.script {
        Some(script.clone())
    } else if let Some(ref demo_name) = args.demo {
        // Find the demo
        DEMOS.iter()
            .find(|(name, _, _, _)| *name == demo_name.as_str())
            .map(|(_, script, _, _)| script.to_string())
    } else {
        None
    };
    
    // Add feature-specific plugins based on demo
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
                #[cfg(feature = "networking")]
                app.add_plugins(HelloNetworkingPlugin);
                #[cfg(not(feature = "networking"))]
                error!("Networking demo requires 'networking' feature");
            }
            _ => {}
        }
    }
    
    // Set initial script if we have one
    if let Some(path) = script_path {
        app.insert_resource(InitialScript(path));
    }
    
    app.run();
}
