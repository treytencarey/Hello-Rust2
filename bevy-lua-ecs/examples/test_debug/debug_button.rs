use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LuaSpawnPlugin)
        .add_systems(Startup, (setup, load_and_run_script).chain())
        .add_systems(Update, debug_button_system)
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
    // Test button with Node positioning
    let script = r#"
spawn({
    Button = {},
    BackgroundColor = { color = { r = 0.2, g = 0.6, b = 0.8, a = 1.0 } },
    Node = {
        width = { Px = 200 },
        height = { Px = 60 },
        position_type = "Absolute",
        top = { Px = 100 },
        left = { Px = 50 }
    },
    Text = { text = "Click Me!" },
    TextFont = { font_size = 32 },
    TextColor = { color = { r = 1.0, g = 1.0, b = 1.0, a = 1.0 } }
})
    "#;
    
    if let Err(e) = lua_ctx.execute_script(script, "test", "test", &script_instance, &script_registry) {
        error!("Failed to execute script: {}", e);
    }
    
    info!("Script executed");
}

fn debug_button_system(
    query: Query<(Entity, Option<&Button>, Option<&Node>, Option<&BackgroundColor>)>,
) {
    for (entity, button, node, bg) in query.iter() {
        if button.is_some() {
            info!("Button Entity {:?}:", entity);
            if let Some(node) = node {
                info!(
                    "  Node: width={:?}, height={:?}, position_type={:?}, top={:?}, left={:?}",
                    node.width, node.height, node.position_type, node.top, node.left
                );
            } else {
                info!("  Node: MISSING!");
            }
            if let Some(bg) = bg {
                info!("  BackgroundColor: {:?}", bg.0);
            }
        }
    }
}
