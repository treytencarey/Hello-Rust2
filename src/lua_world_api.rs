use bevy::prelude::*;
use bevy::ecs::reflect::ReflectComponent;
use bevy::reflect::ReflectRef;
use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use crate::component_update_queue::ComponentUpdateQueue;
use crate::components::ComponentRegistry;

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
            if let Some(data) = this.component_data.get(&component_name) {
                Ok(LuaValue::String(lua.create_string(data)?))
            } else {
                Ok(LuaValue::Nil)
            }
        });
        
        methods.add_method("has", |_, this, component_name: String| {
            Ok(this.lua_components.contains_key(&component_name) || 
               this.component_data.contains_key(&component_name))
        });
        
        methods.add_method("id", |_, this, ()| {
            Ok(format!("{:?}", this.entity))
        });
        
        methods.add_method("set", |lua, this, (component_name, component_data): (String, LuaTable)| {
            // Create a registry key for the component data
            let registry_key = lua.create_registry_value(component_data)?;
            
            // Queue the update
            this.update_queue.queue_update(this.entity, component_name, registry_key);
            
            Ok(())
        });
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
    
    // Iterate all entities
    for entity_ref in world.iter_entities() {
        let mut matches = true;
        let mut component_data = HashMap::new();
        let mut lua_components = HashMap::new();
        
        // Check with() filters and collect component data
        for component_name in &query_builder.with_components {
            // 1. Check if it's a known Rust component
            if let Some(type_path) = component_registry.get_type_path(component_name) {
                if let Some(registration) = type_registry.get_with_type_path(&type_path) {
                    if let Some(reflect_component) = registration.data::<ReflectComponent>() {
                        if let Some(component) = reflect_component.reflect(Into::<bevy::ecs::world::FilteredEntityRef>::into(&entity_ref)) {
                            // Smart serialization
                            let value_str = match component.reflect_ref() {
                                ReflectRef::Enum(enum_ref) => enum_ref.variant_name().to_string(),
                                ReflectRef::TupleStruct(tuple) if tuple.field_len() == 1 => {
                                    format!("{:?}", tuple.field(0).unwrap())
                                }
                                _ => format!("{:?}", component),
                            };
                            component_data.insert(component_name.clone(), value_str);
                            continue;
                        }
                    }
                }
                // If known type but not found on entity -> mismatch
                matches = false;
                break;
            } 
            
            // 2. Check if it's a generic Lua component
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
