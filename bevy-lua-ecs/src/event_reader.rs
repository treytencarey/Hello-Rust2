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
            // For opaque values, we can't easily downcast since PartialReflect doesn't have downcast_ref
            // Instead, we'll try to use the debug representation or return a simple string
            // This is a limitation of the generic event reading approach
            
            // Try to get a string representation
            let debug_str = format!("{:?}", v);
            Ok(LuaValue::String(lua.create_string(&debug_str)?))
        }
        ReflectRef::Set(_) => {
            // Sets are not commonly used in events, return empty table
            Ok(LuaValue::Table(lua.create_table()?))
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
                                let _ = lua_to_reflection(_lua, &nested_lua_val, nested_field, _registry);
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
