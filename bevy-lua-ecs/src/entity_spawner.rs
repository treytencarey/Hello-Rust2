use crate::components::ComponentRegistry;
use crate::lua_integration::LuaScriptContext;
use crate::spawn_queue::SpawnQueue;
use bevy::core_pipeline::core_2d::graph::Core2d;
use bevy::core_pipeline::core_3d::graph::Core3d;
use bevy::ecs::system::SystemChangeTick;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use mlua::prelude::*;

/// System that processes the spawn queue and creates entities
pub fn process_spawn_queue(
    mut commands: Commands,
    queue: Res<SpawnQueue>,
    component_registry: Res<ComponentRegistry>,
    serde_registry: Res<crate::serde_components::SerdeComponentRegistry>,
    lua_ctx: Res<LuaScriptContext>,
    query: Query<Entity>,
    system_tick: SystemChangeTick,
) {
    let requests = queue.drain();

    if requests.is_empty() {
        return;
    }

    let entity_count_before = query.iter().count();
    debug!(
        "[SPAWN_QUEUE] Processing {} spawn requests (current world has {} entities)",
        requests.len(),
        entity_count_before
    );

    let mut spawned_count = 0;

    for request in requests {
        // Spawn entity
        let entity_id = commands.spawn_empty().id();
        debug!(
            "[SPAWN_QUEUE] Spawning entity {:?} with {} components",
            entity_id,
            request.components.len()
        );

        // Store the entity ID as u64 in Lua globals so scripts can access it
        // This allows spawn() to return the entity ID
        if let Err(e) = lua_ctx
            .lua
            .globals()
            .set("__LAST_SPAWNED_ENTITY__", entity_id.to_bits() as u64)
        {
            warn!("Failed to set last spawned entity: {}", e);
        }

        let mut entity = commands.entity(entity_id);
        let mut lua_custom_components = crate::components::LuaCustomComponents::default();
        let mut _has_interaction = false;

        // Track the spawned entity for returning to Lua
        queue.add_spawned_entity(entity_id);

        // Register temp_id -> Entity mapping for entity references (like UiTargetCamera)
        queue.register_entity(request.temp_id, entity_id);
        debug!(
            "[SPAWN_QUEUE] Registered temp_id {} -> {:?}",
            request.temp_id, entity_id
        );

        // Sort components to ensure consistent ordering for Required Components
        // This ensures "Camera" is processed before "Camera2d", so when Camera2d is
        // inserted and its Required Components are checked, our Camera already exists
        // and won't be replaced by Bevy's default Camera.
        let mut components = request.components;
        components.sort_by(|a, b| a.0.cmp(&b.0));

        // Apply each component
        for (component_name, registry_key) in components {
            debug!("[SPAWN_QUEUE] Processing component: {}", component_name);
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
            if let Some(result) =
                serde_registry.try_handle(&component_name, &data_value, &mut entity)
            {
                if let Err(e) = result {
                    error!("Failed to add serde component {}: {}", component_name, e);
                }
            }
            // Check if it's a known Rust component (Reflect)
            else if let Some(handler) = component_registry.get(&component_name) {
                debug!(
                    "[SPAWN_QUEUE] Applying component {} via Reflect handler",
                    component_name
                );

                // Pre-resolve entity references: If data has an 'entity' field,
                // resolve it using the unified method that handles both temp_ids and real entity bits
                let resolved_data = if let LuaValue::Table(ref table) = data_value {
                    if let Ok(entity_id) = table.get::<u64>("entity") {
                        // Use resolve_entity which handles both temp_ids and real entity bits
                        let resolved_entity = queue.resolve_entity(entity_id);
                        // Create a new table with resolved entity bits
                        let new_table = lua_ctx.lua.create_table().unwrap();
                        new_table.set("entity", resolved_entity.to_bits()).unwrap();
                        debug!(
                            "[SPAWN_QUEUE] Resolved entity reference: {} -> {:?} (bits: {})",
                            entity_id,
                            resolved_entity,
                            resolved_entity.to_bits()
                        );
                        LuaValue::Table(new_table)
                    } else {
                        data_value.clone()
                    }
                } else {
                    data_value.clone()
                };

                // Apply component via Reflect
                if let Err(e) = handler(&resolved_data, &mut entity) {
                    error!("Failed to add component {}: {}", component_name, e);
                }

                if component_name == "Interaction" {
                    _has_interaction = true;
                }
            }
            // It's a generic Lua component! Store it.
            else {
                debug!("[SPAWN_QUEUE] WARNING: Component {} not found in registry, treating as custom Lua component", component_name);
                // We keep the registry key alive in the component
                lua_custom_components
                    .components
                    .insert(component_name.clone(), std::sync::Arc::new(registry_key));
                // Mark as changed at current tick so change detection picks it up
                let current_tick = system_tick.this_run().get();
                lua_custom_components.changed_ticks.insert(component_name, current_tick);
                // We don't remove the registry value here because it's stored in Arc
                // But wait, we retrieved it above. We need to be careful about ownership.
                // The registry_key passed in the loop is owned by us.
                // We should NOT remove it if we store it in lua_custom_components.
                continue;
            }

            // Remove the registry value to free memory (only if NOT stored in custom components)
            if let Err(e) = lua_ctx.lua.remove_registry_value(registry_key) {
                warn!(
                    "Failed to remove registry value for {}: {}",
                    component_name, e
                );
            }
        }

        // Add generic Lua components if any
        if !lua_custom_components.components.is_empty() {
            entity.insert(lua_custom_components);
        }

        // Tag entity with script ownership using the captured instance_id
        if let Some(instance_id) = request.instance_id {
            entity.insert(crate::script_entities::ScriptOwned { instance_id });
        }

        // Set parent after all components are added
        // Use resolve_entity which handles both temp_ids and real entity bits
        if let Some(parent_id) = request.parent_temp_id {
            let parent_entity = queue.resolve_entity(parent_id);
            debug!(
                "[SPAWN_QUEUE] Setting parent for {:?}: {} -> {:?}",
                entity_id, parent_id, parent_entity
            );
            commands.entity(parent_entity).add_child(entity_id);
        }

        spawned_count += 1;
    }
    let entity_count = entity_count_before + spawned_count;
    debug!("[SPAWN_QUEUE] Total entities in the game: {}", entity_count);
}
