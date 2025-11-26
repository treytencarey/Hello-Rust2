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
            for i in 0..m.len() {
                let key = m.get_at(i).unwrap().0;
                let value = m.get_at(i).unwrap().1;
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
