//! SystemParam Lua Trait
//!
//! Provides infrastructure for exposing Bevy SystemParams to Lua.
//! Similar to LuaResourceRegistry but uses SystemState<P> for access.
//!
//! # Example
//! ```ignore
//! registry.register_systemparam::<MeshRayCast>("MeshRayCast", |methods| {
//!     methods.add("cast_ray", |raycast, lua, ray_data: LuaTable| {
//!         let ray = Ray3d::new(...);
//!         let hits: Vec<_> = raycast.cast_ray(ray, &default()).collect();
//!         // Convert hits to Lua table
//!         Ok(...)
//!     });
//! });
//! ```

use bevy::ecs::system::{SystemParam, SystemState};
use bevy::prelude::*;
use mlua::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Method handler for a SystemParam
/// Takes: Lua context, mutable World, method arguments
/// Returns: Lua value result
pub type SystemParamMethod =
    Arc<dyn Fn(&Lua, &mut World, LuaMultiValue) -> LuaResult<LuaValue> + Send + Sync>;

/// Function signature for the auto-generated SystemParam method dispatcher
pub type SystemParamDispatchFn =
    fn(&Lua, &mut World, &str, &str, LuaMultiValue) -> LuaResult<LuaValue>;

/// Global dispatch function set by the parent crate's generated code
static SYSTEMPARAM_DISPATCHER: std::sync::OnceLock<SystemParamDispatchFn> =
    std::sync::OnceLock::new();

/// Set the global SystemParam method dispatcher
/// This should be called by the parent crate's initialization code
/// to register the auto-generated dispatch function
pub fn set_systemparam_dispatcher(dispatcher: SystemParamDispatchFn) {
    let _ = SYSTEMPARAM_DISPATCHER.set(dispatcher);
}

/// Call the registered SystemParam method dispatcher
/// Returns an error if no dispatcher has been set
pub fn call_systemparam_method_global(
    lua: &Lua,
    world: &mut World,
    param_name: &str,
    method_name: &str,
    args: LuaMultiValue,
) -> LuaResult<LuaValue> {
    if let Some(dispatcher) = SYSTEMPARAM_DISPATCHER.get() {
        dispatcher(lua, world, param_name, method_name, args)
    } else {
        Err(LuaError::RuntimeError(format!(
            "SystemParam dispatch not configured. Call set_systemparam_dispatcher() at initialization."
        )))
    }
}

/// Function signature for the auto-generated event reader dispatcher
pub type EventDispatchFn = fn(&Lua, &mut World, &str) -> LuaResult<LuaValue>;

/// Global dispatch function set by the parent crate's generated code
static EVENT_DISPATCHER: std::sync::OnceLock<EventDispatchFn> = std::sync::OnceLock::new();

/// Set the global event reader dispatcher
/// This should be called by the parent crate's initialization code
/// to register the auto-generated dispatch_read_events function
pub fn set_event_dispatcher(dispatcher: EventDispatchFn) {
    let _ = EVENT_DISPATCHER.set(dispatcher);
}

/// Call the registered event reader dispatcher
/// Returns an error if no dispatcher has been set
pub fn call_read_events_global(
    lua: &Lua,
    world: &mut World,
    event_type: &str,
) -> LuaResult<LuaValue> {
    if let Some(dispatcher) = EVENT_DISPATCHER.get() {
        dispatcher(lua, world, event_type)
    } else {
        Err(LuaError::RuntimeError(format!(
            "Event dispatch not configured. Call set_event_dispatcher() at initialization."
        )))
    }
}

/// Function signature for the auto-generated event writer dispatcher
pub type EventWriteDispatchFn = fn(&Lua, &mut World, &str, &LuaTable) -> Result<(), String>;

/// Global dispatch function for event writing set by the parent crate's generated code
static EVENT_WRITE_DISPATCHER: std::sync::OnceLock<EventWriteDispatchFn> =
    std::sync::OnceLock::new();

/// Set the global event writer dispatcher
/// This should be called by the parent crate's initialization code
/// to register the auto-generated dispatch_write_events function
pub fn set_event_write_dispatcher(dispatcher: EventWriteDispatchFn) {
    let _ = EVENT_WRITE_DISPATCHER.set(dispatcher);
}

/// Call the registered event writer dispatcher
/// Returns an error if no dispatcher has been set
pub fn call_write_events_global(
    lua: &Lua,
    world: &mut World,
    event_type: &str,
    data: &LuaTable,
) -> Result<(), String> {
    if let Some(dispatcher) = EVENT_WRITE_DISPATCHER.get() {
        dispatcher(lua, world, event_type, data)
    } else {
        Err(format!(
            "Event write dispatch not configured. Call set_event_write_dispatcher() at initialization."
        ))
    }
}

/// Function signature for the auto-generated message writer dispatcher
pub type MessageWriteDispatchFn = fn(&Lua, &mut World, &str, &LuaTable) -> Result<(), String>;

/// Global dispatch function for message writing set by the parent crate's generated code
static MESSAGE_WRITE_DISPATCHER: std::sync::OnceLock<MessageWriteDispatchFn> =
    std::sync::OnceLock::new();

/// Set the global message writer dispatcher
/// This should be called by the parent crate's initialization code
/// to register the auto-generated dispatch_write_message function
pub fn set_message_write_dispatcher(dispatcher: MessageWriteDispatchFn) {
    let _ = MESSAGE_WRITE_DISPATCHER.set(dispatcher);
}

