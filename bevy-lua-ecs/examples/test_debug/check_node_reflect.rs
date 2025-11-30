use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    
    let type_registry = app.world().resource::<AppTypeRegistry>();
    let registry = type_registry.read();
    
    // Check if Node is registered
    let node_paths = vec!["Node", "bevy_ui::ui_node::Node", "bevy_ui::Node"];
    for path in node_paths {
        if let Some(registration) = registry.get_with_short_type_path(path) {
            println!("Found: {}", path);
            println!("Full type path: {}", registration.type_info().type_path());
            
            if registration.data::<ReflectDefault>().is_some() {
                println!("  - Has ReflectDefault");
            } else {
                println!("  - MISSING ReflectDefault");
            }
            
            if registration.data::<bevy::ecs::reflect::ReflectComponent>().is_some() {
                println!("  - Has ReflectComponent");
            } else {
                println!("  - MISSING ReflectComponent");
            }
            
            // Print fields
            if let bevy::reflect::TypeInfo::Struct(struct_info) = registration.type_info() {
                println!("  Fields:");
                for i in 0..struct_info.field_len() {
                    if let Some(field) = struct_info.field_at(i) {
                        println!("    - {}: {}", field.name(), field.type_path());
                    }
                }
            }
            break;
        }
    }
}
