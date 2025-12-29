use crate::component_update_queue::ComponentUpdateQueue;
use crate::components::LuaCustomComponents;
use crate::lua_integration::LuaScriptContext;
use bevy::prelude::*;
use bevy::reflect::ReflectFromPtr;
use mlua::prelude::*;
use std::sync::Arc;

/// System that processes the component update queue
/// Uses reflection for updating ANY Bevy component (Text2d, Transform, etc.)
pub fn process_component_updates(
    world: &mut World,
) {
    use std::time::Instant;
    
    // Collect requests from the queue
    let requests = {
        let queue = world.resource::<ComponentUpdateQueue>();
        queue.drain()
    };

    if requests.is_empty() {
        return;
    }
    
    let batch_start = Instant::now();
    let request_count = requests.len();
    
    // Get resources we need BEFORE any mutable entity access
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let asset_registry = world.get_resource::<crate::asset_loading::AssetRegistry>().cloned();
    let spawn_queue = world.resource::<crate::spawn_queue::SpawnQueue>().clone();
    
    for request in requests {
        let type_path = request.component_name.clone();
        debug!("[COMPONENT_UPDATE] Processing request for component '{}' on entity {:?} (bits={})", type_path, request.entity, request.entity.to_bits());
        
        // Get Lua data before we start mutable access
        let data_value: LuaValue = {
            let lua_ctx = world.resource::<LuaScriptContext>();
            match lua_ctx.lua.registry_value(&request.data) {
                Ok(value) => value,
                Err(e) => {
                    error!("Failed to retrieve Lua value for {}: {}", type_path, e);
                    continue;
                }
            }
        };
        
        // Pre-resolve entity references in the data (handles temp_id -> Entity conversion)
        // This is critical for components like UiTargetCamera that reference entities from spawn()
        let resolved_data = {
            let lua_ctx = world.resource::<LuaScriptContext>();
            resolve_entity_references_in_table(&lua_ctx.lua, &data_value, &spawn_queue)
        };
        
        // Check entity exists
        if world.get_entity(request.entity).is_err() {
            let lua_ctx = world.resource::<LuaScriptContext>();
            let _ = lua_ctx.lua.remove_registry_value(request.data);
            continue;
        }
        
        // Try reflection-based update using in-place mutation
        // This is the generic path that works for all reflected components like Text2d
        let registry = type_registry.read();
        let registration = registry.get_with_type_path(&type_path)
            .or_else(|| registry.get_with_short_type_path(&type_path));
        
        debug!("[COMPONENT_UPDATE] Looking up type: '{}' -> found: {}", type_path, registration.is_some());
        
        let mut component_updated = false;
        
        if let Some(registration) = registration {
            if let (Some(_reflect_component), Some(reflect_from_ptr)) = (
                registration.data::<ReflectComponent>(),
                registration.data::<ReflectFromPtr>().cloned(),
            ) {
                let component_id = world.components().get_id(registration.type_id());
                
                if let Some(comp_id) = component_id {
                    // Release read lock before mutable access
                    drop(registry);
                    
                    // Now do mutable entity access
                    if let Ok(mut entity_mut) = world.get_entity_mut(request.entity) {
                        if let Ok(mut component_ptr) = entity_mut.get_mut_by_id(comp_id) {
                            // SAFETY: We have the correct TypeId and exclusive access via EntityMut
                            let component_mut = unsafe { 
                                reflect_from_ptr.as_reflect_mut(component_ptr.as_mut()) 
                            };
                            
                            // Update fields from Lua table (using resolved_data with temp_ids converted)
                            if let LuaValue::Table(ref table) = resolved_data {
                                if let Err(e) = update_component_from_lua(
                                    component_mut.as_partial_reflect_mut(),
                                    table,
                                    asset_registry.as_ref(),
                                    &type_registry,
                                ) {
                                    error!("Failed to update component {} from Lua: {}", type_path, e);
                                } else {
                                    debug!("[COMPONENT_UPDATE] Updated {} via reflection", type_path);
                                    component_updated = true;
                                }
                            }
                        } else {
                            debug!("[COMPONENT_UPDATE] Component {} not found on entity", type_path);
                        }
                    }
                } else {
                    drop(registry);
                }
            } else {
                drop(registry);
            }
        } else {
            drop(registry);
        }
        
        if component_updated {
            let lua_ctx = world.resource::<LuaScriptContext>();
            let _ = lua_ctx.lua.remove_registry_value(request.data);
            continue;
        }
        
        // Fallback: It's a generic Lua component - store in LuaCustomComponents
        if let Ok(mut entity_mut) = world.get_entity_mut(request.entity) {
            if let Some(mut lua_components) = entity_mut.get_mut::<LuaCustomComponents>() {
                lua_components.components.insert(request.component_name.clone(), Arc::new(request.data));
            } else {
                let mut lua_components = LuaCustomComponents::default();
                lua_components.components.insert(request.component_name.clone(), Arc::new(request.data));
                entity_mut.insert(lua_components);
            }
        } else {
            warn!("Entity {:?} not found for component update", request.entity);
            let lua_ctx = world.resource::<LuaScriptContext>();
            let _ = lua_ctx.lua.remove_registry_value(request.data);
        }
    }
    
    let batch_time = batch_start.elapsed();
    if batch_time.as_millis() >= 1 {
        debug!("[COMPONENT_UPDATE] Processed {} updates in {:?}", request_count, batch_time);
    }
}

