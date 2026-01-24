// Hello library exports for examples and main binary
//
// This file exports modules that need to be accessible

// Plugin groups - always available
pub mod plugins;

// Auto-generated Lua bindings - always required for LuaBindingsPlugin
pub mod auto_resource_bindings;

// Text input wrapper for Lua spawning
pub mod text_input;

// Frame profiling for performance analysis
pub mod frame_profiler;

#[cfg(feature = "physics")]
pub mod rapier;

#[cfg(feature = "tiled")]
pub mod tiled;

#[cfg(feature = "networking")]
pub mod networking;

#[cfg(feature = "networking")]
pub mod network_asset_client;

#[cfg(feature = "networking")]
pub mod asset_server_delivery;

#[cfg(feature = "networking")]
pub mod network_asset_integration;

#[cfg(feature = "networking")]
pub mod subscription_registry;

#[cfg(feature = "networking")]
pub mod asset_events;

#[cfg(feature = "networking")]
pub mod server_hash_tracker;

#[cfg(feature = "networking")]
pub mod upload_state;
