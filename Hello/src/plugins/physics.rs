//! Physics plugin wrapper for Rapier integration

use bevy::prelude::*;

/// Rapier 2D physics integration plugin
pub struct HelloPhysicsPlugin;

impl Plugin for HelloPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::rapier::RapierIntegrationPlugin);
        info!("✓ Physics plugin loaded");
    }
}
