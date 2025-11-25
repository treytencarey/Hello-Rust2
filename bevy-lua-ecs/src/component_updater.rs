use bevy::prelude::*;
use crate::component_update_queue::ComponentUpdateQueue;
use crate::components::{ComponentRegistry, LuaCustomComponents};
use crate::lua_integration::LuaScriptContext;
use mlua::prelude::*;
use std::sync::Arc;

/// System that processes the component update queue
pub fn process_component_updates(
    mut commands: Commands,
    queue: Res<ComponentUpdateQueue>,
    component_registry: Res<ComponentRegistry>,
    lua_ctx: Res<LuaScriptContext>,
    mut query: Query<(Entity, Option<&mut LuaCustomComponents>)>,
) {
    let requests = queue.drain();
    
    if requests.is_empty() {
        return;
    }
    
    for request in requests {
        // Check if it's a known Rust component
        if let Some(handler) = component_registry.get(&request.component_name) {
            // Check if entity still exists before trying to update it
            // This prevents crashes when entities are despawned (e.g., client disconnect in networking)
            if commands.get_entity(request.entity).is_err() {
                // Entity was despawned, skip this update and clean up
                if let Err(e) = lua_ctx.lua.remove_registry_value(request.data) {
                    warn!("Failed to remove registry value for despawned entity: {}", e);
                }
                continue;
            }
            
            // Retrieve the Lua value from the registry (can be string, table, number, etc.)
            let data_value: LuaValue = match lua_ctx.lua.registry_value(&request.data) {
                Ok(value) => value,
                Err(e) => {
                    error!("Failed to retrieve Lua value for {}: {}", request.component_name, e);
                    continue;
                }
            };
            
            // Get entity commands
            let mut entity_commands = commands.entity(request.entity);
            
            // Apply component update
            if let Err(e) = handler(&data_value, &mut entity_commands) {
                error!("Failed to update component {}: {}", request.component_name, e);
            }
            
            // Remove the registry value to free memory
            if let Err(e) = lua_ctx.lua.remove_registry_value(request.data) {
                warn!("Failed to remove registry value for {}: {}", request.component_name, e);
            }
        } else {
            // It's a generic Lua component - update it
            if let Ok((entity, lua_components_opt)) = query.get_mut(request.entity) {
                if let Some(mut lua_components) = lua_components_opt {
                    // Update the registry key for this component
                    lua_components.components.insert(request.component_name.clone(), Arc::new(request.data));
                } else {
                    // Entity doesn't have LuaCustomComponents yet, add it
                    let mut lua_components = LuaCustomComponents::default();
                    lua_components.components.insert(request.component_name.clone(), Arc::new(request.data));
                    commands.entity(entity).insert(lua_components);
                }
            } else {
                warn!("Entity {:?} not found for component update", request.entity);
                // Clean up the registry value
                if let Err(e) = lua_ctx.lua.remove_registry_value(request.data) {
                    warn!("Failed to remove registry value: {}", e);
                }
            }
        }
    }
}
