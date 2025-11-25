use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

#[cfg(feature = "networking")]
use bevy_replicon::prelude::*;
#[cfg(feature = "networking")]
use bevy_replicon_renet::RepliconRenetPlugins;

#[cfg(feature = "networking")]
use bevy_replicon_renet::renet::{ConnectionConfig, RenetClient, RenetServer};
#[cfg(feature = "networking")]
use bevy_renet::netcode::{
    ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
    ServerAuthentication, ServerConfig,
};
#[cfg(feature = "networking")]
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
#[cfg(feature = "networking")]
use std::time::SystemTime;
#[cfg(feature = "networking")]
use mlua::prelude::*;

fn main() {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins);
    
    // Add replicon plugins (third-party plugins, not game logic)
    #[cfg(feature = "networking")]
    {
        app.add_plugins(RepliconPlugins)
            .add_plugins(RepliconRenetPlugins);
        
        // Register components for replication (configuration)
        // Only replicate Transform - Sprite will be added by client locally
        app.replicate::<Transform>();
    }
    
    // Create component registry
    let component_registry = ComponentRegistry::from_type_registry(
        app.world().resource::<AppTypeRegistry>().clone()
    );
    
    let mut serde_registry = SerdeComponentRegistry::default();
    
    // Register Replicated marker component for this example
    #[cfg(feature = "networking")]
    serde_registry.register_marker::<Replicated>("Replicated");
    
    let mut builder_registry = ResourceBuilderRegistry::default();
    
    // Register networking resource constructors for this example
    #[cfg(feature = "networking")]
    register_networking_resources(&mut builder_registry);
    
    app.insert_resource(component_registry)
        .init_resource::<SpawnQueue>()
        .init_resource::<ResourceQueue>()
        .init_resource::<ComponentUpdateQueue>()
        .insert_resource(serde_registry)
        .insert_resource(builder_registry);
        
    app.add_plugins(LuaSpawnPlugin)
        .add_systems(Update, (
            process_spawn_queue,
            run_lua_systems,
            bevy_lua_ecs::component_updater::process_component_updates,
        ))
        .add_systems(PostStartup, load_and_run_script)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn load_and_run_script(lua_ctx: Res<LuaScriptContext>) {
    let script_path = "Hello/assets/scripts/networking_example.lua";
    match fs::read_to_string(script_path) {
        Ok(script_content) => {
            if let Err(e) = lua_ctx.execute_script(&script_content, "networking_example.lua") {
                error!("Failed to execute script: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to load script {}: {}", script_path, e);
        }
    }
}

// Example-specific networking resource registration
// These wrap OS-level operations (socket binding, system time) for this example
#[cfg(feature = "networking")]
fn register_networking_resources(registry: &mut ResourceBuilderRegistry) {
    // RenetServer constructor
    registry.register("RenetServer", |_lua, _data: LuaValue, world: &mut World| {
        world.insert_resource(RenetServer::new(ConnectionConfig::default()));
        Ok(())
    });
    
    // RenetClient constructor
    registry.register("RenetClient", |_lua, _data: LuaValue, world: &mut World| {
        world.insert_resource(RenetClient::new(ConnectionConfig::default()));
        Ok(())
    });
    
    // NetcodeServerTransport - wraps OS-level socket binding
    registry.register("NetcodeServerTransport", |_lua, data: LuaValue, world: &mut World| {
        let table = data.as_table()
            .ok_or_else(|| LuaError::RuntimeError("Expected table for NetcodeServerTransport".to_string()))?;
        
        let port: u16 = table.get("port")?;
        let max_clients: usize = table.get("max_clients")?;
        
        let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        let socket = UdpSocket::bind(server_addr)
            .map_err(|e| LuaError::RuntimeError(format!("Failed to bind socket: {}", e)))?;
        
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        
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
    
    // NetcodeClientTransport - wraps OS-level socket binding
    registry.register("NetcodeClientTransport", |_lua, data: LuaValue, world: &mut World| {
        let table = data.as_table()
            .ok_or_else(|| LuaError::RuntimeError("Expected table for NetcodeClientTransport".to_string()))?;
        
        let server_addr_str: String = table.get("server_addr")?;
        let port: u16 = table.get("port")?;
        
        let server_addr: SocketAddr = format!("{}:{}", server_addr_str, port)
            .parse()
            .map_err(|e| LuaError::RuntimeError(format!("Invalid address: {}", e)))?;
        
        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| LuaError::RuntimeError(format!("Failed to bind socket: {}", e)))?;
        
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        
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
