use crate::component_update_queue::ComponentUpdateQueue;
use crate::components::ComponentRegistry;
use crate::lua_integration::LuaScriptContext;
use crate::lua_world_api::{execute_query, LuaQueryBuilder};
use crate::spawn_queue::SpawnQueue;
use bevy::prelude::*;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};

/// Entry for a registered Lua system with per-system tick tracking
/// 
/// Each system has its own `last_run` tick, updated after IT runs.
/// This ensures cross-system change detection works correctly even when
/// budget time-slicing spreads systems across multiple frames.
#[derive(Clone)]
pub struct LuaSystemEntry {
    pub instance_id: u64,
    pub system_key: Arc<LuaRegistryKey>,
    pub last_run: u32,  // Per-system tick for change detection
}

/// Resource that stores registered Lua systems
#[derive(Resource, Clone)]
pub struct LuaSystemRegistry {
    pub update_systems: Arc<Mutex<Vec<LuaSystemEntry>>>,
    /// Pending coroutines waiting for downloads: path -> list of (coroutine_key, instance_id)
    pub pending_system_coroutines: Arc<Mutex<std::collections::HashMap<String, Vec<(Arc<LuaRegistryKey>, u64)>>>>,
}

impl Default for LuaSystemRegistry {
    fn default() -> Self {
        Self {
            update_systems: Arc::new(Mutex::new(Vec::new())),
            pending_system_coroutines: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
}

impl LuaSystemRegistry {
    /// Register a new system with initial tick of 0
    pub fn register_system(&self, instance_id: u64, system_key: Arc<LuaRegistryKey>) {
        let mut systems = self.update_systems.lock().unwrap();
        systems.push(LuaSystemEntry {
            instance_id,
            system_key,
            last_run: 0,
        });
    }
    
    /// Clear all systems registered by a specific script instance
    pub fn clear_instance_systems(&self, instance_id: u64) {
        let mut systems = self.update_systems.lock().unwrap();
        let initial_count = systems.len();

        systems.retain(|entry| entry.instance_id != instance_id);

        let removed_count = initial_count - systems.len();
        if removed_count > 0 {
            debug!(
                "Cleared {} systems from instance {}",
                removed_count, instance_id
            );
        }
    }

    /// Register a system coroutine waiting for a download
    pub fn register_pending_coroutine(&self, path: String, coroutine_key: Arc<LuaRegistryKey>, instance_id: u64) {
        debug!("📥 [SYSTEM] Registering pending coroutine for '{}'", path);
        self.pending_system_coroutines
            .lock()
            .unwrap()
            .entry(path)
            .or_insert_with(Vec::new)
            .push((coroutine_key, instance_id));
    }

    /// Take all system coroutines waiting for a specific path
    pub fn take_pending_coroutines(&self, path: &str) -> Vec<(Arc<LuaRegistryKey>, u64)> {
        self.pending_system_coroutines
            .lock()
            .unwrap()
            .remove(path)
            .unwrap_or_default()
    }

    /// Get all paths with pending system coroutines
    pub fn get_all_pending_paths(&self) -> Vec<String> {
        self.pending_system_coroutines
            .lock()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }

    /// Check if a path has pending system coroutines
    pub fn has_pending_coroutines(&self, path: &str) -> bool {
        self.pending_system_coroutines
            .lock()
            .unwrap()
            .get(path)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }
}

/// System that runs registered Lua update systems with time-sliced execution
/// 
/// Systems are executed in round-robin order. If the frame budget is exceeded,
/// remaining systems are deferred to run first on the next frame. This ensures
/// fairness and prevents any single system from starving others.
///
/// The time budget is controlled by the `LuaFrameBudget` resource (default: 4ms).
pub fn run_lua_systems(world: &mut World) {
    // Get resources we need
    let lua_ctx = world.resource::<LuaScriptContext>().clone();
    let registry = world.resource::<LuaSystemRegistry>().clone();
    let component_registry = world.resource::<ComponentRegistry>();
    let update_queue = world.resource::<ComponentUpdateQueue>().clone();
    let spawn_queue = world.resource::<SpawnQueue>().clone();
    let serde_registry = world
        .resource::<crate::serde_components::SerdeComponentRegistry>()
        .clone();
    let _builder_registry = world
        .resource::<crate::resource_builder::ResourceBuilderRegistry>()
        .clone();
    let script_registry = world
        .resource::<crate::script_registry::ScriptRegistry>()
        .clone();
    let despawn_queue = world
        .resource::<crate::despawn_queue::DespawnQueue>()
        .clone();
    let pending_messages = world
        .get_resource::<crate::event_sender::PendingLuaMessages>()
        .cloned()
        .unwrap_or_default();
    
    // Get frame budget and progress tracker
    let frame_budget = world
        .get_resource::<crate::lua_frame_budget::LuaFrameBudget>()
        .cloned()
        .unwrap_or_default();
    let progress = world
        .get_resource::<crate::lua_frame_budget::LuaSystemProgress>()
        .cloned()
        .unwrap_or_default();
    
    // Get query cache and current frame for query result caching
    let query_cache = world.get_resource::<crate::query_cache::LuaQueryCache>().cloned();
    let current_frame = world.get_resource::<bevy::diagnostic::FrameCount>()
        .map(|f| f.0 as u64)
        .unwrap_or(0);
    
    // Signal new frame to progress tracker (auto-resets per-frame state)
    progress.new_frame(current_frame);
    
    // Get current change tick (all systems will use this as their this_run)
    let this_run = world.read_change_tick().get();

    // Get the list of systems (with per-system last_run ticks)
    let systems = registry.update_systems.lock().unwrap().clone();
    let total_systems = systems.len();
    
    if total_systems == 0 {
        return;
    }
    
    // Start from where we left off last frame (for fairness)
    let start_index = progress.next_index();
    
    // Track how many systems we've run this frame
    let mut systems_run = 0;
    
    // Track which system indices need their ticks updated (index -> new_tick)
    let mut systems_to_update: Vec<(usize, u32)> = Vec::new();
    
    // Track systems that should be removed (one-shot systems that returned true)
    let mut systems_to_remove: Vec<usize> = Vec::new();
    
    // Run systems in round-robin order
    for i in 0..total_systems {
        let actual_index = (start_index + i) % total_systems;
        let entry = &systems[actual_index];
        
        // Get this system's own last_run tick (per-system, not per-instance!)
        let last_run_for_system = entry.last_run;
        
        // Time this system
        let timer = crate::lua_frame_budget::SystemTimer::start();
        
        match run_single_lua_system_fast(
            &lua_ctx.lua,
            &entry.system_key,
            world,
            component_registry,
            &update_queue,
            &spawn_queue,
            &serde_registry,
            &script_registry,
            &registry,
            &despawn_queue,
            &pending_messages,
            last_run_for_system,
            this_run,
            query_cache.as_ref(),
            current_frame,
        ) {
            Ok(should_remove) => {
                if should_remove {
                    // One-shot system - mark for removal
                    systems_to_remove.push(actual_index);
                }
            }
            Err(e) => {
                error!("Error running Lua system: {}", e);
            }
        }
        
        // Mark this system for tick update (updated immediately after it runs)
        systems_to_update.push((actual_index, this_run));
        
        // Record elapsed time and check budget
        let elapsed = timer.elapsed();
        
        // Log slow systems (>1ms)
        if elapsed.as_millis() >= 1 {
            let script_path = script_registry
                .get_instance_path(entry.instance_id)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            debug!(
                "[LUA_SYSTEM] instance={} script={} took {:?}",
                entry.instance_id, script_path, elapsed
            );
        }
        
        systems_run += 1;
        
        // Advance progress tracker
        progress.advance(total_systems);
        
        // Check if we should continue or defer remaining systems
        if !progress.record_time(elapsed, &frame_budget) {
            // Budget exceeded - remaining systems deferred to next frame
            debug!(
                "[LUA_BUDGET] Budget exceeded after {} of {} systems ({:?} total)",
                systems_run, total_systems, progress.time_this_frame()
            );
            break;
        }
    }
    
    // Update ticks for systems that ran (per-SYSTEM, not per-instance!)
    // Each system gets its tick updated so cross-system changes are detected
    // Also remove one-shot systems
    {
        let mut systems_lock = registry.update_systems.lock().unwrap();
        
        // First, update ticks
        for (idx, new_tick) in &systems_to_update {
            if *idx < systems_lock.len() {
                systems_lock[*idx].last_run = *new_tick;
            }
        }
        
        // Then, remove one-shot systems (in reverse order to avoid index shifting issues)
        if !systems_to_remove.is_empty() {
            // Sort in reverse order (highest index first)
            let mut remove_sorted = systems_to_remove.clone();
            remove_sorted.sort_by(|a, b| b.cmp(a));
            
            for idx in remove_sorted {
                if idx < systems_lock.len() {
                    debug!("[LUA_SYSTEM] Removing one-shot system at index {}", idx);
                    systems_lock.remove(idx);
                }
            }
        }
    }
}

fn run_single_lua_system(
    lua: &Lua,
    system_key: &LuaRegistryKey,
    world: &World,
    component_registry: &ComponentRegistry,
    update_queue: &ComponentUpdateQueue,
    spawn_queue: &SpawnQueue,
    serde_registry: &crate::serde_components::SerdeComponentRegistry,
    _builder_registry: &crate::resource_builder::ResourceBuilderRegistry,
    last_run: u32,
    this_run: u32,
    query_cache: Option<&crate::query_cache::LuaQueryCache>,
    current_frame: u64,
) -> LuaResult<()> {
    // Get resource registry
    let resource_registry = world.resource::<crate::resource_lua_trait::LuaResourceRegistry>();
    // Get the Lua function
    let func: LuaFunction = lua.registry_value(system_key)?;

    // Use scope to ensure all closures are cleaned up
    lua.scope(|scope| {
        // Create world table
        let world_table = lua.create_table()?;

        // Apply _world_mt metatable if it exists (allows extending world API from Lua)
        if let Ok(mt) = lua.globals().get::<LuaTable>("_world_mt") {
            world_table.set_metatable(Some(mt));
        }

        // delta_time() - returns delta time in seconds
        world_table.set(
            "delta_time",
            scope.create_function(|_lua_ctx, _self: LuaTable| {
                let time = world.resource::<Time>();
                Ok(time.delta_secs())
            })?,
        )?;

        // query(with_components, changed_components) - executes immediately and returns results
        world_table.set(
            "query",
            scope.create_function(
                move |lua_ctx,
                      (_self, with_comps, changed_comps): (
                    LuaTable,
                    LuaTable,
                    Option<LuaTable>,
                )| {
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

                    let results = execute_query(
                        lua_ctx,
                        world,
                        &builder,
                        component_registry,
                        update_queue,
                        last_run,
                        this_run,
                        query_cache,
                        current_frame,
                    )?;

                    let results_table = lua_ctx.create_table()?;
                    for (i, entity) in results.into_iter().enumerate() {
                        results_table.set(i + 1, entity)?;
                    }

                    Ok(results_table)
                },
            )?,
        )?;

        // query_resource(resource_name) - check if a resource exists
        world_table.set(
            "query_resource",
            scope.create_function({
                let serde_registry = serde_registry.clone();
                move |_lua_ctx, (_self, resource_name): (LuaTable, String)| {
                    Ok(serde_registry.has_resource(&resource_name))
                }
            })?,
        )?;

        // get_entity(bits) - get an entity wrapper from entity bits (from entity:id() or MeshRayCast results)
        // Returns an entity wrapper with :has(), :get(), :id() methods
        world_table.set(
            "get_entity",
            scope.create_function({
                let update_queue_clone = update_queue.clone();
                let spawn_queue_for_get = spawn_queue.clone();
                move |lua_ctx, (_self, entity_bits): (LuaTable, i64)| {
                    // Resolve temp_id or raw entity bits using spawn_queue
                    // This handles both spawn() temp_ids and query() entity bits
                    let entity = spawn_queue_for_get.resolve_entity(entity_bits as u64);
                    
                    // Check if entity exists in world
                    let entity_ref = match world.get_entity(entity) {
                        Ok(e) => e,
                        Err(_) => {
                            debug!("[GET_ENTITY] Entity {:?} does not exist", entity);
                            return Ok(LuaValue::Nil);
                        }
                    };
                    
                    // Get Lua custom components (for :has() checks like VrPanelMarker)
                    let lua_components = if let Some(custom) = entity_ref.get::<crate::LuaCustomComponents>() {
                        // Just clone the Arc references
                        custom.components.clone()
                    } else {
                        std::collections::HashMap::new()
                    };
                    
                    let snapshot = crate::lua_world_api::LuaEntitySnapshot {
                        entity,
                        component_data: std::collections::HashMap::new(), // Not populating for now
                        lua_components,
                        update_queue: update_queue_clone.clone(),
                    };
                    
                    debug!("[GET_ENTITY] Returning snapshot for {:?}", entity);
                    Ok(LuaValue::UserData(lua_ctx.create_userdata(snapshot)?))
                }
            })?,
        )?;

        // has_component(entity_index, entity_generation, component_name) - check if an entity has a specific component
        // entity_index and entity_generation come from MeshRayCast result format "77v0" -> index=77, generation=0
        // This iterates through all entities to find the one matching index/generation
        world_table.set(
            "has_component",
            scope.create_function(
                |_lua_ctx, (_self, target_index, target_gen_display, component_name): (LuaTable, u32, u32, String)| {
                    // Bevy Debug format shows generation - 1, so "77v0" means generation()=1
                    // But entity.generation() returns the actual NonZeroU32 value
                    let target_gen = target_gen_display + 1;
                    
                    debug!("[HAS_COMPONENT] Looking for entity index={} gen_display={} (actual_gen={}) with '{}'", 
                        target_index, target_gen_display, target_gen, component_name);
                    
                    // Iterate all entities to find one matching index and generation
                    let mut found_entity: Option<Entity> = None;
                    for entity_ref in world.iter_entities() {
                        let entity = entity_ref.id();
                        if entity.index() == target_index && entity.generation().to_bits() == target_gen {
                            found_entity = Some(entity);
                            break;
                        }
                    }
                    
                    let entity = match found_entity {
                        Some(e) => e,
                        None => {
                            debug!("[HAS_COMPONENT] No entity found with index={} generation={}", target_index, target_gen);
                            return Ok(false);
                        }
                    };
                    
                    debug!("[HAS_COMPONENT] Found entity {:?}, checking for component", entity);
                    
                    // Get entity reference for component check
                    let entity_ref = match world.get_entity(entity) {
                        Ok(eref) => eref,
                        Err(_) => return Ok(false),
                    };
                    
                    // Look up component type in registry
                    let type_registry = component_registry.type_registry();
                    let registry = type_registry.read();
                    
                    // Try short name first, then full path
                    let type_registration = registry
                        .get_with_short_type_path(&component_name)
                        .or_else(|| registry.get_with_type_path(&component_name));
                    
                    if let Some(registration) = type_registration {
                        // Get ReflectComponent to check if entity has it
                        if let Some(reflect_component) = registration.data::<ReflectComponent>() {
                            let filtered_ref: bevy::ecs::world::FilteredEntityRef = (&entity_ref).into();
                            let has = reflect_component.reflect(filtered_ref).is_some();
                            debug!("[HAS_COMPONENT] Rust component check: {}", has);
                            if has {
                                return Ok(true);
                            }
                        }
                    }
                    
                    // Check custom Lua components
                    if let Some(lua_comps) = entity_ref.get::<crate::components::LuaCustomComponents>() {
                        let has = lua_comps.components.contains_key(&component_name);
                        debug!("[HAS_COMPONENT] Lua component check: {}, available: {:?}", 
                            has, lua_comps.components.keys().collect::<Vec<_>>());
                        if has {
                            return Ok(true);
                        }
                    }
                    
                    Ok(false)
                },
            )?,
        )?;

        // get_resource(resource_type_name) - get a resource by type name via reflection
        // Returns the resource as a Lua table, or nil if not found
        // Usage: local ray_map = world:get_resource("bevy::picking::backend::ray::RayMap")
        world_table.set(
            "get_resource",
            scope.create_function({
                move |lua_ctx, (_self, resource_type_name): (LuaTable, String)| {
                    let type_registry = component_registry.type_registry();
                    let registry = type_registry.read();

                    // Look up the resource type by name (try short name first, then full path)
                    let type_registration = registry
                        .get_with_short_type_path(&resource_type_name)
                        .or_else(|| registry.get_with_type_path(&resource_type_name))
                        .ok_or_else(|| {
                            LuaError::RuntimeError(format!(
                                "Resource type '{}' not found in TypeRegistry",
                                resource_type_name
                            ))
                        })?;

                    // Get ReflectResource data
                    let reflect_resource = type_registration
                        .data::<bevy::ecs::reflect::ReflectResource>()
                        .ok_or_else(|| {
                            LuaError::RuntimeError(format!(
                        "'{}' doesn't have ReflectResource. Add #[reflect(Resource)] to the type.", 
                        resource_type_name
                    ))
                        })?;

                    // Access the resource via reflection (same pattern as read_events)
                    #[allow(invalid_reference_casting)]
                    let resource_ref = unsafe {
                        let world_mut = &mut *(world as *const bevy::ecs::world::World
                            as *mut bevy::ecs::world::World);
                        let world_cell = world_mut.as_unsafe_world_cell();
                        reflect_resource.reflect_unchecked_mut(world_cell)
                    };

                    match resource_ref {
                        Some(resource) => {
                            // Convert to Lua using reflection_to_lua
                            let lua_value = crate::event_reader::reflection_to_lua(
                                lua_ctx,
                                resource.as_partial_reflect(),
                                &type_registry,
                            )?;
                            Ok(lua_value)
                        }
                        None => {
                            // Resource doesn't exist
                            Ok(LuaValue::Nil)
                        }
                    }
                }
            })?,
        )?;

        // call_resource_method(resource_name, method_name, ...args) - call a registered method on a resource
        world_table.set(
            "call_resource_method",
            scope.create_function({
                let resource_registry = resource_registry.clone();
                move |lua_ctx,
                      (_, resource_name, method_name, args): (
                    LuaTable,
                    String,
                    String,
                    mlua::MultiValue,
                )| {
                    resource_registry.call_method(
                        lua_ctx,
                        world,
                        &resource_name,
                        &method_name,
                        args,
                    )
                }
            })?,
        )?;

        // call_component_method(entity, component_name, method_name, ...args) - call a registered method on an entity's component
        world_table.set(
            "call_component_method",
            scope.create_function({
                let component_registry_for_method = world
                    .resource::<crate::component_lua_trait::LuaComponentRegistry>()
                    .clone();
                move |lua_ctx,
                      (_, entity_snapshot, component_name, method_name, args): (
                    LuaTable,
                    mlua::AnyUserData,
                    String,
                    String,
                    mlua::MultiValue,
                )| {
                    // Get the entity from the snapshot
                    let snapshot =
                        entity_snapshot.borrow::<crate::lua_world_api::LuaEntitySnapshot>()?;
                    let entity = snapshot.entity;
                    drop(snapshot); // Drop borrow before calling method

                    // SAFETY: We need mutable access to call component methods
                    #[allow(invalid_reference_casting)]
                    let world_mut = unsafe { &mut *(world as *const World as *mut World) };

                    component_registry_for_method.call_method(
                        lua_ctx,
                        world_mut,
                        entity,
                        &component_name,
                        &method_name,
                        args,
                    )
                }
            })?,
        )?;

        // call_systemparam_method(param_name, method_name, ...args) - call a registered method on a SystemParam
        // Uses the global dispatcher set by the parent crate's initialization
        world_table.set(
            "call_systemparam_method",
            scope.create_function({
                move |lua_ctx,
                      (_, param_name, method_name, args): (
                    LuaTable,
                    String,
                    String,
                    mlua::MultiValue,
                )| {
                    // SAFETY: SystemParams require mutable world access via SystemState
                    #[allow(invalid_reference_casting)]
                    let world_mut = unsafe { &mut *(world as *const World as *mut World) };

                    // Use the global dispatch callback set by the parent crate
                    crate::systemparam_lua_trait::call_systemparam_method_global(
                        lua_ctx,
                        world_mut,
                        &param_name,
                        &method_name,
                        args,
                    )
                }
            })?,
        )?;

        // call_component_method(entity_id, type_name, method_name, ...args) - call a method on a Component
        // Supports Transform::looking_at, etc. Uses the global dispatcher set by the parent crate's initialization
        world_table.set(
            "call_component_method",
            scope.create_function({
                move |lua_ctx,
                      (_, entity_id, type_name, method_name, args): (
                    LuaTable,
                    u64,
                    String,
                    String,
                    mlua::MultiValue,
                )| {
                    // SAFETY: Entity mutation requires mutable world access
                    #[allow(invalid_reference_casting)]
                    let world_mut = unsafe { &mut *(world as *const World as *mut World) };

                    // Resolve temp_id to real entity (handles both spawn() temp IDs and query() real IDs)
                    let spawn_queue = world.resource::<crate::spawn_queue::SpawnQueue>();
                    let resolved_entity = spawn_queue.resolve_entity(entity_id);
                    let resolved_id = resolved_entity.to_bits();

                    // Use the global dispatch callback set by the parent crate
                    crate::systemparam_lua_trait::call_component_method_global(
                        lua_ctx,
                        world_mut,
                        resolved_id,
                        &type_name,
                        &method_name,
                        args,
                    )
                }
            })?,
        )?;

        let cleanup_script_instance =
            |lua_ctx: &Lua, instance_id: u64, _script_name: &str| -> Result<(), LuaError> {
                let script_registry = world
                    .resource::<crate::script_registry::ScriptRegistry>()
                    .clone();
                let system_registry = world.resource::<LuaSystemRegistry>().clone();
                let resource_queue = world
                    .resource::<crate::resource_queue::ResourceQueue>()
                    .clone();
                let serde_registry = world
                    .resource::<crate::serde_components::SerdeComponentRegistry>()
                    .clone();
                let builder_registry = world
                    .resource::<crate::resource_builder::ResourceBuilderRegistry>()
                    .clone();

                // Clear all systems registered by this instance
                // Note: Per-system ticks are cleaned up automatically with the system entries
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
                    let mut query_state =
                        world_mut.query::<(Entity, &crate::script_entities::ScriptOwned)>();
                    query_state
                        .iter(world_mut)
                        .filter(|(_, script_owned)| script_owned.instance_id == instance_id)
                        .map(|(entity, _)| entity)
                        .collect()
                };

                // Clear pending component updates for these entities
                if !entities_to_despawn.is_empty() {
                    let component_update_queue = world
                        .resource::<crate::component_update_queue::ComponentUpdateQueue>()
                        .clone();
                    let cleared_keys =
                        component_update_queue.clear_for_entities(&entities_to_despawn);

                    // Clean up the Lua registry keys
                    // Arc ensures we only clean up once all references are dropped
                    for key_arc in cleared_keys {
                        // Try to unwrap Arc - if this is the last reference, clean up
                        if let Ok(key) = Arc::try_unwrap(key_arc) {
                            let _ = lua_ctx.remove_registry_value(key);
                        }
                    }
                }

                // Despawn all entities from this instance
                crate::script_entities::despawn_instance_entities(world_mut, instance_id);

                // Remove old instance from registry
                script_registry.remove_instance(instance_id);

                Ok(())
            };

        // reload_current_script() - cleanup and reload the current script instance
        world_table.set(
            "reload_current_script",
            scope.create_function({
                let script_registry = world
                    .resource::<crate::script_registry::ScriptRegistry>()
                    .clone();
                let cleanup = cleanup_script_instance.clone();
                move |lua_ctx, _self: LuaTable| {
                    // Get the instance ID from Lua global (set by execute_script_tracked)
                    let instance_id: u64 = lua_ctx.globals().get("__INSTANCE_ID__")?;
                    let script_name: String = lua_ctx.globals().get("__SCRIPT_NAME__")?;

                    debug!(
                        "Manual reload requested for script instance {} ('{}')",
                        instance_id, script_name
                    );

                    // Get script path and content from registry
                    let script_path =
                        script_registry
                            .get_instance_path(instance_id)
                            .ok_or_else(|| {
                                LuaError::RuntimeError(format!(
                                    "Script instance {} not found in registry",
                                    instance_id
                                ))
                            })?;

                    let script_content = script_registry
                        .get_instance_content(instance_id)
                        .ok_or_else(|| {
                            LuaError::RuntimeError(format!(
                                "Script content for instance {} not found",
                                instance_id
                            ))
                        })?;

                    // Use shared cleanup logic
                    cleanup(lua_ctx, instance_id, &script_name)?;

                    // Get Lua context and re-execute the script
                    let lua_ctx_res = world
                        .resource::<crate::lua_integration::LuaScriptContext>()
                        .clone();
                    let script_instance = world
                        .resource::<crate::script_entities::ScriptInstance>()
                        .clone();

                    // Re-execute the script
                    match lua_ctx_res.execute_script_tracked(
                        &script_content,
                        &script_name,
                        &script_instance,
                    ) {
                        Ok(new_instance_id) => {
                            debug!(
                                "✓ Script reloaded: instance {} -> {}",
                                instance_id, new_instance_id
                            );

                            // Register the new instance
                            script_registry.register_script(
                                script_path,
                                new_instance_id,
                                script_content,
                            );

                            Ok(())
                        }
                        Err(e) => Err(LuaError::RuntimeError(format!(
                            "Failed to reload script: {}",
                            e
                        ))),
                    }
                }
            })?,
        )?;

        // stop_current_script() - cleanup and stop the current script (no reload)
        world_table.set(
            "stop_current_script",
            scope.create_function({
                let script_registry = world
                    .resource::<crate::script_registry::ScriptRegistry>()
                    .clone();
                let cleanup = cleanup_script_instance.clone();
                move |lua_ctx, _self: LuaTable| {
                    // Get the instance ID from Lua global (set by execute_script_tracked)
                    let instance_id: u64 = lua_ctx.globals().get("__INSTANCE_ID__")?;
                    let script_name: String = lua_ctx.globals().get("__SCRIPT_NAME__")?;

                    debug!(
                        "Stop requested for script instance {} ('{}')",
                        instance_id, script_name
                    );

                    // Mark as stopped in registry (prevents auto-reload)
                    script_registry.mark_stopped(instance_id);

                    // Use shared cleanup logic
                    cleanup(lua_ctx, instance_id, &script_name)?;

                    debug!("✓ Script instance {} stopped", instance_id);
                    Ok(())
                }
            })?,
        )?;

        // stop_owning_script(entity_id) - stop the script that owns the given entity
        // Reads ScriptOwned component from the entity to get instance_id
        world_table.set(
            "stop_owning_script",
            scope.create_function({
                let script_registry = world
                    .resource::<crate::script_registry::ScriptRegistry>()
                    .clone();
                let spawn_queue = world.resource::<SpawnQueue>().clone();
                let cleanup = cleanup_script_instance.clone();
                move |lua_ctx, (_self, entity_id): (LuaTable, u64)| {
                    // Use resolve_entity to handle both temp_id (from spawn) and entity bits (from query)
                    let entity = spawn_queue.resolve_entity(entity_id);

                    // Get ScriptOwned component from the entity
                    let script_owned = world
                        .get::<crate::script_entities::ScriptOwned>(entity)
                        .ok_or_else(|| {
                            LuaError::RuntimeError(format!(
                                "Entity {:?} has no ScriptOwned component (id: {})",
                                entity, entity_id
                            ))
                        })?;

                    let instance_id = script_owned.instance_id;

                    debug!(
                        "Stop requested for script instance {} (via entity {:?}, original id: {})",
                        instance_id, entity, entity_id
                    );

                    // Mark as stopped in registry (prevents auto-reload)
                    script_registry.mark_stopped(instance_id);

                    // Use shared cleanup logic
                    cleanup(lua_ctx, instance_id, "stop_owning_script")?;

                    debug!("✓ Script instance {} stopped via entity", instance_id);
                    Ok(())
                }
            })?,
        )?;

        // Legacy reload_script() - kept for backwards compatibility, calls reload_current_script()
        world_table.set(
            "reload_script",
            scope.create_function({
                move |_lua_ctx, self_table: LuaTable| {
                    // Just delegate to reload_current_script
                    let reload_fn: LuaFunction = self_table.get("reload_current_script")?;
                    reload_fn.call::<()>(self_table)
                }
            })?,
        )?;

        // despawn_all(tag_name) - despawn all entities with a specific tag component
        world_table.set(
            "despawn_all",
            scope.create_function({
                move |_lua_ctx, (_self, tag_name): (LuaTable, String)| {
                    let despawn_queue = world
                        .resource::<crate::despawn_queue::DespawnQueue>()
                        .clone();

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
                        world_cell
                            .world_mut()
                            .query::<(Entity, &crate::components::LuaCustomComponents)>()
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
            })?,
        )?;

        // read_events(event_type_name) - read any Bevy event via generated dispatch
        // Uses the dispatch_read_events function generated at build time which uses EventReader
        let read_events_fn =
            scope.create_function(|lua_ctx, (_self, event_type_name): (LuaTable, String)| {
                bevy::log::debug!("[READ_EVENTS] Reading events: '{}'", event_type_name);

                // Use unsafe world access - the dispatch function handles proper SystemState management
                #[allow(invalid_reference_casting)]
                let world_mut = unsafe {
                    &mut *(world as *const bevy::ecs::world::World as *mut bevy::ecs::world::World)
                };

                // Call the generated dispatch function via the global callback
                // Result is already LuaValue::Table, convert to LuaTable for return
                match crate::systemparam_lua_trait::call_read_events_global(
                    lua_ctx,
                    world_mut,
                    &event_type_name,
                ) {
                    Ok(mlua::Value::Table(t)) => Ok(t),
                    Ok(_) => Err(LuaError::RuntimeError(
                        "Expected table from read_events".into(),
                    )),
                    Err(e) => Err(e),
                }
            })?;

        world_table.set("read_events", read_events_fn.clone())?;

        // query_events(event_type_name) - alias for read_events for API consistency
        world_table.set("query_events", read_events_fn)?;

        // invoke_observer(entity_id, event_type, position_table) - directly invoke Lua observer callbacks
        // This bypasses Bevy's picking input pipeline and directly calls registered observer callbacks
        // Useful for RTT picking where we need manual event dispatch
        let lua_script_ctx = world
            .get_resource::<crate::lua_integration::LuaScriptContext>()
            .cloned()
            .expect("LuaScriptContext resource not found");
        let observer_registry = world
            .get_resource::<crate::lua_observers::LuaObserverRegistry>()
            .cloned()
            .unwrap_or_default();
        let update_queue_for_observer = world
            .get_resource::<crate::component_update_queue::ComponentUpdateQueue>()
            .cloned()
            .unwrap_or_default();
        let spawn_queue_for_observer = world
            .get_resource::<crate::spawn_queue::SpawnQueue>()
            .cloned()
            .unwrap();

        let invoke_observer_fn = scope.create_function({
            let lua_ctx = lua_script_ctx.clone();
            let observer_registry = observer_registry.clone();
            let update_queue = update_queue_for_observer.clone();
            let spawn_queue = spawn_queue_for_observer.clone();
            move |_lua,
                  (_self, entity_id, event_type, position_table): (
                LuaTable,
                u64,
                String,
                Option<LuaTable>,
            )| {
                // Resolve temp_id to real entity using SpawnQueue
                // This handles both temp_ids from spawn() and real entity bits from query()
                let entity = spawn_queue.resolve_entity(entity_id);

                // Extract position from table if provided
                let position = if let Some(pos_table) = position_table {
                    let x: f32 = pos_table.get("x").unwrap_or(0.0);
                    let y: f32 = pos_table.get("y").unwrap_or(0.0);
                    Some(bevy::math::Vec2::new(x, y))
                } else {
                    None
                };

                bevy::log::debug!(
                    "[INVOKE_OBSERVER] Invoking '{}' on entity {:?} (from id {}) at {:?}",
                    event_type,
                    entity,
                    entity_id,
                    position
                );

                // Direct call to the internal observer dispatch
                crate::lua_observers::dispatch_lua_observer_internal(
                    &lua_ctx,
                    &observer_registry,
                    &update_queue,
                    entity,
                    &event_type,
                    position,
                );

                Ok(())
            }
        })?;

        world_table.set("invoke_observer", invoke_observer_fn)?;

        // send_event(event_type_name, data_table) - send an event immediately using reflection
        // Uses the dispatch_write_events function generated at build time which uses EventWriter
        let send_event_fn = scope.create_function(
            |lua_ctx, (_self, event_type_name, data_table): (LuaTable, String, LuaTable)| {
                bevy::log::debug!("[SEND_EVENT] Writing event: '{}'", event_type_name);

                // Use unsafe world access - the dispatch function handles proper SystemState management
                #[allow(invalid_reference_casting)]
                let world_mut = unsafe {
                    &mut *(world as *const bevy::ecs::world::World as *mut bevy::ecs::world::World)
                };

                // Call the generated dispatch function via the global callback
                match crate::systemparam_lua_trait::call_write_events_global(
                    lua_ctx,
                    world_mut,
                    &event_type_name,
                    &data_table,
                ) {
                    Ok(()) => {
                        bevy::log::debug!(
                            "[SEND_EVENT] Successfully sent event: '{}'",
                            event_type_name
                        );
                        Ok(())
                    }
                    Err(e) => Err(LuaError::RuntimeError(format!(
                        "Failed to send event '{}': {}",
                        event_type_name, e
                    ))),
                }
            },
        )?;

        world_table.set("send_event", send_event_fn.clone())?;
        world_table.set("write_event", send_event_fn)?;

        // write_message(message_type_name, data_table) - queue a message to be sent
        // Messages use MessageWriter<M> instead of EventWriter<T>
        // Used for types like PointerInput that use the message system
        let pending_messages = world
            .get_resource::<crate::event_sender::PendingLuaMessages>()
            .cloned()
            .unwrap_or_default();

        let write_message_fn = scope.create_function({
            let pending_messages = pending_messages.clone();
            move |_lua_ctx, (_self, message_type_name, data_table): (LuaTable, String, LuaTable)| {
                // Convert Lua table to JSON value (reuse the same conversion logic)
                fn lua_to_json(value: &LuaValue) -> serde_json::Value {
                    match value {
                        LuaValue::Nil => serde_json::Value::Null,
                        LuaValue::Boolean(b) => serde_json::Value::Bool(*b),
                        LuaValue::Integer(i) => serde_json::json!(*i),
                        LuaValue::Number(n) => serde_json::json!(*n),
                        LuaValue::String(s) => match s.to_str() {
                            Ok(cow) => serde_json::Value::String(cow.to_string()),
                            Err(_) => serde_json::Value::String(String::new()),
                        },
                        LuaValue::Table(t) => {
                            // Determine if array or object
                            let mut is_array = true;
                            let mut max_key = 0i64;
                            for pair in t.clone().pairs::<LuaValue, LuaValue>() {
                                if let Ok((key, _)) = pair {
                                    match key {
                                        LuaValue::Integer(i) if i > 0 => {
                                            max_key = max_key.max(i);
                                        }
                                        _ => {
                                            is_array = false;
                                            break;
                                        }
                                    }
                                }
                            }

                            if is_array && max_key > 0 {
                                let mut arr = Vec::new();
                                for i in 1..=max_key {
                                    if let Ok(val) = t.get::<LuaValue>(i) {
                                        arr.push(lua_to_json(&val));
                                    }
                                }
                                serde_json::Value::Array(arr)
                            } else {
                                let mut map = serde_json::Map::new();
                                for pair in t.clone().pairs::<String, LuaValue>() {
                                    if let Ok((key, val)) = pair {
                                        map.insert(key, lua_to_json(&val));
                                    }
                                }
                                serde_json::Value::Object(map)
                            }
                        }
                        _ => serde_json::Value::Null,
                    }
                }

                let json_data = lua_to_json(&LuaValue::Table(data_table.clone()));

                debug!(
                    "[WRITE_MESSAGE] Queueing message '{}': {:?}",
                    message_type_name, json_data
                );
                pending_messages.queue_message(message_type_name.clone(), json_data);

                Ok(())
            }
        })?;

        world_table.set("write_message", write_message_fn.clone())?;
        world_table.set("send_message", write_message_fn)?;

        // Call the Lua system function
        func.call::<()>(world_table)?;

        Ok(())
    })
}

/// Optimized version of run_single_lua_system using LuaWorldContext userdata
/// This eliminates the overhead of creating 10+ closures per system call
/// by using statically-defined userdata methods instead
/// 
/// Returns (Ok(true), _) if the system returned `true` indicating it should be removed (one-shot),
/// Returns (Ok(false), _) if the system should continue running normally.
pub fn run_single_lua_system_fast<'w>(
    lua: &Lua,
    system_key: &LuaRegistryKey,
    world: &'w World,
    component_registry: &'w ComponentRegistry,
    update_queue: &ComponentUpdateQueue,
    spawn_queue: &SpawnQueue,
    serde_registry: &crate::serde_components::SerdeComponentRegistry,
    script_registry: &crate::script_registry::ScriptRegistry,
    system_registry: &LuaSystemRegistry,
    despawn_queue: &crate::despawn_queue::DespawnQueue,
    pending_messages: &crate::event_sender::PendingLuaMessages,
    last_run: u32,
    this_run: u32,
    query_cache: Option<&crate::query_cache::LuaQueryCache>,
    current_frame: u64,
) -> LuaResult<bool> {  // Returns true if system should be REMOVED (one-shot)
    use std::time::Instant;
    
    let t0 = Instant::now();
    
    // Get the Lua function
    let func: LuaFunction = lua.registry_value(system_key)?;
    
    let t1 = Instant::now();

    // Use scope to ensure userdata is cleaned up after the call
    let result = lua.scope(|scope| {
        let t2 = Instant::now();
        
        // Create LuaWorldContext with all methods defined statically
        let world_ctx = crate::lua_world_context::LuaWorldContext::new(
            world,
            component_registry,
            update_queue.clone(),
            spawn_queue.clone(),
            serde_registry.clone(),
            script_registry.clone(),
            system_registry.clone(),
            despawn_queue.clone(),
            pending_messages.clone(),
            last_run,
            this_run,
            query_cache.cloned(),
            current_frame,
        );
        
        let t3 = Instant::now();

        // Create scoped userdata that won't escape this scope
        let world_ud = scope.create_userdata(world_ctx)?;
        
        let t4 = Instant::now();

        // Run the system function inside a coroutine so it can yield for asset downloads
        let thread = lua.create_thread(func)?;
        
        // Track if this is a one-shot system (returns true)
        let mut should_remove = false;
        
        match thread.resume::<mlua::Value>(world_ud) {
            Ok(yield_value) => {
                match thread.status() {
                    mlua::ThreadStatus::Finished => {
                        // System completed normally
                        // Check if it returned `true` to signal one-shot removal
                        if let mlua::Value::Boolean(true) = yield_value {
                            should_remove = true;
                            debug!("[LUA_SYSTEM] System returned true - marking for removal (one-shot)");
                        }
                    }
                    mlua::ThreadStatus::Resumable => {
                        // System yielded - needs to wait for a download
                        // The yield value should be the download path
                        if let mlua::Value::String(path_str) = yield_value {
                            if let Ok(path) = path_str.to_str() {
                                debug!("📥 [LUA_SYSTEM] System yielded for download: {}", path);
                                
                                // Store the coroutine for later resumption
                                // Get script cache from Lua context
                                if let Ok(script_cache_global) = lua.globals().get::<mlua::Value>("__SCRIPT_CACHE__") {
                                    if let mlua::Value::LightUserData(_ptr) = script_cache_global {
                                        debug!("📥 [LUA_SYSTEM] System coroutine registered for resumption");
                                        // For now, just warn - full implementation would store coroutine
                                        // TODO: Store coroutine in pending_download_coroutines for resumption
                                    }
                                }
                                
                                // For now, warn but don't error - the download is queued
                                // The asset may not be available THIS frame but will load eventually
                                warn!("⚠️ System yielded for download '{}'. Asset loading deferred.", path);
                            }
                        } else {
                            warn!("⚠️ System yielded with unexpected value: {:?}", yield_value);
                        }
                    }
                    mlua::ThreadStatus::Error => {
                        warn!("⚠️ System coroutine in error state");
                    }
                    mlua::ThreadStatus::Running => {
                        // This shouldn't happen
                    }
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
        
        let t5 = Instant::now();
        
        // Log timing breakdown for slow calls (>0.5ms)
        let total = t5.duration_since(t2);
        if total.as_micros() >= 500 {
            let ctx_create = t3.duration_since(t2).as_micros();
            let ud_create = t4.duration_since(t3).as_micros();
            let lua_call = t5.duration_since(t4).as_micros();
            debug!(
                "[FAST_TIMING] ctx={:>4}us ud={:>4}us call={:>4}us",
                ctx_create, ud_create, lua_call
            );
        }

        Ok(should_remove)
    });
    
    // Log function lookup time if significant
    let fn_lookup = t1.duration_since(t0).as_micros();
    if fn_lookup >= 100 {
        debug!("[FAST_TIMING] fn_lookup={}us", fn_lookup);
    }
    
    result
}
