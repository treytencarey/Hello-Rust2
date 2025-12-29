//! VR Input Plugin
//! 
//! Provides VR controller tracking, button state polling, and action bindings.
//! All VR state is accessible from Lua via world:get_resource().
//! Uses bevy_mod_openxr for OpenXR integration.

use bevy::prelude::*;
use std::ops::Deref;

use bevy_mod_openxr::{
    action_binding::{OxrSendActionBindings, OxrSuggestActionBinding},
    action_set_attaching::OxrAttachActionSet,
    action_set_syncing::{OxrActionSetSyncSet, OxrSyncActionSet},
    openxr_session_running,
    resources::OxrInstance,
    session::OxrSession,
};
use bevy_mod_xr::session::session_available;
use openxr::Posef;

// =============================================================================
// Components and Enums
// =============================================================================

/// Hand identifier for VR controllers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum Hand {
    Left,
    Right,
}

/// Marker component for VR controller entities.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Controller {
    pub hand: Hand,
}

// =============================================================================
// Resources
// =============================================================================

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
    /// Right trigger is currently pressed (with hysteresis)
    pub right_trigger_pressed: bool,
    /// Right trigger was just pressed this frame
    pub right_trigger_just_pressed: bool,
    /// Right trigger was just released this frame
    pub right_trigger_just_released: bool,
    /// Left trigger value (0.0 to 1.0)
    pub left_trigger: f32,
    /// Left trigger is currently pressed (with hysteresis)
    pub left_trigger_pressed: bool,
    /// Left trigger was just pressed this frame
    pub left_trigger_just_pressed: bool,
    /// Left trigger was just released this frame
    pub left_trigger_just_released: bool,
    /// Left grip value (0.0 to 1.0)
    pub left_grip: f32,
    /// Left grip is currently pressed (with hysteresis)
    pub left_grip_pressed: bool,
    /// Left grip was just pressed this frame
    pub left_grip_just_pressed: bool,
    /// Left grip was just released this frame
    pub left_grip_just_released: bool,
    /// Right grip value (0.0 to 1.0)
    pub right_grip: f32,
    /// Right grip is currently pressed (with hysteresis)
    pub right_grip_pressed: bool,
    /// Right grip was just pressed this frame
    pub right_grip_just_pressed: bool,
    /// Right grip was just released this frame
    pub right_grip_just_released: bool,
}

/// VR controller positions and orientations.
/// 
/// Updated each frame by poll_controller_transforms system.
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
    /// Left controller rotation (world space)
    pub left_rotation: Quat,
    /// Right controller position (world space)
    pub right_position: Vec3,
    /// Right controller forward direction (normalized)
    pub right_forward: Vec3,
    /// Right controller rotation (world space)
    pub right_rotation: Quat,
}

/// OpenXR controller actions for button and pose tracking.
#[derive(Resource)]
pub struct ControllerActions {
    pub set: openxr::ActionSet,
    pub left_pose: openxr::Action<Posef>,
    pub right_pose: openxr::Action<Posef>,
    pub x_button: openxr::Action<bool>,
    pub y_button: openxr::Action<bool>,
    pub a_button: openxr::Action<bool>,
    pub b_button: openxr::Action<bool>,
    pub right_trigger: openxr::Action<f32>,
    pub left_trigger: openxr::Action<f32>,
    pub left_grip: openxr::Action<f32>,
    pub right_grip: openxr::Action<f32>,
}

// =============================================================================
// Systems
// =============================================================================

