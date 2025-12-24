//! VR UI Panel Example
//! 
//! Press X button on left controller to open a UI panel in front of you.
//! Click the "Expand" button to extend the panel horizontally with more items.

use bevy::prelude::*;
use bevy_lua_ecs::*;
use hello::plugins::{VrInputPlugin, VrButtonState};
use bevy::color::palettes::css::{BLUE, GRAY, RED};
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
    
    // Configure default plugins to disable prepass (required for VR)
    app.add_plugins(add_xr_plugins(DefaultPlugins.set(bevy::pbr::PbrPlugin {
        prepass_enabled: false,
        ..default()
    })));
    app.add_plugins(bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin);
    
    // VR session setup
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
    
    // UI Panel systems
    app.add_systems(Update, poll_button_states.run_if(openxr_session_running));
    app.add_systems(Update, toggle_panel_on_x_button);
    app.add_systems(Update, handle_panel_button_click);
    
    // Controller raycasting for UI interaction
    app.add_systems(First, controller_raycast_ui.run_if(openxr_session_running).in_set(bevy::picking::PickingSystems::Input));
    app.add_systems(Update, update_laser_pointer);
    
    // Pointer state resource
    app.init_resource::<TriggerState>();
    
    // Lua bindings plugin
    app.add_plugins(crate::auto_resource_bindings::LuaBindingsPlugin);
    
    // VR Input plugin (provides VrButtonState for Lua access)
    app.add_plugins(VrInputPlugin);
    
    // Initialize panel state
    app.init_resource::<UiPanelState>();
    
    // Spawn custom pointer ID on startup
    app.add_systems(XrSessionCreated, spawn_pointer_and_laser);

    app.run()
}

// =============================================================================
// VR Controller Actions
// =============================================================================

#[derive(Resource)]
struct ControllerActions {
    set: openxr::ActionSet,
    left_pose: openxr::Action<Posef>,
    right_pose: openxr::Action<Posef>,
    x_button: openxr::Action<bool>,
    y_button: openxr::Action<bool>,
    a_button: openxr::Action<bool>,
    b_button: openxr::Action<bool>,
    right_trigger: openxr::Action<f32>,  // Analog trigger
}

fn create_actions(instance: Res<OxrInstance>, mut cmds: Commands) {
    let set = instance.create_action_set("vr_ui_panel", "VR UI Panel Controls", 0).unwrap();
    
    // Hand poses
    let left_pose = set
        .create_action("left_pose", "Left Hand Grip Pose", &[])
        .unwrap();
    let right_pose = set
        .create_action("right_pose", "Right Hand Grip Pose", &[])
        .unwrap();
    
    // Face buttons
    let x_button = set
        .create_action("x_button", "X Button", &[])
        .unwrap();
    let y_button = set
        .create_action("y_button", "Y Button", &[])
        .unwrap();
    let a_button = set
        .create_action("a_button", "A Button", &[])
        .unwrap();
    let b_button = set
        .create_action("b_button", "B Button", &[])
        .unwrap();
    
    // Trigger (analog, 0.0-1.0)
    let right_trigger: openxr::Action<f32> = set
        .create_action("right_trigger", "Right Trigger", &[])
        .unwrap();

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

fn attach_set(actions: Res<ControllerActions>, mut attach: MessageWriter<OxrAttachActionSet>) {
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
    
    // Hand poses
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
    
    // Face buttons (X/Y on left controller, A/B on right)
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
    
    // Right trigger (analog)
    bindings.write(OxrSuggestActionBinding {
        action: actions.right_trigger.as_raw(),
        interaction_profile: profile.into(),
        bindings: vec!["/user/hand/right/input/trigger/value".into()],
    });
}

fn spawn_hands(
    actions: Res<ControllerActions>,
    mut cmds: Commands,
    session: Res<OxrSession>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let left_space = XrSpace::from_openxr_space(
        actions
            .left_pose
            .create_space(
                session.deref().deref().clone(),
                openxr::Path::NULL,
                Posef::IDENTITY,
            )
            .unwrap(),
    );
    let right_space = session
        .create_action_space(&actions.right_pose, openxr::Path::NULL, Isometry3d::IDENTITY)
        .unwrap();
    
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.05))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        left_space,
        Controller { hand: Hand::Left },
    ));
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.05))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        right_space,
        Controller { hand: Hand::Right },
    ));
}

