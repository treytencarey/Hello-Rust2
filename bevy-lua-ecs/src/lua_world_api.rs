use crate::component_update_queue::ComponentUpdateQueue;
use crate::components::ComponentRegistry;
use bevy::ecs::reflect::ReflectComponent;
use bevy::prelude::*;
use mlua::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Filter types that can be combined with Or combinator
/// Matches Bevy's Or<(Changed<T1>, Changed<T2>, Added<T3>, ...)>
#[derive(Clone, Default, Debug)]
pub struct OrFilters {
    /// Or<(Changed<T1>, Changed<T2>, ...)> - at least one must have changed
    pub changed: Vec<String>,
    /// Or<(Added<T1>, Added<T2>, ...)> - at least one must be newly added
    pub added: Vec<String>,
    /// Or<(Removed<T1>, Removed<T2>, ...)> - at least one must have been removed
    pub removed: Vec<String>,
}

impl OrFilters {
    pub fn is_empty(&self) -> bool {
        self.changed.is_empty() && self.added.is_empty() && self.removed.is_empty()
    }
}

/// Lua userdata representing a query builder
/// Supports Bevy-style filters: With, Without, Changed, Added, AnyOf, Or
#[derive(Clone)]
pub struct LuaQueryBuilder {
    /// With<T> - Required components (AND logic, all must be present)
    pub with_components: Vec<String>,
    /// Without<T> - Excluded components (none must be present)
    pub without_components: Vec<String>,
    /// AnyOf<(&T1, &T2, ...)> - Optional components (at least one must be present)
    pub any_of_components: Vec<String>,
    /// Optional<T> - Component that might be present (doesn't filter)
    pub optional_components: Vec<String>,
    /// Changed<T> - Components that must have changed (AND logic)
    pub changed_components: Vec<String>,
    /// Added<T> - Components that must be newly added (AND logic)
    pub added_components: Vec<String>,
    /// Or<(F1, F2, ...)> - Union filter combinator
    pub or_filters: OrFilters,
}

impl LuaQueryBuilder {
    pub fn new() -> Self {
        Self {
            with_components: Vec::new(),
            without_components: Vec::new(),
            any_of_components: Vec::new(),
            optional_components: Vec::new(),
            changed_components: Vec::new(),
            added_components: Vec::new(),
            or_filters: OrFilters::default(),
        }
    }

    /// Check if this query has any change-detection filters (Changed, Added, Or)
    pub fn has_change_detection(&self) -> bool {
        !self.changed_components.is_empty()
            || !self.added_components.is_empty()
            || !self.or_filters.is_empty()
    }
}

impl LuaUserData for LuaQueryBuilder {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // With<T> - require component (AND)
        methods.add_method("with", |_, this, component_name: String| {
            let mut new_builder = this.clone();
            new_builder.with_components.push(component_name);
            Ok(new_builder)
        });

        // Without<T> - exclude component
        methods.add_method("without", |_, this, component_name: String| {
            let mut new_builder = this.clone();
            new_builder.without_components.push(component_name);
            Ok(new_builder)
        });

        // AnyOf - optional components (at least one must be present)
        methods.add_method("any_of", |_, this, component_name: String| {
            let mut new_builder = this.clone();
            new_builder.any_of_components.push(component_name);
            Ok(new_builder)
        });
    
        // Optional<T> - component that might be present
        methods.add_method("optional", |_, this, component_name: String| {
            let mut new_builder = this.clone();
            new_builder.optional_components.push(component_name);
            Ok(new_builder)
        });

        // Changed<T> - component must have changed (AND)
        methods.add_method("changed", |_, this, component_name: String| {
            let mut new_builder = this.clone();
            new_builder.changed_components.push(component_name);
            Ok(new_builder)
        });

        // Added<T> - component must be newly added (AND)
        methods.add_method("added", |_, this, component_name: String| {
            let mut new_builder = this.clone();
            new_builder.added_components.push(component_name);
            Ok(new_builder)
        });

        // Or changed - at least one must have changed
        methods.add_method("or_changed", |_, this, component_name: String| {
            let mut new_builder = this.clone();
            new_builder.or_filters.changed.push(component_name);
            Ok(new_builder)
        });

        // Or added - at least one must be newly added
        methods.add_method("or_added", |_, this, component_name: String| {
            let mut new_builder = this.clone();
            new_builder.or_filters.added.push(component_name);
            Ok(new_builder)
        });