/// Create OpenXR actions for controller poses and buttons.
fn create_actions(instance: Res<OxrInstance>, mut cmds: Commands) {
    let set = instance.create_action_set("vr_input", "VR Input Plugin", 0).unwrap();
    
    let left_pose = set.create_action("left_pose", "Left Hand Grip Pose", &[]).unwrap();
    let right_pose = set.create_action("right_pose", "Right Hand Grip Pose", &[]).unwrap();
    let x_button = set.create_action("x_button", "X Button", &[]).unwrap();
    let y_button = set.create_action("y_button", "Y Button", &[]).unwrap();
    let a_button = set.create_action("a_button", "A Button", &[]).unwrap();
    let b_button = set.create_action("b_button", "B Button", &[]).unwrap();
    let right_trigger = set.create_action("right_trigger", "Right Trigger", &[]).unwrap();
    let left_trigger = set.create_action("left_trigger", "Left Trigger", &[]).unwrap();
    let left_grip = set.create_action("left_grip", "Left Grip", &[]).unwrap();
    let right_grip = set.create_action("right_grip", "Right Grip", &[]).unwrap();

    cmds.insert_resource(ControllerActions {
        set,
        left_pose,
        right_pose,
        x_button,
        y_button,
        a_button,
        b_button,
        right_trigger,
        left_trigger,
        left_grip,
        right_grip,
    });
}

/// Attach the action set to the OpenXR session.
fn attach_action_set(actions: Res<ControllerActions>, mut attach: MessageWriter<OxrAttachActionSet>) {
    attach.write(OxrAttachActionSet(actions.set.clone()));
}

/// Sync the action set each frame.
fn sync_actions(actions: Res<ControllerActions>, mut sync: MessageWriter<OxrSyncActionSet>) {
    sync.write(OxrSyncActionSet(actions.set.clone()));
}

/// Suggest action bindings for Oculus Touch controllers.
fn suggest_action_bindings(
    actions: Res<ControllerActions>,
    mut bindings: MessageWriter<OxrSuggestActionBinding>,
) {
    let profile = "/interaction_profiles/oculus/touch_controller";
    
    bindings.write(OxrSuggestActionBinding {
        action: actions.left_pose.as_raw(),
        interaction_profile: profile.into(),
        bindings: vec!["/user/hand/left/input/grip/pose".into()],
    });
    bindings.write(OxrSuggestActionBinding {
        action: actions.right_pose.as_raw(),
        interaction_profile: profile.into(),
        bindings: vec!["/user/hand/right/input/grip/pose".into()],
    });
    bindings.write(OxrSuggestActionBinding {
        action: actions.x_button.as_raw(),
        interaction_profile: profile.into(),
        bindings: vec!["/user/hand/left/input/x/click".into()],
    });
    bindings.write(OxrSuggestActionBinding {
        action: actions.y_button.as_raw(),
        interaction_profile: profile.into(),
        bindings: vec!["/user/hand/left/input/y/click".into()],
    });
    bindings.write(OxrSuggestActionBinding {
        action: actions.a_button.as_raw(),
        interaction_profile: profile.into(),
        bindings: vec!["/user/hand/right/input/a/click".into()],
    });
    bindings.write(OxrSuggestActionBinding {
        action: actions.b_button.as_raw(),
        interaction_profile: profile.into(),
        bindings: vec!["/user/hand/right/input/b/click".into()],
    });
    bindings.write(OxrSuggestActionBinding {
        action: actions.right_trigger.as_raw(),
        interaction_profile: profile.into(),
        bindings: vec!["/user/hand/right/input/trigger/value".into()],
    });
    bindings.write(OxrSuggestActionBinding {
        action: actions.left_trigger.as_raw(),
        interaction_profile: profile.into(),
        bindings: vec!["/user/hand/left/input/trigger/value".into()],
    });
    bindings.write(OxrSuggestActionBinding {
        action: actions.left_grip.as_raw(),
        interaction_profile: profile.into(),
        bindings: vec!["/user/hand/left/input/squeeze/value".into()],
    });
    bindings.write(OxrSuggestActionBinding {
        action: actions.right_grip.as_raw(),
        interaction_profile: profile.into(),
        bindings: vec!["/user/hand/right/input/squeeze/value".into()],
    });
}

