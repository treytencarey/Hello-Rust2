// Hello's networking module - game-specific networking constructors
// This provides resource builders for RenetClient/Server and transports
// All networking logic and message passing is handled in Lua via auto-generated method bindings

use bevy::prelude::*;
use mlua::prelude::*;
use std::net::{UdpSocket, SocketAddr};
use std::time::{Duration, SystemTime};

/// Register Lua method bindings for networking resources
/// This manually registers the methods that would be auto-generated if the build script worked
#[cfg(feature = "networking")]
pub fn register_networking_methods(registry: &bevy_lua_ecs::LuaResourceRegistry) {
    use bevy_replicon_renet::renet::{RenetClient, RenetServer};
    
    bevy::log::debug!("🔌 Registering networking methods for Lua...");
    
    // Register RenetClient methods
    registry.register_resource::<RenetClient, _>("RenetClient", |methods| {
        methods.add("send_message", |client: &mut RenetClient, _lua, (channel_id, message): (u8, LuaString)| {
            bevy::log::debug!("📤 RenetClient.send_message called: channel={}, size={}", channel_id, message.as_bytes().len());
            client.send_message(channel_id, message.as_bytes().to_vec());
            Ok(LuaValue::Nil)
        });
        
        methods.add("receive_message", |client: &mut RenetClient, lua, channel_id: u8| {
            if let Some(message) = client.receive_message(channel_id) {
                bevy::log::debug!("📥 RenetClient.receive_message: channel={}, size={}", channel_id, message.len());
                lua.create_string(&message).map(LuaValue::String)
            } else {
                Ok(LuaValue::Nil)
            }
        });
        
        methods.add("is_connected", |client: &mut RenetClient, _lua, ()| {
            let connected = client.is_connected();
            bevy::log::debug!("🔗 RenetClient.is_connected: {}", connected);
            Ok(connected)
        });
    });
    
    // Register RenetServer methods
    registry.register_resource::<RenetServer, _>("RenetServer", |methods| {
        methods.add("send_message", |server: &mut RenetServer, _lua, (client_id, channel_id, message): (u64, u8, LuaString)| {
            bevy::log::debug!("📤 RenetServer.send_message called: client={}, channel={}, size={}", client_id, channel_id, message.as_bytes().len());
            server.send_message(client_id, channel_id, message.as_bytes().to_vec());
            Ok(LuaValue::Nil)
        });
        
        methods.add("receive_message", |server: &mut RenetServer, lua, (client_id, channel_id): (u64, u8)| {
            if let Some(message) = server.receive_message(client_id, channel_id) {
                bevy::log::debug!("📥 RenetServer.receive_message: client={}, channel={}, size={}", client_id, channel_id, message.len());
                lua.create_string(&message).map(LuaValue::String)
            } else {
                Ok(LuaValue::Nil)
            }
        });
        
        methods.add("clients_id", |server: &mut RenetServer, lua, ()| {
            let clients: Vec<u64> = server.clients_id().into_iter().collect();
            bevy::log::debug!("👥 RenetServer.clients_id: {} clients", clients.len());
            lua.create_sequence_from(clients).map(LuaValue::Table)
        });
    });
    
    bevy::log::debug!("✅ Networking methods registered successfully");
}

