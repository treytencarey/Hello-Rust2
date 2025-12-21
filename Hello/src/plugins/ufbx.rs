//! UFbx plugin - FBX file loading support
//! Wraps bevy_ufbx for FBX asset loading

use bevy::prelude::*;
use bevy_ufbx::FbxPlugin;

/// UFbx plugin wrapper - adds FBX file loading support
pub struct HelloUfbxPlugin;

impl Plugin for HelloUfbxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FbxPlugin);
        info!("✓ UFbx plugin loaded");
    }
}