/// Spawn controller entities with visual meshes.
fn spawn_controllers(
    actions: Res<ControllerActions>,
    mut cmds: Commands,
    session: Res<OxrSession>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Left controller
    let left_space = session
        .create_action_space(&actions.left_pose, openxr::Path::NULL, Isometry3d::IDENTITY)
        .unwrap();
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.05))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        left_space,
        Controller { hand: Hand::Left },
    ));
    
    // Right controller
    let right_space = session
        .create_action_space(&actions.right_pose, openxr::Path::NULL, Isometry3d::IDENTITY)
        .unwrap();
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.05))),
        MeshMaterial3d(materials.add(Color::srgb_u8(255, 144, 124))),
        right_space,
        Controller { hand: Hand::Right },
    ));
    
    info!("VrInputPlugin: Spawned VR controllers");
}

/// Poll button states from OpenXR and update VrButtonState resource.
fn poll_button_states(
    actions: Res<ControllerActions>,
    session: Res<OxrSession>,
    mut states: ResMut<VrButtonState>,
) {
    // Get current button states from OpenXR
    let x_current = actions.x_button
        .state(session.deref().deref(), openxr::Path::NULL)
        .map(|s| s.current_state)
        .unwrap_or(false);
    let y_current = actions.y_button
        .state(session.deref().deref(), openxr::Path::NULL)
        .map(|s| s.current_state)
        .unwrap_or(false);
    let a_current = actions.a_button
        .state(session.deref().deref(), openxr::Path::NULL)
        .map(|s| s.current_state)
        .unwrap_or(false);
    let b_current = actions.b_button
        .state(session.deref().deref(), openxr::Path::NULL)
        .map(|s| s.current_state)
        .unwrap_or(false);
    
    // Get trigger values
    let right_trigger_value = actions.right_trigger
        .state(session.deref().deref(), openxr::Path::NULL)
        .map(|s| s.current_state)
        .unwrap_or(0.0);
    let left_trigger_value = actions.left_trigger
        .state(session.deref().deref(), openxr::Path::NULL)
        .map(|s| s.current_state)
        .unwrap_or(0.0);
    
    // Get grip values
    let left_grip_value = actions.left_grip
        .state(session.deref().deref(), openxr::Path::NULL)
        .map(|s| s.current_state)
        .unwrap_or(0.0);
    let right_grip_value = actions.right_grip
        .state(session.deref().deref(), openxr::Path::NULL)
        .map(|s| s.current_state)
        .unwrap_or(0.0);
    
    // Hysteresis for triggers/grips: press at 0.6, release at 0.4 to prevent bounce
    let right_trigger_was_pressed = states.right_trigger_pressed;
    let right_trigger_current = if right_trigger_was_pressed {
        right_trigger_value > 0.4
    } else {
        right_trigger_value > 0.6
    };
    
    let left_trigger_was_pressed = states.left_trigger_pressed;
    let left_trigger_current = if left_trigger_was_pressed {
        left_trigger_value > 0.4
    } else {
        left_trigger_value > 0.6
    };
    
    let left_grip_was_pressed = states.left_grip_pressed;
    let left_grip_current = if left_grip_was_pressed {
        left_grip_value > 0.4
    } else {
        left_grip_value > 0.6
    };
    
    let right_grip_was_pressed = states.right_grip_pressed;
    let right_grip_current = if right_grip_was_pressed {
        right_grip_value > 0.4
    } else {
        right_grip_value > 0.6
    };
    
    // Compute just_pressed (rising edge) and just_released (falling edge)
    states.x_just_pressed = x_current && !states.x_pressed;
    states.y_just_pressed = y_current && !states.y_pressed;
    states.a_just_pressed = a_current && !states.a_pressed;
    states.b_just_pressed = b_current && !states.b_pressed;
    states.right_trigger_just_pressed = right_trigger_current && !right_trigger_was_pressed;
    states.right_trigger_just_released = !right_trigger_current && right_trigger_was_pressed;
    states.left_trigger_just_pressed = left_trigger_current && !left_trigger_was_pressed;
    states.left_trigger_just_released = !left_trigger_current && left_trigger_was_pressed;
    states.left_grip_just_pressed = left_grip_current && !left_grip_was_pressed;
    states.left_grip_just_released = !left_grip_current && left_grip_was_pressed;
    states.right_grip_just_pressed = right_grip_current && !right_grip_was_pressed;
    states.right_grip_just_released = !right_grip_current && right_grip_was_pressed;
    
    // Update held states
    states.x_pressed = x_current;
    states.y_pressed = y_current;
    states.a_pressed = a_current;
    states.b_pressed = b_current;
    states.right_trigger = right_trigger_value;
    states.right_trigger_pressed = right_trigger_current;
    states.left_trigger = left_trigger_value;
    states.left_trigger_pressed = left_trigger_current;
    states.left_grip = left_grip_value;
    states.left_grip_pressed = left_grip_current;
    states.right_grip = right_grip_value;
    states.right_grip_pressed = right_grip_current;
}

