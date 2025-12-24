use bevy::prelude::*;
use mlua::prelude::*;

/// Convert a reflected value to a Lua value
/// This is a helper function for event reading via reflection
/// Used by world:read_events() to convert Bevy events to Lua tables
pub fn reflection_to_lua(
    lua: &Lua,
    value: &dyn bevy::reflect::PartialReflect,
    registry: &AppTypeRegistry,
) -> LuaResult<LuaValue> {
    use bevy::reflect::ReflectRef;

    match value.reflect_ref() {
        ReflectRef::Struct(s) => {
            let table = lua.create_table()?;
            for i in 0..s.field_len() {
                let field_name = s.name_at(i).unwrap();
                let field_value = s.field_at(i).unwrap();
                let lua_value = reflection_to_lua(lua, field_value, registry)?;
                table.set(field_name, lua_value)?;
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::TupleStruct(ts) => {
            let table = lua.create_table()?;
            for i in 0..ts.field_len() {
                let field_value = ts.field(i).unwrap();
                let lua_value = reflection_to_lua(lua, field_value, registry)?;
                table.set(i + 1, lua_value)?;
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Tuple(t) => {
            let table = lua.create_table()?;
            for i in 0..t.field_len() {
                let field_value = t.field(i).unwrap();
                let lua_value = reflection_to_lua(lua, field_value, registry)?;
                table.set(i + 1, lua_value)?;
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::List(l) => {
            let table = lua.create_table()?;
            for i in 0..l.len() {
                let item = l.get(i).unwrap();
                let lua_value = reflection_to_lua(lua, item, registry)?;
                table.set(i + 1, lua_value)?;
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Array(a) => {
            let table = lua.create_table()?;
            for i in 0..a.len() {
                let item = a.get(i).unwrap();
                let lua_value = reflection_to_lua(lua, item, registry)?;
                table.set(i + 1, lua_value)?;
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Map(m) => {
            let table = lua.create_table()?;
            for (key, value) in m.iter() {
                let lua_key = reflection_to_lua(lua, key, registry)?;
                let lua_value = reflection_to_lua(lua, value, registry)?;
                table.set(lua_key, lua_value)?;
            }
            Ok(LuaValue::Table(table))
        }
        ReflectRef::Enum(e) => {
            // For enums, create a table with the variant name as key
            let table = lua.create_table()?;
            let variant_name = e.variant_name();

            match e.variant_type() {
                bevy::reflect::VariantType::Struct => {
                    let variant_table = lua.create_table()?;
                    for i in 0..e.field_len() {
                        let field_name = e.name_at(i).unwrap();
                        let field_value = e.field_at(i).unwrap();
                        let lua_value = reflection_to_lua(lua, field_value, registry)?;
                        variant_table.set(field_name, lua_value)?;
                    }
                    table.set(variant_name, variant_table)?;
                }
                bevy::reflect::VariantType::Tuple => {
                    let variant_table = lua.create_table()?;
                    for i in 0..e.field_len() {
                        let field_value = e.field_at(i).unwrap();
                        let lua_value = reflection_to_lua(lua, field_value, registry)?;
                        variant_table.set(i + 1, lua_value)?;
                    }
                    table.set(variant_name, variant_table)?;
                }
                bevy::reflect::VariantType::Unit => {
                    table.set(variant_name, true)?;
                }
            }

            Ok(LuaValue::Table(table))
        }
        ReflectRef::Opaque(v) => {
            // Handle primitive types explicitly - they are opaque in Bevy reflection
            // but we need to convert them to proper Lua types
            
            // Try to downcast to common primitive types
            if let Some(b) = v.try_downcast_ref::<bool>() {
                return Ok(LuaValue::Boolean(*b));
            }
            if let Some(n) = v.try_downcast_ref::<f32>() {
                return Ok(LuaValue::Number(*n as f64));
            }
            if let Some(n) = v.try_downcast_ref::<f64>() {
                return Ok(LuaValue::Number(*n));
            }
            if let Some(n) = v.try_downcast_ref::<i32>() {
                return Ok(LuaValue::Integer(*n as i64));
            }
            if let Some(n) = v.try_downcast_ref::<i64>() {
                return Ok(LuaValue::Integer(*n));
            }
            if let Some(n) = v.try_downcast_ref::<u32>() {
                return Ok(LuaValue::Integer(*n as i64));
            }
            if let Some(n) = v.try_downcast_ref::<u64>() {
                return Ok(LuaValue::Integer(*n as i64));
            }
            if let Some(n) = v.try_downcast_ref::<usize>() {
                return Ok(LuaValue::Integer(*n as i64));
            }
            if let Some(s) = v.try_downcast_ref::<String>() {
                return Ok(LuaValue::String(lua.create_string(s)?));
            }
            
            // For other opaque values, use debug representation as string
            // This is a fallback for types we don't explicitly handle
            let debug_str = format!("{:?}", v);
            Ok(LuaValue::String(lua.create_string(&debug_str)?))
        }
        ReflectRef::Set(_) => {
            // Sets are not commonly used in events, return empty table
            Ok(LuaValue::Table(lua.create_table()?))
        }
    }
}

/// Build a DynamicStruct from a Lua table using type info from the registry
/// This properly inserts fields into the DynamicStruct for FromReflect conversion
///
/// The optional `asset_registry` parameter enables handle ID lookup - when a Lua integer
/// is provided for a field expecting a Handle<T>, the handle is looked up by ID.
pub fn lua_table_to_dynamic(
    lua: &Lua,
    table: &LuaTable,
    type_info: &bevy::reflect::TypeInfo,
    registry: &AppTypeRegistry,
) -> LuaResult<bevy::reflect::DynamicStruct> {
    lua_table_to_dynamic_with_assets(lua, table, type_info, registry, None)
}

/// Build a DynamicStruct from a Lua table with asset registry for handle lookup
pub fn lua_table_to_dynamic_with_assets(
    lua: &Lua,
    table: &LuaTable,
    type_info: &bevy::reflect::TypeInfo,
    registry: &AppTypeRegistry,
    asset_registry: Option<&crate::asset_loading::AssetRegistry>,
) -> LuaResult<bevy::reflect::DynamicStruct> {
    use bevy::reflect::{DynamicStruct, NamedField, TypeInfo};

    let mut dynamic = DynamicStruct::default();

    if let TypeInfo::Struct(struct_info) = type_info {
        // Set represented_type so FromReflect knows the target type
        // Get from TypeRegistry for 'static lifetime
        {
            let reg = registry.read();
            if let Some(registration) = reg.get(type_info.ty().id()) {
                dynamic.set_represented_type(Some(registration.type_info()));
            }
        }

        for i in 0..struct_info.field_len() {
            let field: &NamedField = struct_info.field_at(i).unwrap();
            let field_name = field.name();

            // Try to get the value from the Lua table
            if let Ok(lua_val) = table.get::<LuaValue>(field_name) {
                // Get type_info - fallback to TypeRegistry lookup if field doesn't have it
                let field_type_info = field.type_info().or_else(|| {
                    let reg = registry.read();
                    reg.get(field.type_id()).map(|r| r.type_info())
                });
                // Convert Lua value to appropriate reflected type
                let field_value = lua_value_to_box_reflect_with_assets(
                    lua,
                    &lua_val,
                    field_type_info,
                    registry,
                    asset_registry,
                )?;
                dynamic.insert_boxed(field_name, field_value);
            }
        }
    }

    Ok(dynamic)
}

/// Convert a Lua value to a boxed Reflect value based on type info
fn lua_value_to_box_reflect(
    lua: &Lua,
    lua_value: &LuaValue,
    type_info: Option<&bevy::reflect::TypeInfo>,
    registry: &AppTypeRegistry,
) -> LuaResult<Box<dyn bevy::reflect::PartialReflect>> {
    lua_value_to_box_reflect_with_assets(lua, lua_value, type_info, registry, None)
}

/// Convert a Lua value to a boxed Reflect value with optional asset registry for handle lookup
fn lua_value_to_box_reflect_with_assets(
    lua: &Lua,
    lua_value: &LuaValue,
    type_info: Option<&bevy::reflect::TypeInfo>,
    registry: &AppTypeRegistry,
    asset_registry: Option<&crate::asset_loading::AssetRegistry>,
) -> LuaResult<Box<dyn bevy::reflect::PartialReflect>> {
    use bevy::reflect::{DynamicStruct, TypeInfo};

    match lua_value {
        LuaValue::Number(n) => {
            // Check if type might expect a Handle or newtype wrapper - purely via reflection
            if let Some(type_info) = type_info {
                let type_path = type_info.ty().path();

                // Only attempt handle lookup if type looks like it could be a Handle or newtype
                // Detected via reflection: Handle types, single-field newtypes, or structs containing Handle fields
                let might_be_handle = type_path.contains("Handle<") 
                    || matches!(type_info, 
                        bevy::reflect::TypeInfo::TupleStruct(ts) if ts.field_len() == 1
                    )
                    || matches!(type_info,
                        bevy::reflect::TypeInfo::Struct(s) if s.field_len() == 1
                    )
                    // Check if struct has any field containing a Handle
                    || matches!(type_info,
                        bevy::reflect::TypeInfo::Struct(s) if s.iter().any(|f| f.type_path().contains("Handle<"))
                    );

                bevy::log::debug!(
                    "[HANDLE_DETECT] Type: {}, might_be_handle: {}",
                    type_path,
                    might_be_handle
                );

                if might_be_handle {
                    if let Some(asset_reg) = asset_registry {
                        let id = *n as u32;
                        if let Some(handle) = asset_reg.get_untyped_handle(id) {
                            bevy::log::debug!(
                                "[HANDLE_LOOKUP] Resolved handle ID {} -> {:?} for type {}",
                                id,
                                handle,
                                type_path
                            );

                            // Try to wrap in newtype using reflection
                            if let Some(wrapped) = asset_reg.try_wrap_in_newtype_with_reflection(
                                type_path,
                                Box::new(handle.clone()),
                                registry,
                            ) {
                                bevy::log::debug!(
                                    "[HANDLE_LOOKUP] Successfully wrapped in newtype {}",
                                    type_path
                                );
                                return Ok(wrapped);
                            }

                            // If not a newtype, return the handle directly
                            return Ok(Box::new(handle) as Box<dyn bevy::reflect::PartialReflect>);
                        }
                    }
                }
            }

            // Check type info to determine if it's f32 or f64
            if let Some(TypeInfo::Opaque(info)) = type_info {
                let type_path = info.type_path();
                if type_path.contains("f32") {
                    return Ok(Box::new(*n as f32));
                }
            }
            Ok(Box::new(*n))
        }
        LuaValue::Integer(i) => {
            // Check if type might expect a Handle or newtype wrapper - purely via reflection
            if let Some(type_info) = type_info {
                let type_path = type_info.ty().path();

                // Only attempt handle lookup if type looks like it could be a Handle or newtype
                // Detected via reflection: Handle types, single-field newtypes, or structs containing Handle fields
                let might_be_handle = type_path.contains("Handle<") 
                    || matches!(type_info, 
                        bevy::reflect::TypeInfo::TupleStruct(ts) if ts.field_len() == 1
                    )
                    || matches!(type_info,
                        bevy::reflect::TypeInfo::Struct(s) if s.field_len() == 1
                    )
                    // Check if struct has any field containing a Handle
                    || matches!(type_info,
                        bevy::reflect::TypeInfo::Struct(s) if s.iter().any(|f| f.type_path().contains("Handle<"))
                    );

                if might_be_handle {
                    if let Some(asset_reg) = asset_registry {
                        let id = *i as u32;
                        if let Some(handle) = asset_reg.get_untyped_handle(id) {
                            bevy::log::debug!(
                                "[HANDLE_LOOKUP] Resolved handle ID {} -> {:?} for type {}",
                                id,
                                handle,
                                type_path
                            );

                            // Try to wrap in newtype using reflection
                            if let Some(wrapped) = asset_reg.try_wrap_in_newtype_with_reflection(
                                type_path,
                                Box::new(handle.clone()),
                                registry,
                            ) {
                                bevy::log::debug!(
                                    "[HANDLE_LOOKUP] Successfully wrapped in newtype {}",
                                    type_path
                                );
                                return Ok(wrapped);
                            }

                            // If not a newtype, return the handle directly
                            return Ok(Box::new(handle) as Box<dyn bevy::reflect::PartialReflect>);
                        }
                    }
                }
            }

            // Default to i64, but check for other int types
            if let Some(TypeInfo::Opaque(info)) = type_info {
                let type_path = info.type_path();
                if type_path.contains("i32") {
                    return Ok(Box::new(*i as i32));
                } else if type_path.contains("u32") {
                    return Ok(Box::new(*i as u32));
                } else if type_path.contains("usize") {
                    return Ok(Box::new(*i as usize));
                } else if type_path == "uuid::Uuid" {
                    // Construct Uuid from integer (used for PointerId::Custom)
                    let uuid = uuid::Uuid::from_u128(*i as u128);
                    bevy::log::debug!("[UUID_HANDLE] Constructed Uuid from integer: {:?}", uuid);
                    return Ok(Box::new(uuid));
                }
            }
            Ok(Box::new(*i))
        }
        LuaValue::Boolean(b) => Ok(Box::new(*b)),
        LuaValue::String(s) => {
            let str_val = s
                .to_str()
                .map_err(|e| LuaError::RuntimeError(format!("Invalid string: {:?}", e)))?;

            // Check if the expected type is an enum - if so, treat string as unit variant name
            if let Some(type_info) = type_info {
                if let TypeInfo::Enum(enum_info) = type_info {
                    // Look up the variant by name using iter() - convert to owned String first
                    let str_val_owned = str_val.to_string();
                    let variant_opt = enum_info
                        .iter()
                        .find(|v| v.name() == str_val_owned.as_str());
                    if let Some(variant_info) = variant_opt {
                        if matches!(variant_info, bevy::reflect::VariantInfo::Unit(_)) {
                            // Create a DynamicEnum with unit variant
                            let mut dyn_enum = bevy::reflect::DynamicEnum::new(
                                str_val.to_string(),
                                bevy::reflect::DynamicVariant::Unit,
                            );
                            // Set represented type so FromReflect works
                            let reg = registry.read();
                            if let Some(registration) = reg.get(enum_info.ty().id()) {
                                dyn_enum.set_represented_type(Some(registration.type_info()));
                            }
                            drop(reg);
                            bevy::log::debug!(
                                "[UNIT_ENUM] Constructed unit enum variant '{}' for type {}",
                                str_val,
                                enum_info.type_path()
                            );
                            return Ok(Box::new(dyn_enum) as Box<dyn bevy::reflect::PartialReflect>);
                        }
                    }
                }
            }

            Ok(Box::new(str_val.to_string()))
        }
        LuaValue::Table(table) => {
            // Check if this is a known type like Vec3 or Dir3
            if let Some(type_info) = type_info {
                let type_path = type_info.ty().path();
                bevy::log::debug!("[TABLE_TYPE] Table value expected type: {}", type_path);

                // Handle Vec3 (short type path in Bevy 0.17)
                if type_path == "glam::Vec3" {
                    let x: f32 = table.get("x").unwrap_or(0.0);
                    let y: f32 = table.get("y").unwrap_or(0.0);
                    let z: f32 = table.get("z").unwrap_or(0.0);
                    return Ok(Box::new(bevy::math::Vec3::new(x, y, z)));
                }

                // Handle Vec2 (short type path in Bevy 0.17)
                if type_path == "glam::Vec2" {
                    let x: f32 = table.get("x").unwrap_or(0.0);
                    let y: f32 = table.get("y").unwrap_or(0.0);
                    bevy::log::debug!("[VEC2_HANDLE] Successfully constructed Vec2({}, {})", x, y);
                    return Ok(Box::new(bevy::math::Vec2::new(x, y)));
                }

                // Handle Dir3 (validated unit direction)
                if type_path == "bevy_math::direction::Dir3" {
                    let x: f32 = table.get("x").unwrap_or(0.0);
                    let y: f32 = table.get("y").unwrap_or(0.0);
                    let z: f32 = table.get("z").unwrap_or(-1.0);
                    let vec = bevy::math::Vec3::new(x, y, z);
                    // Use Dir3::new which normalizes and validates
                    let dir = bevy::math::Dir3::new(vec).unwrap_or(bevy::math::Dir3::NEG_Z);
                    return Ok(Box::new(dir));
                }

                // Handle enums - detect table like { Custom = value } as enum variant
                if let TypeInfo::Enum(enum_info) = type_info {
                    use bevy::reflect::{DynamicEnum, DynamicStruct, DynamicTuple, DynamicVariant};

                    bevy::log::debug!(
                        "[ENUM_REFLECT] Detected enum type: {}",
                        enum_info.ty().path()
                    );

                    // Check each key in the Lua table to find the variant name
                    for pair in table.clone().pairs::<String, LuaValue>() {
                        if let Ok((key, value)) = pair {
                            bevy::log::debug!(
                                "[ENUM_REFLECT] Checking variant '{}' in {}",
                                key,
                                enum_info.ty().path()
                            );
                            // Try to find this variant in the enum
                            if let Some(variant_info) = enum_info.variant(&key) {
                                let variant_name = key.clone();
                                bevy::log::debug!(
                                    "[ENUM_REFLECT] Matched variant '{}' in {}",
                                    variant_name,
                                    enum_info.ty().path()
                                );

                                // Build the variant data based on variant type
                                let dynamic_variant = match variant_info {
                                    bevy::reflect::VariantInfo::Unit(_) => DynamicVariant::Unit,
                                    bevy::reflect::VariantInfo::Tuple(tuple_info) => {
                                        let mut dyn_tuple = DynamicTuple::default();
                                        // For single-field tuple variants, the value is the tuple field
                                        if tuple_info.field_len() == 1 {
                                            let field_info = tuple_info.field_at(0).unwrap();

                                            // Get type_info - fallback to TypeRegistry lookup if field doesn't have it
                                            let field_type_info =
                                                field_info.type_info().or_else(|| {
                                                    let reg = registry.read();
                                                    reg.get(field_info.type_id())
                                                        .map(|r| r.type_info())
                                                });

                                            bevy::log::debug!(
                                                "[ENUM_TUPLE] Field type_path: {}, type_info: {:?}",
                                                field_info.type_path(),
                                                field_type_info.map(|t| t.ty().path())
                                            );
                                            let field_value = lua_value_to_box_reflect_with_assets(
                                                lua,
                                                &value,
                                                field_type_info,
                                                registry,
                                                asset_registry,
                                            )?;
                                            dyn_tuple.insert_boxed(field_value);
                                        }
                                        DynamicVariant::Tuple(dyn_tuple)
                                    }
                                    bevy::reflect::VariantInfo::Struct(struct_info) => {
                                        let mut dyn_struct = DynamicStruct::default();
                                        // If value is a table, populate struct fields
                                        if let LuaValue::Table(variant_table) = &value {
                                            for i in 0..struct_info.field_len() {
                                                let field = struct_info.field_at(i).unwrap();
                                                let field_name = field.name();
                                                if let Ok(field_val) =
                                                    variant_table.get::<LuaValue>(field_name)
                                                {
                                                    // Get type_info - fallback to TypeRegistry lookup if field doesn't have it
                                                    let field_type_info =
                                                        field.type_info().or_else(|| {
                                                            let reg = registry.read();
                                                            reg.get(field.type_id())
                                                                .map(|r| r.type_info())
                                                        });
                                                    let boxed =
                                                        lua_value_to_box_reflect_with_assets(
                                                            lua,
                                                            &field_val,
                                                            field_type_info,
                                                            registry,
                                                            asset_registry,
                                                        )?;
                                                    dyn_struct.insert_boxed(field_name, boxed);
                                                }
                                            }
                                        }
                                        DynamicVariant::Struct(dyn_struct)
                                    }
                                };

                                let mut dyn_enum = DynamicEnum::new(variant_name, dynamic_variant);
                                // Set the represented type so FromReflect knows what to construct
                                // Get from registry for 'static lifetime reference
                                let reg = registry.read();
                                if let Some(registration) = reg.get(enum_info.ty().id()) {
                                    dyn_enum.set_represented_type(Some(registration.type_info()));
                                }
                                drop(reg);
                                return Ok(Box::new(dyn_enum));
                            }
                        }
                    }

                    // If no variant found in the table, maybe it's just a string for unit variants
                    // Return a default empty enum
                    bevy::log::debug!("[ENUM_REFLECT] No matching variant found in table for enum");
                }

                // Recursively build nested struct
                if let TypeInfo::Struct(struct_info) = type_info {
                    let nested = lua_table_to_dynamic_with_assets(
                        lua,
                        table,
                        type_info,
                        registry,
                        asset_registry,
                    )?;
                    return Ok(Box::new(nested));
                }
            }

            // Fallback: create empty DynamicStruct
            Ok(Box::new(DynamicStruct::default()))
        }
        _ => {
            // Return nil-like value
            Ok(Box::new(()))
        }
    }
}

/// Convert a Lua value to a reflected value (reverse of reflection_to_lua)
/// This is a helper function for event writing via reflection
/// Used by world:write_event() to convert Lua tables to Bevy event fields
pub fn lua_to_reflection(
    _lua: &Lua,
    lua_value: &LuaValue,
    field: &mut dyn bevy::reflect::PartialReflect,
    _registry: &AppTypeRegistry,
) -> LuaResult<()> {
    match lua_value {
        LuaValue::Number(n) => {
            if let Some(f32_field) = field.try_downcast_mut::<f32>() {
                *f32_field = *n as f32;
            } else if let Some(f64_field) = field.try_downcast_mut::<f64>() {
                *f64_field = *n;
            }
        }
        LuaValue::Integer(i) => {
            if let Some(i32_field) = field.try_downcast_mut::<i32>() {
                *i32_field = *i as i32;
            } else if let Some(i64_field) = field.try_downcast_mut::<i64>() {
                *i64_field = *i;
            } else if let Some(u32_field) = field.try_downcast_mut::<u32>() {
                *u32_field = *i as u32;
            } else if let Some(usize_field) = field.try_downcast_mut::<usize>() {
                *usize_field = *i as usize;
            }
        }
        LuaValue::Boolean(b) => {
            if let Some(bool_field) = field.try_downcast_mut::<bool>() {
                *bool_field = *b;
            }
        }
        LuaValue::String(s) => {
            if let Ok(str_val) = s.to_str() {
                if let Some(string_field) = field.try_downcast_mut::<String>() {
                    *string_field = str_val.to_string();
                }
            }
        }
        LuaValue::Table(table) => {
            // Handle nested structs
            if let bevy::reflect::ReflectMut::Struct(struct_mut) = field.reflect_mut() {
                for i in 0..struct_mut.field_len() {
                    if let Some(field_name) = struct_mut.name_at(i) {
                        if let Ok(nested_lua_val) = table.get::<LuaValue>(field_name) {
                            if let Some(nested_field) = struct_mut.field_at_mut(i) {
                                let _ = lua_to_reflection(
                                    _lua,
                                    &nested_lua_val,
                                    nested_field,
                                    _registry,
                                );
                            }
                        }
                    }
                }
            }
            // Handle Vec2, Vec3 specially
            if let Some(vec2_field) = field.try_downcast_mut::<bevy::math::Vec2>() {
                let x: f32 = table.get("x").unwrap_or(0.0);
                let y: f32 = table.get("y").unwrap_or(0.0);
                *vec2_field = bevy::math::Vec2::new(x, y);
            } else if let Some(vec3_field) = field.try_downcast_mut::<bevy::math::Vec3>() {
                let x: f32 = table.get("x").unwrap_or(0.0);
                let y: f32 = table.get("y").unwrap_or(0.0);
                let z: f32 = table.get("z").unwrap_or(0.0);
                *vec3_field = bevy::math::Vec3::new(x, y, z);
            }
        }
        _ => {}
    }

    Ok(())
}