#[derive(Component)]
struct Controller {
    hand: Hand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hand {
    Left,
    Right,
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
    
    // Compute just_pressed (rising edge)
    states.x_just_pressed = x_current && !states.x_pressed;
    states.y_just_pressed = y_current && !states.y_pressed;
    states.a_just_pressed = a_current && !states.a_pressed;
    states.b_just_pressed = b_current && !states.b_pressed;
    
    // Update held states
    states.x_pressed = x_current;
    states.y_pressed = y_current;
    states.a_pressed = a_current;
    states.b_pressed = b_current;
}

// =============================================================================
// UI Panel
// =============================================================================

/// Panel state and entity references
#[derive(Resource, Default)]
struct UiPanelState {
    is_open: bool,
    panel_entity: Option<Entity>,
    camera_entity: Option<Entity>,
    rtt_image: Option<Handle<Image>>,
    expansion_level: u32,
    needs_respawn: bool,
}

/// Marker for the UI panel plane
#[derive(Component)]
struct UiPanel;

/// Marker for the expand button
#[derive(Component)]
struct ExpandButton;

/// Marker for UI surfaces that can be clicked with controller
/// Stores the RTT image handle and texture size for UV→pixel conversion
#[derive(Component)]
struct VrUiSurface {
    image_handle: Handle<Image>,
    texture_width: f32,
    texture_height: f32,
    // Physical panel dimensions for raycast bounds checking
    panel_half_width: f32,
    panel_half_height: f32,
}

/// Marker for the laser pointer visual
#[derive(Component)]
struct LaserPointer;

/// Custom pointer ID for VR controller
const VR_POINTER_UUID: u64 = 90870999;

/// Tracks trigger state with edge detection
#[derive(Resource, Default)]
struct TriggerState {
    pressed: bool,
    just_pressed: bool,
    just_released: bool,
}

fn toggle_panel_on_x_button(
    mut commands: Commands,
    button_states: Res<VrButtonState>,
    mut panel_state: ResMut<UiPanelState>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    controller_query: Query<&GlobalTransform, With<Controller>>,
    panel_query: Query<Entity, With<UiPanel>>,
    ui_root_query: Query<Entity, With<PanelUiRoot>>,
) {
    if !button_states.x_just_pressed {
        return;
    }
    
    if panel_state.is_open {
        // Close panel - despawn panel mesh, camera, and UI
        info!("Closing UI panel");
        for entity in panel_query.iter() {
            commands.entity(entity).despawn();
        }
        // Also despawn the RTT camera
        if let Some(camera_entity) = panel_state.camera_entity {
            commands.entity(camera_entity).despawn();
        }
        // And the UI root
        for entity in ui_root_query.iter() {
            commands.entity(entity).despawn();
        }
        panel_state.is_open = false;
        panel_state.panel_entity = None;
        panel_state.camera_entity = None;
        panel_state.expansion_level = 0;
    } else {
        // Open panel - spawn in front of left controller
        info!("Opening UI panel");
        
        // Get controller position (use first controller found, or default position)
        let spawn_transform = controller_query.iter().next()
            .map(|gt| {
                let pos = gt.translation();
                let forward = gt.forward();
                // Spawn 0.3m in front of controller, facing toward the user
                let panel_pos = pos + forward.as_vec3() * 0.3;
                // Look away from controller (toward user) - panel faces the opposite direction
                Transform::from_translation(panel_pos)
                    .looking_at(panel_pos + forward.as_vec3(), Vec3::Y)
            })
            .unwrap_or(Transform::from_xyz(0.0, 1.0, -0.5));
        
        // Base dimensions - expand horizontally with each level
        let base_width = 256u32;
        let base_height = 256u32;
        let initial_expansion = 1;
        let texture_width = base_width * initial_expansion;
        let texture_height = base_height;
        
        // Physical panel size (in meters) - width scales with texture
        let panel_width = 0.2 * initial_expansion as f32;
        let panel_height = 0.2;
        
        // Create RTT image
        let size = bevy::render::render_resource::Extent3d {
            width: texture_width,
            height: texture_height,
            depth_or_array_layers: 1,
        };
        let mut rtt_image = Image::default();
        rtt_image.resize(size);
        rtt_image.texture_descriptor.usage = 
            bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
            | bevy::render::render_resource::TextureUsages::COPY_DST
            | bevy::render::render_resource::TextureUsages::RENDER_ATTACHMENT;
        rtt_image.texture_descriptor.format = bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb;
        
        let rtt_handle = images.add(rtt_image);
        panel_state.rtt_image = Some(rtt_handle.clone());
        
        // Create panel mesh (plane)
        let panel_mesh = meshes.add(Rectangle::new(panel_width, panel_height));
        
        // Create material with RTT texture (double-sided so visible from both sides)
        let panel_material = materials.add(StandardMaterial {
            base_color_texture: Some(rtt_handle.clone()),
            unlit: true,
            cull_mode: None,  // Double-sided
            ..default()
        });
        
        // Spawn the panel with VrUiSurface for controller raycasting
        let panel_entity = commands.spawn((
            Mesh3d(panel_mesh),
            MeshMaterial3d(panel_material),
            spawn_transform,
            UiPanel,
            VrUiSurface {
                image_handle: rtt_handle.clone(),
                texture_width: texture_width as f32,
                texture_height: texture_height as f32,
                panel_half_width: panel_width / 2.0,
                panel_half_height: panel_height / 2.0,
            },
        )).id();
        
        panel_state.panel_entity = Some(panel_entity);
        panel_state.is_open = true;
        panel_state.expansion_level = 1;
        
        // Spawn RTT camera and store its entity ID
        let camera_entity = commands.spawn((
            Camera2d,
            Camera {
                order: -1,
                target: rtt_handle.clone().into(),
                ..default()
            },
        )).id();
        
        panel_state.camera_entity = Some(camera_entity);
        
        // Spawn UI content targeting the RTT camera
        spawn_panel_ui(&mut commands, panel_entity, camera_entity, 1);
        
        info!("UI panel spawned at {:?}", spawn_transform.translation);
    }
}

fn spawn_panel_ui(commands: &mut Commands, _panel_entity: Entity, camera_entity: Entity, expansion_level: u32) {
    // UI root - target the RTT camera
    let ui_root = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.9)),
        UiTargetCamera(camera_entity),
        PanelUiRoot,  // Marker for respawn cleanup
    )).id();
    
    // Spawn button sections based on expansion level
    for i in 0..expansion_level {
        let button = commands.spawn((
            Button,
            Node {
                width: Val::Px(200.0),  // Much larger button
                height: Val::Px(180.0), // Nearly fill the 256px height
                margin: UiRect::all(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.3, 0.5, 0.8)),
            BorderRadius::all(Val::Px(16.0)),
            ExpandButton,
        ))
        .observe(
            |drag: On<Pointer<Drag>>, mut nodes: Query<(&mut Node, &ComputedNode)>| {
                let (mut node, computed) = nodes.get_mut(drag.entity).unwrap();
                node.left =
                    Val::Px(drag.pointer_location.position.x - computed.size.x / 2.0);
                node.top = Val::Px(drag.pointer_location.position.y - 50.0);
            },
        )
        .observe(
            |over: On<Pointer<Over>>, mut colors: Query<&mut BackgroundColor>| {
                if let Ok(mut bg) = colors.get_mut(over.entity) {
                    bg.0 = RED.into();
                }
            },
        )
        .observe(
            |out: On<Pointer<Out>>, mut colors: Query<&mut BackgroundColor>| {
                if let Ok(mut bg) = colors.get_mut(out.entity) {
                    bg.0 = BLUE.into();
                }
            },
        )
        .observe(
            |_click: On<Pointer<Click>>, mut panel_state: ResMut<UiPanelState>| {
                info!("Expand button clicked via observer!");
                panel_state.needs_respawn = true;
                panel_state.expansion_level += 1;
            },
        )
        .id();
        
        // Button text
        let button_text = if i == expansion_level - 1 { 
            "Expand →".to_string() 
        } else { 
            format!("Item {}", i + 1) 
        };
        let text = commands.spawn((
            Text::new(button_text),
            TextFont {
                font_size: 32.0,  // Much larger text
                ..default()
            },
            TextColor(Color::WHITE),
        )).id();
        
        commands.entity(button).add_child(text);
        commands.entity(ui_root).add_child(button);
    }
}

