// Tiled map integration using bevy_ecs_tiled
// This module provides integration with the Tiled map editor

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

/// Plugin that adds Tiled map support to the application
pub struct TiledIntegrationPlugin;

impl Plugin for TiledIntegrationPlugin {
    fn build(&self, app: &mut App) {
        // Add the bevy_ecs_tiled plugin with default configuration
        app.add_plugins(TiledPlugin::default());
        
        info!("✓ Tiled map plugin initialized");
    }
}