/// Update a component's fields from a Lua table using reflection
fn update_component_from_lua(
    component: &mut dyn bevy::reflect::PartialReflect,
    table: &LuaTable,
    asset_registry: Option<&crate::asset_loading::AssetRegistry>,
    type_registry: &AppTypeRegistry,
) -> LuaResult<()> {
    use bevy::reflect::ReflectMut;
    
    match component.reflect_mut() {
        ReflectMut::Struct(struct_mut) => {
            // Update each field from the table
            for pair in table.pairs::<String, LuaValue>() {
                let (key, value) = pair?;
                debug!("[COMPONENT_UPDATE] Updating struct field '{}' with value type: {:?}", key, std::mem::discriminant(&value));
                if let Some(field) = struct_mut.field_mut(&key) {
                    crate::components::set_field_from_lua(field, &value, asset_registry, type_registry, Some(&key))?;
                }
            }
        }
        ReflectMut::TupleStruct(tuple_mut) => {
            // For tuple structs like Text2d(String) or BackgroundColor(Color)
            // Check for explicit field names like _0, _1, or aliases like "text"
            let mut handled = false;
            
            for pair in table.pairs::<String, LuaValue>() {
                let (key, value) = pair?;
                // Handle _0, _1, etc. field names
                if key.starts_with('_') {
                    if let Ok(index) = key[1..].parse::<usize>() {
                        if let Some(field) = tuple_mut.field_mut(index) {
                            crate::components::set_field_from_lua(field, &value, asset_registry, type_registry, Some(&key))?;
                            handled = true;
                        }
                    }
                } else if key == "text" && tuple_mut.field_len() == 1 {
                    // Special case: "text" alias for single-field tuple struct like Text2d
                    if let Some(field) = tuple_mut.field_mut(0) {
                        crate::components::set_field_from_lua(field, &value, asset_registry, type_registry, Some(&key))?;
                        handled = true;
                    }
                }
            }
            
            // Fallback: if single-field tuple struct and only one key in table,
            // use that key's VALUE for field 0 (matches spawn behavior for BackgroundColor = { color = {...} })
            if !handled && tuple_mut.field_len() == 1 {
                let mut pairs: Vec<_> = table.pairs::<String, LuaValue>().filter_map(|r| r.ok()).collect();
                if pairs.len() == 1 {
                    let (key, value) = pairs.remove(0);
                    debug!("[COMPONENT_UPDATE] Using single-key fallback: '{}' value for tuple struct field 0", key);
                    if let Some(field) = tuple_mut.field_mut(0) {
                        crate::components::set_field_from_lua(field, &value, asset_registry, type_registry, Some(&key))?;
                    }
                }
            }
        }
        ReflectMut::Enum(_enum_mut) => {
            // Handle enum updates - for now, support unit variants (like Visibility::Visible)
            // The table should have exactly one key (the component name) with a string value (variant name)
            // OR just a single string value representing the variant name
            
            // First, check if there's a single key in the table that might be the variant value
            let pairs: Vec<_> = table.pairs::<String, LuaValue>().filter_map(|r| r.ok()).collect();
            
            let variant_name = if pairs.len() == 1 {
                // Single key-value pair - the value should be the variant name
                match &pairs[0].1 {
                    LuaValue::String(s) => s.to_str().ok().map(|s| s.to_string()),
                    _ => None,
                }
            } else {
                None
            };
            
            if let Some(variant_name) = variant_name {
                // Create a DynamicEnum with the unit variant
                use bevy::reflect::{DynamicEnum, DynamicVariant};
                
                let type_path = component.reflect_type_path().to_string();
                let registry = type_registry.read();
                
                if let Some(registration) = registry.get_with_type_path(&type_path) {
                    // Check if this variant exists and is a unit variant
                    if let Some(reflect_from_reflect) = registration.data::<bevy::reflect::ReflectFromReflect>() {
                        let dynamic_enum = DynamicEnum::new(&variant_name, DynamicVariant::Unit);
                        
                        if let Some(concrete) = reflect_from_reflect.from_reflect(&dynamic_enum) {
                            component.apply(concrete.as_ref());
                            debug!("[COMPONENT_UPDATE] Applied enum variant '{}' to {}", variant_name, type_path);
                            return Ok(());
                        } else {
                            debug!("[COMPONENT_UPDATE] Failed to create enum {} from variant '{}'", type_path, variant_name);
                        }
                    }
                }
            } else {
                debug!("[COMPONENT_UPDATE] Enum update expected single key with string variant name");
            }
        }
        _ => {
            debug!("Unsupported reflect type for component update");
        }
    }
    
    Ok(())
}

