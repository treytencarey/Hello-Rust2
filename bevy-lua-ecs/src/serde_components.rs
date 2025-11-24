use bevy::prelude::*;
use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Function that deserializes a component from Lua and inserts it
type SerdeComponentHandler = Box<dyn Fn(&LuaValue, &mut EntityCommands) -> LuaResult<()> + Send + Sync>;

/// Registry for components that use serde instead of Reflect
/// This is required for types like `Collider` that implement `Deserialize` but not `Reflect`.
#[derive(Resource, Default, Clone)]
pub struct SerdeComponentRegistry {
    handlers: Arc<Mutex<HashMap<String, SerdeComponentHandler>>>,
}

impl SerdeComponentRegistry {
    /// Register a component that implements Deserialize
    pub fn register<T>(&mut self, name: impl Into<String>)
    where
        T: Component + for<'de> serde::Deserialize<'de>,
    {
        let name = name.into();
        let handler = Box::new(move |data: &LuaValue, entity: &mut EntityCommands| {
            // We need to convert LuaValue to T using serde
            // Since we don't have the Lua context here, we use serde_json as an intermediate format.
            // mlua::Value implements Serialize, so we can convert it to serde_json::Value.
            // Then we deserialize T from serde_json::Value.
            
            // 1. Serialize LuaValue to JSON Value
            // 1. Serialize LuaValue to JSON Value
            let json_value = serde_json::to_value(data)
                .map_err(|e| LuaError::SerializeError(format!("Failed to serialize Lua value: {}", e)))?;
            
            // 2. Deserialize T from JSON Value
            let component: T = serde_json::from_value(json_value)
                .map_err(|e| LuaError::DeserializeError(format!("Failed to deserialize component: {}", e)))?;
            
            entity.insert(component);
            Ok(())
        });
        
        self.handlers.lock().unwrap().insert(name, handler);
    }
    
    /// Try to handle a component via serde
    pub fn try_handle(&self, name: &str, data: &LuaValue, entity: &mut EntityCommands) -> Option<LuaResult<()>> {
        let handlers = self.handlers.lock().unwrap();
        if let Some(handler) = handlers.get(name) {
            Some(handler(data, entity))
        } else {
            None
        }
    }
}

/// Macro to create a SerdeComponentRegistry with multiple components
/// 
/// This macro simplifies registration by automatically extracting component names
/// from their type paths.
/// 
/// # Example
/// ```
/// use bevy_lua_entity::serde_components;
/// 
/// app.insert_resource(serde_components![
///     bevy_rapier2d::prelude::Collider,
///     bevy_rapier2d::prelude::Velocity,
/// ]);
/// ```
#[macro_export]
macro_rules! serde_components {
    ($($ty:ty),* $(,)?) => {{
        let mut registry = $crate::serde_components::SerdeComponentRegistry::default();
        $(
            registry.register::<$ty>(
                std::any::type_name::<$ty>()
                    .split("::")
                    .last()
                    .unwrap()
            );
        )*
        registry
    }};
}
