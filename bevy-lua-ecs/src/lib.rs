// Modules
pub mod asset_loading;
pub mod auto_bindings;
pub mod bitflags_registry;
pub mod component_lua_trait;
pub mod component_update_queue;
pub mod component_updater;
pub mod components;
pub mod despawn_queue;
pub mod entity_spawner;
pub mod event_reader;
pub mod event_sender;
pub mod lua_file_watcher;
pub mod lua_integration;
pub mod lua_observers;
pub mod lua_spawn_builder;
pub mod lua_systems;
pub mod lua_world_api;
pub mod network_asset_trait;
pub mod os_utilities;
pub mod path_utils;
pub mod reflection;
pub mod resource_builder;
pub mod resource_constructors;
pub mod resource_inserter;
pub mod resource_lua_trait;
pub mod resource_queue;
pub mod script_cache;
pub mod script_entities;
pub mod script_registry;
pub mod serde_components;
pub mod spawn_queue;
pub mod systemparam_lua_trait;

// Re-export commonly used types
pub use asset_loading::{
    add_asset_loading_to_lua, parse_enum_from_string, register_asset_types_runtime,
    register_cloner_if_clone, AssetAdder, AssetCloner, AssetRegistry, HandleCreator, HandleSetter,
    NewtypeWrapperCreator, ReflectDirectAssetAdd,
};
pub use auto_bindings::{
    dispatch_lua_events, dispatch_lua_messages, dispatch_systemparam_method,
    register_auto_bindings, register_auto_events,
};
pub use bitflags_registry::BitflagsRegistry;
pub use component_lua_trait::LuaComponentRegistry;
pub use component_update_queue::ComponentUpdateQueue;
pub use component_updater::process_component_updates;
pub use components::{register_entity_wrappers_runtime, ComponentRegistry, LuaCustomComponents};
pub use despawn_queue::{process_despawn_queue, DespawnQueue};
pub use entity_spawner::process_spawn_queue;
pub use event_reader::{
    lua_table_to_dynamic, lua_table_to_dynamic_with_assets, lua_to_reflection, reflection_to_lua,
};
pub use event_sender::{LuaEventSenderPlugin, PendingLuaEvents, PendingLuaMessages};
pub use lua_file_watcher::{LuaFileChangeEvent, LuaFileWatcherPlugin};
pub use lua_integration::{LuaScriptContext, LuaSpawnPlugin};
pub use lua_observers::{
    attach_lua_observers, dispatch_lua_observer_internal, process_observer_registrations,
    set_observer_attacher, LuaObserverRegistry, LuaObserversAttached,
};
pub use lua_spawn_builder::LuaSpawnBuilder;
pub use lua_systems::{run_lua_systems, LuaSystemRegistry};
pub use lua_world_api::{execute_query, LuaEntitySnapshot, LuaQueryBuilder};
pub use network_asset_trait::{AssetDownloadStatus, NetworkAssetLoader, NetworkAssetRequestor};
pub use path_utils::{normalize_path, normalize_path_separators, to_forward_slash};
pub use resource_builder::ResourceBuilderRegistry;
pub use resource_constructors::{OsUtilities, ResourceConstructorRegistry};
pub use resource_inserter::process_resource_queue;
pub use resource_lua_trait::LuaResourceRegistry;
pub use resource_queue::ResourceQueue;
pub use script_cache::ScriptCache;
pub use script_entities::{despawn_instance_entities, ScriptInstance, ScriptOwned};
pub use script_registry::ScriptRegistry;
pub use serde_components::SerdeComponentRegistry;
pub use spawn_queue::SpawnQueue;
pub use systemparam_lua_trait::{
    call_component_method_global, call_read_events_global, call_systemparam_method_global,
    call_write_events_global, call_write_messages_global, set_component_method_dispatcher,
    set_event_dispatcher, set_event_write_dispatcher, set_message_write_dispatcher,
    set_systemparam_dispatcher, LuaSystemParamMethods, LuaSystemParamRegistry,
};

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
