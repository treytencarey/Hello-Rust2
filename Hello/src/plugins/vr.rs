//! VR Input Plugin
//! 
//! Provides VR button state tracking accessible from Lua via world:get_resource("VrButtonState").
//! And VR controller positions via world:get_resource("VrControllerState").
//! Uses bevy_mod_openxr for OpenXR button state polling.

use bevy::prelude::*;

/// VR controller button states with edge detection.
/// 
/// Accessible from Lua via:
/// ```lua
/// local vr = world:get_resource("VrButtonState")
/// if vr and vr.x_just_pressed then
///     -- X button was just pressed this frame
/// end
/// ```
#[derive(Resource, Reflect, Default, Debug, Clone)]
#[reflect(Resource)]
pub struct VrButtonState {
    /// X button (left controller) is currently held
    pub x_pressed: bool,
    /// X button was just pressed this frame
    pub x_just_pressed: bool,
    /// Y button (left controller) is currently held
    pub y_pressed: bool,
    /// Y button was just pressed this frame
    pub y_just_pressed: bool,
    /// A button (right controller) is currently held
    pub a_pressed: bool,
    /// A button was just pressed this frame
    pub a_just_pressed: bool,
    /// B button (right controller) is currently held
    pub b_pressed: bool,
    /// B button was just pressed this frame
    pub b_just_pressed: bool,
    /// Right trigger value (0.0 to 1.0)
    pub right_trigger: f32,
    /// Right trigger was just pressed (crossed 0.5 threshold)
    pub right_trigger_just_pressed: bool,
}

/// VR controller positions and orientations.
/// 
/// Updated each frame by the VR example's poll_controller_transforms system.
/// Accessible from Lua via:
/// ```lua
/// local ctrl = world:get_resource("VrControllerState")
/// if ctrl then
///     local left_pos = ctrl.left_position
///     local left_fwd = ctrl.left_forward
///     -- Spawn panel 0.3m in front of left controller
///     local panel_pos = {
///         x = left_pos.x + left_fwd.x * 0.3,
///         y = left_pos.y + left_fwd.y * 0.3,
///         z = left_pos.z + left_fwd.z * 0.3
///     }
/// end
/// ```
#[derive(Resource, Reflect, Default, Debug, Clone)]
#[reflect(Resource)]
pub struct VrControllerState {
    /// HMD/Camera position (world space)
    pub hmd_position: Vec3,
    /// Left controller position (world space)
    pub left_position: Vec3,
    /// Left controller forward direction (normalized)
    pub left_forward: Vec3,
    /// Right controller position (world space)
    pub right_position: Vec3,
    /// Right controller forward direction (normalized)
    pub right_forward: Vec3,
}

/// VR Input Plugin - registers VrButtonState and VrControllerState resources for Lua access.
/// 
/// Note: The actual button polling and transform updates are done by the VR example's systems
/// using OpenXR actions. This plugin just sets up the resources.
pub struct VrInputPlugin;

impl Plugin for VrInputPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<VrButtonState>()
           .init_resource::<VrButtonState>()
           .register_type::<VrControllerState>()
           .init_resource::<VrControllerState>();
        
        info!("VrInputPlugin: VrButtonState and VrControllerState registered for Lua access");
    }
}

