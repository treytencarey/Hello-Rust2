use bevy::prelude::*;

fn main() {
    // Spawn a real button to see what components it has
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let entity = app.world_mut().spawn((
        Button,
        Node {
            width: Val::Px(200.0),
            height: Val::Px(60.0),
            ..default()
        },
    )).id();
    
    println!("Button entity spawned: {:?}", entity);
    
    // List all components
    let entity_ref = app.world().entity(entity);
    println!("Components on button:");
    for component_id in entity_ref.archetype().components() {
        if let Some(info) = app.world().components().get_info(component_id) {
            println!("  - {}", info.name());
        }
    }
}
