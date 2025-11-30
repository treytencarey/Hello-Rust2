use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2d);
    
    // Method 1: 2D World Text (using Transform) - REQUIRES Text2d!
    commands.spawn((
        Text2d::new("World Text"),  // Text2d, not Text!
        TextFont {
            font_size: 64.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.8, 0.2)),
        Transform::from_xyz(0.0, 100.0, 0.0),
    ));
    
    // Method 2: UI Text (Node-based)
    commands.spawn((
        Text::new("UI Text"),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
    ));
    
    info!("Text entities spawned!");
}
