// Modules
pub mod components;
pub mod component_update_queue;
pub mod entity_spawner;
pub mod lua_integration;
pub mod lua_systems;
pub mod lua_world_api;
pub mod spawn_queue;
pub mod serde_components;
pub mod asset_loading;

// Re-export commonly used types
pub use components::{ComponentRegistry, LuaCustomComponents};
pub use entity_spawner::process_spawn_queue;
pub use lua_integration::{LuaScriptContext, LuaSpawnPlugin};
pub use spawn_queue::SpawnQueue;
pub use component_update_queue::ComponentUpdateQueue;
pub use serde_components::SerdeComponentRegistry;
pub use lua_systems::{LuaSystemRegistry, run_lua_systems};
pub use asset_loading::{AssetRegistry, add_asset_loading_to_lua};
pub use lua_world_api::{LuaQueryBuilder, LuaEntitySnapshot, execute_query};
