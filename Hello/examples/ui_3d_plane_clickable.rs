use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            update_button_bounds,
            raycast_plane_interaction,
            button_system,
            rotate_plane,
        ).chain())
        .run();
}

#[derive(Component)]
struct RotatingPlane;

#[derive(Component)]
struct InteractivePlane {
    texture_size: Vec2,
}

#[derive(Component)]
struct VirtualPointer {
    position: Vec2,
    is_pressed: bool,
    was_pressed: bool,
}

#[derive(Component)]
struct TextureButton {
    rect: Rect,
}

#[derive(Component)]
struct UiButton;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Create a render texture for the UI
    let size = Extent3d {
        width: 512,
        height: 512,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);

    let image_handle = images.add(image);

    // 2D Camera that renders UI to the texture
    let ui_camera = commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            target: image_handle.clone().into(),
            clear_color: ClearColorConfig::Custom(Color::srgba(0.2, 0.2, 0.3, 1.0)),
            ..default()
        },
    )).id();

    // Virtual pointer for simulating interactions on the UI texture
    commands.spawn(VirtualPointer {
        position: Vec2::ZERO,
        is_pressed: false,
        was_pressed: false,
    });

    // UI Root node with button
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            UiTargetCamera(ui_camera),
        ))
        .with_children(|parent| {
            // Title text
            parent.spawn((
                Text::new("3D Plane with UI"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    TextureButton {
                        rect: Rect::default(), // Will be calculated dynamically
                    },
                    UiButton,
                ))
                .with_children(|button_parent| {
                    button_parent.spawn((
                        Text::new("Click Me!"),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });

    // 3D Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Plane with the UI texture
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(3.0, 3.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(image_handle.clone()),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RotatingPlane,
        InteractivePlane {
            texture_size: Vec2::new(size.width as f32, size.height as f32),
        },
    ));

    info!("Setup complete! Click on the plane to interact with the UI.");
}

fn update_button_bounds(
    mut button_query: Query<(&Node, &ComputedNode, &mut TextureButton, &GlobalTransform), With<UiButton>>,
) {
    for (_node, computed, mut texture_button, global_transform) in &mut button_query {
        // Get the button's computed size
        let size = computed.size();
        
        // Get the full translation (including Z for debugging)
        let translation = global_transform.translation();
        let center = translation.truncate();
        
        // Calculate top-left corner from center
        let min = center - size / 2.0;
        let max = min + size;
        
        let new_rect = Rect { min, max };
        
        // Always log to debug the coordinate system
        if texture_button.rect.min == Vec2::ZERO || (new_rect.min - texture_button.rect.min).length() > 0.1 {
            info!("=== UI NODE DEBUG ===");
            info!("  GlobalTransform.translation: ({:.2}, {:.2}, {:.2})", translation.x, translation.y, translation.z);
            info!("  ComputedNode.size: ({:.2}, {:.2})", size.x, size.y);
            info!("  Calculated center (2D): ({:.2}, {:.2})", center.x, center.y);
            info!("  Calculated bounds: min({:.2},{:.2}) max({:.2},{:.2})", 
                new_rect.min.x, new_rect.min.y, new_rect.max.x, new_rect.max.y);
            info!("  Expected texture space: 0-512 for 512x512 texture");
        }
        
        texture_button.rect = new_rect;
    }
}

fn raycast_plane_interaction(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    plane_query: Query<(&GlobalTransform, &InteractivePlane, &Mesh3d), With<RotatingPlane>>,
    mut virtual_pointer: Query<&mut VirtualPointer>,
) {
    let Ok(window) = windows.single() else {
        info!("No window found");
        return;
    };
    
    let Some(cursor_position) = window.cursor_position() else {
        return; // Cursor not in window
    };

    let Some((camera, camera_transform)) = cameras.iter().find(|(cam, _)| cam.order == 0) else {
        info!("No 3D camera found");
        return;
    };

    let Ok((plane_transform, interactive_plane, _mesh_handle)) = plane_query.single() else {
        info!("No plane found");
        return;
    };

    let Ok(mut virtual_pointer) = virtual_pointer.single_mut() else {
        info!("No virtual pointer found");
        return;
    };

    // Convert cursor position to ray
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        info!("Failed to convert cursor to ray");
        return;
    };

    // Get plane's world transform
    let plane_translation = plane_transform.translation();
    let plane_rotation = plane_transform.to_scale_rotation_translation().1;
    let plane_normal = plane_rotation * Vec3::Y;
    
    // Ray-plane intersection
    let denom = plane_normal.dot(*ray.direction);
    if denom.abs() > 1e-6 {
        let t = (plane_translation - ray.origin).dot(plane_normal) / denom;
        if t >= 0.0 {
            let hit_point = ray.origin + *ray.direction * t;
            
            // Transform hit point to plane's local space
            let local_hit = plane_transform.affine().inverse().transform_point3(hit_point);
            
            // Convert to UV coordinates (plane is 3x3, so range is -1.5 to 1.5)
            // Map to 0-1 range
            let u = (local_hit.x / 3.0) + 0.5;
            let v = 1.0 - ((local_hit.z / 3.0) + 0.5); // Flip V
            
            // Check if within plane bounds
            if u >= 0.0 && u <= 1.0 && v >= 0.0 && v <= 1.0 {
                // Convert UV to texture pixel coordinates
                let texture_x = u * interactive_plane.texture_size.x;
                let texture_y = v * interactive_plane.texture_size.y;
                
                virtual_pointer.position = Vec2::new(texture_x, texture_y);
                virtual_pointer.was_pressed = virtual_pointer.is_pressed;
                virtual_pointer.is_pressed = buttons.pressed(MouseButton::Left);
                
                if buttons.just_pressed(MouseButton::Left) {
                    info!("=== RAYCAST HIT DEBUG ===");
                    info!("  Hit point (world): ({:.2}, {:.2}, {:.2})", hit_point.x, hit_point.y, hit_point.z);
                    info!("  Hit point (local): ({:.2}, {:.2}, {:.2})", local_hit.x, local_hit.y, local_hit.z);
                    info!("  Plane size: 3.0 x 3.0 (local range: -1.5 to 1.5)");
                    info!("  UV coordinates: ({:.4}, {:.4})", u, v);
                    info!("  Texture coordinates: ({:.1}, {:.1})", texture_x, texture_y);
                    info!("  Texture size: {}x{}", interactive_plane.texture_size.x, interactive_plane.texture_size.y);
                }
                
                return;
            } else {
                if buttons.just_pressed(MouseButton::Left) {
                    info!("Hit plane but outside bounds: UV({:.2}, {:.2})", u, v);
                }
            }
        } else {
            if buttons.just_pressed(MouseButton::Left) {
                info!("Hit behind camera (t={})", t);
            }
        }
    } else {
        if buttons.just_pressed(MouseButton::Left) {
            info!("Ray parallel to plane (denom={})", denom);
        }
    }
    
    // No hit - reset pointer
    virtual_pointer.was_pressed = virtual_pointer.is_pressed;
    virtual_pointer.is_pressed = false;
}

fn button_system(
    virtual_pointer: Query<&VirtualPointer>,
    mut button_query: Query<
        (&TextureButton, &mut BackgroundColor, &Children)
    >,
    mut text_query: Query<&mut TextColor>,
) {
    let Ok(virtual_pointer) = virtual_pointer.single() else {
        return;
    };

    for (texture_button, mut color, children) in &mut button_query {
        let button_rect = &texture_button.rect;
        
        // Check if pointer is over button
        let is_hovered = virtual_pointer.position.x >= button_rect.min.x
            && virtual_pointer.position.x <= button_rect.max.x
            && virtual_pointer.position.y >= button_rect.min.y
            && virtual_pointer.position.y <= button_rect.max.y;
        
        let is_pressed = is_hovered && virtual_pointer.is_pressed;
        let was_just_pressed = is_hovered && virtual_pointer.is_pressed && !virtual_pointer.was_pressed;
        
        // Debug logging for hit testing
        if virtual_pointer.is_pressed && !virtual_pointer.was_pressed {
            info!("=== BUTTON HIT TEST ===");
            info!("  Pointer position: ({:.1}, {:.1})", virtual_pointer.position.x, virtual_pointer.position.y);
            info!("  Button bounds: ({:.1},{:.1}) to ({:.1},{:.1})", 
                button_rect.min.x, button_rect.min.y, button_rect.max.x, button_rect.max.y);
            info!("  Is hovered: {}", is_hovered);
            info!("  Hit test details:");
            info!("    X: {:.1} >= {:.1} && {:.1} <= {:.1} = {}", 
                virtual_pointer.position.x, button_rect.min.x, 
                virtual_pointer.position.x, button_rect.max.x,
                virtual_pointer.position.x >= button_rect.min.x && virtual_pointer.position.x <= button_rect.max.x);
            info!("    Y: {:.1} >= {:.1} && {:.1} <= {:.1} = {}", 
                virtual_pointer.position.y, button_rect.min.y, 
                virtual_pointer.position.y, button_rect.max.y,
                virtual_pointer.position.y >= button_rect.min.y && virtual_pointer.position.y <= button_rect.max.y);
        }
        
        // Update button appearance
        if let Ok(mut text_color) = text_query.get_mut(children[0]) {
            if is_pressed {
                *color = BackgroundColor(Color::srgb(0.35, 0.75, 0.35));
                text_color.0 = Color::srgb(0.1, 0.1, 0.1);
                if was_just_pressed {
                    info!("Button pressed via raycast interaction!");
                }
            } else if is_hovered {
                *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
                text_color.0 = Color::srgb(1.0, 1.0, 1.0);
            } else {
                *color = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));
                text_color.0 = Color::srgb(0.9, 0.9, 0.9);
            }
        }
    }
}

fn rotate_plane(time: Res<Time>, mut query: Query<&mut Transform, With<RotatingPlane>>) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_rotation_y(time.elapsed_secs() * 0.5);
    }
}
