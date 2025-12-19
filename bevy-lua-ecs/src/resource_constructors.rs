use bevy::prelude::*;
use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Generic OS-level utilities for Lua (reusable across ANY game)
/// These wrap OS operations that Lua cannot do directly
pub struct OsUtilities;

impl OsUtilities {
    // Reserved for future generic utilities
}

/// Type for constructor functions that create resources from Lua data
/// Takes Lua context, data table, and returns a boxed reflected resource
type ConstructorFn = Arc<dyn Fn(&Lua, LuaValue) -> LuaResult<Box<dyn Reflect>> + Send + Sync>;

/// Registry for resource constructor functions
/// This is GENERIC - works for ANY resource type that needs construction
#[derive(Resource, Clone)]
pub struct ResourceConstructorRegistry {
    constructors: Arc<Mutex<HashMap<String, ConstructorFn>>>,
}

impl Default for ResourceConstructorRegistry {
    fn default() -> Self {
        Self {
            constructors: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl ResourceConstructorRegistry {
    /// Register a constructor function for a resource type
    /// The constructor should create a reflected resource from Lua data
    pub fn register<F>(&self, name: impl Into<String>, constructor: F)
    where
        F: Fn(&Lua, LuaValue) -> LuaResult<Box<dyn Reflect>> + Send + Sync + 'static,
    {
        self.constructors
            .lock()
            .unwrap()
            .insert(name.into(), Arc::new(constructor));
    }

    /// Try to construct a resource from Lua data
    /// Returns None if no constructor is registered for this type
    pub fn try_construct(
        &self,
        lua: &Lua,
        name: &str,
        data: LuaValue,
    ) -> Option<LuaResult<Box<dyn Reflect>>> {
        let constructors = self.constructors.lock().unwrap();
        constructors
            .get(name)
            .map(|constructor| constructor(lua, data))
    }

    /// Check if a constructor is registered for a resource type
    pub fn has_constructor(&self, name: &str) -> bool {
        self.constructors.lock().unwrap().contains_key(name)
    }
}