/// Call the registered message writer dispatcher
/// Returns an error if no dispatcher has been set
pub fn call_write_messages_global(
    lua: &Lua,
    world: &mut World,
    message_type: &str,
    data: &LuaTable,
) -> Result<(), String> {
    if let Some(dispatcher) = MESSAGE_WRITE_DISPATCHER.get() {
        dispatcher(lua, world, message_type, data)
    } else {
        Err(format!(
            "Message write dispatch not configured. Call set_message_write_dispatcher() at initialization."
        ))
    }
}

/// Registry for Lua-accessible SystemParams
/// This is the main infrastructure for exposing SystemParam methods to Lua
#[derive(Resource, Clone, Default)]
pub struct LuaSystemParamRegistry {
    /// Maps TypeId -> method_name -> method handler
    params: Arc<Mutex<HashMap<TypeId, HashMap<String, SystemParamMethod>>>>,
    /// Maps string name -> TypeId for lookup
    type_names: Arc<Mutex<HashMap<String, TypeId>>>,
}

impl LuaSystemParamRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a SystemParam type with its methods
    ///
    /// # Example
    /// ```ignore
    /// registry.register_systemparam::<MeshRayCast>("MeshRayCast", |methods| {
    ///     methods.add("cast_ray", |param, lua, args: LuaTable| {
    ///         // param is &mut MeshRayCast, accessed via SystemState
    ///         Ok(())
    ///     });
    /// });
    /// ```
    pub fn register_systemparam<P, F>(&self, type_name: &str, register_fn: F)
    where
        P: SystemParam + 'static,
        for<'w, 's> <P as SystemParam>::Item<'w, 's>: 'static,
        F: FnOnce(&mut LuaSystemParamMethods<P>),
    {
        let mut methods_builder = LuaSystemParamMethods::new();
        register_fn(&mut methods_builder);

        let type_id = TypeId::of::<P>();

        self.params
            .lock()
            .unwrap()
            .insert(type_id, methods_builder.into_map());
        self.type_names
            .lock()
            .unwrap()
            .insert(type_name.to_string(), type_id);

        debug!("✓ Registered Lua systemparam: {}", type_name);
    }

    /// Call a method on a SystemParam
    pub fn call_method(
        &self,
        lua: &Lua,
        world: &mut World,
        type_name: &str,
        method_name: &str,
        args: LuaMultiValue,
    ) -> LuaResult<LuaValue> {
        let type_names = self.type_names.lock().unwrap();
        let type_id = type_names.get(type_name).ok_or_else(|| {
            LuaError::RuntimeError(format!("SystemParam type '{}' not registered", type_name))
        })?;

        let params = self.params.lock().unwrap();
        let methods = params.get(type_id).ok_or_else(|| {
            LuaError::RuntimeError(format!("SystemParam type '{}' has no methods", type_name))
        })?;

        let method = methods.get(method_name).ok_or_else(|| {
            LuaError::RuntimeError(format!(
                "Method '{}' not found on '{}'",
                method_name, type_name
            ))
        })?;

        // Clone the Arc since we need to release the lock before calling
        let method = method.clone();
        drop(params);
        drop(type_names);

        method(lua, world, args)
    }

    /// List all registered SystemParam types
    pub fn list_types(&self) -> Vec<String> {
        self.type_names.lock().unwrap().keys().cloned().collect()
    }

    /// List all methods for a SystemParam type
    pub fn list_methods(&self, type_name: &str) -> Option<Vec<String>> {
        let type_names = self.type_names.lock().unwrap();
        let type_id = type_names.get(type_name)?;

        let params = self.params.lock().unwrap();
        params.get(type_id).map(|m| m.keys().cloned().collect())
    }
}

/// Builder for registering SystemParam methods (type-safe version)
pub struct LuaSystemParamMethods<P: SystemParam + 'static> {
    methods: HashMap<String, SystemParamMethod>,
    _phantom: std::marker::PhantomData<P>,
}

impl<P> LuaSystemParamMethods<P>
where
    P: SystemParam + 'static,
    for<'w, 's> <P as SystemParam>::Item<'w, 's>: 'static,
{
    pub fn new() -> Self {
        Self {
            methods: HashMap::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Add a method - uses SystemState for access
    ///
    /// The handler receives the SystemParam item directly.
    ///
    /// # Example
    /// ```ignore
    /// methods.add("cast_ray", |raycast, lua, ray_data: LuaTable| {
    ///     // raycast is the SystemParamItem for MeshRayCast
    ///     Ok(())
    /// });
    /// ```
    pub fn add<F, Args, Ret>(&mut self, name: &str, handler: F)
    where
        F: for<'w, 's> Fn(&mut <P as SystemParam>::Item<'w, 's>, &Lua, Args) -> LuaResult<Ret>
            + Send
            + Sync
            + 'static,
        Args: FromLuaMulti,
        Ret: IntoLua,
    {
        let handler = Arc::new(move |lua: &Lua, world: &mut World, args: LuaMultiValue| {
            // Parse arguments
            let parsed_args: Args = FromLuaMulti::from_lua_multi(args, lua)?;

            // Create SystemState and get the param
            let mut system_state = SystemState::<P>::new(world);
            let mut param = system_state.get_mut(world);

            // Call the handler
            let result = handler(&mut param, lua, parsed_args)?;
            result.into_lua(lua)
        });

        self.methods.insert(name.to_string(), handler);
    }

    pub fn into_map(self) -> HashMap<String, SystemParamMethod> {
        self.methods
    }
}
