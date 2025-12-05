// Lightyear Chat Example - Zero Rust Philosophy
// Rust only sets up Lightyear, all chat logic in Lua

use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;
use std::time::Duration;
use std::sync::{Arc, Mutex};

#[cfg(feature = "lightyear_net")]
use lightyear::prelude::*;
#[cfg(feature = "lightyear_net")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "lightyear_net")]
use mlua::prelude::*;

const FIXED_TIMESTEP_HZ: f64 = 60.0;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let is_client = args.len() > 1 && args[1] == "client";

    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: if is_client { 
                "Lightyear Chat - Client".to_string() 
            } else { 
                "Lightyear Chat - Server".to_string() 
            },
            resolution: (800.0, 600.0).into(),
            ..default()
        }),
        ..default()
    }));
    
    info!("🚀 Starting {} mode", if is_client { "CLIENT" } else { "SERVER" });
    
    app.insert_resource(IsClientMode(is_client));
    app.insert_resource(MessageQueue::default());
    register_common_bevy_events(&mut app);
    
    #[cfg(feature = "lightyear_net")]
    {
        if is_client {
            info!("🌐 Setting up Lightyear CLIENT");
            app.add_plugins(client::ClientPlugins {
                tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
            });
        } else {
            info!("🌐 Setting up Lightyear SERVER");
            app.add_plugins(server::ServerPlugins {
                tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
            });
        }
    }
    
    app.add_plugins(LuaSpawnPlugin)
        .add_systems(Startup, setup)
        .add_systems(PostStartup, register_lua_message_functions)
        .add_systems(PostStartup, load_and_run_script.after(register_lua_message_functions))
        .add_systems(Update, echo_messages_system)  // MVP: local echo for testing
        .run();
}

#[cfg(feature = "lightyear_net")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LuaMessage {
    pub message_type: String,
    pub data: String, // JSON string for Lua
}

// Temporary message queue (MVP until full Lightyear bindings)
#[derive(Resource, Clone)]
struct MessageQueue {
    outgoing: Arc<Mutex<Vec<LuaMessage>>>,
    incoming: Arc<Mutex<Vec<LuaMessage>>>,
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self {
            outgoing: Arc::new(Mutex::new(Vec::new())),
            incoming: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[derive(Resource)]
struct IsClientMode(bool);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    info!("✅ Camera spawned");
}

/// Register Lua functions for sending/receiving messages
fn register_lua_message_functions(
    lua_ctx: Res<LuaScriptContext>,
    queue: Res<MessageQueue>,
) {
    let lua = &lua_ctx.lua;
    let globals = lua.globals();
    
    // Clone Arc for closure
    let outgoing = queue.outgoing.clone();
    let incoming = queue.incoming.clone();
    
    // send_lua_message(message_type, data_json)
    let send_fn = lua.create_function(move |_lua, (msg_type, data): (String, String)| {
        let message = LuaMessage {
            message_type: msg_type.clone(),
            data: data.clone(),
        };
        
        if let Ok(mut queue) = outgoing.lock() {
            queue.push(message);
            info!("📤 Queued message: {} ({})", msg_type, data.len());
        }
        Ok(())
    }).expect("Failed to create send_lua_message function");
    
    globals.set("send_lua_message", send_fn).expect("Failed to set send_lua_message");
    
    // receive_lua_messages() -> array of {message_type, data}
    let receive_fn = lua.create_function(move |lua, ()| {
        let table = lua.create_table()?;
        
        if let Ok(mut queue) = incoming.lock() {
            for (i, msg) in queue.drain(..).enumerate() {
                let msg_table = lua.create_table()?;
                msg_table.set("message_type", msg.message_type)?;
                msg_table.set("data", msg.data)?;
                table.set(i + 1, msg_table)?;
            }
        }
        
        Ok(table)
    }).expect("Failed to create receive_lua_messages function");
    
    globals.set("receive_lua_messages", receive_fn).expect("Failed to set receive_lua_messages");
    
    info!("✅ Registered Lua message functions");
}

/// MVP Echo System - moves outgoing to incoming for local testing
/// (Will be replaced with real Lightyear client<->server communication)
fn echo_messages_system(queue: Res<MessageQueue>) {
    // Move messages from outgoing to incoming as simple echo
    if let (Ok(mut outgoing), Ok(mut incoming)) = (queue.outgoing.lock(), queue.incoming.lock()) {
        if !outgoing.is_empty() {
            for msg in outgoing.drain(..) {
                info!("📡 Echo: {} -> {}", msg.message_type, msg.data.chars().take(50).collect::<String>());
                incoming.push(msg);
            }
        }
    }
}

fn load_and_run_script(
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<crate::script_entities::ScriptInstance>,
    script_registry: Res<crate::script_registry::ScriptRegistry>,
    is_client_mode: Res<IsClientMode>,
) {
    // Try both paths
    let paths = vec![
        std::path::PathBuf::from("Hello/assets/scripts/lightyear_chat.lua"),
        std::path::PathBuf::from("assets/scripts/examples/lightyear_chat.lua"),
    ];
    
    let mut script_content = None;
    let mut used_path = None;
    
    for path in &paths {
        if let Ok(content) = fs::read_to_string(path) {
            script_content = Some(content);
            used_path = Some(path);
            break;
        }
    }
    
    match script_content {
        Some(mut content) => {
            let mode_injection = if is_client_mode.0 {
                "IS_CLIENT_MODE = true\nIS_SERVER_MODE = false\n"
            } else {
                "IS_CLIENT_MODE = false\nIS_SERVER_MODE = true\n"
            };
            content = format!("{}\n{}", mode_injection, content);
            
            info!("📜 Loading Lightyear chat script: {:?}", used_path.unwrap());
            match lua_ctx.execute_script(
                &content,
                "lightyear_chat.lua",
                used_path.unwrap().clone(),
                &script_instance,
                &script_registry,
            ) {
                Ok(instance_id) => {
                    info!("✅ Script executed with instance ID: {}", instance_id);
                }
                Err(e) => {
                    error!("❌ Failed to execute script: {}", e);
                }
            }
        }
        None => {
            error!("❌ Could not find lightyear_chat.lua in any of the expected paths");
        }
    }
}
