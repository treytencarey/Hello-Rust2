//! Networking plugin for unified peer (server/client/both) network asset integration
//!
//! Supports three modes:
//! - ServerOnly: Serves assets to clients, watches files for changes
//! - ClientOnly: Downloads assets from server, executes scripts
//! - Both: Full peer - serves and downloads assets

use bevy::prelude::*;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::Duration;

use bevy_renet::{RenetClientPlugin, RenetServerPlugin};
use bevy_renet::netcode::{NetcodeClientPlugin, NetcodeServerPlugin};
use bevy_renet::netcode::{NetcodeClientTransport, NetcodeServerTransport};
use bevy_renet::netcode::{ClientAuthentication, ServerAuthentication, ServerConfig, ConnectToken};
use bevy_replicon_renet::renet::{RenetClient, RenetServer, ConnectionConfig, ChannelConfig, SendType};

use crate::subscription_registry::{AssetSubscriptionRegistry, FileWatcherResource};
use crate::asset_server_delivery::ConnectedClients;
use crate::network_asset_integration::NetworkAssetPlugin;

/// Network modes for peer configuration
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum NetworkMode {
    /// Server only - serves assets, no script execution
    ServerOnly,
    /// Client only - downloads assets and runs scripts
    ClientOnly,
    /// Both server and client (full peer) - default for main binary
    #[default]
    Both,
}

/// Network configuration resource
#[derive(Resource, Clone, Debug)]
pub struct NetworkConfig {
    pub mode: NetworkMode,
    pub server_addr: String,
    pub server_port: u16,
    /// Private key for authentication (must match between server/clients)
    pub private_key: [u8; 32],
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            mode: NetworkMode::Both,
            server_addr: "127.0.0.1".to_string(),
            server_port: 5000,
            private_key: [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
                17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
            ],
        }
    }
}

/// Tracks files received from network to prevent broadcast loops
/// When a client receives a file update, it shouldn't trigger a hot reload broadcast
#[derive(Resource, Default)]
pub struct NetworkReceivedFiles {
    /// Files received from network with timestamp
    received: HashMap<String, std::time::Instant>,
}

impl NetworkReceivedFiles {
    /// Mark a file as received from network
    pub fn mark_received(&mut self, path: &str) {
        self.received.insert(path.to_string(), std::time::Instant::now());
    }
    
    /// Check if file was received from network recently (within duration)
    pub fn was_received_recently(&self, path: &str, within: Duration) -> bool {
        self.received.get(path)
            .map(|t| t.elapsed() < within)
            .unwrap_or(false)
    }
    
    /// Clean up old entries
    pub fn cleanup_old(&mut self, older_than: Duration) {
        self.received.retain(|_, t| t.elapsed() < older_than);
    }
}

/// Unified networking plugin that supports server, client, or both modes
pub struct HelloNetworkingPlugin {
    pub config: NetworkConfig,
}

impl Default for HelloNetworkingPlugin {
    fn default() -> Self {
        Self { config: NetworkConfig::default() }
    }
}

impl HelloNetworkingPlugin {
    pub fn server_only() -> Self {
        Self {
            config: NetworkConfig {
                mode: NetworkMode::ServerOnly,
                ..Default::default()
            }
        }
    }
    
    pub fn client_only() -> Self {
        Self {
            config: NetworkConfig {
                mode: NetworkMode::ClientOnly,
                ..Default::default()
            }
        }
    }
    
    pub fn with_config(config: NetworkConfig) -> Self {
        Self { config }
    }
}

impl Plugin for HelloNetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone());
        app.init_resource::<NetworkReceivedFiles>();
        
        // Register networking resource constructors and methods for Lua
        // Must run in Startup (before PostStartup when Lua scripts execute)
        app.add_systems(Startup, register_networking_registries);
        
        match self.config.mode {
            NetworkMode::ServerOnly => {
                info!("🖥️ Starting in SERVER ONLY mode");
                // Server plugins
                app.add_plugins(RenetServerPlugin);
                app.add_plugins(NetcodeServerPlugin);
                app.init_resource::<AssetSubscriptionRegistry>();
                app.insert_resource(FileWatcherResource::new());
                app.init_resource::<ConnectedClients>();
                app.add_systems(Startup, setup_server);
                app.add_systems(Update, (
                    crate::asset_server_delivery::handle_asset_requests_global,
                    crate::asset_server_delivery::broadcast_file_updates,
                    crate::asset_server_delivery::cleanup_disconnected_clients,
                    cleanup_received_files,
                ));
            }
            NetworkMode::ClientOnly => {
                info!("📱 Starting in CLIENT ONLY mode");
                // Client plugins
                app.add_plugins(RenetClientPlugin);
                app.add_plugins(NetcodeClientPlugin);
                app.add_plugins(NetworkAssetPlugin);
                app.add_systems(Startup, setup_client);
                app.add_systems(Update, cleanup_received_files);
            }
            NetworkMode::Both => {
                info!("🔄 Starting in PEER mode (server + client)");
                // Both server and client
                // RenetServerPlugin must come before client for proper resource ordering
                app.add_plugins(RenetServerPlugin);
                app.add_plugins(NetcodeServerPlugin);
                app.add_plugins(RenetClientPlugin);
                app.add_plugins(NetcodeClientPlugin);
                // NetworkAssetPlugin already adds server/client systems including:
                // - handle_asset_requests_global
                // - broadcast_file_updates  
                // - cleanup_disconnected_clients (via subscription registry init)
                // So we only add the Renet plugins and our filtered cleanup
                app.add_plugins(NetworkAssetPlugin);
                app.add_systems(Startup, (setup_server, setup_client));
                app.add_systems(Update, cleanup_received_files);
            }
        }
        
        info!("✓ Networking plugin loaded");
    }
}

