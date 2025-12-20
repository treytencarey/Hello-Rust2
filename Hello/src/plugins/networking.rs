//! Networking plugin for Renet/Replicon and network asset integration

use bevy::prelude::*;
use bevy_renet::RenetClientPlugin;
use bevy_renet::netcode::NetcodeClientPlugin;

/// Network asset downloading and hot reload plugin
pub struct HelloNetworkingPlugin;

impl Plugin for HelloNetworkingPlugin {
    fn build(&self, app: &mut App) {
        // Add Renet networking plugins
        app.add_plugins(RenetClientPlugin);
        app.add_plugins(NetcodeClientPlugin);
        
        // Add network asset integration (handles downloads and hot reload)
        app.add_plugins(crate::network_asset_integration::NetworkAssetPlugin);
        
        info!("✓ Networking plugin loaded");
    }
}
