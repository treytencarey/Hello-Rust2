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
    pub update_systems: Arc<Mutex<Vec<Arc<LuaRegistryKey>>>>,
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

/// System that runs registered Lua update systems with full ECS query API
pub fn run_lua_systems(
    world: &mut World,
) {
    // Get resources we need
    let lua_ctx = world.resource::<LuaScriptContext>().clone();
    let registry = world.resource::<LuaSystemRegistry>().clone();
    let component_registry = world.resource::<ComponentRegistry>();
    let update_queue = world.resource::<ComponentUpdateQueue>().clone();
    
    // Get change detection ticks
    let this_run = world.read_change_tick().get();
    let mut last_run = registry.last_run.lock().unwrap();
    let last_run_tick = *last_run;
    *last_run = this_run;
    drop(last_run);
    
    // Run each registered Lua system
    let systems = registry.update_systems.lock().unwrap().clone();
    for system_key in systems.iter() {
        if let Err(e) = run_single_lua_system(
            &lua_ctx.lua,
            system_key,
            world,
            &component_registry,
            &update_queue,
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
    last_run: u32,
    this_run: u32,
) -> LuaResult<()> {
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
        // with_components: table of component names to filter by
        // changed_components: optional table of component names that must have changed
        world_table.set("query", scope.create_function(move |lua_ctx, (_self, with_comps, changed_comps): (LuaTable, LuaTable, Option<LuaTable>)| {
            // Build query from parameters
            let mut builder = LuaQueryBuilder::new();
            
            // Add with components - use sequence_values for array iteration
            for comp_name in with_comps.sequence_values::<String>() {
                let name = comp_name?;
                builder.with_components.push(name);
            }
            
            // Add changed components if provided
            if let Some(changed_table) = changed_comps {
                for comp_name in changed_table.sequence_values::<String>() {
                    builder.changed_components.push(comp_name?);
                }
            }
            
            // Execute query immediately
            let results = execute_query(lua_ctx, world, &builder, component_registry, update_queue, last_run, this_run)?;
            
            // Return results as table
            let results_table = lua_ctx.create_table()?;
            for (i, entity) in results.into_iter().enumerate() {
                results_table.set(i + 1, entity)?;
            }
            
            Ok(results_table)
        })?)?;
        
        // Call the Lua system function
        func.call::<()>(world_table)?;
        
        Ok(())
    })
}