/// Register networking resource constructors for Hello
/// These handle the complex setup (sockets, tokens, configs) that can't be auto-generated
#[cfg(feature = "networking")]
pub fn register_networking_constructors(registry: &bevy_lua_ecs::ResourceBuilderRegistry) {
    use bevy_renet::netcode::{
        ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
        ServerAuthentication, ServerConfig,
    };
    use bevy_replicon_renet::renet::{ConnectionConfig, RenetClient, RenetServer, ChannelConfig, SendType};
    use std::net::{IpAddr, Ipv4Addr};
    
    // Simple protocol ID for testing
    const PROTOCOL_ID: u64 = 0;
    // Fixed private key for local testing (in production, use proper key management)
    const PRIVATE_KEY: [u8; 32] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
        17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
    ];
    
    // Create a connection config with proper channels
    // Bevy Replicon uses channels 0 (replication) and 2 (init) by default
    fn create_connection_config() -> ConnectionConfig {
        ConnectionConfig {
            available_bytes_per_tick: 1024 * 1024,  // 1MB per tick
            client_channels_config: vec![
                // Channel 0: For replication updates
                ChannelConfig {
                    channel_id: 0,
                    max_memory_usage_bytes: 5 * 1024 * 1024,  // 5MB
                    send_type: SendType::ReliableOrdered {
                        resend_time: Duration::from_millis(200),
                    },
                },
                // Channel 1: For file uploads (our custom channel)
                ChannelConfig {
                    channel_id: 1,
                    max_memory_usage_bytes: 10 * 1024 * 1024,  // 10MB for large files
                    send_type: SendType::ReliableOrdered {
                        resend_time: Duration::from_millis(200),
                    },
                },
                // Channel 2: For replication init (bevy_replicon default)
                ChannelConfig {
                    channel_id: 2,
                    max_memory_usage_bytes: 5 * 1024 * 1024,  // 5MB
                    send_type: SendType::ReliableOrdered {
                        resend_time: Duration::from_millis(200),
                    },
                },
                // Channel 3: For player movement (our custom channel)
                ChannelConfig {
                    channel_id: 3,
                    max_memory_usage_bytes: 1 * 1024 * 1024,  // 1MB
                    send_type: SendType::ReliableOrdered {
                        resend_time: Duration::from_millis(200),
                    },
                },
                // Channel 4: For generic Lua events
                ChannelConfig {
                    channel_id: 4,
                    max_memory_usage_bytes: 1 * 1024 * 1024,  // 1MB
                    send_type: SendType::ReliableOrdered {
                        resend_time: Duration::from_millis(200),
                    },
                },
                // Channel 5: For on-demand asset delivery (scripts, images, etc.)
                ChannelConfig {
                    channel_id: 5,
                    max_memory_usage_bytes: 50 * 1024 * 1024,  // 50MB for large assets
                    send_type: SendType::ReliableOrdered {
                        resend_time: Duration::from_millis(200),
                    },
                },
            ],
            server_channels_config: vec![
                // Channel 0: For replication updates
                ChannelConfig {
                    channel_id: 0,
                    max_memory_usage_bytes: 5 * 1024 * 1024,  // 5MB
                    send_type: SendType::ReliableOrdered {
                        resend_time: Duration::from_millis(200),
                    },
                },
                // Channel 1: For file uploads (our custom channel)
                ChannelConfig {
                    channel_id: 1,
                    max_memory_usage_bytes: 10 * 1024 * 1024,  // 10MB for large files
                    send_type: SendType::ReliableOrdered {
                        resend_time: Duration::from_millis(200),
                    },
                },
                // Channel 2: For replication init (bevy_replicon default)
                ChannelConfig {
                    channel_id: 2,
                    max_memory_usage_bytes: 5 * 1024 * 1024,  // 5MB
                    send_type: SendType::ReliableOrdered {
                        resend_time: Duration::from_millis(200),
                    },
                },
                // Channel 3: For player movement (our custom channel)
                ChannelConfig {
                    channel_id: 3,
                    max_memory_usage_bytes: 1 * 1024 * 1024,  // 1MB
                    send_type: SendType::ReliableOrdered {
                        resend_time: Duration::from_millis(200),
                    },
                },
                // Channel 4: For generic Lua events
                ChannelConfig {
                    channel_id: 4,
                    max_memory_usage_bytes: 1 * 1024 * 1024,  // 1MB
                    send_type: SendType::ReliableOrdered {
                        resend_time: Duration::from_millis(200),
                    },
                },
                // Channel 5: For on-demand asset delivery (scripts, images, etc.)
                ChannelConfig {
                    channel_id: 5,
                    max_memory_usage_bytes: 50 * 1024 * 1024,  // 50MB for large assets
                    send_type: SendType::ReliableOrdered {
                        resend_time: Duration::from_millis(200),
                    },
                },
            ],
        }
    }
    
    /// OS-level utilities for networking
    fn bind_udp_socket(addr: &str) -> Result<UdpSocket, String> {
        UdpSocket::bind(addr).map_err(|e| format!("Failed to bind socket: {}", e))
    }
    
    fn current_time() -> Duration {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
    }
    
    fn parse_socket_addr(addr: &str) -> Result<SocketAddr, String> {
        addr.parse().map_err(|e| format!("Invalid address: {}", e))
    }
    
    // RenetServer constructor
    registry.register("RenetServer", |_lua, _data: LuaValue, world: &mut World| {
        world.insert_resource(RenetServer::new(create_connection_config()));
        Ok(())
    });
    
    // RenetClient constructor
    registry.register("RenetClient", |_lua, _data: LuaValue, world: &mut World| {
        world.insert_resource(RenetClient::new(create_connection_config()));
        Ok(())
    });
    
    // NetcodeServerTransport constructor
    registry.register("NetcodeServerTransport", |_lua, data: LuaValue, world: &mut World| {
        let table = data.as_table()
            .ok_or_else(|| LuaError::RuntimeError("Expected table for NetcodeServerTransport".to_string()))?;
        
        let port: u16 = table.get("port")?;
        let max_clients: usize = table.get("max_clients")?;
        
        // Bind to 0.0.0.0 to accept connections from any interface
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        let socket = bind_udp_socket(&bind_addr.to_string())
            .map_err(|e| LuaError::RuntimeError(e))?;
        
        let current_time = current_time();
        
        // Public address for local testing - use localhost
        // This is the address that will be embedded in connect tokens
        let public_addresses = vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port),
        ];
        
        let server_config = ServerConfig {
            current_time,
            max_clients,
            protocol_id: PROTOCOL_ID,
            public_addresses,
            authentication: ServerAuthentication::Secure { private_key: PRIVATE_KEY },
        };
        
        let transport = NetcodeServerTransport::new(server_config, socket)
            .map_err(|e| LuaError::RuntimeError(format!("Failed to create transport: {}", e)))?;
        
        world.insert_resource(transport);
        Ok(())
    });
    
    // NetcodeClientTransport constructor
    registry.register("NetcodeClientTransport", |_lua, data: LuaValue, world: &mut World| {
        let table = data.as_table()
            .ok_or_else(|| LuaError::RuntimeError("Expected table for NetcodeClientTransport".to_string()))?;
        
        let server_addr_str: String = table.get("server_addr")?;
        let port: u16 = table.get("port")?;
        
        let server_addr = parse_socket_addr(&format!("{}:{}", server_addr_str, port))
            .map_err(|e| LuaError::RuntimeError(e))?;
        
        let socket = bind_udp_socket("0.0.0.0:0")
            .map_err(|e| LuaError::RuntimeError(e))?;
        
        let current_time = current_time();
        
        let client_id = current_time.as_millis() as u64;
        
        // Generate a connect token for secure authentication
        use bevy_renet::netcode::ConnectToken;
        
        let connect_token = ConnectToken::generate(
            current_time,
            PROTOCOL_ID,
            300, // Token expires in 300 seconds (5 minutes)
            client_id,
            15,  // Timeout seconds
            vec![server_addr],
            None, // No user data
            &PRIVATE_KEY,
        ).map_err(|e| LuaError::RuntimeError(format!("Failed to generate connect token: {}", e)))?;
        
        let authentication = ClientAuthentication::Secure { connect_token };
        
        let transport = NetcodeClientTransport::new(current_time, authentication, socket)
            .map_err(|e| LuaError::RuntimeError(format!("Failed to create transport: {}", e)))?;
        
        world.insert_resource(transport);
        Ok(())
    });
}
