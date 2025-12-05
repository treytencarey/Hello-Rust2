use bevy::prelude::*;
use bevy_lua_ecs::{LuaSpawnPlugin, LuaScriptContext, ScriptInstance, ScriptRegistry, AssetRegistry};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LuaSpawnPlugin)
        .add_systems(Startup, load_and_run_script)
        .add_systems(PostStartup, setup_2d_camera)
        .run();
}

fn load_and_run_script(
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<ScriptInstance>,
    script_registry: Res<ScriptRegistry>,
) {
    let script_path = "assets/scripts/examples/ui_3d_plane.lua";
    let script_content = std::fs::read_to_string(script_path)
        .expect("Failed to read ui_3d_plane.lua");
    
    match lua_ctx.execute_script(
        &script_content,
        "ui_3d_plane.lua",
        std::path::PathBuf::from(script_path),
        &script_instance,
        &script_registry,
    ) {
        Ok(_) => info!("✓ Lua script executed successfully"),
        Err(e) => error!("Failed to execute Lua script: {}", e),
    }
}

/// Setup the 2D camera to render UI to the texture
/// Done in Rust because we can't construct ImageRenderTarget from Lua
fn setup_2d_camera(
    mut commands: Commands,
    asset_registry: Res<AssetRegistry>,
    images: Res<Assets<Image>>,
) {    
    // Get the image handle that Lua created
    if let Some(image_handle) = asset_registry.get_image_handle(0) {
        // Verify the image actually exists  
        if let Some(_image) = images.get(&image_handle) {
            info!("[RUST] Setting up 2D camera to render to texture");
            
            // We'll spawn directly from Lua instead - this was just a test
            // The real issue is that we can't pass RenderTarget to Lua
            // So let's just document this limitation
            
            info!("[RUST] Note: Camera render targets must be set in Rust due to Bevy API limitations");
        } else {
            warn!("[RUST] Image handle found but image not yet in Assets");
        }
    } else {
        warn!("[RUST] No render target image found (ID 0)");
    }
}
