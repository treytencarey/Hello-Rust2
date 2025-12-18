// Modules
pub mod components;
pub mod component_update_queue;
pub mod component_updater;
pub mod entity_spawner;
pub mod lua_integration;
pub mod lua_systems;
pub mod lua_world_api;
pub mod spawn_queue;
pub mod despawn_queue;
pub mod serde_components;
pub mod asset_loading;
pub mod resource_queue;
pub mod resource_inserter;
pub mod resource_builder;
pub mod resource_constructors;
pub mod resource_lua_trait;
pub mod systemparam_lua_trait;
pub mod component_lua_trait;
pub mod event_reader;
pub mod auto_bindings;
pub mod os_utilities;
pub mod reflection;
pub mod script_entities;
pub mod script_registry;
pub mod script_cache;
pub mod lua_file_watcher;
pub mod event_sender;
pub mod lua_spawn_builder;
pub mod lua_observers;
pub mod bitflags_registry;
pub mod network_asset_trait;


// Re-export commonly used types
pub use components::{ComponentRegistry, LuaCustomComponents, register_entity_wrappers_runtime};
pub use entity_spawner::process_spawn_queue;
pub use lua_integration::{LuaScriptContext, LuaSpawnPlugin};
pub use spawn_queue::SpawnQueue;
pub use despawn_queue::{DespawnQueue, process_despawn_queue};
pub use component_update_queue::ComponentUpdateQueue;
pub use component_updater::process_component_updates;
pub use serde_components::SerdeComponentRegistry;
pub use lua_systems::{LuaSystemRegistry, run_lua_systems};
pub use asset_loading::{AssetRegistry, add_asset_loading_to_lua, HandleSetter, AssetAdder, AssetCloner, HandleCreator, NewtypeWrapperCreator, ReflectDirectAssetAdd, register_cloner_if_clone, register_asset_types_runtime, parse_enum_from_string};
pub use lua_world_api::{LuaQueryBuilder, LuaEntitySnapshot, execute_query};
pub use resource_queue::ResourceQueue;
pub use resource_inserter::process_resource_queue;
pub use resource_builder::ResourceBuilderRegistry;
pub use resource_constructors::{ResourceConstructorRegistry, OsUtilities};
pub use resource_lua_trait::LuaResourceRegistry;
pub use systemparam_lua_trait::{LuaSystemParamRegistry, LuaSystemParamMethods, set_systemparam_dispatcher, call_systemparam_method_global, set_event_dispatcher, call_read_events_global, set_event_write_dispatcher, call_write_events_global, set_message_write_dispatcher, call_write_messages_global};
pub use component_lua_trait::LuaComponentRegistry;
pub use event_reader::{reflection_to_lua, lua_to_reflection, lua_table_to_dynamic, lua_table_to_dynamic_with_assets};
pub use auto_bindings::{register_auto_bindings, register_auto_events, dispatch_lua_events, dispatch_lua_messages, dispatch_systemparam_method};
pub use script_entities::{ScriptOwned, ScriptInstance, despawn_instance_entities};
pub use lua_spawn_builder::LuaSpawnBuilder;
pub use lua_observers::{LuaObserverRegistry, process_observer_registrations, attach_lua_observers, LuaObserversAttached, set_observer_attacher, dispatch_lua_observer_internal};
pub use script_registry::ScriptRegistry;
pub use script_cache::ScriptCache;
pub use lua_file_watcher::{LuaFileWatcherPlugin, LuaFileChangeEvent};
pub use event_sender::{PendingLuaEvents, PendingLuaMessages, LuaEventSenderPlugin};
pub use bitflags_registry::BitflagsRegistry;
pub use network_asset_trait::{NetworkAssetLoader, NetworkAssetRequestor, AssetDownloadStatus};


/// Register common Bevy event types for Lua access via world:read_events()
/// 
/// This function uses auto-generated event registrations from the build script.
/// The events are defined in [package.metadata.lua_events] in Cargo.toml.
/// 
/// **IMPORTANT for Bevy Replicon users:** Call this after DefaultPlugins but BEFORE
/// adding RepliconPlugins to ensure consistent event registration order between
/// client and server. Protocol mismatches will occur if registration order differs.
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
    // Use the auto-generated event registration
    register_auto_events(app);
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
