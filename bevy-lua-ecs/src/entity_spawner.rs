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
) {
    let requests = queue.drain();
    
    if requests.is_empty() {
        return;
    }
    
    for request in requests {
        // Spawn entity
        let mut entity = commands.spawn_empty();
        let mut lua_custom_components = crate::components::LuaCustomComponents::default();
        let mut _has_interaction = false;
        
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
            
            // Check if it's a known Rust component (Reflect)
            if let Some(handler) = component_registry.get(&component_name) {
                // Apply component via Reflect
                if let Err(e) = handler(&data_value, &mut entity) {
                    error!("Failed to add component {}: {}", component_name, e);
                }
                
                if component_name == "Interaction" {
                    _has_interaction = true;
                }
            } 
            // Check if it's a known Serde component (Non-Reflect)
            else if let Some(result) = serde_registry.try_handle(&component_name, &data_value, &mut entity) {
                if let Err(e) = result {
                    error!("Failed to add serde component {}: {}", component_name, e);
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
            
            // If we have custom components and interaction, we might want PrevInteraction
            // But since we don't know which custom component is a callback, we can't be sure.
            // However, the user script handles interaction state changes manually via query.changed("Interaction").
            // So we might NOT need PrevInteraction anymore if the Lua script does the logic!
            // The previous PrevInteraction was for the Rust event system.
            // If the user wants to track previous state in Lua, they can do it in Lua.
            // But let's keep it if Interaction is present, just in case?
            // No, "Zero Rust" means we shouldn't add magic components unless necessary.
            // The Lua script example uses `changed("Interaction")` query filter, which works on the Interaction component itself.
            // So we don't need PrevInteraction!
        }
    }
}
