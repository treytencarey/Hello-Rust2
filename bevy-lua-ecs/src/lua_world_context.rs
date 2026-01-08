//! LuaWorldContext - Optimized world API for Lua systems
//!
//! This module provides a userdata-based world API that avoids creating
//! closures on every system call. The methods are defined statically
//! on the LuaUserData trait, so only the userdata instance needs to be
//! created per call.

use bevy::prelude::*;
use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::component_update_queue::ComponentUpdateQueue;
use crate::components::LuaCustomComponents;
use crate::lua_world_api::{execute_query, LuaQueryBuilder, LuaEntitySnapshot};
use crate::spawn_queue::SpawnQueue;
use crate::serde_components::SerdeComponentRegistry;
use crate::ComponentRegistry;
use crate::lua_systems::LuaSystemRegistry;

/// World userdata context - wraps references that are only valid during system execution
/// SAFETY: This MUST only be used within a lua.scope() to ensure the world reference is valid
pub struct LuaWorldContext<'w> {
    /// Reference to World - only valid during system execution!
    world: &'w World,
    /// Reference to ComponentRegistry - only valid during system execution!
    pub component_registry: &'w ComponentRegistry,
    pub update_queue: ComponentUpdateQueue,
    pub spawn_queue: SpawnQueue,
    pub serde_registry: SerdeComponentRegistry,
    pub script_registry: crate::script_registry::ScriptRegistry,
    pub system_registry: LuaSystemRegistry,
    pub despawn_queue: crate::despawn_queue::DespawnQueue,
    pub pending_messages: crate::event_sender::PendingLuaMessages,
    pub last_run: u32,
    pub this_run: u32,
    pub query_cache: Option<crate::query_cache::LuaQueryCache>,
    pub current_frame: u64,
}

impl<'w> LuaWorldContext<'w> {
    /// Create a new context - this is only valid during the current system call!
    pub fn new(
        world: &'w World,
        component_registry: &'w ComponentRegistry,
        update_queue: ComponentUpdateQueue,
        spawn_queue: SpawnQueue,
        serde_registry: SerdeComponentRegistry,
        script_registry: crate::script_registry::ScriptRegistry,
        system_registry: LuaSystemRegistry,
        despawn_queue: crate::despawn_queue::DespawnQueue,
        pending_messages: crate::event_sender::PendingLuaMessages,
        last_run: u32,
        this_run: u32,
        query_cache: Option<crate::query_cache::LuaQueryCache>,
        current_frame: u64,
    ) -> Self {
        Self {
            world,
            component_registry,
            update_queue,
            spawn_queue,
            serde_registry,
            script_registry,
            system_registry,
            despawn_queue,
            pending_messages,
            last_run,
            this_run,
            query_cache,
            current_frame,
        }
    }

    #[inline]
    fn world(&self) -> &World {
        self.world
    }
}