/// Marker for UI root node that should be despawned on respawn
#[derive(Component)]
struct PanelUiRoot;

fn handle_panel_button_click(
    mut commands: Commands,
    mut panel_state: ResMut<UiPanelState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    ui_query: Query<Entity, With<PanelUiRoot>>,
    mut panel_query: Query<(&mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>, &mut VrUiSurface, &Transform), With<UiPanel>>,
) {
    if !panel_state.needs_respawn {
        return;
    }
    panel_state.needs_respawn = false;
    
    info!("Respawning UI with expansion level {}", panel_state.expansion_level);
    
    // Despawn old UI root (children despawn automatically via hierarchy)
    for entity in ui_query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Calculate new dimensions based on expansion level
    let base_width = 256u32;
    let base_height = 256u32;
    let texture_width = base_width * panel_state.expansion_level;
    let texture_height = base_height;
    
    let panel_width = 0.2 * panel_state.expansion_level as f32;
    let panel_height = 0.2;
    
    // Resize the RTT image
    let size = bevy::render::render_resource::Extent3d {
        width: texture_width,
        height: texture_height,
        depth_or_array_layers: 1,
    };
    let mut rtt_image = Image::default();
    rtt_image.resize(size);
    rtt_image.texture_descriptor.usage = 
        bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
        | bevy::render::render_resource::TextureUsages::COPY_DST
        | bevy::render::render_resource::TextureUsages::RENDER_ATTACHMENT;
    rtt_image.texture_descriptor.format = bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb;
    
    let new_rtt_handle = images.add(rtt_image);
    
    // Update panel mesh, material, and VrUiSurface
    if let Ok((mut mesh3d, mut material3d, mut vr_surface, transform)) = panel_query.single_mut() {
        // Create new mesh with updated dimensions
        let new_mesh = meshes.add(Rectangle::new(panel_width, panel_height));
        mesh3d.0 = new_mesh;
        
        // Create new material with new RTT texture
        let new_material = materials.add(StandardMaterial {
            base_color_texture: Some(new_rtt_handle.clone()),
            unlit: true,
            cull_mode: None,
            ..default()
        });
        material3d.0 = new_material;
        
        // Update VrUiSurface dimensions
        vr_surface.image_handle = new_rtt_handle.clone();
        vr_surface.texture_width = texture_width as f32;
        vr_surface.texture_height = texture_height as f32;
        vr_surface.panel_half_width = panel_width / 2.0;
        vr_surface.panel_half_height = panel_height / 2.0;
        
        // Update camera target to new RTT image
        if let Some(camera_entity) = panel_state.camera_entity {
            commands.entity(camera_entity).insert(Camera {
                order: -1,
                target: new_rtt_handle.clone().into(),
                ..default()
            });
        }
        
        panel_state.rtt_image = Some(new_rtt_handle.clone());
    }
    
    // Respawn UI with new level
    if let (Some(panel_entity), Some(camera_entity)) = (panel_state.panel_entity, panel_state.camera_entity) {
        spawn_panel_ui(&mut commands, panel_entity, camera_entity, panel_state.expansion_level);
    }
}

