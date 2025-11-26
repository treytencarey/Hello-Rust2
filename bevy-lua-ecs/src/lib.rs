// Modules
pub mod components;
pub mod component_update_queue;
pub mod component_updater;
pub mod entity_spawner;
pub mod lua_integration;
pub mod lua_systems;
pub mod lua_world_api;
pub mod spawn_queue;
pub mod serde_components;
pub mod asset_loading;
pub mod resource_queue;
pub mod resource_inserter;
pub mod resource_builder;
pub mod resource_constructors;
pub mod event_reader;

// Re-export commonly used types
pub use components::{ComponentRegistry, LuaCustomComponents};
pub use entity_spawner::process_spawn_queue;
pub use lua_integration::{LuaScriptContext, LuaSpawnPlugin};
pub use spawn_queue::SpawnQueue;
pub use component_update_queue::ComponentUpdateQueue;
pub use component_updater::process_component_updates;
pub use serde_components::SerdeComponentRegistry;
pub use lua_systems::{LuaSystemRegistry, run_lua_systems};
pub use asset_loading::{AssetRegistry, add_asset_loading_to_lua};
pub use lua_world_api::{LuaQueryBuilder, LuaEntitySnapshot, execute_query};
pub use resource_queue::ResourceQueue;
pub use resource_inserter::process_resource_queue;
pub use resource_builder::ResourceBuilderRegistry;
pub use resource_constructors::{ResourceConstructorRegistry, OsUtilities};
pub use event_reader::reflection_to_lua;

#[cfg(feature = "networking")]
pub use resource_constructors::register_networking_constructors;

/// Register common Bevy event types for Lua access via world:read_events()
/// 
/// This is a convenience function that registers the most commonly used Bevy events.
/// Call this after adding DefaultPlugins and before adding LuaSpawnPlugin.
/// 
/// Events registered:
/// - Window: CursorMoved, FileDragAndDrop, WindowResized, WindowFocused, WindowClosed
/// - Keyboard: KeyboardInput
/// - Mouse: MouseButtonInput, MouseWheel, MouseMotion
/// 
/// # Example
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_lua_ecs::*;
/// 
/// fn main() {
///     let mut app = App::new();
///     app.add_plugins(DefaultPlugins);
///     
///     // Register common events for Lua
///     register_common_bevy_events(&mut app);
///     
///     app.add_plugins(LuaSpawnPlugin);
///     app.run();
/// }
/// ```
pub fn register_common_bevy_events(app: &mut bevy::prelude::App) {
    use bevy::prelude::*;
    
    // Window events
    app.register_type::<bevy::window::CursorMoved>();
    app.register_type::<Events<bevy::window::CursorMoved>>();
    
    app.register_type::<bevy::window::FileDragAndDrop>();
    app.register_type::<Events<bevy::window::FileDragAndDrop>>();
    
    app.register_type::<bevy::window::WindowResized>();
    app.register_type::<Events<bevy::window::WindowResized>>();
    
    app.register_type::<bevy::window::WindowFocused>();
    app.register_type::<Events<bevy::window::WindowFocused>>();
    
    app.register_type::<bevy::window::WindowClosed>();
    app.register_type::<Events<bevy::window::WindowClosed>>();
    
    // Input events
    app.register_type::<bevy::input::keyboard::KeyboardInput>();
    app.register_type::<Events<bevy::input::keyboard::KeyboardInput>>();
    
    app.register_type::<bevy::input::mouse::MouseButtonInput>();
    app.register_type::<Events<bevy::input::mouse::MouseButtonInput>>();
    
    app.register_type::<bevy::input::mouse::MouseWheel>();
    app.register_type::<Events<bevy::input::mouse::MouseWheel>>();
    
    app.register_type::<bevy::input::mouse::MouseMotion>();
    app.register_type::<Events<bevy::input::mouse::MouseMotion>>();
}

/// Macro to register multiple event types for Lua access at once
/// 
/// This registers both the event type and its Events<T> wrapper for reflection.
/// Call this after adding DefaultPlugins but before LuaSpawnPlugin.
/// 
/// # Example
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_lua_ecs::*;
/// 
/// fn main() {
///     let mut app = App::new();
///     app.add_plugins(DefaultPlugins);
///     
///     // Register common events for Lua access
///     register_lua_events!(app,
///         bevy::window::CursorMoved,
///         bevy::window::FileDragAndDrop,
///         bevy::input::keyboard::KeyboardInput,
///         bevy::input::mouse::MouseButtonInput,
///         bevy::input::mouse::MouseWheel,
///         bevy::input::mouse::MouseMotion,
///     );
///     
///     app.add_plugins(LuaSpawnPlugin);
///     app.run();
/// }
/// ```
#[macro_export]
macro_rules! register_lua_events {
    ($app:expr, $($event_type:ty),+ $(,)?) => {
        $(
            $app.register_type::<$event_type>();
            $app.register_type::<bevy::prelude::Events<$event_type>>();
        )+
    };
}
