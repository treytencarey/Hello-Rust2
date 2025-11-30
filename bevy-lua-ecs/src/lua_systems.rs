use bevy::prelude::*;
use mlua::prelude::*;
use crate::lua_integration::LuaScriptContext;
use crate::components::ComponentRegistry;
use crate::component_update_queue::ComponentUpdateQueue;
use crate::lua_world_api::{LuaQueryBuilder, execute_query};
use std::sync::{Arc, Mutex};

/// Resource that stores registered Lua systems
#[derive(Resource, Clone)]
pub struct LuaSystemRegistry {
    pub update_systems: Arc<Mutex<Vec<(u64, Arc<LuaRegistryKey>)>>>, // (instance_id, system)
    pub last_run: Arc<Mutex<u32>>,
}

impl Default for LuaSystemRegistry {
    fn default() -> Self {
        Self {
            update_systems: Arc::new(Mutex::new(Vec::new())),
            last_run: Arc::new(Mutex::new(0)),
        }
    }
}

impl LuaSystemRegistry {
    /// Clear all systems registered by a specific script instance
    pub fn clear_instance_systems(&self, instance_id: u64) {
        let mut systems = self.update_systems.lock().unwrap();
        let initial_count = systems.len();
        
        systems.retain(|(id, _key)| {
            *id != instance_id
        });
        
        let removed_count = initial_count - systems.len();
        if removed_count > 0 {
            info!("Cleared {} systems from instance {}", removed_count, instance_id);
        }
    }
}

/// System that runs registered Lua update systems with full ECS query API
pub fn run_lua_systems(
    world: &mut World,
) {
    // Get resources we need
    let lua_ctx = world.resource::<LuaScriptContext>().clone();
    let registry = world.resource::<LuaSystemRegistry>().clone();
    let component_registry = world.resource::<ComponentRegistry>();
    let update_queue = world.resource::<ComponentUpdateQueue>().clone();
    let serde_registry = world.resource::<crate::serde_components::SerdeComponentRegistry>().clone();
    let builder_registry = world.resource::<crate::resource_builder::ResourceBuilderRegistry>().clone();
    
    // Get change detection ticks
    let this_run = world.read_change_tick().get();
    let mut last_run = registry.last_run.lock().unwrap();
    let last_run_tick = *last_run;
    *last_run = this_run;
    drop(last_run);
    
    // Run each registered Lua system
    let systems = registry.update_systems.lock().unwrap().clone();
    for (_instance_id, system_key) in systems.iter() {
        if let Err(e) = run_single_lua_system(
            &lua_ctx.lua,
            system_key,
            world,
            &component_registry,
            &update_queue,
            &serde_registry,
            &builder_registry,
            last_run_tick,
            this_run,
        ) {
            error!("Error running Lua system: {}", e);
        }
    }
}

