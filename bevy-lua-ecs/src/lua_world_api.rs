use crate::component_update_queue::ComponentUpdateQueue;
use crate::components::ComponentRegistry;
use bevy::ecs::reflect::ReflectComponent;
use bevy::prelude::*;
use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Lua userdata representing a query builder
#[derive(Clone)]
pub struct LuaQueryBuilder {
    pub with_components: Vec<String>,
    pub changed_components: Vec<String>,
}

impl LuaQueryBuilder {
    pub fn new() -> Self {
        Self {
            with_components: Vec::new(),
            changed_components: Vec::new(),
        }
    }
}

impl LuaUserData for LuaQueryBuilder {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("with", |_, this, component_name: String| {
            let mut new_builder = this.clone();
            new_builder.with_components.push(component_name);
            Ok(new_builder)
        });

        methods.add_method("changed", |_, this, component_name: String| {
            let mut new_builder = this.clone();
            new_builder.changed_components.push(component_name);
            Ok(new_builder)
        });
    }
}

// ... (imports)
use crate::components::LuaCustomComponents;

// ... (LuaQueryBuilder)

/// Snapshot of entity data for Lua access
pub struct LuaEntitySnapshot {
    pub entity: Entity,
    pub component_data: HashMap<String, String>,
    pub lua_components: HashMap<String, Arc<LuaRegistryKey>>,
    pub update_queue: ComponentUpdateQueue,
}

impl Clone for LuaEntitySnapshot {
    fn clone(&self) -> Self {
        Self {
            entity: self.entity,
            component_data: self.component_data.clone(),
            lua_components: HashMap::new(), // Can't clone registry keys easily without context
            update_queue: self.update_queue.clone(),
        }
    }
}

impl LuaUserData for LuaEntitySnapshot {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get", |lua, this, component_name: String| {
            // Check generic Lua components first
            if let Some(key) = this.lua_components.get(&component_name) {
                let value: LuaValue = lua.registry_value(&**key)?;
                return Ok(value);
            }

            // Check reflected Rust components
            if let Some(data_str) = this.component_data.get(&component_name) {
                // Try to deserialize JSON string to Lua table
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(data_str) {
                    // Convert JSON to Lua value - field names are preserved from serialization
                    let lua_value = json_to_lua(lua, &json_value)?;
                    return Ok(lua_value);
                }
                // Fallback to string if not JSON
                Ok(LuaValue::String(lua.create_string(data_str)?))
            } else {
                Ok(LuaValue::Nil)
            }
        });

        methods.add_method("has", |_, this, component_name: String| {
            Ok(this.lua_components.contains_key(&component_name)
                || this.component_data.contains_key(&component_name))
        });

        methods.add_method("id", |_, this, ()| Ok(this.entity.to_bits()));

        // Set/update components using spawn-style syntax
        // Usage: entity:set({ Text2d = { text = "..." }, Transform = {...} })
        methods.add_method(
            "set",
            |lua, this, components: LuaTable| {
                // Iterate through the table - keys are component names, values are component data
                for pair in components.pairs::<String, LuaValue>() {
                    let (component_name, component_value) = pair?;
                    
                    // Convert the value to a table if possible, or create a wrapper for scalar values
                    let component_data = match component_value {
                        LuaValue::Table(table) => table,
                        _ => {
                            // For non-table values, create a wrapper table with _0 (tuple struct style)
                            let wrapper = lua.create_table()?;
                            wrapper.set("_0", component_value)?;
                            wrapper
                        }
                    };
                    
                    // Create a registry key for the component data
                    let registry_key = lua.create_registry_value(component_data)?;
                    
                    // Queue the update
                    this.update_queue
                        .queue_update(this.entity, component_name, registry_key);
                }

                Ok(())
            },
        );
    }
}

