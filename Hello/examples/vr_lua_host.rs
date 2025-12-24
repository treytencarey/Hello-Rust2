//! VR Lua Host Example
//! 
//! A minimal Rust bootstrap for VR Lua applications.
//! Sets up OpenXR, controller tracking, button state polling, and runs a Lua script.
//!
//! Usage: cargo run --example vr_lua_host --features bevy_mod_xr

use bevy::prelude::*;
use bevy_lua_ecs::*;
use hello::plugins::{VrInputPlugin, VrButtonState, VrControllerState};
use std::fs;
use std::ops::Deref;

use bevy_mod_openxr::{
    action_binding::{OxrSendActionBindings, OxrSuggestActionBinding},
    action_set_attaching::OxrAttachActionSet,
    action_set_syncing::{OxrActionSetSyncSet, OxrSyncActionSet},
    add_xr_plugins, openxr_session_running,
    resources::OxrInstance,
    session::OxrSession,
};
use bevy_mod_xr::{
    session::{session_available, XrSessionCreated},
    spaces::XrSpace,
};
use openxr::Posef;

// Include auto-generated bindings
#[path = "../src/auto_resource_bindings.rs"]
mod auto_resource_bindings;

// Controller marker component
#[derive(Component, Reflect)]
#[reflect(Component)]
struct Controller {
    hand: Hand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
enum Hand {
    Left,
    Right,
}

// Default script path - can be overridden via command line
const DEFAULT_SCRIPT: &str = "assets/scripts/examples/vr_ui_panel.lua";

fn main() -> AppExit {
    let mut app = App::new();
    
    // Get script path from args or use default
    let script_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_SCRIPT.to_string());
    
    // Store script path as resource
    app.insert_resource(ScriptPath(script_path));
    
    // Configure default plugins with XR
    app.add_plugins(add_xr_plugins(DefaultPlugins.set(bevy::pbr::PbrPlugin {
        prepass_enabled: false,
        ..default()
    })));
    app.add_plugins(bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin);
    
    // VR input plugin for button states
    app.add_plugins(VrInputPlugin);
    
    // Lua bindings
    app.add_plugins(crate::auto_resource_bindings::LuaBindingsPlugin);
    
    // OpenXR action setup
    app.add_systems(XrSessionCreated, spawn_controllers);
    app.add_systems(XrSessionCreated, attach_action_set);
    app.add_systems(
        PreUpdate,
        sync_actions
            .before(OxrActionSetSyncSet)
            .run_if(openxr_session_running),
    );
    app.add_systems(OxrSendActionBindings, suggest_action_bindings);
    app.add_systems(Startup, create_actions.run_if(session_available));
    
    // Button polling - run in PreUpdate to ensure state is ready before Lua Update systems
    app.add_systems(PreUpdate, poll_button_states.run_if(openxr_session_running));
    
    // Controller transform polling - also run in PreUpdate for Lua access
    app.add_systems(PreUpdate, poll_controller_transforms.run_if(openxr_session_running));
    
    // Script loading
    app.add_systems(PostStartup, load_and_run_script);

    app.run()
}

#[derive(Resource)]
struct ScriptPath(String);

/// OpenXR controller actions
#[derive(Resource)]
struct ControllerActions {
    set: openxr::ActionSet,
    left_pose: openxr::Action<Posef>,
    right_pose: openxr::Action<Posef>,
    x_button: openxr::Action<bool>,
    y_button: openxr::Action<bool>,
    a_button: openxr::Action<bool>,
    b_button: openxr::Action<bool>,
    right_trigger: openxr::Action<f32>,
}

fn create_actions(instance: Res<OxrInstance>, mut cmds: Commands) {
    let set = instance.create_action_set("vr_lua", "VR Lua Host", 0).unwrap();
    
    let left_pose = set.create_action("left_pose", "Left Hand Grip Pose", &[]).unwrap();
    let right_pose = set.create_action("right_pose", "Right Hand Grip Pose", &[]).unwrap();
    let x_button = set.create_action("x_button", "X Button", &[]).unwrap();
    let y_button = set.create_action("y_button", "Y Button", &[]).unwrap();
    let a_button = set.create_action("a_button", "A Button", &[]).unwrap();
    let b_button = set.create_action("b_button", "B Button", &[]).unwrap();
    let right_trigger = set.create_action("right_trigger", "Right Trigger", &[]).unwrap();

    cmds.insert_resource(ControllerActions {
        set,
        left_pose,
        right_pose,
        x_button,
        y_button,
        a_button,
        b_button,
        right_trigger,
    });
}

fn attach_action_set(actions: Res<ControllerActions>, mut attach: MessageWriter<OxrAttachActionSet>) {
    attach.write(OxrAttachActionSet(actions.set.clone()));
}

fn sync_actions(actions: Res<ControllerActions>, mut sync: MessageWriter<OxrSyncActionSet>) {
    sync.write(OxrSyncActionSet(actions.set.clone()));
}

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
}

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
    
    info!("Spawned VR controllers for Lua access");
}

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
    
    // Get right trigger value
    let trigger_value = actions.right_trigger
        .state(session.deref().deref(), openxr::Path::NULL)
        .map(|s| s.current_state)
        .unwrap_or(0.0);
    let trigger_current = trigger_value > 0.5;  // Threshold for "pressed"
    
    // Compute just_pressed (rising edge)
    states.x_just_pressed = x_current && !states.x_pressed;
    states.y_just_pressed = y_current && !states.y_pressed;
    states.a_just_pressed = a_current && !states.a_pressed;
    states.b_just_pressed = b_current && !states.b_pressed;
    states.right_trigger_just_pressed = trigger_current && states.right_trigger < 0.5;
    
    // Update held states
    states.x_pressed = x_current;
    states.y_pressed = y_current;
    states.a_pressed = a_current;
    states.b_pressed = b_current;
    states.right_trigger = trigger_value;
}

/// Poll controller transforms and update VrControllerState for Lua access
fn poll_controller_transforms(
    controllers: Query<(&GlobalTransform, &Controller)>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    mut ctrl_state: ResMut<VrControllerState>,
) {
    // Update HMD position from camera
    if let Ok(camera_transform) = cameras.single() {
        ctrl_state.hmd_position = camera_transform.translation();
    }
    
    // Update controller positions
    for (global_transform, controller) in controllers.iter() {
        let pos = global_transform.translation();
        let forward = global_transform.forward().as_vec3();
        
        match controller.hand {
            Hand::Left => {
                ctrl_state.left_position = pos;
                ctrl_state.left_forward = forward;
            }
            Hand::Right => {
                ctrl_state.right_position = pos;
                ctrl_state.right_forward = forward;
            }
        }
    }
}

fn load_and_run_script(
    script_path: Res<ScriptPath>,
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<crate::script_entities::ScriptInstance>,
    script_registry: Res<crate::script_registry::ScriptRegistry>,
) {
    let path = std::path::PathBuf::from(&script_path.0);
    match fs::read_to_string(&path) {
        Ok(script_content) => {
            info!("Loading Lua script: {:?}", path);
            match lua_ctx.execute_script(
                &script_content,
                path.file_name().unwrap().to_str().unwrap(),
                path.clone(),
                &script_instance,
                &script_registry,
            ) {
                Ok(instance_id) => {
                    info!("Script executed with instance ID: {}", instance_id);
                }
                Err(e) => {
                    error!("Failed to execute script: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to load script {:?}: {}", path, e);
        }
    }
}
