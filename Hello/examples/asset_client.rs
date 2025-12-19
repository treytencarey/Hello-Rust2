//! Asset Client Example with Lua Execution
//!
//! Run with: cargo run --example asset_client --features networking
//!
//! This connects to an asset server, downloads a script, and executes it with Lua.
//! The script can require other scripts which will also be downloaded.
//! The server should be running first via: cargo run --example asset_server --features networking

use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::net::UdpSocket;
use std::time::Duration;

use bevy_renet::netcode::{NetcodeClientTransport, ClientAuthentication, ConnectToken};
use bevy_renet::{RenetClientPlugin, netcode::NetcodeClientPlugin};
use bevy_replicon_renet::renet::{RenetClient, ConnectionConfig, ChannelConfig, SendType};

// Import from hello library crate
use hello::auto_resource_bindings;
use hello::network_asset_client;
use hello::network_asset_integration;

// Channel 5 is used for asset delivery
const ASSET_CHANNEL: u8 = 5;

/// The main script we want to download and execute
const MAIN_SCRIPT_PATH: &str = "scripts/examples/network_test_module.lua";

fn main() {
    println!("=== Asset Client with Lua Execution ===");
    println!("Connecting to asset server at 127.0.0.1:5000");
    println!("Will download and execute: {}", MAIN_SCRIPT_PATH);
    println!("");
    
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(auto_resource_bindings::LuaBindingsPlugin)
        .add_plugins(RenetClientPlugin)
        .add_plugins(NetcodeClientPlugin)
        .add_plugins(network_asset_integration::NetworkAssetPlugin)
        .init_resource::<TestState>()
        .add_systems(Startup, setup_client)
        .add_systems(Update, (
            wait_for_connection,
            run_script_when_ready,
        ).chain())
        .run();
}

#[derive(Resource, Default)]
struct TestState {
    connected: bool,
    request_queued: bool,
    script_executed: bool,
}

fn setup_client(mut commands: Commands) {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    
    // Create connection config with asset channel (channel 5)
    let connection_config = ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: create_channel_config(),
        server_channels_config: create_channel_config(),
    };
    
    let client = RenetClient::new(connection_config);
    
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind client socket");
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    
    // Fixed private key for testing (must match server's)
    let private_key: [u8; 32] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
        17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
    ];
    
    // Create connect token
    let client_id = fastrand::u64(..);
    let token = ConnectToken::generate(
        current_time,
        0, // protocol_id
        300, // Token expires in 300 seconds
        client_id,
        15, // Timeout seconds
        vec![server_addr],
        None,
        &private_key,
    ).expect("Failed to generate token");
    
    let authentication = ClientAuthentication::Secure { connect_token: token };
    
    let transport = NetcodeClientTransport::new(current_time, authentication, socket)
        .expect("Failed to create transport");
    
    commands.insert_resource(client);
    commands.insert_resource(transport);
    
    info!("✓ Client created, connecting to server...");
}

fn create_channel_config() -> Vec<ChannelConfig> {
    vec![
        // Channels 0-4 for compatibility
        ChannelConfig {
            channel_id: 0,
            max_memory_usage_bytes: 5 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(200),
            },
        },
        ChannelConfig {
            channel_id: 1,
            max_memory_usage_bytes: 10 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(200),
            },
        },
        ChannelConfig {
            channel_id: 2,
            max_memory_usage_bytes: 5 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(200),
            },
        },
        ChannelConfig {
            channel_id: 3,
            max_memory_usage_bytes: 1 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(200),
            },
        },
        ChannelConfig {
            channel_id: 4,
            max_memory_usage_bytes: 1 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(200),
            },
        },
        // Channel 5 for asset delivery (50MB)
        ChannelConfig {
            channel_id: 5,
            max_memory_usage_bytes: 50 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(200),
            },
        },
    ]
}

