use bevy::prelude::*;
use bevy::reflect::{TypeInfo, PartialReflect, ReflectMut};
use bevy::ecs::reflect::{ReflectComponent, ReflectCommandExt};
use mlua::prelude::*;
use std::collections::HashMap;

use std::sync::Arc;

/// Component handler function type
type ComponentHandler = Box<dyn Fn(&LuaValue, &mut EntityCommands) -> LuaResult<()> + Send + Sync>;

/// Generic container for components defined purely in Lua
#[derive(Component, Default, Clone)]
pub struct LuaCustomComponents {
    pub components: HashMap<String, Arc<LuaRegistryKey>>,
}

/// Registry of component handlers using reflection
#[derive(Resource)]
pub struct ComponentRegistry {
    handlers: HashMap<String, ComponentHandler>,
    type_registry: AppTypeRegistry,
    asset_registry: Option<crate::asset_loading::AssetRegistry>,
}

impl ComponentRegistry {
    /// Create registry from app's type registry
    pub fn from_type_registry(type_registry: AppTypeRegistry) -> Self {
        let mut registry = Self {
            handlers: HashMap::new(),
            type_registry: type_registry.clone(),
            asset_registry: None,
        };
        
        // Auto-discover and register all components
        registry.discover_components();
        
        registry
    }
    
    /// Set the asset registry (called after AssetRegistry is created)
    pub fn set_asset_registry(&mut self, asset_registry: crate::asset_loading::AssetRegistry) {
        self.asset_registry = Some(asset_registry);
        // Re-discover components to update handlers with asset registry
        self.discover_components();
    }
    
    /// Automatically discover all registered components via reflection
    fn discover_components(&mut self) {
        let type_registry = self.type_registry.read();
        
        for registration in type_registry.iter() {
            // Check if this type is a component
            if registration.data::<ReflectComponent>().is_some() {
                let type_info = registration.type_info();
                let short_name = type_info.type_path_table().short_path().to_string();
                let full_path = type_info.type_path().to_string();
                
                // Clone what we need for the closure
                let registry_clone = self.type_registry.clone();
                let full_path_clone = full_path.clone();
                let asset_registry_clone = self.asset_registry.clone();
                
                // Create handler using reflection
                let handler = Box::new(move |data: &LuaValue, entity: &mut EntityCommands| {
                    spawn_component_via_reflection(
                        data,
                        entity,
                        &full_path_clone,
                        &registry_clone,
                        asset_registry_clone.as_ref(),
                    )
                });
                
                self.handlers.insert(short_name, handler);
            }
        }
    }
    
    /// Get a component handler by name
    pub fn get(&self, name: &str) -> Option<&ComponentHandler> {
        self.handlers.get(name)
    }
    
    /// Get the full type path for a component by short name
    pub fn get_type_path(&self, short_name: &str) -> Option<String> {
        let type_registry = self.type_registry.read();
        for registration in type_registry.iter() {
            if registration.data::<ReflectComponent>().is_some() {
                let type_info = registration.type_info();
                if type_info.type_path_table().short_path() == short_name {
                    return Some(type_info.type_path().to_string());
                }
            }
        }
        None
    }
    
    /// Get access to the type registry
    pub fn type_registry(&self) -> &AppTypeRegistry {
        &self.type_registry
    }
}

