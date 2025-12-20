//! Tiled map plugin wrapper

use bevy::prelude::*;

/// Tiled map integration plugin
pub struct HelloTiledPlugin;

impl Plugin for HelloTiledPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::tiled::TiledIntegrationPlugin);
        info!("✓ Tiled plugin loaded");
    }
}
