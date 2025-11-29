use bevy::prelude::*;
use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A resource builder function that inserts a resource directly into the World
/// Takes Lua context, data, and World, inserts the resource
pub type ResourceBuilderFn = Arc<dyn Fn(&Lua, LuaValue, &mut World) -> LuaResult<()> + Send + Sync>;

/// A resource removal function that removes a resource from the World
pub type ResourceRemovalFn = Arc<dyn Fn(&mut World) + Send + Sync>;

/// Registry for resource builders
/// This is GENERIC - works for ANY resource type
#[derive(Resource, Clone)]
pub struct ResourceBuilderRegistry {
    builders: Arc<Mutex<HashMap<String, ResourceBuilderFn>>>,
    removers: Arc<Mutex<HashMap<String, ResourceRemovalFn>>>,
}

impl Default for ResourceBuilderRegistry {
    fn default() -> Self {
        Self {
            builders: Arc::new(Mutex::new(HashMap::new())),
            removers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl ResourceBuilderRegistry {
    /// Register a builder function
    /// The builder should insert the resource directly into the World
    pub fn register<F>(&self, name: impl Into<String>, builder: F)
    where
        F: Fn(&Lua, LuaValue, &mut World) -> LuaResult<()> + Send + Sync + 'static,
    {
        self.builders.lock().unwrap().insert(name.into(), Arc::new(builder));
    }
    
    /// Register a removal function
    pub fn register_removal<F>(&self, name: impl Into<String>, remover: F)
    where
        F: Fn(&mut World) + Send + Sync + 'static,
    {
        self.removers.lock().unwrap().insert(name.into(), Arc::new(remover));
    }
    
    /// Try to build and insert a resource from Lua data
    /// Returns None if no builder is registered for this type
    pub fn try_build(&self, lua: &Lua, name: &str, data: LuaValue, world: &mut World) -> Option<LuaResult<()>> {
        let builders = self.builders.lock().unwrap();
        builders.get(name).map(|builder| builder(lua, data, world))
    }
    
    /// Try to remove a resource by name
    /// Returns true if a remover was found and executed
    pub fn try_remove(&self, name: &str, world: &mut World) -> bool {
        let removers = self.removers.lock().unwrap();
        if let Some(remover) = removers.get(name) {
            remover(world);
            true
        } else {
            false
        }
    }
    
    /// Check if a builder is registered for a resource type
    pub fn has_builder(&self, name: &str) -> bool {
        self.builders.lock().unwrap().contains_key(name)
    }
}
