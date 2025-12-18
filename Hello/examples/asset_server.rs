//! Asset Server Example
//!
//! Run with: cargo run --example asset_server --features networking
//!
//! This starts an asset server that serves files from the assets/ directory.
//! Clients can connect and request assets via Renet channel 5.

use bevy::prelude::*;
use std::net::UdpSocket;
use std::time::Duration;

use bevy_renet::netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::{RenetServerPlugin, netcode::NetcodeServerPlugin};
use bevy_replicon_renet::renet::{RenetServer, ConnectionConfig, ChannelConfig, SendType};

// Channel 5 is used for asset delivery
const ASSET_CHANNEL: u8 = 5;

fn main() {
    println!("=== Asset Server ===");
    println!("Serving assets from assets/ directory");
    println!("Listening on 127.0.0.1:5000");
    println!("Press Ctrl+C to stop");
    println!("");
    
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(bevy::log::LogPlugin::default())
        .add_plugins(RenetServerPlugin)
        .add_plugins(NetcodeServerPlugin)
        .add_systems(Startup, setup_server)
        .add_systems(Update, (
            handle_asset_requests,
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

/// Compute FNV-1a hash of data (fast, good for change detection)
fn compute_hash(data: &[u8]) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    
    let mut hash = FNV_OFFSET;
    for byte in data {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{:016x}", hash)
}

/// Magic bytes to identify encrypted data (must match Hello's "ASET")
const ENCRYPTION_MAGIC: [u8; 4] = [0xAE, 0x53, 0x45, 0x54];

/// XOR encryption key (must match Hello's)
const ENCRYPTION_KEY: [u8; 32] = [
    0x1A, 0x2B, 0x3C, 0x4D, 0x5E, 0x6F, 0x70, 0x81,
    0x92, 0xA3, 0xB4, 0xC5, 0xD6, 0xE7, 0xF8, 0x09,
    0x10, 0x21, 0x32, 0x43, 0x54, 0x65, 0x76, 0x87,
    0x98, 0xA9, 0xBA, 0xCB, 0xDC, 0xED, 0xFE, 0x0F,
];

/// Simple XOR encryption (must match Hello's encrypt_data)
fn encrypt_data(data: &[u8]) -> Vec<u8> {
    let mut encrypted = Vec::with_capacity(ENCRYPTION_MAGIC.len() + data.len());
    encrypted.extend_from_slice(&ENCRYPTION_MAGIC);
    
    for (i, byte) in data.iter().enumerate() {
        encrypted.push(byte ^ ENCRYPTION_KEY[i % ENCRYPTION_KEY.len()]);
    }
    
    encrypted
}

/// Asset type enum - must match Hello's AssetType
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
enum AssetType {
    Script,
    Image,
    Binary,
}

/// Asset request message - must match Hello's AssetRequestMessage
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct AssetRequest {
    request_id: u64,
    path: String,
    asset_type: AssetType,
    /// Hash of local file (for up-to-date check)
    local_hash: Option<String>,
}

/// Asset response message - must match Hello's AssetResponseMessage
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct AssetResponse {
    request_id: u64,
    path: String,
    is_up_to_date: bool,
    server_hash: Option<String>,
    chunk_index: u32,
    total_chunks: u32,
    total_size: usize,
    data: Vec<u8>,
    error: Option<String>,
}

fn handle_asset_requests(mut server: Option<ResMut<RenetServer>>) {
    let Some(ref mut server) = server else { return };
    
    let client_ids: Vec<u64> = server.clients_id().into_iter().collect();
    
    for client_id in client_ids {
        while let Some(message_bytes) = server.receive_message(client_id, ASSET_CHANNEL) {
            match bincode::deserialize::<AssetRequest>(&message_bytes) {
                Ok(request) => {
                    info!("📥 [SERVER] Asset request from client {}: {} (local_hash: {:?})", 
                        client_id, request.path, request.local_hash);
                    
                    // Try to read file from assets/scripts/ first, then assets/
                    let file_path = std::path::Path::new("assets/scripts").join(&request.path);
                    let data = std::fs::read(&file_path)
                        .or_else(|_| std::fs::read(std::path::Path::new("assets").join(&request.path)));
                    
                    let response = match data {
                        Ok(data) => {
                            // Compute server hash
                            let server_hash = compute_hash(&data);
                            let data_len = data.len();
                            
                            // Check if client's hash matches (up-to-date)
                            if let Some(ref client_hash) = request.local_hash {
                                if client_hash == &server_hash {
                                    info!("✓ [SERVER] Asset '{}' is UP-TO-DATE (hash: {})", request.path, server_hash);
                                    AssetResponse {
                                        request_id: request.request_id,
                                        path: request.path,
                                        is_up_to_date: true,
                                        server_hash: Some(server_hash),
                                        chunk_index: 0,
                                        total_chunks: 0,
                                        total_size: 0,
                                        data: vec![],
                                        error: None,
                                    }
                                } else {
                                    info!("📤 [SERVER] Hash MISMATCH - sending {} bytes for '{}' (client: {}, server: {})", 
                                        data_len, request.path, client_hash, server_hash);
                                    let encrypted = encrypt_data(&data);
                                    AssetResponse {
                                        request_id: request.request_id,
                                        path: request.path,
                                        is_up_to_date: false,
                                        server_hash: Some(server_hash),
                                        chunk_index: 0,
                                        total_chunks: 1,
                                        total_size: data_len,
                                        data: encrypted,
                                        error: None,
                                    }
                                }
                            } else {
                                info!("📤 [SERVER] No local hash - sending {} bytes for '{}'", data_len, request.path);
                                let encrypted = encrypt_data(&data);
                                AssetResponse {
                                    request_id: request.request_id,
                                    path: request.path,
                                    is_up_to_date: false,
                                    server_hash: Some(server_hash),
                                    chunk_index: 0,
                                    total_chunks: 1,
                                    total_size: data_len,
                                    data: encrypted,
                                    error: None,
                                }
                            }
                        }
                        Err(e) => {
                            warn!("❌ [SERVER] File not found: {} ({})", request.path, e);
                            AssetResponse {
                                request_id: request.request_id,
                                path: request.path,
                                is_up_to_date: false,
                                server_hash: None,
                                chunk_index: 0,
                                total_chunks: 0,
                                total_size: 0,
                                data: vec![],
                                error: Some(format!("File not found: {}", e)),
                            }
                        }
                    };
                    
                    if let Ok(response_bytes) = bincode::serialize(&response) {
                        server.send_message(client_id, ASSET_CHANNEL, bytes::Bytes::from(response_bytes));
                    }
                }
                Err(e) => {
                    warn!("❌ [SERVER] Failed to deserialize request: {}", e);
                }
            }
        }
    }
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
