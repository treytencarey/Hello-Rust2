use bevy::prelude::*;
use crate::spawn_queue::SpawnQueue;
use crate::components::ComponentRegistry;
use crate::lua_integration::LuaScriptContext;
use mlua::prelude::*;

/// System that processes the spawn queue and creates entities
pub fn process_spawn_queue(
    mut commands: Commands,
    queue: Res<SpawnQueue>,
    component_registry: Res<ComponentRegistry>,
    serde_registry: Res<crate::serde_components::SerdeComponentRegistry>,
    lua_ctx: Res<LuaScriptContext>,
    current_script: Res<crate::script_entities::ScriptInstance>,
) {
    let requests = queue.drain();
    
    if requests.is_empty() {
        return;
    }
    
    for request in requests {
        // Spawn entity
        let entity_id = commands.spawn_empty().id();
        
        // Store the entity ID as u64 in Lua globals so scripts can access it
        // This allows spawn() to return the entity ID
        if let Err(e) = lua_ctx.lua.globals().set("__LAST_SPAWNED_ENTITY__", entity_id.to_bits() as u64) {
            warn!("Failed to set last spawned entity: {}", e);
        }
        
        let mut entity = commands.entity(entity_id);
        let mut lua_custom_components = crate::components::LuaCustomComponents::default();
        let mut _has_interaction = false;
        
        // Track the spawned entity for returning to Lua
        queue.add_spawned_entity(entity_id);
        
        // Apply each component
        for (component_name, registry_key) in request.components {
            // Retrieve the Lua value from the registry (can be string, table, number, etc.)
            let data_value: LuaValue = match lua_ctx.lua.registry_value(&registry_key) {
                Ok(value) => value,
                Err(e) => {
                    error!("Failed to retrieve Lua value for {}: {}", component_name, e);
                    continue;
                }
            };
            
            // Check if it's a known Serde component (Non-Reflect) FIRST
            // This handles special cases like Replicated which may be in both registries
            // but don't implement Default (required for Reflect-based insertion)
            if let Some(result) = serde_registry.try_handle(&component_name, &data_value, &mut entity) {
                if let Err(e) = result {
                    error!("Failed to add serde component {}: {}", component_name, e);
                }
            }
            // Check if it's a known Rust component (Reflect)
            else if let Some(handler) = component_registry.get(&component_name) {
                // Apply component via Reflect
                if let Err(e) = handler(&data_value, &mut entity) {
                    error!("Failed to add component {}: {}", component_name, e);
                }
                
                if component_name == "Interaction" {
                    _has_interaction = true;
                }
            }
            // It's a generic Lua component! Store it.
            else {
                // We keep the registry key alive in the component
                lua_custom_components.components.insert(component_name, std::sync::Arc::new(registry_key));
                // We don't remove the registry value here because it's stored in Arc
                // But wait, we retrieved it above. We need to be careful about ownership.
                // The registry_key passed in the loop is owned by us.
                // We should NOT remove it if we store it in lua_custom_components.
                continue; 
            }
            
            // Remove the registry value to free memory (only if NOT stored in custom components)
            if let Err(e) = lua_ctx.lua.remove_registry_value(registry_key) {
                warn!("Failed to remove registry value for {}: {}", component_name, e);
            }
        }
        
        // Add generic Lua components if any
        if !lua_custom_components.components.is_empty() {
            entity.insert(lua_custom_components);
        }
        
        // Automatically tag entity with script ownership if a script instance is executing
        if let Some(instance_id) = current_script.get_id() {
            entity.insert(crate::script_entities::ScriptOwned { instance_id });
        }
        
        // Set parent after all components are added
        if let Some(parent_entity) = request.parent {
            commands.entity(parent_entity).add_child(entity_id);
        }
    }
}