/// Poll controller transforms and update VrControllerState for Lua access.
fn poll_controller_transforms(
    controllers: Query<(&GlobalTransform, &Controller)>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    mut ctrl_state: ResMut<VrControllerState>,
) {
    // Update HMD position from camera
    if let Ok(camera_transform) = cameras.single() {
        ctrl_state.hmd_position = camera_transform.translation();
    }
    
    // Update controller positions and rotations
    for (global_transform, controller) in controllers.iter() {
        let pos = global_transform.translation();
        let forward = global_transform.forward().as_vec3();
        let rotation = global_transform.to_scale_rotation_translation().1;
        
        match controller.hand {
            Hand::Left => {
                ctrl_state.left_position = pos;
                ctrl_state.left_forward = forward;
                ctrl_state.left_rotation = rotation;
            }
            Hand::Right => {
                ctrl_state.right_position = pos;
                ctrl_state.right_forward = forward;
                ctrl_state.right_rotation = rotation;
            }
        }
    }
}

// =============================================================================
// Plugin
// =============================================================================

/// VR Input Plugin - provides complete VR controller and button tracking.
/// 
/// This plugin:
/// - Creates OpenXR action bindings for Oculus Touch controllers
/// - Spawns controller visual meshes with tracking
/// - Polls button states each frame
/// - Updates controller positions for Lua access
/// 
/// All state is accessible from Lua via world:get_resource():
/// - "VrButtonState" for button states
/// - "VrControllerState" for controller positions
pub struct VrInputPlugin;

impl Plugin for VrInputPlugin {
    fn build(&self, app: &mut App) {
        // Register types for reflection/Lua access
        app.register_type::<VrButtonState>()
           .init_resource::<VrButtonState>()
           .register_type::<VrControllerState>()
           .init_resource::<VrControllerState>()
           .register_type::<Controller>()
           .register_type::<Hand>();
        
        // OpenXR action setup
        app.add_systems(bevy_mod_xr::session::XrSessionCreated, spawn_controllers);
        app.add_systems(bevy_mod_xr::session::XrSessionCreated, attach_action_set);
        app.add_systems(
            PreUpdate,
            sync_actions
                .before(OxrActionSetSyncSet)
                .run_if(openxr_session_running),
        );
        app.add_systems(OxrSendActionBindings, suggest_action_bindings);
        app.add_systems(Startup, create_actions.run_if(session_available));
        
        // Button and transform polling - run in PreUpdate AFTER action sync so we read fresh state
        app.add_systems(
            PreUpdate, 
            (poll_button_states, poll_controller_transforms)
                .after(OxrActionSetSyncSet)
                .run_if(openxr_session_running)
        );
        
        info!("VrInputPlugin: Initialized with controller tracking and button polling");
    }
}