// =============================================================================
// VR Controller Raycasting for UI Interaction
// =============================================================================

/// Spawn the custom PointerId entity and laser pointer visual
fn spawn_pointer_and_laser(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn custom PointerId for VR controller
    commands.spawn(bevy::picking::pointer::PointerId::Custom(
        uuid::Uuid::from_u64_pair(VR_POINTER_UUID, 0)
    ));
    info!("Spawned VR controller PointerId");
    
    // Spawn laser pointer visual (thin cylinder)
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.002, 1.0))),  // 2mm radius, 1m default length
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.2, 0.2, 0.8),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::default(),
        Visibility::Hidden,
        LaserPointer,
    ));
    info!("Spawned laser pointer visual");
}

/// Raycast from right controller and send pointer input to UI surfaces
fn controller_raycast_ui(
    actions: Res<ControllerActions>,
    session: Res<OxrSession>,
    mut trigger_state: ResMut<TriggerState>,
    controller_query: Query<(&GlobalTransform, &Controller)>,
    surface_query: Query<(&GlobalTransform, &VrUiSurface)>,
    camera_query: Query<&Camera, With<Camera2d>>,
    primary_window: Query<Entity, With<bevy::window::PrimaryWindow>>,
    mut pointer_writer: MessageWriter<bevy::picking::pointer::PointerInput>,
) {
    use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
    
    // Get trigger value and compute edge detection
    let trigger_value = actions.right_trigger
        .state(session.deref().deref(), openxr::Path::NULL)
        .map(|s| s.current_state)
        .unwrap_or(0.0);
    let trigger_current = trigger_value > 0.5;
    
    trigger_state.just_pressed = trigger_current && !trigger_state.pressed;
    trigger_state.just_released = !trigger_current && trigger_state.pressed;
    trigger_state.pressed = trigger_current;
    
    // Get right controller transform
    let Some((controller_transform, _)) = controller_query.iter()
        .find(|(_, c)| c.hand == Hand::Right)
    else {
        // This is normal if controller not tracked yet
        return;
    };
    
    let ray_origin = controller_transform.translation();
    let ray_direction = controller_transform.forward().as_vec3();
    
    // Get RTT camera's normalized render target (pass primary window like Bevy example)
    let Ok(rtt_camera) = camera_query.single() else {
        return;
    };
    let Some(normalized_target) = rtt_camera.target.normalize(primary_window.single().ok()) else {
        warn!("VR raycast: target.normalize() returned None - target is {:?}", rtt_camera.target);
        return;
    };
    
    // Check if any surfaces exist
    let surface_count = surface_query.iter().count();
    if surface_count == 0 {
        // Only warn once when trigger pressed
        if trigger_state.just_pressed {
            warn!("VR raycast: No VrUiSurface entities found");
        }
    }
    
    // Simple ray-plane intersection for each UI surface
    for (surface_transform, vr_surface) in surface_query.iter() {
        let plane_pos = surface_transform.translation();
        let plane_normal = surface_transform.forward().as_vec3();
        
        let denom = plane_normal.dot(ray_direction);
        if denom.abs() < 1e-6 {
            continue;  // Ray parallel to plane
        }
        
        let t = (plane_pos - ray_origin).dot(plane_normal) / denom;
        if t < 0.0 || t > 5.0 {
            continue;  // Hit behind controller or too far
        }
        
        let hit_point = ray_origin + ray_direction * t;
        
        // Transform hit to local space
        let local_hit = surface_transform.affine().inverse().transform_point3(hit_point);
        
        // Use panel dimensions from VrUiSurface for bounds checking
        let panel_half_width = vr_surface.panel_half_width;
        let panel_half_height = vr_surface.panel_half_height;
        
        // Check if within panel bounds
        if local_hit.x.abs() > panel_half_width || local_hit.y.abs() > panel_half_height {
            continue;
        }
        
        // Convert local coords to UV (0-1)
        let u = (local_hit.x / panel_half_width + 1.0) / 2.0;
        let v = 1.0 - (local_hit.y / panel_half_height + 1.0) / 2.0;  // Flip Y
        
        // Convert UV to texture pixel coords
        let tex_x = u * vr_surface.texture_width;
        let tex_y = v * vr_surface.texture_height;
        
        // Build pointer ID
        let pointer_id = PointerId::Custom(uuid::Uuid::from_u64_pair(VR_POINTER_UUID, 0));
        
        // Build location using the camera's normalized render target
        let location = Location {
            target: normalized_target.clone(),
            position: Vec2::new(tex_x, tex_y),
        };
        
        // Send move event
        pointer_writer.write(PointerInput::new(
            pointer_id.clone(),
            location.clone(),
            PointerAction::Move { delta: Vec2::ZERO },
        ));
        
        // Send press/release events
        if trigger_state.just_pressed {
            pointer_writer.write(PointerInput::new(
                pointer_id.clone(),
                location.clone(),
                PointerAction::Press(PointerButton::Primary),
            ));
            info!("VR controller pointer press at ({:.1}, {:.1})", tex_x, tex_y);
        }
        if trigger_state.just_released {
            pointer_writer.write(PointerInput::new(
                pointer_id.clone(),
                location.clone(),
                PointerAction::Release(PointerButton::Primary),
            ));
            info!("VR controller pointer release at ({:.1}, {:.1})", tex_x, tex_y);
        }
        
        break;  // Only process first hit
    }
}

/// Update laser pointer visual based on right controller pose
fn update_laser_pointer(
    controller_query: Query<(&GlobalTransform, &Controller)>,
    mut laser_query: Query<(&mut Transform, &mut Visibility), With<LaserPointer>>,
) {
    let Ok((mut laser_transform, mut visibility)) = laser_query.single_mut() else {
        return;
    };
    
    // Get right controller transform
    let Some((controller_transform, _)) = controller_query.iter()
        .find(|(_, c)| c.hand == Hand::Right)
    else {
        *visibility = Visibility::Hidden;
        return;
    };
    
    // Make laser visible
    *visibility = Visibility::Visible;
    
    // Position laser at controller, pointing forward
    let origin = controller_transform.translation();
    let forward = controller_transform.forward().as_vec3();
    let laser_length = 2.0;  // 2 meter laser
    
    // Laser center is halfway along the ray
    let laser_center = origin + forward * (laser_length / 2.0);
    
    // Rotate cylinder to point along ray (cylinder default points along Y)
    let rotation = Quat::from_rotation_arc(Vec3::Y, forward);
    
    laser_transform.translation = laser_center;
    laser_transform.rotation = rotation;
    laser_transform.scale = Vec3::new(1.0, laser_length, 1.0);
}
