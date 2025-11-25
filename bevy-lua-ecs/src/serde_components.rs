use bevy::prelude::*;
use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Function that deserializes a component from Lua and inserts it
type SerdeComponentHandler = Box<dyn Fn(&LuaValue, &mut EntityCommands) -> LuaResult<()> + Send + Sync>;

/// Function that deserializes a resource from Lua and inserts it into World
type SerdeResourceHandler = Box<dyn Fn(&LuaValue, &mut World) -> LuaResult<()> + Send + Sync>;

/// Registry for components and resources that use serde instead of Reflect
/// This is required for types like `Collider` that implement `Deserialize` but not `Reflect`.
#[derive(Resource, Default, Clone)]
pub struct SerdeComponentRegistry {
    pub component_handlers: Arc<Mutex<HashMap<String, SerdeComponentHandler>>>,
    resource_handlers: Arc<Mutex<HashMap<String, SerdeResourceHandler>>>,
    /// Track which resources have been inserted (generic tracking)
    inserted_resources: Arc<Mutex<std::collections::HashSet<String>>>,
}

impl SerdeComponentRegistry {
    /// Register a component that implements Deserialize
    pub fn register<T>(&mut self, name: impl Into<String>)
    where
        T: Component + for<'de> serde::Deserialize<'de>,
    {
        let name = name.into();
        let handler = Box::new(move |data: &LuaValue, entity: &mut EntityCommands| {
            // Convert LuaValue to T using serde via JSON intermediate
            let json_value = serde_json::to_value(data)
                .map_err(|e| LuaError::SerializeError(format!("Failed to serialize Lua value: {}", e)))?;
            
            let component: T = serde_json::from_value(json_value)
                .map_err(|e| LuaError::DeserializeError(format!("Failed to deserialize component: {}", e)))?;
            
            entity.insert(component);
            Ok(())
        });
        
        self.component_handlers.lock().unwrap().insert(name, handler);
    }
    
    /// Register a marker component (zero-sized type, no fields)
    pub fn register_marker<T>(&mut self, name: impl Into<String>)
    where
        T: Component + Default,
    {
        let name = name.into();
        let handler = Box::new(move |_data: &LuaValue, entity: &mut EntityCommands| {
            // Marker component - ignore data and just insert it
            entity.insert(T::default());
            Ok(())
        });
        
        self.component_handlers.lock().unwrap().insert(name, handler);
    }
    
    /// Register a newtype wrapper component (tuple struct with single field)
    /// Accepts a scalar value from Lua and wraps it in an array for deserialization
    pub fn register_newtype<T>(&mut self, name: impl Into<String>)
    where
        T: Component + for<'de> serde::Deserialize<'de>,
    {
        let name = name.into();
        let handler = Box::new(move |data: &LuaValue, entity: &mut EntityCommands| {
            // Wrap scalar value in array for tuple struct deserialization
            let json_value = serde_json::to_value(data)
                .map_err(|e| LuaError::SerializeError(format!("Failed to serialize Lua value: {}", e)))?;
            
            // Wrap in array to match tuple struct format
            let wrapped = serde_json::json!([json_value]);
            
            let component: T = serde_json::from_value(wrapped)
                .map_err(|e| LuaError::DeserializeError(format!("Failed to deserialize newtype component: {}", e)))?;
            
            entity.insert(component);
            Ok(())
        });
        
        self.component_handlers.lock().unwrap().insert(name, handler);
    }
    
    /// Register a resource that implements Deserialize
    pub fn register_resource<T>(&mut self, name: impl Into<String>)
    where
        T: Resource + for<'de> serde::Deserialize<'de>,
    {
        let name = name.into();
        let name_clone = name.clone();
        let inserted_resources = self.inserted_resources.clone();
        let handler = Box::new(move |data: &LuaValue, world: &mut World| {
            // Convert LuaValue to T using serde via JSON intermediate
            let json_value = serde_json::to_value(data)
                .map_err(|e| LuaError::SerializeError(format!("Failed to serialize Lua value: {}", e)))?;
            
            let resource: T = serde_json::from_value(json_value)
                .map_err(|e| LuaError::DeserializeError(format!("Failed to deserialize resource: {}", e)))?;
            
            world.insert_resource(resource);
            
            // Track that this resource has been inserted (generic tracking)
            inserted_resources.lock().unwrap().insert(name_clone.clone());
            
            Ok(())
        });
        
        self.resource_handlers.lock().unwrap().insert(name, handler);
    }
    
    /// Try to handle a component via serde
    pub fn try_handle(&self, name: &str, data: &LuaValue, entity: &mut EntityCommands) -> Option<LuaResult<()>> {
        let handlers = self.component_handlers.lock().unwrap();
        if let Some(handler) = handlers.get(name) {
            Some(handler(data, entity))
        } else {
            None
        }
    }
    
    /// Try to insert a resource via serde
    pub fn try_insert_resource(&self, name: &str, data: &LuaValue, world: &mut World) -> Option<LuaResult<()>> {
        let handlers = self.resource_handlers.lock().unwrap();
        if let Some(handler) = handlers.get(name) {
            Some(handler(data, world))
        } else {
            None
        }
    }
    
    /// Check if a resource has been inserted (generic tracking)
    pub fn has_resource(&self, name: &str) -> bool {
        self.inserted_resources.lock().unwrap().contains(name)
    }
    
    /// Mark a resource as inserted (for tracking by Rust code)
    /// This is generic - works for any resource name
    pub fn mark_resource_inserted(&self, name: impl Into<String>) {
        self.inserted_resources.lock().unwrap().insert(name.into());
    }
}

/// Macro to create a SerdeComponentRegistry with multiple components
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

/// Macro to create a SerdeComponentRegistry with multiple resources
#[macro_export]
macro_rules! serde_resources {
    ($($ty:ty),* $(,)?) => {{
        let mut registry = $crate::serde_components::SerdeComponentRegistry::default();
        $(
            registry.register_resource::<$ty>(
                std::any::type_name::<$ty>()
                    .split("::")
                    .last()
                    .unwrap()
            );
        )*
        registry
    }};
}
