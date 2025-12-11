use bevy::picking::PickingSystems;
use bevy::{
    asset::uuid::Uuid,
    camera::RenderTarget,
    picking::{
        backend::ray::RayMap,
        pointer::{Location, PointerAction, PointerId, PointerInput, PointerButton},
    },
    prelude::*,
    window::{PrimaryWindow, WindowEvent},
};
use bevy_lua_ecs::*;
use std::fs;

// Include auto-generated bindings
#[path = "../src/auto_resource_bindings.rs"]
mod auto_resource_bindings;

// Custom pointer ID for the RTT texture interaction
const SPHERE_POINTER_ID: PointerId = PointerId::Custom(Uuid::from_u128(90870988));

// Marker component for the 3D sphere that has the RTT texture
#[derive(Component)]
struct Sphere3d;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Use LuaBindingsPlugin - automatically handles all registration
        .add_plugins(crate::auto_resource_bindings::LuaBindingsPlugin)
        .add_systems(Startup, setup)
        // Run script loading AFTER PostStartup so register_asset_constructors has completed
        // This ensures UiTargetCamera and other entity wrappers are registered before the script uses them
        .add_systems(Update, load_and_run_script.run_if(run_once))
        // Add raycasting system for RTT UI picking
        .add_systems(First, drive_diegetic_pointer.in_set(PickingSystems::Input))
        // Add system to tag the sphere mesh once spawned
        .add_systems(Update, tag_sphere_mesh)
        // DEBUG: Check camera targets after spawn
        // .add_systems(Update, debug_camera_targets)
        .run();
}

/// DEBUG SYSTEM: Check what Camera::target is set to for all Camera2d entities
// fn debug_camera_targets(
//     cameras: Query<(Entity, &Camera), With<Camera2d>>,
//     materials: Res<Assets<StandardMaterial>>,
//     images: Res<Assets<Image>>,
//     material_handles: Query<(Entity, &MeshMaterial3d<StandardMaterial>)>,
// ) {
//     for (entity, camera) in cameras.iter() {
//         if let RenderTarget::Image(image_target) = &camera.target {
//             let handle = &image_target.handle;
//             if let Some(image) = images.get(handle) {
//                 info!("[DEBUG_CAMERA] Entity {:?} Camera2d target Image: {}x{}, usage: {:?}, order: {}", 
//                     entity, 
//                     image.texture_descriptor.size.width, 
//                     image.texture_descriptor.size.height,
//                     image.texture_descriptor.usage, 
//                     camera.order);
//             } else {
//                 info!("[DEBUG_CAMERA] Entity {:?} Camera2d target Image NOT LOADED yet", entity);
//             }
//         } else {
//             info!("[DEBUG_CAMERA] Entity {:?} Camera2d target: {:?}", entity, camera.target);
//         }
//     }
    
//     for (entity, mat_handle) in material_handles.iter() {
//         if let Some(material) = materials.get(&mat_handle.0) {
//             if let Some(ref tex_handle) = material.base_color_texture {
//                 if let Some(image) = images.get(tex_handle) {
//                     info!("[DEBUG_MATERIAL] Entity {:?} base_color_texture usage: {:?}", entity, image.texture_descriptor.usage);
//                 }
//             }
//         }
//     }
// }

fn setup(mut commands: Commands) {
    // Spawn the custom pointer entity
    commands.spawn(SPHERE_POINTER_ID);
    info!("Lua RTT UI example starting...");
}