/// Convert a reflected value to Lua using Bevy's reflection API directly.
/// This preserves struct field names that would be lost through serde serialization.
pub fn reflection_to_lua(lua: &Lua, value: &dyn bevy::reflect::PartialReflect) -> LuaResult<LuaValue> {
    use bevy::reflect::ReflectRef;
    
    match value.reflect_ref() {
        ReflectRef::Struct(s) => {
            let table = lua.create_table()?;
            for i in 0..s.field_len() {
                if let Some(field) = s.field_at(i) {
                    let field_name = match s.name_at(i) {
                        Some(name) => name.to_string(),
                        None => format!("{}", i),
                    };
                    table.set(field_name, reflection_to_lua(lua, field)?)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::TupleStruct(ts) => {
            let table = lua.create_table()?;
            for i in 0..ts.field_len() {
                if let Some(field) = ts.field(i) {
                    // Use _0, _1, etc. for tuple struct fields (like bevy-lua-ecs convention)
                    table.set(format!("_{}", i), reflection_to_lua(lua, field)?)?;
                    // Also set numeric indices for array-style access
                    table.set(i + 1, reflection_to_lua(lua, field)?)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Tuple(t) => {
            let table = lua.create_table()?;
            for i in 0..t.field_len() {
                if let Some(field) = t.field(i) {
                    table.set(i + 1, reflection_to_lua(lua, field)?)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::List(list) => {
            let table = lua.create_table()?;
            for i in 0..list.len() {
                if let Some(item) = list.get(i) {
                    table.set(i + 1, reflection_to_lua(lua, item)?)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Array(arr) => {
            let table = lua.create_table()?;
            for i in 0..arr.len() {
                if let Some(item) = arr.get(i) {
                    table.set(i + 1, reflection_to_lua(lua, item)?)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Map(map) => {
            let table = lua.create_table()?;
            for (key, val) in map.iter() {
                // Try to get a string key
                if let Some(key_str) = key.try_downcast_ref::<String>() {
                    table.set(key_str.clone(), reflection_to_lua(lua, val)?)?;
                } else if let Some(key_str) = key.try_downcast_ref::<&str>() {
                    table.set(*key_str, reflection_to_lua(lua, val)?)?;
                } else {
                    // Use debug format for non-string keys
                    table.set(format!("{:?}", key), reflection_to_lua(lua, val)?)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Set(set) => {
            let table = lua.create_table()?;
            for (i, item) in set.iter().enumerate() {
                table.set(i + 1, reflection_to_lua(lua, item)?)?;
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Enum(e) => {
            // For enums, create a table with the variant name as key
            let table = lua.create_table()?;
            let variant_name = e.variant_name();
            
            if e.field_len() == 0 {
                // Unit variant
                table.set(variant_name, true)?;
            } else if e.field_len() == 1 {
                // Newtype variant
                if let Some(field) = e.field_at(0) {
                    table.set(variant_name, reflection_to_lua(lua, field)?)?;
                }
            } else {
                // Tuple or struct variant
                let inner = lua.create_table()?;
                for i in 0..e.field_len() {
                    if let Some(field) = e.field_at(i) {
                        if let Some(name) = e.name_at(i) {
                            inner.set(name.to_string(), reflection_to_lua(lua, field)?)?;
                        } else {
                            inner.set(i + 1, reflection_to_lua(lua, field)?)?;
                        }
                    }
                }
                table.set(variant_name, inner)?;
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Opaque(opaque) => {
            // Try to extract primitive values
            if let Some(v) = opaque.try_downcast_ref::<f32>() {
                return Ok(LuaValue::Number(*v as f64));
            }
            if let Some(v) = opaque.try_downcast_ref::<f64>() {
                return Ok(LuaValue::Number(*v));
            }
            if let Some(v) = opaque.try_downcast_ref::<i32>() {
                return Ok(LuaValue::Integer(*v as i64));
            }
            if let Some(v) = opaque.try_downcast_ref::<i64>() {
                return Ok(LuaValue::Integer(*v));
            }
            if let Some(v) = opaque.try_downcast_ref::<u32>() {
                return Ok(LuaValue::Integer(*v as i64));
            }
            if let Some(v) = opaque.try_downcast_ref::<u64>() {
                return Ok(LuaValue::Integer(*v as i64));
            }
            if let Some(v) = opaque.try_downcast_ref::<bool>() {
                return Ok(LuaValue::Boolean(*v));
            }
            if let Some(v) = opaque.try_downcast_ref::<String>() {
                return Ok(LuaValue::String(lua.create_string(v)?));
            }
            // Fallback to debug string for unknown opaque types
            Ok(LuaValue::String(lua.create_string(&format!("{:?}", opaque))?))
        }
    }
}

/// Helper function to convert serde_json::Value to Lua value
fn json_to_lua(lua: &Lua, value: &serde_json::Value) -> LuaResult<LuaValue> {
    json_to_lua_impl(lua, value, None)
}

/// Helper function to convert serde_json::Value to Lua value with context about parent key
fn json_to_lua_impl(
    lua: &Lua,
    value: &serde_json::Value,
    parent_key: Option<&str>,
) -> LuaResult<LuaValue> {
    match value {
        serde_json::Value::Null => Ok(LuaValue::Nil),
        serde_json::Value::Bool(b) => Ok(LuaValue::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(LuaValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(LuaValue::Number(f))
            } else {
                Ok(LuaValue::Nil)
            }
        }
        serde_json::Value::String(s) => Ok(LuaValue::String(lua.create_string(s)?)),
        serde_json::Value::Array(arr) => {
            // Check if this is a vector type based on parent key and array length
            if arr.iter().all(|v| v.is_number()) {
                let field_names: Option<&[&str]> = match (parent_key, arr.len()) {
                    (Some("translation" | "scale"), 3) => Some(&["x", "y", "z"]),
                    (Some("rotation"), 4) => Some(&["x", "y", "z", "w"]),
                    (_, 2) if arr.iter().all(|v| v.is_number()) => Some(&["x", "y"]),
                    (_, 3) if arr.iter().all(|v| v.is_number()) => Some(&["x", "y", "z"]),
                    (_, 4) if arr.iter().all(|v| v.is_number()) => Some(&["x", "y", "z", "w"]),
                    _ => None,
                };

                if let Some(names) = field_names {
                    let table = lua.create_table()?;
                    for (i, (name, item)) in names.iter().zip(arr.iter()).enumerate() {
                        // Set both named field and numeric index
                        table.set(*name, json_to_lua_impl(lua, item, None)?)?;
                        table.set(i + 1, json_to_lua_impl(lua, item, None)?)?;
                    }
                    return Ok(LuaValue::Table(table));
                }
            }

            // Regular array (1-indexed for Lua)
            let table = lua.create_table()?;
            for (i, item) in arr.iter().enumerate() {
                table.set(i + 1, json_to_lua_impl(lua, item, None)?)?;
            }
            Ok(LuaValue::Table(table))
        }
        serde_json::Value::Object(obj) => {
            // Objects are converted to Lua tables with their field names preserved
            let table = lua.create_table()?;
            for (key, value) in obj {
                // Pass the key name as context for array field name inference
                table.set(key.as_str(), json_to_lua_impl(lua, value, Some(key))?)?;
            }
            Ok(LuaValue::Table(table))
        }
    }
}

/// Execute a query and collect entity snapshots
pub fn execute_query(
    lua: &Lua,
    world: &World,
    query_builder: &LuaQueryBuilder,
    component_registry: &ComponentRegistry,
    update_queue: &ComponentUpdateQueue,
    last_run: u32,
    this_run: u32,
) -> LuaResult<Vec<LuaEntitySnapshot>> {
    let mut results = Vec::new();
    let type_registry = component_registry.type_registry().read();

    // Iterate all entities using world entities iterator
    // In Bevy 0.17, we can use world.iter_entities() but it's deprecated
    // The recommended way is to use a system with Query, but since we're in a Lua context
    // we'll use the entities iterator
    for entity_ref in world.iter_entities() {
        let mut matches = true;
        let mut component_data = HashMap::new();
        let mut lua_components = HashMap::new();

        // Check with() filters and collect component data
        for component_name in &query_builder.with_components {
            // 1. Check if it's a non-reflected component (like Lightyear components)
            if let Some(type_id) = component_registry.get_non_reflected_type_id(component_name) {
                // Check if entity has this component via world.components()
                if let Some(component_id) = world.components().get_id(*type_id) {
                    if entity_ref.contains_id(component_id) {
                        // Component exists on this entity, add placeholder data
                        component_data.insert(component_name.clone(), "{}".to_string());
                        continue;
                    }
                }
                // Component not found on entity
                matches = false;
                break;
            }

            // 2. Check if it's a known Rust component (with Reflect)
            if let Some(type_path) = component_registry.get_type_path(component_name) {
                if let Some(registration) = type_registry.get_with_type_path(&type_path) {
                    if let Some(reflect_component) = registration.data::<ReflectComponent>() {
                        if let Some(component) =
                            reflect_component.reflect(
                                Into::<bevy::ecs::world::FilteredEntityRef>::into(&entity_ref),
                            )
                        {
                            // Use reflection_to_lua to convert directly with proper field names
                            match reflection_to_lua(lua, component) {
                                Ok(lua_value) => {
                                    // Store as registry key (same as Lua components)
                                    if let Ok(registry_key) = lua.create_registry_value(lua_value) {
                                        lua_components.insert(component_name.clone(), Arc::new(registry_key));
                                        continue;
                                    }
                                }
                                Err(_) => {
                                    // Fallback to JSON if reflection_to_lua fails
                                    use bevy::reflect::serde::TypedReflectSerializer;
                                    let serializer =
                                        TypedReflectSerializer::new(component.as_reflect(), &type_registry);
                                    if let Ok(json_value) = serde_json::to_value(serializer) {
                                        if let Ok(json_string) = serde_json::to_string(&json_value) {
                                            component_data.insert(component_name.clone(), json_string);
                                            continue;
                                        }
                                    }
                                    // Final fallback to Debug
                                    component_data
                                        .insert(component_name.clone(), format!("{:?}", component));
                                    continue;
                                }
                            }
                        }
                    }
                }
                // If known type but not found on entity -> mismatch
                matches = false;
                break;
            }

            // 3. Check if it's a generic Lua component
            if let Some(custom_components) = entity_ref.get::<LuaCustomComponents>() {
                if let Some(key) = custom_components.components.get(component_name) {
                    // Found it!
                    lua_components.insert(component_name.clone(), key.clone());
                    continue;
                }
            }

            // Not found anywhere
            matches = false;
            break;
        }

        // Check changed() filters
        if matches {
            for component_name in &query_builder.changed_components {
                // Only Rust components support change detection efficiently
                if let Some(type_path) = component_registry.get_type_path(component_name) {
                    if let Some(registration) = type_registry.get_with_type_path(&type_path) {
                        if let Some(_reflect_component) = registration.data::<ReflectComponent>() {
                            let component_id = world.components().get_id(registration.type_id());
                            if let Some(comp_id) = component_id {
                                if let Some(ticks) = entity_ref.get_change_ticks_by_id(comp_id) {
                                    use bevy::ecs::component::Tick;
                                    if !ticks.is_changed(Tick::new(last_run), Tick::new(this_run)) {
                                        matches = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                // TODO: Change detection for Lua components?
                // For now, we assume they don't change or we can't detect it easily without a wrapper.
            }
        }

        if matches {
            results.push(LuaEntitySnapshot {
                entity: entity_ref.id(),
                component_data,
                lua_components,
                update_queue: update_queue.clone(),
            });
        }
    }

    Ok(results)
}
