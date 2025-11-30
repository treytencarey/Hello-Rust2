use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(LuaSpawnPlugin)
		.add_systems(Startup, (setup, load_and_run_script).chain())
		.add_systems(Update, debug_text_entities)
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
    // Execute the spawn_text script
    let script = r#"
spawn({
    Text = { text = "Hello from Lua!" },
    TextFont = { font_size = 64 },
    TextColor = { color = {r = 1.0, g = 0.8, b = 0.2, a = 1.0} },
    Transform = { translation = {x = 0, y = 100, z = 0} }
})
    "#;
    
    if let Err(e) = lua_ctx.execute_script(script, "test", "test", &script_instance, &script_registry) {
        error!("Failed to execute script: {}", e);
    }
    
    info!("Script executed");
}

fn debug_text_entities(
    query: Query<(Entity, Option<&Text>, Option<&TextFont>, Option<&Transform>, Option<&Node>)>,
    mut ran: Local<bool>,
) {
    if *ran {
        return;
    }
    
    let mut count = 0;
    for (entity, text, font, transform, node) in query.iter() {
        if text.is_some() || font.is_some() {
            count += 1;
            info!(
                "Text Entity {:?}: Text={:?}, Font={:?}, Transform={:?}, Node={:?}",
                entity,
                text.map(|t| &t.0),
                font.map(|f| f.font_size),
                transform.map(|t| t.translation),
                node.is_some()
            );
        }
    }
    
    if count > 0 {
        *ran = true;
        info!("Found {} text entities", count);
    }
}