fn load_and_run_script(
    lua_ctx: Res<LuaScriptContext>,
    script_instance: Res<ScriptInstance>,
    script_registry: Res<ScriptRegistry>,
) {
    let script_path = std::path::PathBuf::from("assets/scripts/render_ui_to_texture.lua");
    match fs::read_to_string(&script_path) {
        Ok(script_content) => {
            info!("Loaded RTT UI script: {:?}", script_path);
            match lua_ctx.execute_script(
                &script_content,
                "render_ui_to_texture.lua",
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

/// System to tag the sphere mesh entity once it's spawned by Lua
/// The sphere should have a Mesh3d component but no Sphere3d marker yet
fn tag_sphere_mesh(
    mut commands: Commands,
    spheres: Query<Entity, (With<Mesh3d>, Without<Sphere3d>)>,
    meshes: Res<Assets<Mesh>>,
    mesh_handles: Query<&Mesh3d>,
) {
    for entity in spheres.iter() {
        // Check if this is a sphere mesh by looking at its structure
        // For now, tag any mesh that doesn't have Sphere3d marker yet
        // This is a simple heuristic - in production you'd use a marker from Lua
        if let Ok(mesh3d) = mesh_handles.get(entity) {
            if let Some(_mesh) = meshes.get(&mesh3d.0) {
                // Tag this entity as our sphere
                commands.entity(entity).insert(Sphere3d);
                debug!("Tagged sphere mesh entity: {:?}", entity);
            }
        }
    }
}

/// System that casts rays to the sphere and converts UV hits to pointer input
/// This enables UI picking on the RTT texture displayed on the 3D sphere
fn drive_diegetic_pointer(
    mut cursor_last: Local<Vec2>,
    mut raycast: MeshRayCast,
    rays: Res<RayMap>,
    spheres: Query<&Mesh3d, With<Sphere3d>>,
    ui_camera: Query<&Camera, With<Camera2d>>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    windows: Query<(Entity, &Window)>,
    images: Res<Assets<Image>>,
    manual_texture_views: Res<ManualTextureViews>,
    mut window_events: MessageReader<WindowEvent>,
    mut pointer_inputs: MessageWriter<PointerInput>,
) {
    // Get the UI camera's render target info
    let Ok(camera) = ui_camera.single() else {
        warn!("[RTT_PICK] No Camera2d found, returning");
        return;
    };
    
    let Some(target) = camera.target.normalize(primary_window.single().ok()) else {
        warn!("[RTT_PICK] Could not normalize camera target, returning");
        return;
    };
    
    let Some(target_info) = target.get_render_target_info(windows, &images, &manual_texture_views).ok() else {
        warn!("[RTT_PICK] Could not get render target info, returning");
        return;
    };
    
    let size = target_info.physical_size.as_vec2();
    
    // Debug: log how many spheres we have to raycast against
    let sphere_count = spheres.iter().count();
    info!("[RTT_PICK] Spheres: {}, size: {:?}", sphere_count, size);
    
    // Cast rays and find hits on the sphere
    let raycast_settings = MeshRayCastSettings {
        visibility: RayCastVisibility::VisibleInView,
        filter: &|entity| spheres.contains(entity),
        early_exit_test: &|_| false,
    };
    
    let ray_count = rays.iter().count();
    
    for (id, ray) in rays.iter() {
        let hits = raycast.cast_ray(*ray, &raycast_settings);
        if !hits.is_empty() {
            info!("[RTT_PICK] Ray {:?} hit {} entities", id, hits.len());
        }
        
        for (entity, hit) in hits {
            info!("[RTT_PICK] Hit entity {:?}, uv: {:?}", entity, hit.uv);
            if let Some(uv) = hit.uv {
                let position = size * uv;
                if position != *cursor_last {
                    info!("[RTT_PICK] Pointer moved to {:?}", position);
                    pointer_inputs.write(PointerInput::new(
                        SPHERE_POINTER_ID,
                        Location {
                            target: target.clone(),
                            position,
                        },
                        PointerAction::Move {
                            delta: position - *cursor_last,
                        },
                    ));
                    *cursor_last = position;
                }
            } else {
                warn!("[RTT_PICK] Hit has no UV coordinates! Entity: {:?}", entity);
            }
        }
    }
    
    // Pipe mouse button presses to the virtual pointer
    for window_event in window_events.read() {
        if let WindowEvent::MouseButtonInput(input) = window_event {
            let button = match input.button {
                MouseButton::Left => PointerButton::Primary,
                MouseButton::Right => PointerButton::Secondary,
                MouseButton::Middle => PointerButton::Middle,
                _ => continue,
            };
            let action = match input.state {
                bevy::input::ButtonState::Pressed => PointerAction::Press(button),
                bevy::input::ButtonState::Released => PointerAction::Release(button),
            };
            pointer_inputs.write(PointerInput::new(
                SPHERE_POINTER_ID,
                Location {
                    target: target.clone(),
                    position: *cursor_last,
                },
                action,
            ));
        }
    }
}

