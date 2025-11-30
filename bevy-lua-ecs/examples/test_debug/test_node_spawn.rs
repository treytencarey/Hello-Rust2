use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(LuaSpawnPlugin)
		.add_systems(Startup, (setup, load_and_run_script).chain())
		.add_systems(Update, check_nodes)
		.run();
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2d);
}

fn load_and_run_script(
	lua_ctx: Res<LuaScriptContext>,
	script_instance: Res<crate::script_entities::ScriptInstance>,
	script_registry: Res<crate::script_registry::ScriptRegistry>,
) {
	// Test spawning a Node with Val fields
    let script = r#"
spawn({
    Node = {
        width = { Px = 200 },
        height = { Px = 60 },
        position_type = "Absolute",
        top = { Px = 100 },
        left = { Px = 50 }
    }
})
print("Spawned node")
    "#;
    
    if let Err(e) = lua_ctx.execute_script(script, "test", "test", &script_instance, &script_registry) {
        eprintln!("Error: {}", e);
    }
}

fn check_nodes(query: Query<&Node>, mut ran: Local<bool>) {
	if *ran {
		return;
	}
	for node in query.iter() {
		*ran = true;
		println!("Node found:");
		println!("  width: {:?}", node.width);
		println!("  height: {:?}", node.height);
		println!("  position_type: {:?}", node.position_type);
		println!("  top: {:?}", node.top);
		println!("  left: {:?}", node.left);
		std::process::exit(0);
	}
}
// ...existing code...
// This file should use the new load_and_run_script pattern as in basic.rs