/// Spawn component using Bevy's reflection system (with serde fallback)
fn spawn_component_via_reflection(
    data: &LuaValue,
    entity: &mut EntityCommands,
    type_path: &str,
    type_registry: &AppTypeRegistry,
    asset_registry: Option<&crate::asset_loading::AssetRegistry>,
) -> LuaResult<()> {
    let registry = type_registry.read();
    
    // Get type registration
    let Some(registration) = registry.get_with_type_path(type_path) else {
        return Err(LuaError::RuntimeError(format!("Type not found: {}", type_path)));
    };
    
    // PATH 1: Try Reflect-based creation (existing system)
    if let Some(reflect_default) = registration.data::<ReflectDefault>() {
        let mut component = reflect_default.default();
        let type_info = registration.type_info();
        
        // Patch component with Lua data
        match type_info {
            TypeInfo::Struct(struct_info) => {
                // Structs require a table
                let data_table = match data {
                    LuaValue::Table(t) => t,
                    _ => return Err(LuaError::RuntimeError(
                        format!("{} requires a table, got {:?}", type_path, data)
                    )),
                };
                
                // Get mutable reflection
                let reflect_mut = component.reflect_mut();
                
                // Pattern match to get struct
                if let ReflectMut::Struct(struct_mut) = reflect_mut {
                    
                    // Iterate through struct fields
                    for i in 0..struct_info.field_len() {
                        let field_info = struct_info.field_at(i).unwrap();
                        let field_name = field_info.name();
                        
                        // Try to get value from Lua table
                        if let Ok(lua_value) = data_table.get::<LuaValue>(field_name) {
                            // Get mutable field
                            if let Some(field) = struct_mut.field_at_mut(i) {
                                set_field_from_lua(field, &lua_value, asset_registry)?;
                            }
                        }
                    }
                }
            }
            
            TypeInfo::TupleStruct(struct_info) => {
                // Tuple structs require a table
                let data_table = match data {
                    LuaValue::Table(t) => t,
                    _ => return Err(LuaError::RuntimeError(
                        format!("{} requires a table, got {:?}", type_path, data)
                    )),
                };
                
                // Handle single-field tuple structs like Text(String)
                // Try to get the value from common keys
                // Handle single-field tuple structs like Text(String)
                // Try to get the value from common keys
                let lua_value: LuaValue = 'search: {
                    // Check "value"
                    if let Ok(val) = data_table.raw_get::<LuaValue>("value") {
                        if !matches!(val, LuaValue::Nil) { break 'search val; }
                    }
                    
                    // Check "0"
                    if let Ok(val) = data_table.raw_get::<LuaValue>("0") {
                        if !matches!(val, LuaValue::Nil) { break 'search val; }
                    }
                    
                    // Check index 1
                    if let Ok(val) = data_table.get::<LuaValue>(1) {
                        if !matches!(val, LuaValue::Nil) { break 'search val; }
                    }
                    
                    // Fallback: if the tuple struct has 1 field, check if we can find a single value
                    if struct_info.field_len() == 1 {
                        let mut pairs = data_table.pairs::<LuaValue, LuaValue>();
                        
                        // Get first pair
                        if let Some(Ok((_, val))) = pairs.next() {
                            // Ensure there is no second pair (to avoid ambiguity)
                            if pairs.next().is_none() {
                                break 'search val;
                            } else {
                                return Err(LuaError::RuntimeError("Ambiguous tuple struct data: multiple keys found".to_string()));
                            }
                        }
                    }
                    
                    LuaValue::Nil
                };
                
                if matches!(lua_value, LuaValue::Nil) {
                     return Err(LuaError::RuntimeError("Failed to access tuple struct data".to_string()));
                }
                
                let reflect_mut = component.reflect_mut();
                if let ReflectMut::TupleStruct(tuple_mut) = reflect_mut {
                    if let Some(field) = tuple_mut.field_mut(0) {
                        set_field_from_lua(field, &lua_value, asset_registry)?;
                    }
                }
            }
            
            TypeInfo::Enum(enum_info) => {
                // Enums can be specified as strings (variant name)
                let variant_name = match data {
                    LuaValue::String(s) => s.to_str()?.to_string(),
                    _ => return Err(LuaError::RuntimeError(
                        format!("{} enum requires a string variant name, got {:?}", type_path, data)
                    )),
                };
                
                // Find matching variant (case-sensitive)
                let variant_info = enum_info.variant(&variant_name)
                    .ok_or_else(|| LuaError::RuntimeError(
                        format!("Unknown variant '{}' for enum {}. Available variants: {}",
                            variant_name,
                            type_path,
                            enum_info.iter().map(|v| v.name()).collect::<Vec<_>>().join(", ")
                        )
                    ))?;
                
                // Handle unit variants
                use bevy::reflect::{DynamicEnum, DynamicVariant};
                
                let dynamic_variant = match variant_info {
                    bevy::reflect::VariantInfo::Unit(_) => DynamicVariant::Unit,
                    _ => return Err(LuaError::RuntimeError(
                        format!("Only unit enum variants are supported via this path. Use serde path for complex enums.")
                    )),
                };
                
                let dynamic_enum = DynamicEnum::new(&variant_name, dynamic_variant);
                
                // Convert to concrete type via ReflectFromReflect
                let reflect_from_reflect = registration.data::<bevy::reflect::ReflectFromReflect>()
                    .ok_or_else(|| LuaError::RuntimeError(
                        format!("{} doesn't implement FromReflect", type_path)
                    ))?;
                
                let new_component = reflect_from_reflect.from_reflect(&dynamic_enum)
                    .ok_or_else(|| LuaError::RuntimeError(
                        format!("Failed to create {} from reflection.", type_path)
                    ))?;
                
                component.apply(new_component.as_ref());
            }
            
            _ => {
                return Err(LuaError::RuntimeError(format!("Unsupported type: {}", type_path)));
            }
        }
        
        // Insert component via reflection
        entity.insert_reflect(component);
        return Ok(());
    }
    
    // PATH 2: Try Serde deserialization (fallback for non-Reflect types like Collider)
    // Note: This path is for types that ARE in the TypeRegistry (implement Reflect) 
    // but use serde for deserialization. Collider is NOT in TypeRegistry, so it 
    // will be handled by SerdeComponentRegistry in entity_spawner.rs instead.
    
    // Neither Reflect nor Serde available
    Err(LuaError::RuntimeError(format!(
        "{} doesn't implement Reflect (with Default). Cannot create from Lua via reflection.",
        type_path
    )))
}

/// Set a reflected field value from a Lua value
fn set_field_from_lua(
    field: &mut dyn PartialReflect,
    lua_value: &LuaValue,
    asset_registry: Option<&crate::asset_loading::AssetRegistry>,
) -> LuaResult<()> {
    // Check if this is any Handle<T> field - generic handle resolution
    let type_path = field.reflect_type_path().to_string();
    if type_path.contains("Handle<") {
        if let LuaValue::Integer(asset_id) = lua_value {
            if let Some(registry) = asset_registry {
                // Generic handle resolution: try Handle<Image> as it's the most common
                if type_path.contains("Image>") {
                    if let Some(image_handle) = registry.get_image_handle(*asset_id as u32) {
                        if let Some(handle_field) = field.try_downcast_mut::<Handle<Image>>() {
                            *handle_field = image_handle;
                            return Ok(());
                        }
                    }
                }
                // For other handle types, this is a limitation of the current generic system
                // Future: Could use TypeRegistry to dynamically downcast to any Handle<T>
            }
        }
    }
    
    // Try to downcast to common types
    if let Some(f32_field) = field.try_downcast_mut::<f32>() {
        match lua_value {
            LuaValue::Number(n) => {
                *f32_field = *n as f32;
            }
            LuaValue::Integer(i) => {
                *f32_field = *i as f32;
            }
            _ => {}
        }
    } else if let Some(i32_field) = field.try_downcast_mut::<i32>() {
        if let LuaValue::Integer(i) = lua_value {
            *i32_field = *i as i32;
        }
    } else if let Some(string_field) = field.try_downcast_mut::<String>() {
        if let LuaValue::String(s) = lua_value {
            *string_field = s.to_str()?.to_string();
        }
    } else if let Some(bool_field) = field.try_downcast_mut::<bool>() {
        if let LuaValue::Boolean(b) = lua_value {
            *bool_field = *b;
        }
    } else if let Some(color_field) = field.try_downcast_mut::<Color>() {
        if let LuaValue::Table(color_table) = lua_value {
            let r: f32 = color_table.get("r").unwrap_or(1.0);
            let g: f32 = color_table.get("g").unwrap_or(1.0);
            let b: f32 = color_table.get("b").unwrap_or(1.0);
            let a: f32 = color_table.get("a").unwrap_or(1.0);
            *color_field = Color::srgba(r, g, b, a);
        }
    } else if let Some(vec3_field) = field.try_downcast_mut::<Vec3>() {
        if let LuaValue::Table(vec_table) = lua_value {
            let x: f32 = vec_table.get("x").unwrap_or(0.0);
            let y: f32 = vec_table.get("y").unwrap_or(0.0);
            let z: f32 = vec_table.get("z").unwrap_or(0.0);
            *vec3_field = Vec3::new(x, y, z);
        }
    } else if let Some(vec2_field) = field.try_downcast_mut::<Vec2>() {
        if let LuaValue::Table(vec_table) = lua_value {
            let x: f32 = vec_table.get("x").unwrap_or(0.0);
            let y: f32 = vec_table.get("y").unwrap_or(0.0);
            *vec2_field = Vec2::new(x, y);
        }
    } else if let Some(quat_field) = field.try_downcast_mut::<Quat>() {
        if let LuaValue::Table(quat_table) = lua_value {
            let x: f32 = quat_table.get("x").unwrap_or(0.0);
            let y: f32 = quat_table.get("y").unwrap_or(0.0);
            let z: f32 = quat_table.get("z").unwrap_or(0.0);
            let w: f32 = quat_table.get("w").unwrap_or(1.0);
            *quat_field = Quat::from_xyzw(x, y, z, w);
        }
    } else if let LuaValue::Table(nested_table) = lua_value {
        // Generic nested struct/enum handling using reflection
        if let Err(e) = set_nested_field_from_lua(field, nested_table, asset_registry) {
            // Silently continue if nested field setting fails - might not be a nested struct
            let _ = e;
        }
    } else {
        // warn!(
        //     "Could not downcast field of type {} to any known type",
        //     field.reflect_type_path()
        // );
    }
    
    Ok(())
}

/// Generic handler for nested structs and enums using reflection
fn set_nested_field_from_lua(
    field: &mut dyn PartialReflect,
    table: &LuaTable,
    asset_registry: Option<&crate::asset_loading::AssetRegistry>,
) -> LuaResult<()> {
    use bevy::reflect::{DynamicStruct, DynamicTuple, DynamicEnum, DynamicVariant};
    
    match field.reflect_mut() {
        ReflectMut::Struct(struct_mut) => {
            // Set each field in the struct from the table
            for i in 0..struct_mut.field_len() {
                if let Some(field_name) = struct_mut.name_at(i) {
                    if let Ok(lua_value) = table.get::<LuaValue>(field_name) {
                        if let Some(nested_field) = struct_mut.field_at_mut(i) {
                            set_field_from_lua(nested_field, &lua_value, asset_registry)?;
                        }
                    }
                }
            }
        }
        ReflectMut::Enum(_enum_mut) => {
            // Generic Option<T> handling: Try to create a Some(T) variant
            // This handles Option<Rect>, Option<TextureAtlas>, etc.
            
            // Get the type info to understand the enum structure
            let type_path = field.reflect_type_path();
            
            // For Option types, we want to create Some(inner_value)
            // The table represents the inner value (e.g., Rect { min, max })
            if type_path.contains("Option<") {
                // Create a dynamic struct for the inner value
                let mut inner_struct = DynamicStruct::default();
                
                // Populate the inner struct from all table fields
                for pair in table.pairs::<String, LuaValue>() {
                    if let Ok((key, value)) = pair {
                        match &value {
                            LuaValue::Integer(i) => {
                                inner_struct.insert_boxed(&key, Box::new(*i as i32));
                            }
                            LuaValue::Number(n) => {
                                inner_struct.insert_boxed(&key, Box::new(*n as f32));
                            }
                            LuaValue::Boolean(b) => {
                                inner_struct.insert_boxed(&key, Box::new(*b));
                            }
                            LuaValue::String(s) => {
                                if let Ok(string) = s.to_str() {
                                    inner_struct.insert_boxed(&key, Box::new(string.to_string()));
                                }
                            }
                            LuaValue::Table(nested_table) => {
                                // Handle nested tables (e.g., min/max in Rect)
                                if key == "min" || key == "max" {
                                    // Try to create Vec2
                                    if let (Ok(x), Ok(y)) = (
                                        nested_table.get::<f32>("x"),
                                        nested_table.get::<f32>("y")
                                    ) {
                                        inner_struct.insert_boxed(&key, Box::new(Vec2::new(x, y)));
                                    }
                                } else {
                                    // Generic nested struct handling
                                    let mut nested_struct = DynamicStruct::default();
                                    for nested_pair in nested_table.pairs::<String, LuaValue>() {
                                        if let Ok((nested_key, nested_value)) = nested_pair {
                                            match &nested_value {
                                                LuaValue::Number(n) => {
                                                    nested_struct.insert_boxed(&nested_key, Box::new(*n as f32));
                                                }
                                                LuaValue::Integer(i) => {
                                                    nested_struct.insert_boxed(&nested_key, Box::new(*i as i32));
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    inner_struct.insert_boxed(&key, Box::new(nested_struct));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                
                // Wrap in tuple for Some(.0)
                let mut some_tuple = DynamicTuple::default();
                some_tuple.insert_boxed(Box::new(inner_struct));
                
                let some_variant = DynamicVariant::Tuple(some_tuple);
                let some_enum = DynamicEnum::new("Some", some_variant);
                
                field.apply(&some_enum);
            }
        }
        _ => {}
    }
    
    Ok(())
}
