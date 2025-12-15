use bevy::prelude::*;
use bevy_lua_ecs::*;
use std::fs;

use std::ops::Deref;
use bevy_mod_openxr::{
    action_binding::{OxrSendActionBindings, OxrSuggestActionBinding},
    action_set_attaching::OxrAttachActionSet,
    action_set_syncing::{OxrActionSetSyncSet, OxrSyncActionSet},
    add_xr_plugins, openxr_session_running,
    resources::OxrInstance,
    session::OxrSession,
    spaces::OxrSpaceExt,
};
use bevy_mod_xr::{
    session::{session_available, XrSessionCreated},
    spaces::XrSpace,
};
use openxr::Posef;

// Include auto-generated bindings
#[path = "../src/auto_resource_bindings.rs"]
mod auto_resource_bindings;

fn main() -> AppExit {
    let mut app = App::new();
    
    // Configure default plugins to disable prepass
    app.add_plugins(add_xr_plugins(DefaultPlugins.set(bevy::pbr::PbrPlugin {
        prepass_enabled: false,
        ..default()
    })));
    app.add_plugins(bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin);
    
    app.add_systems(XrSessionCreated, spawn_hands);
    app.add_systems(XrSessionCreated, attach_set);
    app.add_systems(
        PreUpdate,
        sync_actions
            .before(OxrActionSetSyncSet)
            .run_if(openxr_session_running),
    );
    app.add_systems(OxrSendActionBindings, suggest_action_bindings);
    app.add_systems(Startup, create_actions.run_if(session_available));

    app.add_plugins(crate::auto_resource_bindings::LuaBindingsPlugin);
    
    app.add_systems(PostStartup, load_and_run_script);

    app.run()
}

fn attach_set(actions: Res<ControllerActions>, mut attach: MessageWriter<OxrAttachActionSet>) {
    attach.write(OxrAttachActionSet(actions.set.clone()));
}

#[derive(Resource)]
struct ControllerActions {
    set: openxr::ActionSet,
    left: openxr::Action<Posef>,
    right: openxr::Action<Posef>,
}
fn sync_actions(actions: Res<ControllerActions>, mut sync: MessageWriter<OxrSyncActionSet>) {
    sync.write(OxrSyncActionSet(actions.set.clone()));
}
fn suggest_action_bindings(
    actions: Res<ControllerActions>,
    mut bindings: MessageWriter<OxrSuggestActionBinding>,
) {
    bindings.write(OxrSuggestActionBinding {
        action: actions.left.as_raw(),
        interaction_profile: "/interaction_profiles/oculus/touch_controller".into(),
        bindings: vec!["/user/hand/left/input/grip/pose".into()],
    });
    bindings.write(OxrSuggestActionBinding {
        action: actions.right.as_raw(),
        interaction_profile: "/interaction_profiles/oculus/touch_controller".into(),
        bindings: vec!["/user/hand/right/input/grip/pose".into()],
    });
}
fn create_actions(instance: Res<OxrInstance>, mut cmds: Commands) {
    let set = instance.create_action_set("hands", "Hands", 0).unwrap();
    let left = set
        .create_action("left_pose", "Left Hand Grip Pose", &[])
        .unwrap();
    let right = set
        .create_action("right_pose", "Right Hand Grip Pose", &[])
        .unwrap();

    cmds.insert_resource(ControllerActions { set, left, right })
}

fn spawn_hands(
    actions: Res<ControllerActions>,
    mut cmds: Commands,
    session: Res<OxrSession>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // This is a demonstation of how to integrate with the openxr crate, the right space is the
    // recommended way
    let left_space = XrSpace::from_openxr_space(
        actions
            .left
            .create_space(
                session.deref().deref().clone(),
                openxr::Path::NULL,
                Posef::IDENTITY,
            )
            .unwrap(),
    );
    let right_space = session
        .create_action_space(&actions.right, openxr::Path::NULL, Isometry3d::IDENTITY)
        .unwrap();
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.05))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        left_space,
        Controller,
    ));
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.05))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        right_space,
        Controller,
    ));
}

fn load_and_run_script(
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<crate::script_entities::ScriptInstance>,
    script_registry: Res<crate::script_registry::ScriptRegistry>,
) {
    let script_path = std::path::PathBuf::from("assets/scripts/examples/vr_simple.lua");
    match fs::read_to_string(&script_path) {
        Ok(script_content) => {
            info!("Loaded hot reload script: {:?}", script_path);
            match lua_ctx.execute_script(
                &script_content,
                "vr_simple.lua",
                script_path,
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
            error!("Failed to load script {:?}: {}", script_path, e);
        }
    }
}

#[derive(Component)]
struct Controller;