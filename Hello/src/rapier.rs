/// Rapier physics integration module
/// This module contains all Rapier-specific code and is only compiled when the "physics" feature is enabled.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_lua_ecs::*;

/// Plugin that adds Rapier physics support to the application
/// This plugin handles all Rapier setup including physics plugins and serde component registration
pub struct RapierIntegrationPlugin;

impl Plugin for RapierIntegrationPlugin {
    fn build(&self, app: &mut App) {
        // Add Rapier physics plugins
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0));
        app.add_plugins(RapierDebugRenderPlugin::default());
        
        // Register Rapier-specific serde components that don't implement Reflect
        app.insert_resource(bevy_lua_ecs::serde_components![
            Collider,
        ]);
        
        info!("✓ Rapier physics integration enabled");
    }
}