        // Or removed - at least one must have been removed this frame
        methods.add_method("or_removed", |_, this, component_name: String| {
            let mut new_builder = this.clone();
            new_builder.or_filters.removed.push(component_name);
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
    pub changed_components: HashSet<String>,
    pub added_components: HashSet<String>,
    pub update_queue: ComponentUpdateQueue,
}

impl Clone for LuaEntitySnapshot {
    fn clone(&self) -> Self {
        Self {
            entity: self.entity,
            component_data: self.component_data.clone(),
            lua_components: HashMap::new(), // Can't clone registry keys easily without context
            changed_components: self.changed_components.clone(),
            added_components: self.added_components.clone(),
            update_queue: self.update_queue.clone(),
        }
    }
}

impl LuaUserData for LuaEntitySnapshot {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get", |lua, this, component_name: String| {
            // FIRST: Check for pending updates in the queue (read-through cache)
            // This ensures get() returns the most recent set() value, even if not yet applied
            if let Some(pending_key) = this.update_queue.peek_pending(this.entity, &component_name) {
                let value: LuaValue = lua.registry_value(&*pending_key)?;
                return Ok(value);
            }

            // Check generic Lua components (snapshot data)
            if let Some(key) = this.lua_components.get(&component_name) {
                let value: LuaValue = lua.registry_value(&**key)?;
                return Ok(value);
            }

            // Check reflected Rust components (snapshot data)
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
        
        methods.add_method("changed_components", |lua, this, ()| {
            let table = lua.create_table()?;
            for (i, name) in this.changed_components.iter().enumerate() {
                table.set(i + 1, name.clone())?;
            }
            Ok(table)
        });

        methods.add_method("added_components", |lua, this, ()| {
            let table = lua.create_table()?;
            for (i, name) in this.added_components.iter().enumerate() {
                table.set(i + 1, name.clone())?;
            }
            Ok(table)
        });

        methods.add_method("is_changed", |_, this, component_name: String| {
            Ok(this.changed_components.contains(&component_name))
        });

        methods.add_method("is_added", |_, this, component_name: String| {
            Ok(this.added_components.contains(&component_name))
        });

        methods.add_method("get_components", |lua, this, ()| {
            let table = lua.create_table()?;
            let mut index = 1;

            // Add Lua components
            for key in this.lua_components.keys() {
                table.set(index, key.clone())?;
                index += 1;
            }

            // Add Rust components
            for key in this.component_data.keys() {
                // Ensure no duplicates if a component exists in both maps (unlikely but safe)
                if !this.lua_components.contains_key(key) {
                    table.set(index, key.clone())?;
                    index += 1;
                }
            }

            Ok(LuaValue::Table(table))
        });

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

        // Patch/merge components - only updates specified fields, preserves other existing fields
        // Usage: entity:patch({ PlayerState = { velocity = {x=1,y=0,z=0} } })
        // This merges with existing PlayerState, preserving fields like model_path, owner_client, etc.
        methods.add_method(
            "patch",
            |lua, this, components: LuaTable| {
                // Iterate through the table - keys are component names, values are partial component data
                for pair in components.pairs::<String, LuaValue>() {
                    let (component_name, patch_value) = pair?;
                    
                    // Get the current component value
                    // FIRST: Check for pending updates in the queue (read-through cache)
                    // This ensures patch() uses the most recent set/patch value, even if not yet applied
                    let current_value = if let Some(pending_key) = this.update_queue.peek_pending(this.entity, &component_name) {
                        lua.registry_value::<LuaValue>(&*pending_key)?
                    } else if let Some(key) = this.lua_components.get(&component_name) {
                        // Lua component - get from registry
                        lua.registry_value::<LuaValue>(&**key)?
                    } else if let Some(data_str) = this.component_data.get(&component_name) {
                        // Rust component - parse JSON
                        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(data_str) {
                            json_to_lua(lua, &json_value)?
                        } else {
                            LuaValue::Nil
                        }
                    } else {
                        LuaValue::Nil
                    };
                    
                    // Merge the patch into current value
                    let merged_data = match (current_value, patch_value) {
                        (LuaValue::Table(current_table), LuaValue::Table(patch_table)) => {
                            // Deep merge tables
                            let merged = lua.create_table()?;
                            
                            // First, copy all existing fields
                            for pair in current_table.pairs::<LuaValue, LuaValue>() {
                                let (key, value) = pair?;
                                merged.set(key, value)?;
                            }
                            
                            // Then, overlay patch fields (overwrites existing)
                            for pair in patch_table.pairs::<LuaValue, LuaValue>() {
                                let (key, value) = pair?;
                                merged.set(key, value)?;
                            }
                            
                            merged
                        }
                        (_, LuaValue::Table(patch_table)) => {
                            // No existing value or not a table, just use patch as-is
                            patch_table
                        }
                        (_, other) => {
                            // For non-table values, create a wrapper
                            let wrapper = lua.create_table()?;
                            wrapper.set("_0", other)?;
                            wrapper
                        }
                    };
                    
                    // Create a registry key for the merged data
                    let registry_key = lua.create_registry_value(merged_data)?;
                    
                    // Queue the update with merged data
                    this.update_queue
                        .queue_update(this.entity, component_name, registry_key);
                }

                Ok(())
            },
        );

        // Remove components
        // Usage: entity:remove("ComponentName")
        methods.add_method(
            "remove",
            |_lua, this, component_name: String| {
                // Queue the removal
                this.update_queue.queue_removal(this.entity, component_name);
                Ok(())
            },
        );
    }
}

/// Convert a reflected value to Lua using Bevy's reflection API directly.
/// This preserves struct field names that would be lost through serde serialization.
pub fn reflection_to_lua(lua: &Lua, value: &dyn bevy::reflect::PartialReflect) -> LuaResult<LuaValue> {
    reflection_to_lua_with_assets(lua, value, None)
}

/// Convert a reflected value to Lua with optional AssetRegistry for Handle→path serialization.
/// When asset_registry is provided, Handle<T> types are serialized as their asset path strings
/// for network replication.
pub fn reflection_to_lua_with_assets(
    lua: &Lua, 
    value: &dyn bevy::reflect::PartialReflect,
    asset_registry: Option<&crate::asset_loading::AssetRegistry>,
) -> LuaResult<LuaValue> {
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
                    table.set(field_name, reflection_to_lua_with_assets(lua, field, asset_registry)?)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::TupleStruct(ts) => {
            let table = lua.create_table()?;
            for i in 0..ts.field_len() {
                if let Some(field) = ts.field(i) {
                    // Use _0, _1, etc. for tuple struct fields (like bevy-lua-ecs convention)
                    table.set(format!("_{}", i), reflection_to_lua_with_assets(lua, field, asset_registry)?)?;
                    // Also set numeric indices for array-style access
                    table.set(i + 1, reflection_to_lua_with_assets(lua, field, asset_registry)?)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Tuple(t) => {
            let table = lua.create_table()?;
            for i in 0..t.field_len() {
                if let Some(field) = t.field(i) {
                    table.set(i + 1, reflection_to_lua_with_assets(lua, field, asset_registry)?)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::List(list) => {
            let table = lua.create_table()?;
            for i in 0..list.len() {
                if let Some(item) = list.get(i) {
                    table.set(i + 1, reflection_to_lua_with_assets(lua, item, asset_registry)?)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Array(arr) => {
            let table = lua.create_table()?;
            for i in 0..arr.len() {
                if let Some(item) = arr.get(i) {
                    table.set(i + 1, reflection_to_lua_with_assets(lua, item, asset_registry)?)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Map(map) => {
            let table = lua.create_table()?;
            for (key, val) in map.iter() {
                // Try to get a string key
                if let Some(key_str) = key.try_downcast_ref::<String>() {
                    table.set(key_str.clone(), reflection_to_lua_with_assets(lua, val, asset_registry)?)?;
                } else if let Some(key_str) = key.try_downcast_ref::<&str>() {
                    table.set(*key_str, reflection_to_lua_with_assets(lua, val, asset_registry)?)?;
                } else {
                    // Use debug format for non-string keys
                    table.set(format!("{:?}", key), reflection_to_lua_with_assets(lua, val, asset_registry)?)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Set(set) => {
            let table = lua.create_table()?;
            for (i, item) in set.iter().enumerate() {
                table.set(i + 1, reflection_to_lua_with_assets(lua, item, asset_registry)?)?;
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Enum(e) => {
            // Get type info for Handle detection
            let type_path = value.get_represented_type_info()
                .map(|ti| ti.type_path().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            
            // Handle<T> is reflected as an Enum with Strong/Weak variants
            // If this is a Handle type and we have an asset_registry, try to serialize as path
            if type_path.contains("Handle<") {
                if let Some(registry) = asset_registry {
                    // Use generic handle extractor registry (no hardcoded types)
                    if let Some(path) = registry.try_extract_handle_path(value) {
                        debug!("[REFLECTION_TO_LUA] Handle {} serialized as path: '{}'", type_path, path);
                        return Ok(LuaValue::String(lua.create_string(&path)?));
                    }
                    debug!("[REFLECTION_TO_LUA] Handle detected but no extractor found or path not available: {}", type_path);
                }
            }
            
            // For normal enums, create a table with the variant name as key
            let table = lua.create_table()?;
            let variant_name = e.variant_name();
            
            if e.field_len() == 0 {
                // Unit variant
                table.set(variant_name, true)?;
            } else if e.field_len() == 1 {
                // Newtype variant
                if let Some(field) = e.field_at(0) {
                    table.set(variant_name, reflection_to_lua_with_assets(lua, field, asset_registry)?)?;
                }
            } else {
                // Tuple or struct variant
                let inner = lua.create_table()?;
                for i in 0..e.field_len() {
                    if let Some(field) = e.field_at(i) {
                        if let Some(name) = e.name_at(i) {
                            inner.set(name.to_string(), reflection_to_lua_with_assets(lua, field, asset_registry)?)?;
                        } else {
                            inner.set(i + 1, reflection_to_lua_with_assets(lua, field, asset_registry)?)?;
                        }
                    }
                }
                table.set(variant_name, inner)?;
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Opaque(opaque) => {
            // Get type info for logging
            let type_path = value.get_represented_type_info()
                .map(|ti| ti.type_path().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            debug!("[REFLECTION_TO_LUA] Opaque type: {}", type_path);
            
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
            // Entity - return bits as integer for use with world:get_entity()
            if let Some(entity) = opaque.try_downcast_ref::<Entity>() {
                debug!("[REFLECTION_TO_LUA] Opaque: Successfully downcast to Entity: {:?}", entity);
                return Ok(LuaValue::Integer(entity.to_bits() as i64));
            }
            
            // Handle<T> detection - serialize as path for network replication
            if type_path.contains("Handle<") {
                if let Some(registry) = asset_registry {
                    // Try UntypedHandle first (rare but possible)
                    if let Some(handle) = opaque.try_downcast_ref::<bevy::asset::UntypedHandle>() {
                        if let Some(path) = registry.get_path_for_handle(handle) {
                            debug!("[REFLECTION_TO_LUA] UntypedHandle serialized as path: '{}'", path);
                            return Ok(LuaValue::String(lua.create_string(&path)?));
                        }
                    }
                    // Use generic handle extractor registry (no hardcoded types)
                    if let Some(path) = registry.try_extract_handle_path(value) {
                        debug!("[REFLECTION_TO_LUA] Handle {} serialized as path: '{}'", type_path, path);
                        return Ok(LuaValue::String(lua.create_string(&path)?));
                    }
                }
                debug!("[REFLECTION_TO_LUA] Handle detected but no path found: {}", type_path);
            }
            
            // Fallback to debug string for unknown opaque types
            let debug_str = format!("{:?}", opaque);
            debug!("[REFLECTION_TO_LUA] Opaque fallback to debug: '{}'", debug_str);
            Ok(LuaValue::String(lua.create_string(&debug_str)?))
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

/// Helper function to resolve a component name to CachedComponentInfo
/// Uses cache if available, otherwise does full lookup
fn resolve_component_info(
    name: &str,
    query_cache: Option<&crate::query_cache::LuaQueryCache>,
    component_registry: &ComponentRegistry,
    type_registry: &bevy::reflect::TypeRegistry,
    world: &World,
) -> crate::query_cache::CachedComponentInfo {
    // Check cache first
    if let Some(cache) = query_cache {
        if let Some(info) = cache.get_component_info(name) {
            return info;
        }
    }

    // Cache miss - do lookup
    let info = if let Some(type_id) = component_registry.get_non_reflected_type_id(name) {
        if let Some(id) = world.components().get_id(*type_id) {
            crate::query_cache::CachedComponentInfo::Rust(id)
        } else {
            crate::query_cache::CachedComponentInfo::Lua
        }
    } else if let Some(type_path) = component_registry.get_type_path(name) {
        if let Some(registration) = type_registry.get_with_type_path(&type_path) {
            if let Some(id) = world.components().get_id(registration.type_id()) {
                crate::query_cache::CachedComponentInfo::Rust(id)
            } else {
                crate::query_cache::CachedComponentInfo::Lua
            }
        } else {
            crate::query_cache::CachedComponentInfo::Lua
        }
    } else {
        crate::query_cache::CachedComponentInfo::Lua
    };

    // Cache for next time - but only cache Rust components
    // Don't cache Lua/NotFound results since they might just be Rust components
    // that aren't registered in the type registry yet (initialization order race)
    if let Some(cache) = query_cache {
        if matches!(info, crate::query_cache::CachedComponentInfo::Rust(_)) {
            cache.cache_component_info(name, info.clone());
        }
    }

    info
}

/// Execute a query and collect entity snapshots
/// Uses per-frame caching with full component data for performance
pub fn execute_query(
    lua: &Lua,
    world: &World,
    query_builder: &LuaQueryBuilder,
    component_registry: &ComponentRegistry,
    update_queue: &ComponentUpdateQueue,
    last_run: u32,
    this_run: u32,
    query_cache: Option<&crate::query_cache::LuaQueryCache>,
    current_frame: u64,
    asset_registry: Option<&crate::asset_loading::AssetRegistry>,
) -> LuaResult<Vec<LuaEntitySnapshot>> {
    // Special handling for 'removed' queries
    // These return entity_bits for entities that had components removed this frame
    // The entities may no longer exist, so we can't return full snapshots
    if !query_builder.or_filters.removed.is_empty() {
        let removed_tracker = world
            .get_resource::<crate::removed_components::RemovedComponentsTracker>()
            .cloned()
            .unwrap_or_default();

        let mut seen_entities = HashSet::new();
        let mut results = Vec::new();

        // Collect entity_bits for all removed components (OR logic)
        for comp_name in &query_builder.or_filters.removed {
            for entity_bits in removed_tracker.get_removed(comp_name) {
                if seen_entities.insert(entity_bits) {
                    // Create a minimal snapshot with just the entity bits
                    // The entity may not exist anymore, but we provide the bits for state lookup
                    results.push(LuaEntitySnapshot {
                        entity: Entity::from_bits(entity_bits),
                        component_data: HashMap::new(),
                        lua_components: HashMap::new(),
                        changed_components: HashSet::new(),
                        added_components: HashSet::new(),
                        update_queue: update_queue.clone(),
                    });
                }
            }
        }

        return Ok(results);
    }

    let type_registry = component_registry.type_registry().read();
    
    // For queries without change detection, check cache first (full component data)
    // Note: We skip cache for queries with Without/AnyOf filters since those need runtime checks
    let can_use_cache = !query_builder.has_change_detection()
        && query_builder.without_components.is_empty()
        && query_builder.any_of_components.is_empty();

    if can_use_cache {
        if let Some(cache) = query_cache {
            if let Some(cached_results) = cache.get_full(&query_builder.with_components, current_frame) {
                // Fast path: directly use cached registry keys
                // No serialization, no reflection - just clone the Arc<RegistryKey>
                debug!(
                    "[QUERY_CACHE] HIT for {:?} - returning {} cached entities",
                    query_builder.with_components, cached_results.len()
                );
                let mut results = Vec::with_capacity(cached_results.len());
                for cached in cached_results {
                    let entity = Entity::from_bits(cached.entity_bits);
                    // Verify entity still exists (could have been despawned)
                    if let Ok(entity_ref) = world.get_entity(entity) {
                        let mut lua_components = cached.component_keys.clone();
                        let changed_set = HashSet::new(); // Cached results don't have change info
                        let added_set = HashSet::new(); // Cached results don't have added info

                        // Resolving optional components
                        let mut optional_component_keys = HashMap::new();
                        if !query_builder.optional_components.is_empty() {
                            for name in &query_builder.optional_components {
                                let info = resolve_component_info(
                                    name,
                                    query_cache,
                                    component_registry,
                                    &type_registry,
                                    world,
                                );
                                
                                match info {
                                    crate::query_cache::CachedComponentInfo::Rust(id) => {
                                        if let Some(component_info) = world.components().get_info(id) {
                                            // Only try to reflect if component exists on entity
                                            if entity_ref.contains_id(id) {
                                                let type_id = component_info.type_id().unwrap();
                                                if let Some(registration) = type_registry.get(type_id) {
                                                    if let Some(reflect_component) = registration.data::<ReflectComponent>() {
                                                        if let Some(component) = reflect_component.reflect(entity_ref) {
                                                            if let Ok(lua_value) = crate::lua_world_api::reflection_to_lua_with_assets(lua, component, asset_registry) {
                                                                if let Ok(registry_key) = lua.create_registry_value(lua_value) {
                                                                    optional_component_keys.insert(name.clone(), Arc::new(registry_key));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    crate::query_cache::CachedComponentInfo::Lua | crate::query_cache::CachedComponentInfo::NotFound => {
                                        // Lua components are handled later if they are 'with' components.
                                        // For optional, if it's Lua and not in 'with', we don't add it here.
                                        // NotFound means it doesn't exist, so nothing to add.
                                    }
                                }
                            }
                        }

                        // Merge optional components into lua_components
                        for (k, v) in optional_component_keys {
                            lua_components.insert(k, v);
                        }

                        results.push(LuaEntitySnapshot {
                            entity,
                            component_data: HashMap::new(), // Not needed, we have lua_components
                            lua_components,
                            changed_components: changed_set,
                            added_components: added_set,
                            update_queue: update_queue.clone(),
                        });
                    }
                }
                return Ok(results);
            }
        }
    }
    
    // Cache miss - need to do full query
    debug!(
        "[QUERY_CACHE] MISS for {:?} frame={} (changed_components: {:?})",
        query_builder.with_components, current_frame, query_builder.changed_components
    );
    
    // Determine if this query is for Lua-only components using CACHED lookups
    // This avoids the expensive type registry iteration (was 300-500us per query!)
    let t_check = std::time::Instant::now();
    
    let has_rust_components = if let Some(cache) = query_cache {
        // Try to use cached component type info
        let (cached_result, needs_lookup) = cache.check_rust_components(&query_builder.with_components);
        
        if let Some(has_rust) = cached_result {
            // All components were cached - fast path!
            has_rust
        } else {
            // Some components need registry lookup, then cache the result
            // Only cache positive (is_rust=true) results - don't cache false since
            // the component might just not be registered in the type registry yet
            let mut has_rust = false;
            for name in &needs_lookup {
                let is_rust = component_registry.get_type_path(name).is_some() ||
                    component_registry.get_non_reflected_type_id(name).is_some();
                if is_rust {
                    cache.cache_component_type(name, true);
                    has_rust = true;
                }
            }
            // Also check cached ones that returned false
            for name in &query_builder.with_components {
                if !needs_lookup.contains(name) {
                    if cache.is_rust_component(name).unwrap_or(false) {
                        has_rust = true;
                    }
                }
            }
            has_rust
        }
    } else {
        // No cache available - do full lookup (fallback)
        query_builder.with_components.iter().any(|name| {
            component_registry.get_type_path(name).is_some() ||
            component_registry.get_non_reflected_type_id(name).is_some()
        })
    };
    let _check_time = t_check.elapsed().as_micros();
    
    // Log which path we're taking (debug only - logging adds ~150us overhead)
    debug!(
        "[QUERY_PATH] {:?} has_rust={} changed_empty={} check_time={}us",
        query_builder.with_components, has_rust_components, query_builder.changed_components.is_empty(), _check_time
    );
    
    // FAST PATH: For pure Lua component queries (no Rust components, no complex filters)
    // This is a direct iteration without parallel overhead - much faster for small result sets
    // Skip FAST PATH if we have: Rust components, change detection, Without, AnyOf, Added, or Or filters
    let use_fast_path = !has_rust_components
        && !query_builder.has_change_detection()
        && query_builder.without_components.is_empty()
        && query_builder.any_of_components.is_empty();

    if use_fast_path {
        let component_id = world.components().component_id::<LuaCustomComponents>();
        if let Some(comp_id) = component_id {
            use std::time::Instant;
            let t0 = Instant::now();

            let mut results = Vec::new();
            let mut cache_entries = Vec::new();
            let mut entities_checked = 0u32;
            let mut entities_matched = 0u32;
            let mut archetypes_checked = 0u32;
            let mut archetypes_with_lua = 0u32;

            // Direct archetype traversal - no intermediate Vec, no parallel overhead
            for archetype in world.archetypes().iter() {
                archetypes_checked += 1;
                if !archetype.contains(comp_id) {
                    continue;
                }
                archetypes_with_lua += 1;

                for arch_entity in archetype.entities() {
                    entities_checked += 1;
                    let entity = arch_entity.id();
                    if let Ok(entity_ref) = world.get_entity(entity) {
                        if let Some(custom_components) = entity_ref.get::<LuaCustomComponents>() {
                            // Check if entity has ALL required Lua components
                            let has_all = query_builder.with_components.iter()
                                .all(|name| custom_components.components.contains_key(name));

                            if has_all {
                                entities_matched += 1;
                                // Build component map directly - no intermediate allocations
                                let mut lua_components = HashMap::new();
                                for name in &query_builder.with_components {
                                    if let Some(key) = custom_components.components.get(name) {
                                        lua_components.insert(name.clone(), key.clone());
                                    }
                                }

                                // Build cache entry
                                cache_entries.push(crate::query_cache::CachedEntityResult {
                                    entity_bits: entity.to_bits(),
                                    component_keys: lua_components.clone(),
                                });

                                results.push(LuaEntitySnapshot {
                                    entity,
                                    component_data: HashMap::new(),
                                    lua_components,
                                    changed_components: HashSet::new(),
                                    added_components: HashSet::new(),
                                    update_queue: update_queue.clone(),
                                });
                            }
                        }
                    }
                }
            }

            let elapsed = t0.elapsed().as_micros();
            // Log for slow queries or when debugging
            if elapsed >= 1000 { // 1ms
                debug!(
                    "[LUA_PERF] 🔍 FAST_PATH_QUERY: components={:?} archetypes={}/{} entities={}/{} time={}us",
                    query_builder.with_components, archetypes_with_lua, archetypes_checked,
                    entities_matched, entities_checked, elapsed
                );
            }

            // Cache results for later in this frame
            if let Some(cache) = query_cache {
                cache.insert_full(&query_builder.with_components, cache_entries, current_frame);
            }

            return Ok(results);
        }
    }
    
    // ARCHETYPE PATH: Optimized filtering using Bevy's archetype system
    let t_archetype_start = std::time::Instant::now();

    // 1. Identify ComponentIds for all required components (with caching)
    let mut required_ids = Vec::new();
    let mut lua_required = Vec::new();
    let mut rust_required = Vec::new();
    let lua_custom_comp_id = world.components().component_id::<crate::components::LuaCustomComponents>();

    // Resolve component names to ComponentIds with caching
    let mut cache_hits = 0u32;
    let mut cache_misses = 0u32;
    let mut cache_lookup_time_ns = 0u64;
    let mut expensive_lookup_time_ns = 0u64;

    for name in &query_builder.with_components {
        // Check cache first (fast path)
        if let Some(cache) = query_cache {
            let t_cache_start = std::time::Instant::now();
            let cached_info = cache.get_component_info(name);
            cache_lookup_time_ns += t_cache_start.elapsed().as_nanos() as u64;

            if let Some(info) = cached_info {
                cache_hits += 1;
                match info {
                    crate::query_cache::CachedComponentInfo::Rust(id) => {
                        required_ids.push(id);
                        rust_required.push(name.clone());
                        continue;
                    }
                    crate::query_cache::CachedComponentInfo::Lua | crate::query_cache::CachedComponentInfo::NotFound => {
                        let id = match lua_custom_comp_id {
                            Some(id) => id,
                            None => return Ok(Vec::new()),
                        };
                        if !required_ids.contains(&id) {
                            required_ids.push(id);
                        }
                        lua_required.push(name.clone());
                        continue;
                    }
                }
            }
        }

        cache_misses += 1;
        // Cache miss - do expensive lookup and cache result
        let t_expensive_start = std::time::Instant::now();
        let info = if let Some(type_id) = component_registry.get_non_reflected_type_id(name) {
            if let Some(id) = world.components().get_id(*type_id) {
                crate::query_cache::CachedComponentInfo::Rust(id)
            } else {
                crate::query_cache::CachedComponentInfo::Lua
            }
        } else if let Some(type_path) = component_registry.get_type_path(name) {
            if let Some(registration) = type_registry.get_with_type_path(&type_path) {
                if let Some(id) = world.components().get_id(registration.type_id()) {
                    crate::query_cache::CachedComponentInfo::Rust(id)
                } else {
                    crate::query_cache::CachedComponentInfo::Lua
                }
            } else {
                crate::query_cache::CachedComponentInfo::Lua
            }
        } else {
            crate::query_cache::CachedComponentInfo::Lua
        };
        expensive_lookup_time_ns += t_expensive_start.elapsed().as_nanos() as u64;

        // Cache for next time - but only cache Rust components
        // Don't cache Lua/NotFound results since they might just be Rust components
        // that aren't registered in the type registry yet (initialization order race)
        if let Some(cache) = query_cache {
            if matches!(info, crate::query_cache::CachedComponentInfo::Rust(_)) {
                cache.cache_component_info(name, info.clone());
            }
        }

        // Apply the resolved info
        match info {
            crate::query_cache::CachedComponentInfo::Rust(id) => {
                required_ids.push(id);
                rust_required.push(name.clone());
            }
            crate::query_cache::CachedComponentInfo::Lua | crate::query_cache::CachedComponentInfo::NotFound => {
                let id = match lua_custom_comp_id {
                    Some(id) => id,
                    None => return Ok(Vec::new()),
                };
                if !required_ids.contains(&id) {
                    required_ids.push(id);
                }
                lua_required.push(name.clone());
            }
        }
    }

    // Add IDs for changed components (also using cache)
    let mut changed_ids = Vec::new();
    let mut lua_changed = Vec::new();
    for name in &query_builder.changed_components {
        // Check cache first
        if let Some(cache) = query_cache {
            if let Some(info) = cache.get_component_info(name) {
                match info {
                    crate::query_cache::CachedComponentInfo::Rust(id) => {
                        changed_ids.push((name.clone(), id));
                        if !required_ids.contains(&id) {
                            required_ids.push(id);
                        }
                        continue;
                    }
                    crate::query_cache::CachedComponentInfo::Lua | crate::query_cache::CachedComponentInfo::NotFound => {
                        let id = match lua_custom_comp_id {
                            Some(id) => id,
                            None => return Ok(Vec::new()),
                        };
                        lua_changed.push(name.clone());
                        if !required_ids.contains(&id) {
                            required_ids.push(id);
                        }
                        continue;
                    }
                }
            }
        }

        // Cache miss - do lookup
        let info = if let Some(type_path) = component_registry.get_type_path(name) {
            if let Some(registration) = type_registry.get_with_type_path(&type_path) {
                if let Some(id) = world.components().get_id(registration.type_id()) {
                    crate::query_cache::CachedComponentInfo::Rust(id)
                } else {
                    crate::query_cache::CachedComponentInfo::Lua
                }
            } else {
                crate::query_cache::CachedComponentInfo::Lua
            }
        } else {
            crate::query_cache::CachedComponentInfo::Lua
        };

        // Cache for next time - but only cache Rust components
        // Don't cache Lua/NotFound results since they might just be Rust components
        // that aren't registered in the type registry yet (initialization order race)
        if let Some(cache) = query_cache {
            if matches!(info, crate::query_cache::CachedComponentInfo::Rust(_)) {
                cache.cache_component_info(name, info.clone());
            }
        }

        match info {
            crate::query_cache::CachedComponentInfo::Rust(id) => {
                changed_ids.push((name.clone(), id));
                if !required_ids.contains(&id) {
                    required_ids.push(id);
                }
            }
            crate::query_cache::CachedComponentInfo::Lua | crate::query_cache::CachedComponentInfo::NotFound => {
                let id = match lua_custom_comp_id {
                    Some(id) => id,
                    None => return Ok(Vec::new()),
                };
                lua_changed.push(name.clone());
                if !required_ids.contains(&id) {
                    required_ids.push(id);
                }
            }
        }
    }

    // Resolve Without filter components
    let mut excluded_ids = Vec::new();
    let mut lua_without = Vec::new();
    for name in &query_builder.without_components {
        let info = resolve_component_info(name, query_cache, component_registry, &*type_registry, world);
        match info {
            crate::query_cache::CachedComponentInfo::Rust(id) => {
                excluded_ids.push(id);
            }
            crate::query_cache::CachedComponentInfo::Lua | crate::query_cache::CachedComponentInfo::NotFound => {
                lua_without.push(name.clone());
            }
        }
    }

    // Resolve AnyOf filter components
    let mut any_of_ids = Vec::new();
    let mut lua_any_of = Vec::new();
    for name in &query_builder.any_of_components {
        let info = resolve_component_info(name, query_cache, component_registry, &*type_registry, world);
        match info {
            crate::query_cache::CachedComponentInfo::Rust(id) => {
                any_of_ids.push((name.clone(), id));
            }
            crate::query_cache::CachedComponentInfo::Lua | crate::query_cache::CachedComponentInfo::NotFound => {
                lua_any_of.push(name.clone());
            }
        }
    }

    // Resolve Added filter components (AND logic)
    let mut added_ids = Vec::new();
    let mut lua_added = Vec::new();
    for name in &query_builder.added_components {
        let info = resolve_component_info(name, query_cache, component_registry, &*type_registry, world);
        match info {
            crate::query_cache::CachedComponentInfo::Rust(id) => {
                added_ids.push((name.clone(), id));
                if !required_ids.contains(&id) {
                    required_ids.push(id);
                }
            }
            crate::query_cache::CachedComponentInfo::Lua | crate::query_cache::CachedComponentInfo::NotFound => {
                lua_added.push(name.clone());
                if let Some(id) = lua_custom_comp_id {
                    if !required_ids.contains(&id) {
                        required_ids.push(id);
                    }
                }
            }
        }
    }

    // Resolve Or filter components
    let mut or_changed_ids = Vec::new();
    let mut or_changed_lua = Vec::new();
    for name in &query_builder.or_filters.changed {
        let info = resolve_component_info(name, query_cache, component_registry, &*type_registry, world);
        match info {
            crate::query_cache::CachedComponentInfo::Rust(id) => {
                or_changed_ids.push((name.clone(), id));
            }
            crate::query_cache::CachedComponentInfo::Lua | crate::query_cache::CachedComponentInfo::NotFound => {
                or_changed_lua.push(name.clone());
            }
        }
    }

    let mut or_added_ids = Vec::new();
    let mut or_added_lua = Vec::new();
    for name in &query_builder.or_filters.added {
        let info = resolve_component_info(name, query_cache, component_registry, &*type_registry, world);
        match info {
            crate::query_cache::CachedComponentInfo::Rust(id) => {
                or_added_ids.push((name.clone(), id));
            }
            crate::query_cache::CachedComponentInfo::Lua | crate::query_cache::CachedComponentInfo::NotFound => {
                or_added_lua.push(name.clone());
            }
        }
    }

    let has_or_filters = !or_changed_ids.is_empty()
        || !or_changed_lua.is_empty()
        || !or_added_ids.is_empty()
        || !or_added_lua.is_empty();

    let mut results = Vec::new();
    let mut cache_entries = Vec::new();
    let mut archetypes_checked = 0u32;
    let mut archetypes_matched = 0u32;
    let mut entities_checked = 0u32;

    let t_id_resolve = std::time::Instant::now();
    let id_resolve_time = t_id_resolve.duration_since(t_archetype_start).as_micros();

    // 2. Iterate archetypes that contain ALL required components
    for archetype in world.archetypes().iter() {
        archetypes_checked += 1;

        // Check all required components are present (With filter)
        let mut matches = true;
        for &id in &required_ids {
            if !archetype.contains(id) {
                matches = false;
                break;
            }
        }
        if !matches {
            continue;
        }

        // Check no excluded Rust components are present (Without filter - archetype level)
        let mut has_excluded = false;
        for &id in &excluded_ids {
            if archetype.contains(id) {
                has_excluded = true;
                break;
            }
        }
        if has_excluded {
            continue;
        }

        // Check at least one AnyOf Rust component is present (if any specified)
        // Note: Lua AnyOf components are checked at entity level since they share LuaCustomComponents
        if !any_of_ids.is_empty() && lua_any_of.is_empty() {
            let mut has_any_of = false;
            for (_name, id) in &any_of_ids {
                if archetype.contains(*id) {
                    has_any_of = true;
                    break;
                }
            }
            if !has_any_of {
                continue;
            }
        }

        archetypes_matched += 1;
        
        // 3. Collect entities and their data
        for arch_entity in archetype.entities() {
            entities_checked += 1;
            let entity = arch_entity.id();
            let entity_ref = world.get_entity(entity).expect("Entity in archetype must exist");
            
            let mut entity_changed = HashSet::new();
            let mut entity_added = HashSet::new();

            // Determine if we need LuaCustomComponents for any check
            let needs_lua_comps = !lua_required.is_empty()
                || !lua_changed.is_empty()
                || !lua_without.is_empty()
                || !lua_any_of.is_empty()
                || !lua_added.is_empty()
                || !or_changed_lua.is_empty()
                || !or_added_lua.is_empty();

            let custom_comps = if needs_lua_comps {
                entity_ref.get::<crate::components::LuaCustomComponents>()
            } else {
                None
            };

            // Check Lua-specific requirements (With filter)
            if !lua_required.is_empty() {
                let comps = match &custom_comps {
                    Some(c) => c,
                    None => continue, // No LuaCustomComponents means missing Lua components
                };
                let mut lua_missing = false;
                for name in &lua_required {
                    if !comps.components.contains_key(name) {
                        lua_missing = true;
                        break;
                    }
                }
                if lua_missing { continue; }
            }

            // Check Lua Without filter (exclude entities with these Lua components)
            if !lua_without.is_empty() {
                if let Some(comps) = &custom_comps {
                    let mut has_excluded = false;
                    for name in &lua_without {
                        if comps.components.contains_key(name) {
                            has_excluded = true;
                            break;
                        }
                    }
                    if has_excluded { continue; }
                }
            }

            // Check AnyOf filter (at least one must be present)
            if !any_of_ids.is_empty() || !lua_any_of.is_empty() {
                let mut has_any_of = false;

                // Check Rust AnyOf components
                for (_name, id) in &any_of_ids {
                    if entity_ref.contains_id(*id) {
                        has_any_of = true;
                        break;
                    }
                }

                // Check Lua AnyOf components
                if !has_any_of {
                    if let Some(comps) = &custom_comps {
                        for name in &lua_any_of {
                            if comps.components.contains_key(name) {
                                has_any_of = true;
                                break;
                            }
                        }
                    }
                }

                if !has_any_of { continue; }
            }

            // Check Lua Changed filters (AND logic - all must have changed)
            if !lua_changed.is_empty() {
                let comps = match &custom_comps {
                    Some(c) => c,
                    None => continue,
                };
                let mut lua_not_changed = false;
                for name in &lua_changed {
                    if let Some(&tick) = comps.changed_ticks.get(name) {
                        use bevy::ecs::component::Tick;
                        if !Tick::new(tick).is_newer_than(Tick::new(last_run), Tick::new(this_run)) {
                            lua_not_changed = true;
                            break;
                        }
                    } else {
                        lua_not_changed = true;
                        break;
                    }
                }
                if lua_not_changed { continue; }
                for name in &lua_changed {
                    entity_changed.insert(name.clone());
                }
            }

            // Check Bevy Changed filters (AND logic - all must have changed)
            if !changed_ids.is_empty() {
                let mut rust_not_changed = false;
                for (_name, id) in &changed_ids {
                    if let Some(ticks) = entity_ref.get_change_ticks_by_id(*id) {
                        use bevy::ecs::component::Tick;
                        if !ticks.is_changed(Tick::new(last_run), Tick::new(this_run)) {
                            rust_not_changed = true;
                            break;
                        }
                    }
                }
                if rust_not_changed { continue; }
                for (name, _) in &changed_ids {
                    entity_changed.insert(name.clone());
                }
            }

            // Check Added filters (AND logic - all must be newly added)
            if !added_ids.is_empty() {
                let mut rust_not_added = false;
                for (_name, id) in &added_ids {
                    if let Some(ticks) = entity_ref.get_change_ticks_by_id(*id) {
                        use bevy::ecs::component::Tick;
                        if !ticks.is_added(Tick::new(last_run), Tick::new(this_run)) {
                            rust_not_added = true;
                            break;
                        }
                    } else {
                        rust_not_added = true;
                        break;
                    }
                }
                if rust_not_added { continue; }
                for (name, _) in &added_ids {
                    entity_added.insert(name.clone());
                }
            }

            if !lua_added.is_empty() {
                let comps = match &custom_comps {
                    Some(c) => c,
                    None => continue,
                };
                let mut lua_not_added = false;
                for name in &lua_added {
                    if let Some(&tick) = comps.added_ticks.get(name) {
                        use bevy::ecs::component::Tick;
                        if !Tick::new(tick).is_newer_than(Tick::new(last_run), Tick::new(this_run)) {
                            lua_not_added = true;
                            break;
                        }
                    } else {
                        lua_not_added = true;
                        break;
                    }
                }
                if lua_not_added { continue; }
                for name in &lua_added {
                    entity_added.insert(name.clone());
                }
            }

            // Check Or filters (at least one must match - early exit on first match)
            if has_or_filters {
                let mut or_matched = false;

                // Check Or Changed - Rust components
                for (name, id) in &or_changed_ids {
                    if let Some(ticks) = entity_ref.get_change_ticks_by_id(*id) {
                        use bevy::ecs::component::Tick;
                        let changed = ticks.is_changed(Tick::new(last_run), Tick::new(this_run));
                        if changed {
                            or_matched = true;
                            entity_changed.insert(name.clone());
                        }
                    }
                }

                // Check Or Changed - Lua components
                if !or_matched || true { // Continue checking to collect all matches
                    if let Some(comps) = &custom_comps {
                        for name in &or_changed_lua {
                            if let Some(&tick) = comps.changed_ticks.get(name) {
                                use bevy::ecs::component::Tick;
                                if Tick::new(tick).is_newer_than(Tick::new(last_run), Tick::new(this_run)) {
                                    or_matched = true;
                                    entity_changed.insert(name.clone());
                                }
                            }
                        }
                    }
                }

                // Check Or Added - Rust components
                if !or_matched || true {
                    for (name, id) in &or_added_ids {
                        if let Some(ticks) = entity_ref.get_change_ticks_by_id(*id) {
                            use bevy::ecs::component::Tick;
                            if ticks.is_added(Tick::new(last_run), Tick::new(this_run)) {
                                or_matched = true;
                                entity_added.insert(name.clone());
                            }
                        }
                    }
                }

                // Check Or Added - Lua components
                if !or_matched || true {
                    if let Some(comps) = &custom_comps {
                        for name in &or_added_lua {
                            if let Some(&tick) = comps.added_ticks.get(name) {
                                use bevy::ecs::component::Tick;
                                if Tick::new(tick).is_newer_than(Tick::new(last_run), Tick::new(this_run)) {
                                    or_matched = true;
                                    entity_added.insert(name.clone());
                                }
                            }
                        }
                    }
                }

                if !or_matched { continue; }
            }
            
            // 4. Build snapshot and serialize
            let mut component_data = HashMap::new();
            let mut lua_components = HashMap::new();

            // Map Lua components (required + any_of + changed/added that are present)
            if let Some(comps) = &custom_comps {
                // Required Lua components
                for name in &lua_required {
                    if let Some(key) = comps.components.get(name) {
                        lua_components.insert(name.clone(), key.clone());
                    }
                }
                // AnyOf Lua components (include if present)
                for name in &lua_any_of {
                    if let Some(key) = comps.components.get(name) {
                        lua_components.insert(name.clone(), key.clone());
                    }
                }
                // Include components from changed/added filters so they're serialized
                for name in &or_changed_lua {
                    if let Some(key) = comps.components.get(name) {
                        lua_components.insert(name.clone(), key.clone());
                    }
                }
                for name in &or_added_lua {
                    if let Some(key) = comps.components.get(name) {
                        lua_components.insert(name.clone(), key.clone());
                    }
                }
            }

            // Map Rust components (required + any_of + changed/added that are present)
            // Combine required, any_of, and or_filters Rust component names
            let rust_components_to_serialize: Vec<&String> = rust_required.iter()
                .chain(any_of_ids.iter().map(|(name, _)| name))
                .chain(or_changed_ids.iter().map(|(name, _)| name))
                .chain(or_added_ids.iter().map(|(name, _)| name))
                .collect();

            for name in rust_components_to_serialize {
                // Non-reflected component
                if component_registry.get_non_reflected_type_id(name).is_some() {
                    if let Some(serialized) = component_registry.serialize_non_reflected(&entity_ref, name) {
                        component_data.insert(name.clone(), serialized);
                    }
                    continue;
                }

                // Reflected component
                if let Some(type_path) = component_registry.get_type_path(name) {
                    if let Some(registration) = type_registry.get_with_type_path(&type_path) {
                        if let Some(reflect_component) = registration.data::<ReflectComponent>() {
                            if let Some(component) = reflect_component.reflect(
                                Into::<bevy::ecs::world::FilteredEntityRef>::into(&entity_ref),
                            ) {
                                match reflection_to_lua_with_assets(lua, component, asset_registry) {
                                    Ok(lua_val) => {
                                        if let Ok(key) = lua.create_registry_value(lua_val) {
                                            lua_components.insert(name.clone(), Arc::new(key));
                                        }
                                    }
                                    Err(_) => {
                                        // Fallback to JSON if reflection fails
                                        use bevy::reflect::serde::TypedReflectSerializer;
                                        let serializer = TypedReflectSerializer::new(
                                            component.as_reflect(),
                                            &type_registry,
                                        );
                                        if let Ok(json_value) = serde_json::to_value(serializer) {
                                            if let Ok(json_string) = serde_json::to_string(&json_value) {
                                                component_data.insert(name.clone(), json_string);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Store results
            cache_entries.push(crate::query_cache::CachedEntityResult {
                entity_bits: entity.to_bits(),
                component_keys: lua_components.clone(),
            });
            
            results.push(LuaEntitySnapshot {
                entity,
                component_data,
                lua_components,
                changed_components: entity_changed,
                added_components: entity_added,
                update_queue: update_queue.clone(),
            });
        }
    }
    
    // Store in cache if no change detection and no complex filters
    // (Without and AnyOf filters make caching less effective)
    let should_cache = !query_builder.has_change_detection()
        && query_builder.without_components.is_empty()
        && query_builder.any_of_components.is_empty();

    if should_cache {
        if let Some(cache) = query_cache {
            cache.insert_full(&query_builder.with_components, cache_entries, current_frame);
        }
    }

    let t_archetype_end = std::time::Instant::now();
    let total_archetype_time = t_archetype_end.duration_since(t_archetype_start).as_micros();
    if total_archetype_time >= 1000 { // 1ms
        debug!(
            "[LUA_PERF] 🔍 ARCHETYPE_PATH_QUERY: components={:?} changed={:?} id_resolve={}us (cache_lookup={}us expensive={}us) cache_hits={} cache_misses={} cache_avail={} archetypes={}/{} entities_checked={} results={} total={}us",
            query_builder.with_components, query_builder.changed_components,
            id_resolve_time, cache_lookup_time_ns / 1000, expensive_lookup_time_ns / 1000,
            cache_hits, cache_misses, query_cache.is_some(),
            archetypes_matched, archetypes_checked,
            entities_checked, results.len(), total_archetype_time
        );
    }

    Ok(results)
}