fn run_single_lua_system(
    lua: &Lua,
    system_key: &LuaRegistryKey,
    world: &World,
    component_registry: &ComponentRegistry,
    update_queue: &ComponentUpdateQueue,
    serde_registry: &crate::serde_components::SerdeComponentRegistry,
    _builder_registry: &crate::resource_builder::ResourceBuilderRegistry,
    last_run: u32,
    this_run: u32,
) -> LuaResult<()> {
    // Get resource registry
    let resource_registry = world.resource::<crate::resource_lua_trait::LuaResourceRegistry>();
    // Get the Lua function
    let func: LuaFunction = lua.registry_value(system_key)?;
    
    // Use scope to ensure all closures are cleaned up
    lua.scope(|scope| {
        // Create world table
        let world_table = lua.create_table()?;
        
        // delta_time() - returns delta time in seconds
        world_table.set("delta_time", scope.create_function(|_lua_ctx, _self: LuaTable| {
            let time = world.resource::<Time>();
            Ok(time.delta_secs())
        })?)?;
        
        // query(with_components, changed_components) - executes immediately and returns results
        world_table.set("query", scope.create_function(move |lua_ctx, (_self, with_comps, changed_comps): (LuaTable, LuaTable, Option<LuaTable>)| {
            let mut builder = LuaQueryBuilder::new();
            
            for comp_name in with_comps.sequence_values::<String>() {
                let name = comp_name?;
                builder.with_components.push(name);
            }
            
            if let Some(changed_table) = changed_comps {
                for comp_name in changed_table.sequence_values::<String>() {
                    builder.changed_components.push(comp_name?);
                }
            }
            
            let results = execute_query(lua_ctx, world, &builder, component_registry, update_queue, last_run, this_run)?;
            
            let results_table = lua_ctx.create_table()?;
            for (i, entity) in results.into_iter().enumerate() {
                results_table.set(i + 1, entity)?;
            }
            
            Ok(results_table)
        })?)?;
        
        // query_resource(resource_name) - check if a resource exists
        world_table.set("query_resource", scope.create_function({
            let serde_registry = serde_registry.clone();
            move |_lua_ctx, (_self, resource_name): (LuaTable, String)| {
                Ok(serde_registry.has_resource(&resource_name))
            }
        })?)?;
        
        // call_resource_method(resource_name, method_name, ...args) - call a registered method on a resource
        world_table.set("call_resource_method", scope.create_function({
            let resource_registry = resource_registry.clone();
            move |lua_ctx, (_, resource_name, method_name, args): (LuaTable, String, String, mlua::MultiValue)| {
                resource_registry.call_method(lua_ctx, world, &resource_name, &method_name, args)
            }
        })?)?;
        
        // Helper function to cleanup a script instance (shared by reload and stop)
        let cleanup_script_instance = |lua_ctx: &Lua, instance_id: u64, script_name: &str| -> Result<(), LuaError> {
            let script_registry = world.resource::<crate::script_registry::ScriptRegistry>().clone();
            let system_registry = world.resource::<LuaSystemRegistry>().clone();
            let resource_queue = world.resource::<crate::resource_queue::ResourceQueue>().clone();
            let serde_registry = world.resource::<crate::serde_components::SerdeComponentRegistry>().clone();
            let builder_registry = world.resource::<crate::resource_builder::ResourceBuilderRegistry>().clone();
            
            // Clear all systems registered by this instance
            system_registry.clear_instance_systems(instance_id);
            
            // Get list of resources inserted by this instance
            let resources_to_clear = resource_queue.get_instance_resources(instance_id);
            
            if !resources_to_clear.is_empty() {
                // SAFETY: We need mutable access to remove resources
                #[allow(invalid_reference_casting)]
                let world_mut = unsafe { &mut *(world as *const World as *mut World) };
                
                for resource_name in &resources_to_clear {
                    if !builder_registry.try_remove(resource_name, world_mut) {
                        serde_registry.try_remove_resource(resource_name, world_mut);
                    }
                }
            }
            
            // Clear resource tracking for this instance
            resource_queue.clear_instance_tracking(instance_id);
            
            // SAFETY: We need mutable access to despawn entities
            #[allow(invalid_reference_casting)]
            let world_mut = unsafe { &mut *(world as *const World as *mut World) };
            
            // Get list of entities to be despawned BEFORE despawning
            let entities_to_despawn: Vec<Entity> = {
                let mut query_state = world_mut.query::<(Entity, &crate::script_entities::ScriptOwned)>();
                query_state.iter(world_mut)
                    .filter(|(_, script_owned)| script_owned.instance_id == instance_id)
                    .map(|(entity, _)| entity)
                    .collect()
            };
            
            // Clear pending component updates for these entities
            if !entities_to_despawn.is_empty() {
                let component_update_queue = world.resource::<crate::component_update_queue::ComponentUpdateQueue>().clone();
                let cleared_keys = component_update_queue.clear_for_entities(&entities_to_despawn);
                
                // Clean up the Lua registry keys
                for key in cleared_keys {
                    let _ = lua_ctx.remove_registry_value(key);
                }
            }
            
            // Despawn all entities from this instance
            crate::script_entities::despawn_instance_entities(world_mut, instance_id);
            
            // Remove old instance from registry
            script_registry.remove_instance(instance_id);
            
            Ok(())
        };
        
        // reload_current_script() - cleanup and reload the current script instance
        world_table.set("reload_current_script", scope.create_function({
            let script_registry = world.resource::<crate::script_registry::ScriptRegistry>().clone();
            let cleanup = cleanup_script_instance.clone();
            move |lua_ctx, _self: LuaTable| {
                // Get the instance ID from Lua global (set by execute_script_tracked)
                let instance_id: u64 = lua_ctx.globals().get("__INSTANCE_ID__")?;
                let script_name: String = lua_ctx.globals().get("__SCRIPT_NAME__")?;
                
                info!("Manual reload requested for script instance {} ('{}')", instance_id, script_name);
                
                // Get script path and content from registry
                let script_path = script_registry.get_instance_path(instance_id)
                    .ok_or_else(|| LuaError::RuntimeError(format!("Script instance {} not found in registry", instance_id)))?;
                
                let script_content = script_registry.get_instance_content(instance_id)
                    .ok_or_else(|| LuaError::RuntimeError(format!("Script content for instance {} not found", instance_id)))?;
                
                // Use shared cleanup logic
                cleanup(lua_ctx, instance_id, &script_name)?;
                
                // Get Lua context and re-execute the script
                let lua_ctx_res = world.resource::<crate::lua_integration::LuaScriptContext>().clone();
                let script_instance = world.resource::<crate::script_entities::ScriptInstance>().clone();
                
                // Re-execute the script
                match lua_ctx_res.execute_script_tracked(&script_content, &script_name, &script_instance) {
                    Ok(new_instance_id) => {
                        info!("✓ Script reloaded: instance {} -> {}", instance_id, new_instance_id);
                        
                        // Register the new instance
                        script_registry.register_script(script_path, new_instance_id, script_content);
                        
                        Ok(())
                    }
                    Err(e) => {
                        Err(LuaError::RuntimeError(format!("Failed to reload script: {}", e)))
                    }
                }
            }
        })?)?;
        
        // stop_current_script() - cleanup and stop the current script (no reload)
        world_table.set("stop_current_script", scope.create_function({
            let script_registry = world.resource::<crate::script_registry::ScriptRegistry>().clone();
            let cleanup = cleanup_script_instance.clone();
            move |lua_ctx, _self: LuaTable| {
                // Get the instance ID from Lua global (set by execute_script_tracked)
                let instance_id: u64 = lua_ctx.globals().get("__INSTANCE_ID__")?;
                let script_name: String = lua_ctx.globals().get("__SCRIPT_NAME__")?;
                
                info!("Stop requested for script instance {} ('{}')", instance_id, script_name);
                
                // Mark as stopped in registry (prevents auto-reload)
                script_registry.mark_stopped(instance_id);
                
                // Use shared cleanup logic
                cleanup(lua_ctx, instance_id, &script_name)?;
                
                info!("✓ Script instance {} stopped", instance_id);
                Ok(())
            }
        })?)?;
        
        // Legacy reload_script() - kept for backwards compatibility, calls reload_current_script()
        world_table.set("reload_script", scope.create_function({
            move |_lua_ctx, self_table: LuaTable| {
                // Just delegate to reload_current_script
                let reload_fn: LuaFunction = self_table.get("reload_current_script")?;
                reload_fn.call::<()>(self_table)
            }
        })?)?;
        
        // despawn_all(tag_name) - despawn all entities with a specific tag component
        world_table.set("despawn_all", scope.create_function({
            move |_lua_ctx, (_self, tag_name): (LuaTable, String)| {
                let despawn_queue = world.resource::<crate::despawn_queue::DespawnQueue>().clone();

                // SAFETY: We need to access the world mutably for querying.
                // This is safe because we're in an exclusive system context.
                #[allow(invalid_reference_casting)]
                let world_cell = unsafe {
                    let world_mut = &mut *(world as *const World as *mut World);
                    world_mut.as_unsafe_world_cell()
                };

                let mut entities_to_despawn = Vec::new();
                
                // Create a query state
                let mut query_state = unsafe {
                    world_cell.world_mut().query::<(Entity, &crate::components::LuaCustomComponents)>()
                };
                
                for (entity, lua_comps) in unsafe { query_state.iter(world_cell.world()) } {
                    if lua_comps.components.contains_key(&tag_name) {
                        entities_to_despawn.push(entity);
                    }
                }

                // Queue all matching entities for despawn
                for entity in entities_to_despawn {
                    despawn_queue.queue_despawn(entity);
                }

                Ok(())
            }
        })?)?;

        // read_events(event_type_name) - read any Bevy event via reflection
        world_table.set("read_events", scope.create_function({
            move |lua_ctx, (_self, event_type_name): (LuaTable, String)| {
                let type_registry = component_registry.type_registry();
                let registry = type_registry.read();
                
                // Look up the event type
                let _type_registration = registry.get_with_type_path(&event_type_name)
                    .ok_or_else(|| LuaError::RuntimeError(format!("Event type '{}' not found", event_type_name)))?;
                
                // Construct Events<T> type path
                let events_type_path = format!("bevy_ecs::event::collections::Events<{}>", event_type_name);
                
                // Look up Events<T>
                let events_registration = registry.get_with_type_path(&events_type_path)
                    .ok_or_else(|| LuaError::RuntimeError(format!("Events<{}> not found. Make sure it's registered.", event_type_name)))?;
                
                // Get ReflectResource
                let reflect_resource = events_registration.data::<bevy::ecs::reflect::ReflectResource>()
                    .ok_or_else(|| LuaError::RuntimeError(format!("Events<{}> doesn't have ReflectResource", event_type_name)))?;
                
                // Access Events<T> resource using unsafe cast
                #[allow(invalid_reference_casting)]
                let events_resource = unsafe {
                    let world_mut = &mut *(world as *const bevy::ecs::world::World as *mut bevy::ecs::world::World);
                    let world_cell = world_mut.as_unsafe_world_cell();
                    reflect_resource.reflect_unchecked_mut(world_cell)
                }.ok_or_else(|| LuaError::RuntimeError(format!("Events<{}> resource not found", event_type_name)))?;
                
                // Read events from the Events<T> struct
                let results = lua_ctx.create_table()?;
                let mut index = 1;
                
                if let bevy::reflect::ReflectRef::Struct(events_struct) = events_resource.reflect_ref() {
                    // Events<T> has events_a and events_b fields which are EventSequence<T>
                    for field_name in ["events_a", "events_b"] {
                        if let Some(field) = events_struct.field(field_name) {
                            // EventSequence is a struct with an "events" field that contains a Vec
                            if let bevy::reflect::ReflectRef::Struct(sequence_struct) = field.reflect_ref() {
                                if let Some(events_field) = sequence_struct.field("events") {
                                    if let bevy::reflect::ReflectRef::List(event_list) = events_field.reflect_ref() {
                                        for i in 0..event_list.len() {
                                            if let Some(event_instance) = event_list.get(i) {
                                                // EventInstance<T> is a struct with 'event' field
                                                if let bevy::reflect::ReflectRef::Struct(instance_struct) = event_instance.reflect_ref() {
                                                    if let Some(event) = instance_struct.field("event") {
                                                        let lua_value = crate::event_reader::reflection_to_lua(lua_ctx, event, &type_registry)?;
                                                        results.set(index, lua_value)?;
                                                        index += 1;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                Ok(results)
            }
        })?)?;

        // Call the Lua system function
        func.call::<()>(world_table)?;
        
        Ok(())
    })
}