impl LuaUserData for LuaWorldContext<'_> {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // delta_time() - returns delta time in seconds
        methods.add_method("delta_time", |_lua, this, ()| {
            let time = this.world().resource::<Time>();
            Ok(time.delta_secs())
        });

        // query(with_components, changed_components) - executes immediately and returns results
        methods.add_method("query", |lua, this, (with_comps, changed_comps): (LuaTable, Option<LuaTable>)| {
            let t0 = std::time::Instant::now();
            
            let mut builder = LuaQueryBuilder::new();

            for comp_name in with_comps.sequence_values::<String>() {
                builder.with_components.push(comp_name?);
            }

            if let Some(changed_table) = changed_comps {
                for comp_name in changed_table.sequence_values::<String>() {
                    builder.changed_components.push(comp_name?);
                }
            }
            
            let t1 = std::time::Instant::now();

            let results = execute_query(
                lua,
                this.world(),
                &builder,
                &this.component_registry,
                &this.update_queue,
                this.last_run,
                this.this_run,
                this.query_cache.as_ref(),
                this.current_frame,
            )?;
            
            let t2 = std::time::Instant::now();
            let result_count = results.len();

            let results_table = lua.create_table()?;
            for (i, entity) in results.into_iter().enumerate() {
                results_table.set(i + 1, entity)?;
            }
            
            let t3 = std::time::Instant::now();
            
            let elapsed = t3.duration_since(t0).as_micros();
            if elapsed >= 100 {
                let parse_time = t1.duration_since(t0).as_micros();
                let query_time = t2.duration_since(t1).as_micros();
                let table_time = t3.duration_since(t2).as_micros();
                debug!(
                    "[METHOD_TIMING] query({:?}) parse={}us exec={}us table={}us (n={}) total={}us",
                    builder.with_components, parse_time, query_time, table_time, result_count, elapsed
                );
            }

            Ok(results_table)
        });

        // query_resource(resource_name) - check if a resource exists
        methods.add_method("query_resource", |_lua, this, resource_name: String| {
            Ok(this.serde_registry.has_resource(&resource_name))
        });

        // get_entity(bits) - get an entity wrapper from entity bits
        methods.add_method("get_entity", |lua, this, entity_bits: i64| {
            let entity = this.spawn_queue.resolve_entity(entity_bits as u64);
            
            let entity_ref = match this.world().get_entity(entity) {
                Ok(e) => e,
                Err(_) => {
                    return Ok(LuaValue::Nil);
                }
            };
            
            // Read Lua custom components
            let lua_components = if let Some(custom) = entity_ref.get::<LuaCustomComponents>() {
                custom.components.clone()
            } else {
                HashMap::new()
            };
            
            // Read all reflected Rust components on this entity
            let type_registry = this.component_registry.type_registry().read();
            let mut reflected_components: HashMap<String, Arc<LuaRegistryKey>> = HashMap::new();
            
            // Iterate through all archetype components on this entity
            let archetype = entity_ref.archetype();
            for component_id in archetype.components() {
                if let Some(component_info) = this.world().components().get_info(*component_id) {
                    let type_id = component_info.type_id();
                    if let Some(type_id) = type_id {
                        // Find the registration for this type
                        if let Some(registration) = type_registry.get(type_id) {
                            if let Some(reflect_component) = registration.data::<bevy::ecs::reflect::ReflectComponent>() {
                                // Get the reflected component data
                                if let Some(component) = reflect_component.reflect(entity_ref) {
                                    // Get short name for the component
                                    let type_path = registration.type_info().type_path();
                                    let short_name = type_path.rsplit("::").next().unwrap_or(type_path);
                                    
                                    // Convert to Lua via reflection
                                    if let Ok(lua_value) = crate::lua_world_api::reflection_to_lua(lua, component) {
                                        if let Ok(registry_key) = lua.create_registry_value(lua_value) {
                                            reflected_components.insert(short_name.to_string(), Arc::new(registry_key));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Merge reflected components into lua_components  
            let mut all_components = lua_components;
            for (name, key) in reflected_components {
                all_components.entry(name).or_insert(key);
            }
            
            let snapshot = LuaEntitySnapshot {
                entity,
                component_data: HashMap::new(), // Deprecated - using lua_components for all
                lua_components: all_components,
                update_queue: this.update_queue.clone(),
            };
            
            Ok(LuaValue::UserData(lua.create_userdata(snapshot)?))
        });

        // stop_owning_script(entity_id) - stop a script and clean up its resources
        methods.add_method("stop_owning_script", |_lua, this, entity_id: i64| {
            let entity = this.spawn_queue.resolve_entity(entity_id as u64);
            
            // Get the instance_id from the entity's ScriptOwned component
            if let Some(script_owned) = this.world().get::<crate::script_entities::ScriptOwned>(entity) {
                let instance_id = script_owned.instance_id;
                
                // Mark script as stopped in registry
                this.script_registry.mark_stopped(instance_id);
                
                // Clear systems registered by this script
                this.system_registry.clear_instance_systems(instance_id);
                
                // Queue entity for despawn
                this.despawn_queue.queue_despawn(entity);
                
                debug!("[STOP_SCRIPT] Stopped script instance {} for entity {:?}", instance_id, entity);
                Ok(())
            } else {
                Err(LuaError::RuntimeError(format!(
                    "Entity {:?} has no ScriptOwned component (id: {})",
                    entity, entity_id
                )))
            }
        });

        // read_events(event_type_name) - read any Bevy event via generated dispatch
        methods.add_method("read_events", |lua, this, event_type_name: String| {
            bevy::log::debug!("[READ_EVENTS] Reading events: '{}'", event_type_name);

            // Use unsafe world access - the dispatch function handles proper SystemState management
            #[allow(invalid_reference_casting)]
            let world_mut = unsafe {
                &mut *(this.world() as *const bevy::ecs::world::World as *mut bevy::ecs::world::World)
            };

            // Call the generated dispatch function via the global callback
            match crate::systemparam_lua_trait::call_read_events_global(
                lua,
                world_mut,
                &event_type_name,
            ) {
                Ok(mlua::Value::Table(t)) => Ok(t),
                Ok(_) => Err(LuaError::RuntimeError(
                    "Expected table from read_events".into(),
                )),
                Err(e) => Err(e),
            }
        });

        // query_events - alias for read_events
        methods.add_method("query_events", |lua, this, event_type_name: String| {
            // Delegate to read_events logic
            #[allow(invalid_reference_casting)]
            let world_mut = unsafe {
                &mut *(this.world() as *const bevy::ecs::world::World as *mut bevy::ecs::world::World)
            };

            match crate::systemparam_lua_trait::call_read_events_global(
                lua,
                world_mut,
                &event_type_name,
            ) {
                Ok(mlua::Value::Table(t)) => Ok(t),
                Ok(_) => Err(LuaError::RuntimeError(
                    "Expected table from query_events".into(),
                )),
                Err(e) => Err(e),
            }
        });

        // get_resource(resource_type_name) - get a resource by type name via reflection
        methods.add_method("get_resource", |lua, this, resource_type_name: String| {
            let type_registry = this.component_registry.type_registry();
            let registry = type_registry.read();

            // Try direct lookup first (exact short path or full path match)
            let type_registration = registry
                .get_with_short_type_path(&resource_type_name)
                .or_else(|| registry.get_with_type_path(&resource_type_name))
                // Fallback: iterate and find by short name match (handles generics)
                .or_else(|| {
                    // For generic types like "ButtonInput<KeyCode>", the short_type_path might be
                    // "ButtonInput<KeyCode>" but the full path is "bevy_input::ButtonInput<bevy_input::keyboard::KeyCode>"
                    // We need to match the short_type_path which strips crate prefixes from generics
                    registry.iter().find(|reg| {
                        let short = reg.type_info().type_path_table().short_path();
                        short == resource_type_name
                    })
                });

            let registration = match type_registration {
                Some(r) => r,
                None => {
                    // Log available similar types for debugging
                    debug!("[GET_RESOURCE] Type '{}' not found. Looking for similar types...", resource_type_name);
                    for reg in registry.iter() {
                        if reg.data::<bevy::ecs::reflect::ReflectResource>().is_some() {
                            let short = reg.type_info().type_path_table().short_path();
                            let full = reg.type_info().type_path();
                            // Log types that might be similar (contain part of the requested name)
                            if let Some(base_name) = resource_type_name.split(['<', '>']).next() {
                                if short.contains(base_name) || full.contains(base_name) {
                                    debug!("[GET_RESOURCE]   Found similar: short='{}', full='{}'", short, full);
                                }
                            }
                        }
                    }
                    return Ok(LuaValue::Nil);
                }
            };

            let reflect_resource = match registration.data::<bevy::ecs::reflect::ReflectResource>() {
                Some(r) => r,
                None => {
                    debug!("[GET_RESOURCE] Type '{}' found but has no ReflectResource", resource_type_name);
                    return Ok(LuaValue::Nil);
                }
            };

            #[allow(invalid_reference_casting)]
            let resource_ref = unsafe {
                let world_mut = &mut *(this.world() as *const bevy::ecs::world::World
                    as *mut bevy::ecs::world::World);
                let world_cell = world_mut.as_unsafe_world_cell();
                reflect_resource.reflect_unchecked_mut(world_cell)
            };

            match resource_ref {
                Some(resource) => {
                    crate::event_reader::reflection_to_lua(
                        lua,
                        resource.as_partial_reflect(),
                        &type_registry,
                    )
                }
                None => Ok(LuaValue::Nil),
            }
        });

        // call_systemparam_method(param_name, method_name, ...args)
        methods.add_method("call_systemparam_method", |lua, this, (param_name, method_name, args): (String, String, mlua::MultiValue)| {
            #[allow(invalid_reference_casting)]
            let world_mut = unsafe { &mut *(this.world() as *const World as *mut World) };

            crate::systemparam_lua_trait::call_systemparam_method_global(
                lua,
                world_mut,
                &param_name,
                &method_name,
                args,
            )
        });

        // call_resource_method(resource_name, method_name, ...args) - call a registered method on a resource
        methods.add_method("call_resource_method", |lua, this, (resource_name, method_name, args): (String, String, mlua::MultiValue)| {
            let resource_registry = this.world().resource::<crate::resource_lua_trait::LuaResourceRegistry>();
            resource_registry.call_method(
                lua,
                this.world(),
                &resource_name,
                &method_name,
                args,
            )
        });

        // call_component_method(entity_id, type_name, method_name, ...args)
        methods.add_method("call_component_method", |lua, this, (entity_id, type_name, method_name, args): (u64, String, String, mlua::MultiValue)| {
            #[allow(invalid_reference_casting)]
            let world_mut = unsafe { &mut *(this.world() as *const World as *mut World) };

            let resolved_entity = this.spawn_queue.resolve_entity(entity_id);
            let resolved_id = resolved_entity.to_bits();

            crate::systemparam_lua_trait::call_component_method_global(
                lua,
                world_mut,
                resolved_id,
                &type_name,
                &method_name,
                args,
            )
        });

        // call_static_method(type_name, method_name, ...args) - call static methods on math types
        // Unlike call_component_method, this doesn't require an entity or world access
        methods.add_method("call_static_method", |lua, this, (type_name, method_name, args): (String, String, mlua::MultiValue)| {
            crate::systemparam_lua_trait::call_static_method_global(
                lua,
                this.world(),
                &type_name,
                &method_name,
                args,
            )
        });

        // send_event(event_type_name, data_table) - send an event immediately
        methods.add_method("send_event", |lua, this, (event_type_name, data_table): (String, LuaTable)| {
            bevy::log::debug!("[SEND_EVENT] Writing event: '{}'", event_type_name);

            #[allow(invalid_reference_casting)]
            let world_mut = unsafe {
                &mut *(this.world() as *const bevy::ecs::world::World as *mut bevy::ecs::world::World)
            };

            match crate::systemparam_lua_trait::call_write_events_global(
                lua,
                world_mut,
                &event_type_name,
                &data_table,
            ) {
                Ok(()) => Ok(()),
                Err(e) => Err(LuaError::RuntimeError(format!(
                    "Failed to send event '{}': {}",
                    event_type_name, e
                ))),
            }
        });

        // write_event - alias for send_event
        methods.add_method("write_event", |lua, this, (event_type_name, data_table): (String, LuaTable)| {
            #[allow(invalid_reference_casting)]
            let world_mut = unsafe {
                &mut *(this.world() as *const bevy::ecs::world::World as *mut bevy::ecs::world::World)
            };

            match crate::systemparam_lua_trait::call_write_events_global(
                lua,
                world_mut,
                &event_type_name,
                &data_table,
            ) {
                Ok(()) => Ok(()),
                Err(e) => Err(LuaError::RuntimeError(format!(
                    "Failed to write event '{}': {}",
                    event_type_name, e
                ))),
            }
        });

        // write_message(message_type_name, data_table) - queue a message to be sent
        // Messages use MessageWriter<M> instead of EventWriter<T>
        methods.add_method("write_message", |_lua, this, (message_type_name, data_table): (String, LuaTable)| {
            // Convert Lua table to JSON value
            fn lua_to_json(value: &LuaValue) -> serde_json::Value {
                match value {
                    LuaValue::Nil => serde_json::Value::Null,
                    LuaValue::Boolean(b) => serde_json::Value::Bool(*b),
                    LuaValue::Integer(i) => serde_json::json!(*i),
                    LuaValue::Number(n) => serde_json::json!(*n),
                    LuaValue::String(s) => match s.to_str() {
                        Ok(cow) => serde_json::Value::String(cow.to_string()),
                        Err(_) => serde_json::Value::String(String::new()),
                    },
                    LuaValue::Table(t) => {
                        // Determine if array or object
                        let mut is_array = true;
                        let mut max_key = 0i64;
                        for pair in t.clone().pairs::<LuaValue, LuaValue>() {
                            if let Ok((key, _)) = pair {
                                match key {
                                    LuaValue::Integer(i) if i > 0 => {
                                        max_key = max_key.max(i);
                                    }
                                    _ => {
                                        is_array = false;
                                        break;
                                    }
                                }
                            }
                        }

                        if is_array && max_key > 0 {
                            let mut arr = Vec::new();
                            for i in 1..=max_key {
                                if let Ok(val) = t.get::<LuaValue>(i) {
                                    arr.push(lua_to_json(&val));
                                }
                            }
                            serde_json::Value::Array(arr)
                        } else {
                            let mut map = serde_json::Map::new();
                            for pair in t.clone().pairs::<String, LuaValue>() {
                                if let Ok((key, val)) = pair {
                                    map.insert(key, lua_to_json(&val));
                                }
                            }
                            serde_json::Value::Object(map)
                        }
                    }
                    _ => serde_json::Value::Null,
                }
            }

            let json_data = lua_to_json(&LuaValue::Table(data_table.clone()));

            bevy::log::debug!(
                "[WRITE_MESSAGE] Queueing message '{}': {:?}",
                message_type_name, json_data
            );
            this.pending_messages.queue_message(message_type_name.clone(), json_data);

            Ok(())
        });

        // send_message - alias for write_message
        methods.add_method("send_message", |_lua, this, (message_type_name, data_table): (String, LuaTable)| {
            // Re-use the same conversion logic
            fn lua_to_json(value: &LuaValue) -> serde_json::Value {
                match value {
                    LuaValue::Nil => serde_json::Value::Null,
                    LuaValue::Boolean(b) => serde_json::Value::Bool(*b),
                    LuaValue::Integer(i) => serde_json::json!(*i),
                    LuaValue::Number(n) => serde_json::json!(*n),
                    LuaValue::String(s) => match s.to_str() {
                        Ok(cow) => serde_json::Value::String(cow.to_string()),
                        Err(_) => serde_json::Value::String(String::new()),
                    },
                    LuaValue::Table(t) => {
                        let mut is_array = true;
                        let mut max_key = 0i64;
                        for pair in t.clone().pairs::<LuaValue, LuaValue>() {
                            if let Ok((key, _)) = pair {
                                match key {
                                    LuaValue::Integer(i) if i > 0 => {
                                        max_key = max_key.max(i);
                                    }
                                    _ => {
                                        is_array = false;
                                        break;
                                    }
                                }
                            }
                        }

                        if is_array && max_key > 0 {
                            let mut arr = Vec::new();
                            for i in 1..=max_key {
                                if let Ok(val) = t.get::<LuaValue>(i) {
                                    arr.push(lua_to_json(&val));
                                }
                            }
                            serde_json::Value::Array(arr)
                        } else {
                            let mut map = serde_json::Map::new();
                            for pair in t.clone().pairs::<String, LuaValue>() {
                                if let Ok((key, val)) = pair {
                                    map.insert(key, lua_to_json(&val));
                                }
                            }
                            serde_json::Value::Object(map)
                        }
                    }
                    _ => serde_json::Value::Null,
                }
            }

            let json_data = lua_to_json(&LuaValue::Table(data_table.clone()));
            this.pending_messages.queue_message(message_type_name.clone(), json_data);

            Ok(())
        });
    }
}
