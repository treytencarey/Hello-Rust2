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
        
        // Register Rapier components that need serde (don't implement Reflect)
        app.add_systems(PreStartup, register_rapier_serde_components);
        
        debug!("✓ Rapier physics integration enabled");
    }
}

/// Register Rapier-specific components that need serde deserialization
/// Most Rapier components implement Reflect and work automatically.
/// Only register here components that implement Deserialize but not Reflect (like Collider).
fn register_rapier_serde_components(mut serde_registry: ResMut<SerdeComponentRegistry>) {
    // Collider uses complex nested structures and doesn't implement Reflect
    serde_registry.register::<Collider>("Collider");
    
    // Note: GravityScale, Restitution, and other simple Rapier components implement
    // Reflect and are automatically discovered by ComponentRegistry, so they don't
    // need to be registered here. The generic newtype wrapper support in components.rs
    // handles them automatically.
}