/// Once connected, queue the main script download via PendingAssetRequests
/// This uses the same path that nested require() calls use
fn wait_for_connection(
    client: Option<Res<RenetClient>>,
    mut state: ResMut<TestState>,
    pending_requests: Res<network_asset_client::PendingAssetRequests>,
    lua_ctx: Option<Res<LuaScriptContext>>,
) {
    if state.request_queued {
        return;
    }
    
    let Some(client) = client else { return };
    let Some(lua_ctx) = lua_ctx else { return };
    
    if client.is_connected() && !state.connected {
        info!("✓ Connected to server!");
        state.connected = true;
    }
    
    if state.connected {
        // Queue the main script download using the same mechanism as require()
        // This registers a pending download so process_download_requests will pick it up
        info!("📥 Queuing main script download: {}", MAIN_SCRIPT_PATH);
        
        // Register in script_cache so process_download_requests picks it up
        // We use a nil coroutine key since we'll check completion manually
        let nil_key = lua_ctx.lua.create_registry_value(mlua::Value::Nil)
            .expect("Failed to create nil registry value");
        lua_ctx.script_cache.register_pending_download_coroutine(
            MAIN_SCRIPT_PATH.to_string(),
            std::sync::Arc::new(nil_key),
            0, // instance_id - we'll set this when we execute
            false, // is_binary=false - this is a script
            None, // No context - this is the root script
            true, // should_subscribe=true for main script to enable hot reload
        );
        
        state.request_queued = true;
        info!("📋 Main script queued for download via NetworkAssetPlugin");
    }
}

/// Check if the main script is ready (downloaded or locally available)
/// This polls the PendingAssetRequests for completion
fn run_script_when_ready(
    mut state: ResMut<TestState>,
    pending_requests: Res<network_asset_client::PendingAssetRequests>,
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<ScriptInstance>,
    script_registry: Res<ScriptRegistry>,
) {
    if !state.request_queued || state.script_executed {
        return;
    }
    
    // Check if the main script download is complete
    let script_content = if pending_requests.is_completed(MAIN_SCRIPT_PATH) {
        // Take downloaded data
        if let Some(data) = pending_requests.take_completed(MAIN_SCRIPT_PATH) {
            match String::from_utf8(data) {
                Ok(content) => {
                    info!("✅ Main script downloaded successfully");
                    Some(content)
                }
                Err(e) => {
                    error!("❌ Downloaded script is not valid UTF-8: {}", e);
                    return;
                }
            }
        } else {
            return; // Data was taken by someone else?
        }
    } else if pending_requests.is_up_to_date(MAIN_SCRIPT_PATH) {
        // Script is confirmed up-to-date, load from disk
        info!("✓ Main script is up-to-date, loading from disk");
        pending_requests.mark_up_to_date(MAIN_SCRIPT_PATH);
        
        match bevy_lua_ecs::script_cache::load_module_source(MAIN_SCRIPT_PATH) {
            Ok((content, _)) => Some(content),
            Err(e) => {
                error!("❌ Failed to load local script: {}", e);
                return;
            }
        }
    } else {
        // Not ready yet, check if we have it locally as fallback
        // (This handles the case where NetworkAssetPlugin hasn't processed it yet)
        if let Ok((content, _)) = bevy_lua_ecs::script_cache::load_module_source(MAIN_SCRIPT_PATH) {
            // Local file exists - use it and let hot reload handle updates
            info!("📂 Using local script (network check in progress)");
            Some(content)
        } else {
            // Still waiting for download
            return;
        }
    };
    
    let Some(script_content) = script_content else {
        return;
    };
    
    // Mark as executed so we don't run again
    state.script_executed = true;
    
    info!("🚀 Executing main script...");
    
    let script_path = std::path::PathBuf::from(format!("assets/{}", MAIN_SCRIPT_PATH));
    match lua_ctx.execute_script(
        &script_content,
        MAIN_SCRIPT_PATH,
        script_path,
        &script_instance,
        &script_registry,
    ) {
        Ok(instance_id) => {
            info!("✅ Script executed with instance ID: {}", instance_id);
            info!("   (If script yields for nested downloads, NetworkAssetPlugin will resume it)");
        }
        Err(e) => {
            error!("❌ Failed to execute script: {}", e);
        }
    }
}

