use bevy::prelude::*;
use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::net::{UdpSocket, SocketAddr};
use std::time::{Duration, SystemTime};

/// Generic OS-level utilities for Lua (reusable across ANY game)
/// These wrap OS operations that Lua cannot do directly
pub struct OsUtilities;

impl OsUtilities {
    /// Bind a UDP socket to the given address
    /// Generic infrastructure - works for any game needing UDP sockets
    pub fn bind_udp_socket(addr: &str) -> Result<UdpSocket, String> {
        UdpSocket::bind(addr).map_err(|e| format!("Failed to bind socket: {}", e))
    }
    
    /// Get current system time as Duration since UNIX_EPOCH
    /// Generic infrastructure - works for any game needing timestamps
    pub fn current_time() -> Duration {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
    }
    
    /// Parse a socket address from string
    /// Generic infrastructure - works for any game needing network addresses
    pub fn parse_socket_addr(addr: &str) -> Result<SocketAddr, String> {
        addr.parse().map_err(|e| format!("Invalid address: {}", e))
    }
}

/// Type for constructor functions that create resources from Lua data
/// Takes Lua context, data table, and returns a boxed reflected resource
type ConstructorFn = Arc<dyn Fn(&Lua, LuaValue) -> LuaResult<Box<dyn Reflect>> + Send + Sync>;

/// Registry for resource constructor functions
/// This is GENERIC - works for ANY resource type that needs construction
#[derive(Resource, Clone)]
pub struct ResourceConstructorRegistry {
    constructors: Arc<Mutex<HashMap<String, ConstructorFn>>>,
}

impl Default for ResourceConstructorRegistry {
    fn default() -> Self {
        Self {
            constructors: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl ResourceConstructorRegistry {
    /// Register a constructor function for a resource type
    /// The constructor should create a reflected resource from Lua data
    pub fn register<F>(&self, name: impl Into<String>, constructor: F)
    where
        F: Fn(&Lua, LuaValue) -> LuaResult<Box<dyn Reflect>> + Send + Sync + 'static,
    {
        self.constructors
            .lock()
            .unwrap()
            .insert(name.into(), Arc::new(constructor));
    }
    
    /// Try to construct a resource from Lua data
    /// Returns None if no constructor is registered for this type
    pub fn try_construct(
        &self,
        lua: &Lua,
        name: &str,
        data: LuaValue,
    ) -> Option<LuaResult<Box<dyn Reflect>>> {
        let constructors = self.constructors.lock().unwrap();
        constructors.get(name).map(|constructor| constructor(lua, data))
    }
    
    /// Check if a constructor is registered for a resource type
    pub fn has_constructor(&self, name: &str) -> bool {
        self.constructors.lock().unwrap().contains_key(name)
    }
}

/// Register networking resource constructors (generic library infrastructure)
/// These are NOT game-specific - they work for ANY game using bevy_replicon
/// Note: Uses ResourceBuilderRegistry because networking types don't implement Reflect
#[cfg(feature = "networking")]
pub fn register_networking_constructors(registry: &crate::resource_builder::ResourceBuilderRegistry) {
    use bevy_renet::netcode::{
        ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
        ServerAuthentication, ServerConfig,
    };
    use bevy_replicon_renet::renet::{ConnectionConfig, RenetClient, RenetServer};
    use std::net::{IpAddr, Ipv4Addr};
    
    // RenetServer constructor - generic infrastructure
    registry.register("RenetServer", |_lua, _data: LuaValue, world: &mut World| {
        world.insert_resource(RenetServer::new(ConnectionConfig::default()));
        Ok(())
    });
    
    // RenetClient constructor - generic infrastructure
    registry.register("RenetClient", |_lua, _data: LuaValue, world: &mut World| {
        world.insert_resource(RenetClient::new(ConnectionConfig::default()));
        Ok(())
    });
    
    // NetcodeServerTransport constructor - generic infrastructure using OS utilities
    registry.register("NetcodeServerTransport", |_lua, data: LuaValue, world: &mut World| {
        let table = data.as_table()
            .ok_or_else(|| LuaError::RuntimeError("Expected table for NetcodeServerTransport".to_string()))?;
        
        let port: u16 = table.get("port")?;
        let max_clients: usize = table.get("max_clients")?;
        
        // Use generic OS utilities
        let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        let socket = OsUtilities::bind_udp_socket(&server_addr.to_string())
            .map_err(|e| LuaError::RuntimeError(e))?;
        
        let current_time = OsUtilities::current_time();
        
        let server_config = ServerConfig {
            current_time,
            max_clients,
            protocol_id: 0,
            public_addresses: vec![server_addr],
            authentication: ServerAuthentication::Unsecure,
        };
        
        let transport = NetcodeServerTransport::new(server_config, socket)
            .map_err(|e| LuaError::RuntimeError(format!("Failed to create transport: {}", e)))?;
        
        world.insert_resource(transport);
        Ok(())
    });
    
    // NetcodeClientTransport constructor - generic infrastructure using OS utilities
    registry.register("NetcodeClientTransport", |_lua, data: LuaValue, world: &mut World| {
        let table = data.as_table()
            .ok_or_else(|| LuaError::RuntimeError("Expected table for NetcodeClientTransport".to_string()))?;
        
        let server_addr_str: String = table.get("server_addr")?;
        let port: u16 = table.get("port")?;
        
        // Use generic OS utilities
        let server_addr = OsUtilities::parse_socket_addr(&format!("{}:{}", server_addr_str, port))
            .map_err(|e| LuaError::RuntimeError(e))?;
        
        let socket = OsUtilities::bind_udp_socket("0.0.0.0:0")
            .map_err(|e| LuaError::RuntimeError(e))?;
        
        let current_time = OsUtilities::current_time();
        
        let client_id = current_time.as_millis() as u64;
        let authentication = ClientAuthentication::Unsecure {
            client_id,
            protocol_id: 0,
            server_addr,
            user_data: None,
        };
        
        let transport = NetcodeClientTransport::new(current_time, authentication, socket)
            .map_err(|e| LuaError::RuntimeError(format!("Failed to create transport: {}", e)))?;
        
        world.insert_resource(transport);
        Ok(())
    });
}
