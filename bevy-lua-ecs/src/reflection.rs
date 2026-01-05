use bevy::prelude::*;
#[cfg(feature = "auto-reflection")]
use bevy::reflect::{Reflect, StructInfo, TypeInfo, TypeRegistry};
use bevy::reflect::{PartialReflect, ReflectRef};
use mlua::prelude::*;
use std::collections::HashMap;

/// Convert any reflected Bevy value to a Lua value
/// - Entity → u64 bits (use world:get_entity(bits) to get entity wrapper)
/// - Structs → Lua tables with field names as keys
/// - TupleStructs → Lua tables with numeric keys (1-indexed)  
/// - Lists/Arrays → Lua array tables
/// - Tuples → Lua array tables
/// - Primitives → Lua primitives (f32, f64, bool, String, etc.)
/// - Options → nil if None, value if Some
/// Falls back to Debug format if reflection isn't available
pub fn reflect_to_lua_value(lua: &Lua, value: &dyn PartialReflect) -> LuaResult<LuaValue> {
    // Check if this is an Entity by type path
    if let Some(type_info) = value.get_represented_type_info() {
        if type_info.type_path() == "bevy_ecs::entity::Entity" {
            // We can't downcast PartialReflect to Entity directly
            // Use the debug representation to extract the entity bits
            // Entity debug format is "Entity(index, generation)" or similar
            // But we want to return the actual bits - need to use try_downcast_ref on concrete
            // Since we're getting &dyn PartialReflect, try using try_as_reflect
            if let Some(reflect) = value.try_as_reflect() {
                if let Some(entity) = reflect.downcast_ref::<Entity>() {
                    return Ok(LuaValue::Integer(entity.to_bits() as i64));
                }
            }
        }
    }
    
    match value.reflect_ref() {
        ReflectRef::Struct(s) => {
            let table = lua.create_table()?;
            for i in 0..s.field_len() {
                if let (Some(name), Some(field)) = (s.name_at(i), s.field_at(i)) {
                    let lua_value = reflect_to_lua_value(lua, field)?;
                    table.set(name, lua_value)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::TupleStruct(ts) => {
            let table = lua.create_table()?;
            for i in 0..ts.field_len() {
                if let Some(field) = ts.field(i) {
                    let lua_value = reflect_to_lua_value(lua, field)?;
                    table.set((i + 1) as i64, lua_value)?; // Lua is 1-indexed
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Tuple(t) => {
            let table = lua.create_table()?;
            for i in 0..t.field_len() {
                if let Some(field) = t.field(i) {
                    let lua_value = reflect_to_lua_value(lua, field)?;
                    table.set((i + 1) as i64, lua_value)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::List(list) => {
            let table = lua.create_table()?;
            for i in 0..list.len() {
                if let Some(item) = list.get(i) {
                    let lua_value = reflect_to_lua_value(lua, item)?;
                    table.set((i + 1) as i64, lua_value)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Array(arr) => {
            let table = lua.create_table()?;
            for i in 0..arr.len() {
                if let Some(item) = arr.get(i) {
                    let lua_value = reflect_to_lua_value(lua, item)?;
                    table.set((i + 1) as i64, lua_value)?;
                }
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Map(map) => {
            let table = lua.create_table()?;
            for (key, val) in map.iter() {
                let lua_key = reflect_to_lua_value(lua, key)?;
                let lua_val = reflect_to_lua_value(lua, val)?;
                table.set(lua_key, lua_val)?;
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Set(set) => {
            let table = lua.create_table()?;
            for (i, item) in set.iter().enumerate() {
                let lua_value = reflect_to_lua_value(lua, item)?;
                table.set((i + 1) as i64, lua_value)?;
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Enum(e) => {
            // Handle Option specially
            if let Some(type_info) = value.get_represented_type_info() {
                let type_path = type_info.type_path();
                if type_path.starts_with("core::option::Option") {
                    let variant = e.variant_name();
                    if variant == "None" {
                        return Ok(LuaValue::Nil);
                    } else if variant == "Some" && e.field_len() > 0 {
                        if let Some(field) = e.field_at(0) {
                            return reflect_to_lua_value(lua, field);
                        }
                    }
                }
            }
            // For other enums, return table with variant name and fields
            let table = lua.create_table()?;
            table.set("variant", e.variant_name())?;
            let fields = lua.create_table()?;
            for i in 0..e.field_len() {
                if let Some(field) = e.field_at(i) {
                    let lua_value = reflect_to_lua_value(lua, field)?;
                    // Use field name if available, otherwise index
                    if let Some(name) = e.name_at(i) {
                        fields.set(name, lua_value)?;
                    } else {
                        fields.set((i + 1) as i64, lua_value)?;
                    }
                }
            }
            table.set("fields", fields)?;
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Opaque(opaque) => {
            // Try to downcast to common primitive types
            if let Some(reflect) = opaque.try_as_reflect() {
                // Numeric types
                if let Some(v) = reflect.downcast_ref::<f32>() {
                    return Ok(LuaValue::Number(*v as f64));
                }
                if let Some(v) = reflect.downcast_ref::<f64>() {
                    return Ok(LuaValue::Number(*v));
                }
                if let Some(v) = reflect.downcast_ref::<i8>() {
                    return Ok(LuaValue::Integer(*v as i64));
                }
                if let Some(v) = reflect.downcast_ref::<i16>() {
                    return Ok(LuaValue::Integer(*v as i64));
                }
                if let Some(v) = reflect.downcast_ref::<i32>() {
                    return Ok(LuaValue::Integer(*v as i64));
                }
                if let Some(v) = reflect.downcast_ref::<i64>() {
                    return Ok(LuaValue::Integer(*v));
                }
                if let Some(v) = reflect.downcast_ref::<u8>() {
                    return Ok(LuaValue::Integer(*v as i64));
                }
                if let Some(v) = reflect.downcast_ref::<u16>() {
                    return Ok(LuaValue::Integer(*v as i64));
                }
                if let Some(v) = reflect.downcast_ref::<u32>() {
                    return Ok(LuaValue::Integer(*v as i64));
                }
                if let Some(v) = reflect.downcast_ref::<u64>() {
                    return Ok(LuaValue::Integer(*v as i64));
                }
                if let Some(v) = reflect.downcast_ref::<usize>() {
                    return Ok(LuaValue::Integer(*v as i64));
                }
                // Boolean
                if let Some(v) = reflect.downcast_ref::<bool>() {
                    return Ok(LuaValue::Boolean(*v));
                }
                // String types
                if let Some(v) = reflect.downcast_ref::<String>() {
                    return Ok(LuaValue::String(lua.create_string(v)?));
                }
                if let Some(v) = reflect.downcast_ref::<&str>() {
                    return Ok(LuaValue::String(lua.create_string(*v)?));
                }
            }
            // Fallback to Debug format
            Ok(LuaValue::String(lua.create_string(&format!("{:?}", opaque))?))
        }
    }
}

/// Try to convert a generic type implementing PartialReflect to a Lua value
/// This is a convenience wrapper for use from generated code
pub fn try_reflect_to_lua_value<T: PartialReflect + ?Sized>(lua: &Lua, value: &T) -> LuaResult<LuaValue> {
    reflect_to_lua_value(lua, value.as_partial_reflect())
}

/// Convert a string to an enum value using reflection
/// Looks up the enum in the type registry and finds the variant by name
/// Returns an error if the type is not an enum
pub fn string_to_enum<T: bevy::reflect::Reflect + 'static>(
    variant_name: &str,
    type_registry: &bevy::ecs::reflect::AppTypeRegistry,
) -> Result<T, String> {
    use bevy::reflect::{DynamicEnum, DynamicVariant, ReflectFromReflect};
    
    let registry = type_registry.read();
    
    // Get the type registration for T
    let type_id = std::any::TypeId::of::<T>();
    let registration = registry.get(type_id)
        .ok_or_else(|| format!("Type not found in registry"))?;
    
    // Check if it's an enum
    let type_info = registration.type_info();
    let enum_info = match type_info {
        bevy::reflect::TypeInfo::Enum(info) => info,
        _ => return Err(format!("Type is not an enum")),
    };
    
    // Find the variant by name
    let variant_info = enum_info.variant(variant_name)
        .ok_or_else(|| format!("Variant '{}' not found", variant_name))?;
    
    // Create a DynamicEnum for this variant
    let dynamic_variant = match variant_info {
        bevy::reflect::VariantInfo::Unit(_) => DynamicVariant::Unit,
        _ => return Err(format!("Only unit variants are supported, '{}' is not a unit variant", variant_name)),
    };
    
    let dynamic_enum = DynamicEnum::new(variant_name, dynamic_variant);
    
    // Use FromReflect to convert to concrete type
    let from_reflect = registration.data::<ReflectFromReflect>()
        .ok_or_else(|| format!("Type has no FromReflect implementation"))?;
    
    let concrete = from_reflect.from_reflect(&dynamic_enum)
        .ok_or_else(|| format!("Failed to construct enum from reflection"))?;
    
    concrete.downcast::<T>()
        .map(|boxed| *boxed)
        .map_err(|_| format!("Failed to downcast to expected type"))
}

/// Convert a Lua value to a Rust type using reflection
/// Uses TypeInfo at runtime to determine the appropriate conversion strategy:
/// - Primitives (f32, i32, bool, etc.): direct conversion from Lua numbers/booleans
/// - Enums: string -> variant lookup
/// - Structs: table -> FromReflect
pub fn lua_value_to_type<T: bevy::reflect::Reflect + 'static>(
    lua: &mlua::Lua,
    value: mlua::Value,
    type_registry: &bevy::ecs::reflect::AppTypeRegistry,
) -> Result<T, String> {
    use bevy::reflect::{ReflectFromReflect, TypeInfo};
    
    let registry = type_registry.read();
    let type_id = std::any::TypeId::of::<T>();
    
    let registration = registry.get(type_id)
        .ok_or_else(|| format!("Type '{}' not registered in TypeRegistry", std::any::type_name::<T>()))?;
    
    let type_info = registration.type_info();
    
    // Check type kind and get string if enum (to allow dropping registry early for enum case)
    let is_enum = matches!(type_info, TypeInfo::Enum(_));
    let is_opaque = matches!(type_info, TypeInfo::Opaque(_));
    
    // Handle primitives first - no registry needed
    if is_opaque {
        drop(registry);
        return convert_lua_to_primitive::<T>(&value);
    }
    
    // Handle enums - extract string and drop registry before recursive call
    if is_enum {
        drop(registry);
        return match value {
            mlua::Value::String(s) => {
                let s_str = s.to_str().map_err(|e| format!("Invalid UTF-8: {}", e))?;
                string_to_enum::<T>(s_str.as_ref(), type_registry)
            }
            _ => Err(format!("Enum type '{}' requires a string value (variant name), got {:?}", 
                std::any::type_name::<T>(), value.type_name()))
        };
    }
    
    // Handle structs - need registry for FromReflect
    match type_info {
        TypeInfo::Struct(_) | TypeInfo::TupleStruct(_) => {
            match value {
                mlua::Value::Table(ref t) => {
                    let rfr = registration.data::<ReflectFromReflect>()
                        .ok_or_else(|| format!("Type '{}' has no FromReflect", std::any::type_name::<T>()))?;
                    let dynamic = crate::lua_table_to_dynamic(lua, t, type_info, type_registry)
                        .map_err(|e| format!("Failed to build '{}': {}", std::any::type_name::<T>(), e))?;
                    let concrete = rfr.from_reflect(&dynamic)
                        .ok_or_else(|| format!("FromReflect failed for '{}'", std::any::type_name::<T>()))?;
                    concrete.downcast::<T>()
                        .map(|b| *b)
                        .map_err(|_| format!("Downcast failed for '{}'", std::any::type_name::<T>()))
                }
                _ => Err(format!("Struct type '{}' requires a table value, got {:?}",
                    std::any::type_name::<T>(), value.type_name()))
            }
        }
        
        // Lists, Arrays, Tuples - could be extended if needed
        _ => Err(format!("Unsupported type kind for '{}': {:?}", 
            std::any::type_name::<T>(), type_info))
    }
}

/// Convert Lua value to a primitive type
/// Handles f32, f64, i32, u32, i64, u64, bool
fn convert_lua_to_primitive<T: 'static>(value: &mlua::Value) -> Result<T, String> {
    use std::any::TypeId;
    
    let type_id = TypeId::of::<T>();
    
    // Handle numeric types
    if type_id == TypeId::of::<f32>() {
        match value {
            mlua::Value::Number(n) => {
                let v = *n as f32;
                // SAFETY: We checked type_id matches f32
                Ok(unsafe { std::mem::transmute_copy(&v) })
            }
            mlua::Value::Integer(i) => {
                let v = *i as f32;
                Ok(unsafe { std::mem::transmute_copy(&v) })
            }
            _ => Err(format!("f32 requires a number, got {:?}", value.type_name()))
        }
    } else if type_id == TypeId::of::<f64>() {
        match value {
            mlua::Value::Number(n) => {
                let v = *n;
                Ok(unsafe { std::mem::transmute_copy(&v) })
            }
            mlua::Value::Integer(i) => {
                let v = *i as f64;
                Ok(unsafe { std::mem::transmute_copy(&v) })
            }
            _ => Err(format!("f64 requires a number, got {:?}", value.type_name()))
        }
    } else if type_id == TypeId::of::<i32>() {
        match value {
            mlua::Value::Integer(i) => {
                let v = *i as i32;
                Ok(unsafe { std::mem::transmute_copy(&v) })
            }
            mlua::Value::Number(n) => {
                let v = *n as i32;
                Ok(unsafe { std::mem::transmute_copy(&v) })
            }
            _ => Err(format!("i32 requires an integer, got {:?}", value.type_name()))
        }
    } else if type_id == TypeId::of::<u32>() {
        match value {
            mlua::Value::Integer(i) => {
                let v = *i as u32;
                Ok(unsafe { std::mem::transmute_copy(&v) })
            }
            mlua::Value::Number(n) => {
                let v = *n as u32;
                Ok(unsafe { std::mem::transmute_copy(&v) })
            }
            _ => Err(format!("u32 requires an integer, got {:?}", value.type_name()))
        }
    } else if type_id == TypeId::of::<i64>() {
        match value {
            mlua::Value::Integer(i) => {
                let v = *i;
                Ok(unsafe { std::mem::transmute_copy(&v) })
            }
            mlua::Value::Number(n) => {
                let v = *n as i64;
                Ok(unsafe { std::mem::transmute_copy(&v) })
            }
            _ => Err(format!("i64 requires an integer, got {:?}", value.type_name()))
        }
    } else if type_id == TypeId::of::<u64>() {
        match value {
            mlua::Value::Integer(i) => {
                let v = *i as u64;
                Ok(unsafe { std::mem::transmute_copy(&v) })
            }
            mlua::Value::Number(n) => {
                let v = *n as u64;
                Ok(unsafe { std::mem::transmute_copy(&v) })
            }
            _ => Err(format!("u64 requires an integer, got {:?}", value.type_name()))
        }
    } else if type_id == TypeId::of::<bool>() {
        match value {
            mlua::Value::Boolean(b) => {
                let v = *b;
                Ok(unsafe { std::mem::transmute_copy(&v) })
            }
            mlua::Value::Integer(i) => {
                let v = *i != 0;
                Ok(unsafe { std::mem::transmute_copy(&v) })
            }
            _ => Err(format!("bool requires a boolean, got {:?}", value.type_name()))
        }
    } else {
        Err(format!("Unsupported primitive type: {}", std::any::type_name::<T>()))
    }
}


/// Trait for types that can be converted to Lua values from systemparam method results
pub trait ToLuaValue {
    fn to_lua_value(&self, lua: &Lua) -> LuaResult<LuaValue>;
}

// Implementation for slices of (Entity, T) where T: PartialReflect  
impl<T: PartialReflect> ToLuaValue for [(Entity, T)] {
    fn to_lua_value(&self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        for (i, (entity, hit)) in self.iter().enumerate() {
            let hit_table = lua.create_table()?;
            hit_table.set("entity", entity.to_bits())?;
            // Convert the hit data using reflection
            let hit_lua = reflect_to_lua_value(lua, hit.as_partial_reflect())?;
            hit_table.set("data", hit_lua)?;
            table.set((i + 1) as i64, hit_table)?;
        }
        Ok(LuaValue::Table(table))
    }
}

// Implementation for references to slices of (Entity, T)
impl<T: PartialReflect> ToLuaValue for &[(Entity, T)] {
    fn to_lua_value(&self, lua: &Lua) -> LuaResult<LuaValue> {
        <[(Entity, T)] as ToLuaValue>::to_lua_value(*self, lua)
    }
}

/// Convert any result type to Lua value using the ToLuaValue trait
/// This is the main entry point for generated code
pub fn result_to_lua_value<T: ToLuaValue + ?Sized>(lua: &Lua, value: &T) -> LuaResult<LuaValue> {
    value.to_lua_value(lua)
}

/// Bundle definition for Lua spawning
pub struct BundleDefinition {
    pub name: String,
    pub spawn_fn: Box<dyn Fn(&LuaValue, &mut EntityCommands) -> LuaResult<()> + Send + Sync>,
}

/// Registry of available bundles for Lua
#[derive(Resource, Default)]
pub struct BundleRegistry {
    bundles: HashMap<String, BundleDefinition>,
}

impl BundleRegistry {
    /// Register a bundle with a spawn function
    pub fn register<F>(&mut self, name: impl Into<String>, spawn_fn: F)
    where
        F: Fn(&LuaValue, &mut EntityCommands) -> LuaResult<()> + Send + Sync + 'static,
    {
        let name = name.into();
        self.bundles.insert(
            name.clone(),
            BundleDefinition {
                name,
                spawn_fn: Box::new(spawn_fn),
            },
        );
    }

    /// Get a bundle definition by name
    pub fn get(&self, name: &str) -> Option<&BundleDefinition> {
        self.bundles.get(name)
    }

    /// Create registry from TypeRegistry using reflection (auto-reflection feature)
    #[cfg(feature = "auto-reflection")]
    pub fn from_type_registry(type_registry: &AppTypeRegistry) -> Self {
        let mut registry = Self::default();
        let type_registry = type_registry.read();

        for registration in type_registry.iter() {
            let type_info = registration.type_info();

            // Only process struct types
            if let TypeInfo::Struct(struct_info) = type_info {
                let type_name = struct_info.type_path_table().short_path();

                // Clone what we need before moving into closure
                let struct_info_clone = struct_info.clone();
                let type_path = registration.type_path().to_string();

                registry.register(
                    type_name,
                    move |data: &LuaValue, entity: &mut EntityCommands| {
                        spawn_from_reflection(data, entity, &struct_info_clone, &type_path)
                    },
                );
            }
        }

        registry
    }
}

/// Spawn entity from reflected type information
#[cfg(feature = "auto-reflection")]
fn spawn_from_reflection(
    data: &LuaValue,
    entity: &mut EntityCommands,
    struct_info: &StructInfo,
    type_path: &str,
) -> LuaResult<()> {
    debug!("Spawning {} via reflection", type_path);

    // For now, we'll use a simplified approach
    // Full implementation would use DynamicStruct and FromReflect

    // This is a placeholder - actual implementation would:
    // 1. Create DynamicStruct from struct_info
    // 2. Populate fields from Lua table
    // 3. Use FromReflect to convert to concrete type
    // 4. Insert into entity

    Err(LuaError::RuntimeError(
        "Full reflection not yet implemented - use manual LuaSpawnable".to_string(),
    ))
}

/// Trait for types that can be spawned from Lua
pub trait LuaSpawnable {
    fn from_lua(data: &LuaTable, entity: &mut EntityCommands) -> LuaResult<()>;
}

/// Macro to register bundles automatically
#[macro_export]
macro_rules! register_lua_bundles {
    ($registry:expr, $($bundle:ty),* $(,)?) => {
        $(
            {
                let bundle_name = std::any::type_name::<$bundle>()
                    .split("::")
                    .last()
                    .unwrap_or(std::any::type_name::<$bundle>());

                $registry.register(bundle_name, |data: &mlua::Table, entity: &mut bevy::ecs::system::EntityCommands| {
                    <$bundle as $crate::reflection::LuaSpawnable>::from_lua(data, entity)
                });
            }
        )*
    };
}

/// Derive macro helper - converts Lua table to reflected value
#[cfg(feature = "auto-reflection")]
pub fn lua_table_to_reflect(
    table: &LuaTable,
    field_name: &str,
    type_info: &TypeInfo,
    type_registry: &TypeRegistry,
) -> LuaResult<Box<dyn Reflect>> {
    match type_info {
        TypeInfo::Struct(struct_info) => {
            // Handle common Bevy types
            match struct_info.type_path() {
                "bevy_color::srgba::Srgba" | "bevy_color::Color" => {
                    let r: f32 = table.get("r").unwrap_or(1.0);
                    let g: f32 = table.get("g").unwrap_or(1.0);
                    let b: f32 = table.get("b").unwrap_or(1.0);
                    let a: f32 = table.get("a").unwrap_or(1.0);
                    Ok(Box::new(Color::srgba(r, g, b, a)))
                }
                "glam::Vec2" => {
                    let x: f32 = table.get("x").unwrap_or(0.0);
                    let y: f32 = table.get("y").unwrap_or(0.0);
                    Ok(Box::new(Vec2::new(x, y)))
                }
                "glam::Vec3" => {
                    let x: f32 = table.get("x").unwrap_or(0.0);
                    let y: f32 = table.get("y").unwrap_or(0.0);
                    let z: f32 = table.get("z").unwrap_or(0.0);
                    Ok(Box::new(Vec3::new(x, y, z)))
                }
                _ => Err(LuaError::RuntimeError(format!(
                    "Unsupported struct type for reflection: {}",
                    struct_info.type_path()
                ))),
            }
        }
        _ => Err(LuaError::RuntimeError(format!(
            "Unsupported type info for field {}: {:?}",
            field_name, type_info
        ))),
    }
}
