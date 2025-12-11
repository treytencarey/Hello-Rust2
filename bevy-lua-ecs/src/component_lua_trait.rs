use bevy::prelude::*;
use mlua::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Method handler for a component
/// Takes: Lua context, Entity, method arguments
/// Returns: Lua value result
pub type ComponentMethod = Arc<dyn Fn(&Lua, &mut World, Entity, LuaMultiValue) -> LuaResult<LuaValue> + Send + Sync>;

/// Builder for registering component methods (type-safe version)
pub struct LuaComponentMethods<C: Component> {
    methods: HashMap<String, ComponentMethod>,
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Component + 'static> LuaComponentMethods<C> {
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
    /// methods.add("send", |component, lua, (channel, msg): (String, String)| {
    ///     // Direct access to component!
    ///     component.send(channel, msg)?;
    ///     Ok(())
    /// });
    /// ```
    pub fn add<F, Args, Ret>(&mut self, name: &str, handler: F)
    where
        C: Component<Mutability = bevy::ecs::component::Mutable>,  // Auto-detect: only mutable components
        F: Fn(&mut C, &Lua, Args) -> LuaResult<Ret> + Send + Sync + 'static,
        Args: FromLuaMulti,
        Ret: IntoLua,
    {
        let handler = Arc::new(move |lua: &Lua, world: &mut World, entity: Entity, args: LuaMultiValue| {
            // Parse arguments
            let parsed_args: Args = FromLuaMulti::from_lua_multi(args, lua)?;
            
            // Get mutable component via entity_mut (Bevy 0.16 compatible)
            let mut entity_mut = world.entity_mut(entity);
            if let Some(mut component) = entity_mut.get_mut::<C>() {
                let result = handler(&mut component, lua, parsed_args)?;
                result.into_lua(lua)
            } else {
                Err(LuaError::RuntimeError(format!(
                    "Component {} not found on entity {:?}", 
                    std::any::type_name::<C>(),
                    entity
                )))
            }
        });
        
        self.methods.insert(name.to_string(), handler);
    }
    
    pub fn into_map(self) -> HashMap<String, ComponentMethod> {
        self.methods
    }
}

/// Registry for Lua-accessible components
/// This is the main infrastructure for exposing component methods to Lua
#[derive(Resource, Clone, Default)]
pub struct LuaComponentRegistry {
    components: Arc<Mutex<HashMap<TypeId, HashMap<String, ComponentMethod>>>>,
    type_names: Arc<Mutex<HashMap<String, TypeId>>>,
}

impl LuaComponentRegistry {
    /// Register a component type with its methods
    /// This is the main API for registering components
    /// 
    /// # Example
    /// ```ignore
    /// registry.register_component::<MessageSender<LuaMessage>>("MessageSender", |methods| {
    ///     methods.add("send", |sender, lua, (channel, msg): (String, String)| {
    ///         // Implementation
    ///         Ok(())
    ///     });
    /// });
    /// ```
    pub fn register_component<C, F>(&self, type_name: &str, register_fn: F)
    where
        C: Component<Mutability = bevy::ecs::component::Mutable> + 'static,  // Auto-detect: only mutable components
        F: FnOnce(&mut LuaComponentMethods<C>),
    {
        let mut methods_builder = LuaComponentMethods::new();
        register_fn(&mut methods_builder);
        
        let type_id = TypeId::of::<C>();
        
        self.components.lock().unwrap().insert(type_id, methods_builder.into_map());
        self.type_names.lock().unwrap().insert(type_name.to_string(), type_id);
        
        debug!("✓ Registered Lua component: {}", type_name);
    }
    
    /// Call a method on a component
    pub fn call_method(
        &self,
        lua: &Lua,
        world: &mut World,
        entity: Entity,
        type_name: &str,
        method_name: &str,
        args: LuaMultiValue,
    ) -> LuaResult<LuaValue> {
        let type_names = self.type_names.lock().unwrap();
        let type_id = type_names.get(type_name)
            .ok_or_else(|| LuaError::RuntimeError(format!("Component type '{}' not registered", type_name)))?;
        
        let components = self.components.lock().unwrap();
        let methods = components.get(type_id)
            .ok_or_else(|| LuaError::RuntimeError(format!("Component type '{}' has no methods", type_name)))?;
        
        let method = methods.get(method_name)
            .ok_or_else(|| LuaError::RuntimeError(format!("Method '{}' not found on '{}'", method_name, type_name)))?;
        
        method(lua, world, entity, args)
    }
}