/// Setup server networking
fn setup_server(mut commands: Commands, config: Res<NetworkConfig>) {
    let addr = format!("{}:{}", config.server_addr, config.server_port);
    let server_addr = addr.parse().expect("Invalid server address");
    
    let connection_config = ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: create_channel_config(),
        server_channels_config: create_channel_config(),
    };
    
    let server = RenetServer::new(connection_config);
    
    let socket = match UdpSocket::bind(server_addr) {
        Ok(s) => s,
        Err(e) => {
            error!("❌ Failed to bind server socket on {}: {}", addr, e);
            return;
        }
    };
    
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    
    let server_config = ServerConfig {
        current_time,
        max_clients: 10,
        protocol_id: 0,
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Secure { private_key: config.private_key },
    };
    
    let transport = NetcodeServerTransport::new(server_config, socket)
        .expect("Failed to create server transport");
    
    commands.insert_resource(server);
    commands.insert_resource(transport);
    
    info!("✓ Server started on {}", addr);
}

/// Setup client networking  
fn setup_client(mut commands: Commands, config: Res<NetworkConfig>) {
    let server_addr = format!("{}:{}", config.server_addr, config.server_port)
        .parse()
        .expect("Invalid server address");
    
    let connection_config = ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: create_channel_config(),
        server_channels_config: create_channel_config(),
    };
    
    let client = RenetClient::new(connection_config);
    
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind client socket");
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    
    let client_id = fastrand::u64(..);
    let token = ConnectToken::generate(
        current_time,
        0, // protocol_id
        300, // Token expires in 300 seconds
        client_id,
        15, // Timeout seconds
        vec![server_addr],
        None,
        &config.private_key,
    ).expect("Failed to generate connect token");
    
    let authentication = ClientAuthentication::Secure { connect_token: token };
    
    let transport = NetcodeClientTransport::new(current_time, authentication, socket)
        .expect("Failed to create client transport");
    
    commands.insert_resource(client);
    commands.insert_resource(transport);
    
    info!("✓ Client connecting to {}", server_addr);
}

/// Create channel configuration (shared between server and client)
fn create_channel_config() -> Vec<ChannelConfig> {
    vec![
        // Channels 0-4 for compatibility
        ChannelConfig {
            channel_id: 0,
            max_memory_usage_bytes: 5 * 1024 * 1024,
            send_type: SendType::ReliableOrdered { resend_time: Duration::from_millis(200) },
        },
        ChannelConfig {
            channel_id: 1,
            max_memory_usage_bytes: 10 * 1024 * 1024,
            send_type: SendType::ReliableOrdered { resend_time: Duration::from_millis(200) },
        },
        ChannelConfig {
            channel_id: 2,
            max_memory_usage_bytes: 5 * 1024 * 1024,
            send_type: SendType::ReliableOrdered { resend_time: Duration::from_millis(200) },
        },
        ChannelConfig {
            channel_id: 3,
            max_memory_usage_bytes: 1 * 1024 * 1024,
            send_type: SendType::ReliableOrdered { resend_time: Duration::from_millis(200) },
        },
        ChannelConfig {
            channel_id: 4,
            max_memory_usage_bytes: 1 * 1024 * 1024,
            send_type: SendType::ReliableOrdered { resend_time: Duration::from_millis(200) },
        },
        // Channel 5 for asset delivery (50MB)
        ChannelConfig {
            channel_id: 5,
            max_memory_usage_bytes: 50 * 1024 * 1024,
            send_type: SendType::ReliableOrdered { resend_time: Duration::from_millis(200) },
        },
    ]
}

/// System to clean up old received file entries
fn cleanup_received_files(mut received: ResMut<NetworkReceivedFiles>) {
    received.cleanup_old(Duration::from_secs(5));
}

/// Register networking resource constructors and methods for Lua
fn register_networking_registries(
    builder_registry: Res<bevy_lua_ecs::ResourceBuilderRegistry>,
    resource_registry: Res<bevy_lua_ecs::LuaResourceRegistry>,
) {
    crate::networking::register_networking_constructors(&builder_registry);
    crate::networking::register_networking_methods(&resource_registry);
    info!("✓ Registered networking Lua bindings (constructors + methods)");
}
