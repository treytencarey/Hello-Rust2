// Modules
pub mod components;
pub mod component_update_queue;
pub mod component_updater;
pub mod entity_spawner;
pub mod lua_integration;
pub mod lua_systems;
pub mod lua_world_api;
pub mod reflection;
pub mod spawn_queue;
pub mod serde_components;

// Re-exports
pub use spawn_queue::SpawnQueue;
pub use component_update_queue::ComponentUpdateQueue;
pub use entity_spawner::process_spawn_queue;
pub use component_updater::process_component_updates;
pub use lua_integration::{LuaSpawnPlugin, LuaScriptContext};
pub use reflection::BundleRegistry;
pub use components::{ComponentRegistry, LuaCustomComponents};
pub use lua_systems::{LuaSystemRegistry, run_lua_systems};
pub use lua_world_api::{LuaQueryBuilder, LuaEntitySnapshot, execute_query};

