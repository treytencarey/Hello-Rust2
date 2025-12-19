//! Asset Server Example with Full File Sync Subscription
//!
//! Run with: cargo run --example asset_server --features networking
//!
//! This starts an asset server that:
//! - Serves files from the assets/ directory
//! - Handles asset requests from clients via Renet channel 5
//! - Watches subscribed files for changes and broadcasts updates to clients
//!
//! The server should be running first, then clients can connect with:
//! cargo run --example asset_client --features networking

use bevy::prelude::*;
use std::net::UdpSocket;
use std::time::Duration;

use bevy_renet::netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::{RenetServerPlugin, netcode::NetcodeServerPlugin};
use bevy_replicon_renet::renet::{RenetServer, ConnectionConfig, ChannelConfig, SendType};

// Import from hello library for full file sync subscription support
use hello::subscription_registry::{AssetSubscriptionRegistry, FileWatcherResource};
use hello::asset_server_delivery::{handle_asset_requests_global, broadcast_file_updates};

fn main() {
    println!("=== Asset Server with File Sync ===");
    println!("Serving assets from assets/ directory");
    println!("Listening on 127.0.0.1:5000");
    println!("File sync subscriptions enabled!");
    println!("Press Ctrl+C to stop");
    println!("");
    
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(bevy::log::LogPlugin::default())
        .add_plugins(RenetServerPlugin)
        .add_plugins(NetcodeServerPlugin)
        // Initialize server resources for file sync
        .init_resource::<AssetSubscriptionRegistry>()
        .insert_resource(FileWatcherResource::new())
        .init_resource::<hello::asset_server_delivery::ConnectedClients>()
        .add_systems(Startup, setup_server)
        .add_systems(Update, (
            // Use library's systems for full file sync support
            handle_asset_requests_global,
            broadcast_file_updates,
            hello::asset_server_delivery::cleanup_disconnected_clients,
            log_connections,
        ))
        .run();
}

fn setup_server(mut commands: Commands) {
    let port = 5000u16;
    let server_addr = format!("127.0.0.1:{}", port).parse().unwrap();
    
    // Create connection config with asset channel (channel 5)
    let connection_config = ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: create_channel_config(),
        server_channels_config: create_channel_config(),
    };
    
    let server = RenetServer::new(connection_config);
    
    let socket = UdpSocket::bind(server_addr).expect("Failed to bind socket");
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    
    // Fixed private key for testing
    let private_key: [u8; 32] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
        17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
    ];
    
    let server_config = ServerConfig {
        current_time,
        max_clients: 10,
        protocol_id: 0,
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Secure { private_key },
    };
    
    let transport = NetcodeServerTransport::new(server_config, socket)
        .expect("Failed to create transport");
    
    commands.insert_resource(server);
    commands.insert_resource(transport);
    
    info!("✓ Asset server started on {}", server_addr);
}

fn create_channel_config() -> Vec<ChannelConfig> {
    vec![
        // Channels 0-4 for compatibility with other networking
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

fn log_connections(server: Option<Res<RenetServer>>) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static LAST_COUNT: AtomicUsize = AtomicUsize::new(0);
    
    if let Some(server) = server {
        let clients = server.clients_id();
        let count = clients.len();
        let last = LAST_COUNT.load(Ordering::Relaxed);
        
        if count != last {
            if count > last {
                info!("📥 Client connected! Total clients: {}", count);
            } else {
                info!("📤 Client disconnected. Total clients: {}", count);
            }
            LAST_COUNT.store(count, Ordering::Relaxed);
        }
    }
}
