use crate::lua_integration::LuaScriptContext;
use crate::resource_queue::ResourceQueue;
use crate::serde_components::SerdeComponentRegistry;
use bevy::prelude::*;
use mlua::prelude::*;

/// System that processes the resource queue and inserts resources
pub fn process_resource_queue(world: &mut World) {
    // Get resources we need
    let queue = world.resource::<ResourceQueue>().clone();
    let lua_ctx = world.resource::<LuaScriptContext>().clone();
    let serde_registry = world.resource::<SerdeComponentRegistry>().clone();
    let builder_registry = world
        .resource::<crate::resource_builder::ResourceBuilderRegistry>()
        .clone();

    let requests = queue.drain();

    if requests.is_empty() {
        return;
    }

    for request in requests {
        // Track which script instance inserted this resource
        if let Some(instance_id) = request.instance_id {
            queue.track_resource(instance_id, request.resource_name.clone());
        }

        // Retrieve the Lua value from the registry
        let data_value: LuaValue = match lua_ctx.lua.registry_value(&request.data) {
            Ok(value) => value,
            Err(e) => {
                error!(
                    "Failed to retrieve Lua value for resource {}: {}",
                    request.resource_name, e
                );
                continue;
            }
        };

        // Try builder registry first (for resources like RenetServer, NetcodeServerTransport)
        if let Some(result) = builder_registry.try_build(
            &lua_ctx.lua,
            &request.resource_name,
            data_value.clone(),
            world,
        ) {
            match result {
                Ok(()) => {
                    debug!(
                        "✓ Inserted resource '{}' via builder",
                        request.resource_name
                    );
                    // Mark resource as inserted for query_resource tracking
                    serde_registry.mark_resource_inserted(&request.resource_name);
                }
                Err(e) => {
                    error!("Failed to build resource {}: {}", request.resource_name, e);
                }
            }
        }
        // Fall back to serde registry (for simple resources)
        else if let Some(result) =
            serde_registry.try_insert_resource(&request.resource_name, &data_value, world)
        {
            if let Err(e) = result {
                error!("Failed to insert resource {}: {}", request.resource_name, e);
            }
        } else {
            warn!(
                "Resource type '{}' is not registered in builder or serde registry",
                request.resource_name
            );
        }

        // Remove the registry value to free memory
        if let Err(e) = lua_ctx.lua.remove_registry_value(request.data) {
            warn!(
                "Failed to remove registry value for resource {}: {}",
                request.resource_name, e
            );
        }
    }
}
