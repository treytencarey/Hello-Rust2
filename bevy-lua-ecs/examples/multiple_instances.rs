/// Multiple Instances Example
/// Demonstrates loading the same script multiple times with independent tracking
/// Run with: cargo run --example multiple_instances

use bevy::prelude::*;
use bevy_lua_ecs::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LuaSpawnPlugin)
        .add_systems(Startup, setup)
        .add_systems(PostStartup, load_multiple_instances)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    info!("Camera spawned");
}

fn load_multiple_instances(lua_ctx: Res<LuaScriptContext>, script_instance: Res<ScriptInstance>) {
    let script_content = r#"
print("=== Script Instance Starting ===")
print("Instance ID:", __INSTANCE_ID__)
print("Script Name:", __SCRIPT_NAME__)

-- Spawn some entities unique to this instance
local radius = __INSTANCE_ID__ * 100
for i = 1, 3 do
    local angle = (i - 1) * (2 * math.pi / 3)
    local x = math.cos(angle) * radius
    local y = math.sin(angle) * radius
    
    spawn({
        Transform = { translation = { x, y, 0 } },
        Sprite = { 
            color = { 
                r = __INSTANCE_ID__ * 0.2,
                g = 0.5,
                b = 1.0 - (__INSTANCE_ID__ * 0.2),
                a = 1.0
            }
        }
    })
    print(string.format("  Spawned entity at (%.0f, %.0f) for instance %d", x, y, __INSTANCE_ID__))
end

print("=== Script Instance Initialized ===")
"#;

    // Load the same script 3 times with independent tracking
    for i in 1..=3 {
        match lua_ctx.execute_script_tracked(script_content, "test_script.lua", &script_instance) {
            Ok(instance_id) => {
                info!("✓ Loaded script instance {} with ID: {}", i, instance_id);
            }
            Err(e) => {
                error!("Failed to load script instance {}: {}", i, e);
            }
        }
    }
    
    info!("All 3 instances loaded! Each has independent entities.");
    info!("Instance 1: 3 entities at radius 100");
    info!("Instance 2: 3 entities at radius 200");
    info!("Instance 3: 3 entities at radius 300");
}
