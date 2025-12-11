use bevy::prelude::*;
use mlua::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Method handler for a resource
/// Takes: Lua context, World, method arguments
/// Returns: Lua value result
pub type ResourceMethod = Arc<dyn Fn(&Lua, &World, LuaMultiValue) -> LuaResult<LuaValue> + Send + Sync>;

/// Builder for registering resource methods (type-safe version)
pub struct LuaResourceMethods<R: Resource> {
    methods: HashMap<String, ResourceMethod>,
    _phantom: std::marker::PhantomData<R>,
}

impl<R: Resource + 'static> LuaResourceMethods<R> {
    pub fn new() -> Self {
        Self {
            methods: HashMap::new(),
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Add a method - type-safe!
    /// 
    /// # Example
    /// ```ignore
    /// methods.add("receive_message", |resource, lua, channel_id: u8| {
    ///     // Direct access to resource!
    ///     let msg = resource.receive_message(channel_id);
    ///     Ok(msg)
    /// });
    /// ```
    pub fn add<F, Args, Ret>(&mut self, name: &str, handler: F)
    where
        F: Fn(&mut R, &Lua, Args) -> LuaResult<Ret> + Send + Sync + 'static,
        Args: FromLuaMulti,
        Ret: IntoLua,
    {
        let handler = Arc::new(move |lua: &Lua, world: &World, args: LuaMultiValue| {
            // Parse arguments
            let parsed_args: Args = FromLuaMulti::from_lua_multi(args, lua)?;
            
            // Get mutable resource
            #[allow(invalid_reference_casting)]
            unsafe {
                let world_mut = &mut *(world as *const World as *mut World);
                if let Some(mut resource) = world_mut.get_resource_mut::<R>() {
                    let result = handler(&mut resource, lua, parsed_args)?;
                    result.into_lua(lua)
                } else {
                    Err(LuaError::RuntimeError(format!(
                        "Resource {} not found", 
                        std::any::type_name::<R>()
                    )))
                }
            }
        });
        
        self.methods.insert(name.to_string(), handler);
    }
    
    pub fn into_map(self) -> HashMap<String, ResourceMethod> {
        self.methods
    }
}

/// Registry for Lua-accessible resources
/// This is the main infrastructure for exposing resource methods to Lua
#[derive(Resource, Clone, Default)]
pub struct LuaResourceRegistry {
    resources: Arc<Mutex<HashMap<TypeId, HashMap<String, ResourceMethod>>>>,
    type_names: Arc<Mutex<HashMap<String, TypeId>>>,
}

impl LuaResourceRegistry {
    /// Register a resource type with its methods
    /// This is the main API for registering resources
    /// 
    /// # Example
    /// ```ignore
    /// registry.register_resource::<RenetClient>("RenetClient", |methods| {
    ///     methods.add("receive_message", |client, lua, channel_id: u8| {
    ///         // Implementation
    ///         Ok(())
    ///     });
    /// });
    /// ```
    pub fn register_resource<R, F>(&self, type_name: &str, register_fn: F)
    where
        R: Resource + 'static,
        F: FnOnce(&mut LuaResourceMethods<R>),
    {
        let mut methods_builder = LuaResourceMethods::new();
        register_fn(&mut methods_builder);
        
        let type_id = TypeId::of::<R>();
        
        self.resources.lock().unwrap().insert(type_id, methods_builder.into_map());
        self.type_names.lock().unwrap().insert(type_name.to_string(), type_id);
        
        debug!("✓ Registered Lua resource: {}", type_name);
    }
    
    /// Call a method on a resource
    pub fn call_method(
        &self,
        lua: &Lua,
        world: &World,
        type_name: &str,
        method_name: &str,
        args: LuaMultiValue,
    ) -> LuaResult<LuaValue> {
        let type_names = self.type_names.lock().unwrap();
        let type_id = type_names.get(type_name)
            .ok_or_else(|| LuaError::RuntimeError(format!("Resource type '{}' not registered", type_name)))?;
        
        let resources = self.resources.lock().unwrap();
        let methods = resources.get(type_id)
            .ok_or_else(|| LuaError::RuntimeError(format!("Resource type '{}' has no methods", type_name)))?;
        
        let method = methods.get(method_name)
            .ok_or_else(|| LuaError::RuntimeError(format!("Method '{}' not found on '{}'", method_name, type_name)))?;
        
        method(lua, world, args)
    }
}