/// Resolve entity references (temp_ids from spawn()) in a Lua table
/// Looks for 'entity' keys and converts temp_ids to real entity bits
fn resolve_entity_references_in_table(
    lua: &mlua::Lua,
    data: &LuaValue,
    spawn_queue: &crate::spawn_queue::SpawnQueue,
) -> LuaValue {
    match data {
        LuaValue::Table(table) => {
            // Create a new table with resolved references
            match lua.create_table() {
                Ok(new_table) => {
                    for pair in table.pairs::<mlua::Value, LuaValue>() {
                        if let Ok((key, value)) = pair {
                            // Check if this is an 'entity' field that needs resolution
                            let resolved_value = if let mlua::Value::String(key_str) = &key {
                                if key_str.to_str().map(|s| s == "entity").unwrap_or(false) {
                                    // This is an entity reference - resolve temp_id
                                    match &value {
                                        LuaValue::Integer(temp_id) => {
                                            let entity = spawn_queue.resolve_entity(*temp_id as u64);
                                            debug!("[ENTITY_RESOLVE] Resolved temp_id {} -> entity {:?} (bits: {})", 
                                                temp_id, entity, entity.to_bits());
                                            LuaValue::Integer(entity.to_bits() as i64)
                                        }
                                        LuaValue::Number(temp_id) => {
                                            let entity = spawn_queue.resolve_entity(*temp_id as u64);
                                            debug!("[ENTITY_RESOLVE] Resolved temp_id {} -> entity {:?} (bits: {})", 
                                                temp_id, entity, entity.to_bits());
                                            LuaValue::Integer(entity.to_bits() as i64)
                                        }
                                        _ => resolve_entity_references_in_table(lua, &value, spawn_queue),
                                    }
                                } else {
                                    // Recursively resolve nested tables
                                    resolve_entity_references_in_table(lua, &value, spawn_queue)
                                }
                            } else {
                                // Non-string key, recursively resolve
                                resolve_entity_references_in_table(lua, &value, spawn_queue)
                            };
                            let _ = new_table.set(key, resolved_value);
                        }
                    }
                    LuaValue::Table(new_table)
                }
                Err(_) => data.clone(),
            }
        }
        _ => data.clone(),
    }
}
