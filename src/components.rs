use bevy::prelude::*;
use bevy::reflect::{TypeInfo, PartialReflect, ReflectMut};
use bevy::ecs::reflect::{ReflectComponent, ReflectCommandExt};
use mlua::prelude::*;
use std::collections::HashMap;

use std::sync::Arc;

/// Component handler function type
type ComponentHandler = Box<dyn Fn(&LuaTable, &mut EntityCommands) -> LuaResult<()> + Send + Sync>;

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
}

impl ComponentRegistry {
    /// Create registry from app's type registry
    pub fn from_type_registry(type_registry: AppTypeRegistry) -> Self {
        let mut registry = Self {
            handlers: HashMap::new(),
            type_registry: type_registry.clone(),
        };
        
        // Auto-discover and register all components
        registry.discover_components();
        
        registry
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
                
                // Create handler using reflection
                let handler = Box::new(move |data: &LuaTable, entity: &mut EntityCommands| {
                    spawn_component_via_reflection(
                        data,
                        entity,
                        &full_path_clone,
                        &registry_clone,
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

/// Spawn component using Bevy's reflection system
fn spawn_component_via_reflection(
    data: &LuaTable,
    entity: &mut EntityCommands,
    type_path: &str,
    type_registry: &AppTypeRegistry,
) -> LuaResult<()> {
    let registry = type_registry.read();
    
    // Get type registration
    let Some(registration) = registry.get_with_type_path(type_path) else {
        return Err(LuaError::RuntimeError(format!("Type not found: {}", type_path)));
    };
    
    // Create default instance
    let Some(reflect_default) = registration.data::<ReflectDefault>() else {
        return Err(LuaError::RuntimeError(format!("{} doesn't implement Default", type_path)));
    };
    
    let mut component = reflect_default.default();
    let type_info = registration.type_info();
    
    // Patch component with Lua data
    match type_info {
        TypeInfo::Struct(struct_info) => {
            // Get mutable reflection
            let reflect_mut = component.reflect_mut();
            
            // Pattern match to get struct
            if let ReflectMut::Struct(struct_mut) = reflect_mut {
                
                // Iterate through struct fields
                for i in 0..struct_info.field_len() {
                    let field_info = struct_info.field_at(i).unwrap();
                    let field_name = field_info.name();
                    
                    // Try to get value from Lua table
                    if let Ok(lua_value) = data.get::<LuaValue>(field_name) {
                        // Get mutable field
                        if let Some(field) = struct_mut.field_at_mut(i) {
                            set_field_from_lua(field, &lua_value)?;
                        }
                    }
                }
            }
        }
        
        TypeInfo::TupleStruct(_) => {
            // Handle single-field tuple structs like Text(String)
            for pair in data.pairs::<LuaValue, LuaValue>() {
                if let Ok((key, value)) = pair {
                    let _key_str = match &key {
                        LuaValue::String(s) => {
                            match s.to_str() {
                                Ok(s_str) => format!("'{}'", s_str),
                                Err(_) => "?".to_string(),
                            }
                        }
                        LuaValue::Integer(i) => format!("{}", i),
                        _ => "other".to_string(),
                    };
                    let _value_str = match &value {
                        LuaValue::String(s) => {
                            match s.to_str() {
                                Ok(s_str) => format!("String(\"{}\")", s_str),
                                Err(_) => "String(?)".to_string(),
                            }
                        }
                        LuaValue::Table(_) => "Table".to_string(),
                        LuaValue::Nil => "Nil".to_string(),
                        _ => format!("{:?}", value),
                    };
                }
            }
            
            // Try to get the value from common keys
            let lua_value: LuaValue = if let Ok(val) = data.raw_get("value") {
                if !matches!(val, LuaValue::Nil) {
                    val
                } else if let Ok(val) = data.raw_get("text") {
                    if !matches!(val, LuaValue::Nil) {
                        val
                    } else if let Ok(val) = data.raw_get("color") {
                        if !matches!(val, LuaValue::Nil) {
                            val
                        } else if let Ok(val) = data.raw_get("0") {
                            val
                        } else {
                            return Err(LuaError::RuntimeError("No valid value found in tuple struct data".to_string()));
                        }
                    } else {
                        return Err(LuaError::RuntimeError("Failed to access tuple struct data".to_string()));
                    }
                } else {
                    return Err(LuaError::RuntimeError("Failed to access tuple struct data".to_string()));
                }
            } else {
                return Err(LuaError::RuntimeError("Failed to access tuple struct data".to_string()));
            };
            
            let reflect_mut = component.reflect_mut();
            if let ReflectMut::TupleStruct(tuple_mut) = reflect_mut {
                if let Some(field) = tuple_mut.field_mut(0) {
                    set_field_from_lua(field, &lua_value)?;
                }
            }
        }
        
        _ => {
            return Err(LuaError::RuntimeError(format!("Unsupported type: {}", type_path)));
        }
    }
    
    // Insert component via reflection
    entity.insert_reflect(component);

    Ok(())
}

/// Set a reflected field value from a Lua value
fn set_field_from_lua(field: &mut dyn PartialReflect, lua_value: &LuaValue) -> LuaResult<()> {
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
    } else {
        // warn!(
        //     "Could not downcast field of type {} to any known type",
        //     field.reflect_type_path()
        // );
    }
    
    Ok(())
}
