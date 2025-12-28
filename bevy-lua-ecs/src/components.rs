use bevy::ecs::reflect::{ReflectCommandExt, ReflectComponent};
use bevy::prelude::*;
use bevy::reflect::{PartialReflect, ReflectMut, TypeInfo};
use mlua::prelude::*;
use std::collections::HashMap;

use std::sync::Arc;

/// Component handler function type
type ComponentHandler = Box<dyn Fn(&LuaValue, &mut EntityCommands) -> LuaResult<()> + Send + Sync>;

/// Generic container for components defined purely in Lua
#[derive(Component, Default, Clone)]
pub struct LuaCustomComponents {
    pub components: HashMap<String, Arc<LuaRegistryKey>>,
}

/// Registry of component handlers using reflection
#[derive(Resource)]
pub struct ComponentRegistry {
    pub handlers: HashMap<String, ComponentHandler>,
    type_registry: AppTypeRegistry,
    asset_registry: Option<crate::asset_loading::AssetRegistry>,
    /// Map of non-reflected component names to their TypeIds (for components that don't implement Reflect)
    non_reflected_components: HashMap<String, std::any::TypeId>,
}

impl ComponentRegistry {
    /// Create registry from app's type registry
    pub fn from_type_registry(type_registry: AppTypeRegistry) -> Self {
        let mut registry = Self {
            handlers: HashMap::new(),
            type_registry: type_registry.clone(),
            asset_registry: None,
            non_reflected_components: HashMap::new(),
        };

        // Auto-discover and register all components
        registry.discover_components();

        registry
    }

    /// Set the asset registry (called after AssetRegistry is created)
    pub fn set_asset_registry(&mut self, asset_registry: crate::asset_loading::AssetRegistry) {
        self.asset_registry = Some(asset_registry);
        // Re-discover components to update handlers with asset registry
        self.discover_components();
    }

    /// Automatically discover all registered components via reflection
    fn discover_components(&mut self) {
        let type_registry = self.type_registry.read();

        for registration in type_registry.iter() {
            // Check if this type is a component
            if registration.data::<ReflectComponent>().is_some() {
                let type_info = registration.type_info();
                let short_name = type_info.type_path_table().short_path().to_string();
                let full_path = type_info.type_path().to_string();

                // Clone what we need for the closure
                let registry_clone = self.type_registry.clone();
                let full_path_clone = full_path.clone();
                let asset_registry_clone = self.asset_registry.clone();

                // Create handler using reflection
                let handler = Box::new(move |data: &LuaValue, entity: &mut EntityCommands| {
                    spawn_component_via_reflection(
                        data,
                        entity,
                        &full_path_clone,
                        &registry_clone,
                        asset_registry_clone.as_ref(),
                    )
                });

                self.handlers.insert(short_name, handler);
            }
        }
    }

    /// Get a component handler by name
    pub fn get(&self, name: &str) -> Option<&ComponentHandler> {
        self.handlers.get(name)
    }

    /// Get the full type path for a component by short name
    pub fn get_type_path(&self, short_name: &str) -> Option<String> {
        // Check non-reflected components first
        if let Some(type_id) = self.non_reflected_components.get(short_name) {
            return Some(std::any::type_name_of_val(type_id).to_string());
        }

        // Fall back to reflected components
        let type_registry = self.type_registry.read();
        for registration in type_registry.iter() {
            if registration.data::<ReflectComponent>().is_some() {
                let type_info = registration.type_info();
                if type_info.type_path_table().short_path() == short_name {
                    return Some(type_info.type_path().to_string());
                }
            }
        }
        None
    }

    /// Register a component that doesn't implement Reflect for queryability
    pub fn register_non_reflected_component<C: Component + 'static>(&mut self, short_name: &str) {
        use std::any::TypeId;
        self.non_reflected_components
            .insert(short_name.to_string(), TypeId::of::<C>());
    }

    /// Check if a non-reflected component is registered
    pub fn is_non_reflected_component(&self, short_name: &str) -> bool {
        self.non_reflected_components.contains_key(short_name)
    }

    /// Get the TypeId of a non-reflected component
    pub fn get_non_reflected_type_id(&self, short_name: &str) -> Option<&std::any::TypeId> {
        self.non_reflected_components.get(short_name)
    }

    /// Get access to the type registry
    pub fn type_registry(&self) -> &AppTypeRegistry {
        &self.type_registry
    }

    /// Register an entity wrapper component (newtype around Entity)
    /// These components have pattern: pub struct ComponentName(pub Entity);
    /// Lua table should have format: { entity = entity_id }
    ///
    /// # Example
    /// ```ignore
    /// registry.register_entity_component::<UiTargetCamera>("UiTargetCamera", UiTargetCamera);
    /// ```
    pub fn register_entity_component<C, F>(&mut self, short_name: &str, constructor: F)
    where
        C: Component,
        F: Fn(Entity) -> C + Send + Sync + 'static,
    {
        let component_name = short_name.to_string();

        let handler: ComponentHandler = Box::new(
            move |data: &LuaValue, entity_commands: &mut EntityCommands| {
                if let LuaValue::Table(table) = data {
                    // Get entity ID from Lua table
                    if let Ok(entity_id) = table.get::<u64>("entity") {
                        // Create Entity from bits using Bevy 0.17 API
                        // from_bits reconstructs the full Entity (index + generation) from to_bits()
                        let target_entity = Entity::from_bits(entity_id);
                        debug!(
                            "[ENTITY_COMPONENT] {} targeting entity bits {} -> {:?}",
                            component_name, entity_id, target_entity
                        );
                        let component = constructor(target_entity);
                        entity_commands.insert(component);
                        return Ok(());
                    }
                }
                Err(mlua::Error::RuntimeError(format!(
                    "Failed to create {} from Lua. Expected table with 'entity' field.",
                    component_name
                )))
            },
        );

        self.handlers.insert(short_name.to_string(), handler);
        debug!("✓ Registered entity component handler: {}", short_name);
    }

    /// Register a handle wrapper component (newtype around Handle<T>)
    /// These components have pattern: pub struct ComponentName(pub Handle<T>);
    /// Lua table should have format: { id = asset_id } or just the asset_id directly
    ///
    /// Uses AssetRegistry to get asset path, then loads typed Handle<T> at spawn time.
    ///
    /// # Example
    /// ```ignore
    /// registry.register_handle_component::<SceneRoot, Scene>("SceneRoot", SceneRoot);
    /// ```
    pub fn register_handle_component<C, T, F>(&mut self, short_name: &str, constructor: F)
    where
        C: Component,
        T: bevy::asset::Asset,
        F: Fn(bevy::prelude::Handle<T>) -> C + Send + Sync + 'static,
    {
        let component_name = short_name.to_string();
        let asset_registry = self.asset_registry.clone();
        // Wrap constructor in Arc before closure to avoid Clone requirement on F
        let constructor_arc = std::sync::Arc::new(constructor);

        let handler: ComponentHandler = Box::new(
            move |data: &LuaValue, entity_commands: &mut EntityCommands| {
                // Get asset ID from Lua - can be { id = N }, { _0 = N }, or just N
                let asset_id: Option<u32> = match data {
                    LuaValue::Integer(i) => Some(*i as u32),
                    LuaValue::Number(n) => Some(*n as u32),
                    LuaValue::Table(table) => {
                        // Try "id" first, then "_0"
                        table.get::<u32>("id")
                            .ok()
                            .or_else(|| table.get::<u32>("_0").ok())
                    }
                    _ => None,
                };

                if let Some(id) = asset_id {
                    if let Some(ref registry) = asset_registry {
                        // Get the asset path from registry
                        if let Some(path) = registry.get_path(id) {
                            // Queue a command that will run with World access
                            // This allows us to use asset_server.load::<T>() with the correct type
                            let component_name_clone = component_name.clone();
                            let constructor_clone = std::sync::Arc::clone(&constructor_arc);
                            entity_commands.queue(move |mut entity: bevy::prelude::EntityWorldMut| {
                                let asset_server = entity.world().resource::<AssetServer>().clone();
                                let handle: Handle<T> = asset_server.load(&path);
                                debug!(
                                    "[HANDLE_COMPONENT] {} path '{}' -> Handle<{}>",
                                    component_name_clone, path, std::any::type_name::<T>()
                                );
                                let component = constructor_clone(handle);
                                entity.insert(component);
                            });
                            return Ok(());
                        }
                    }
                    return Err(mlua::Error::RuntimeError(format!(
                        "Failed to create {} from Lua. Asset ID {} not found in registry.",
                        component_name, id
                    )));
                }
                
                Err(mlua::Error::RuntimeError(format!(
                    "Failed to create {} from Lua. Expected 'id' field with asset ID or direct asset ID.",
                    component_name
                )))
            },
        );

        self.handlers.insert(short_name.to_string(), handler);
        debug!("✓ Registered handle component handler: {}", short_name);
    }
}

/// Register entity wrapper components at runtime using TypeRegistry
///
/// This function takes a list of discovered type names and looks up each one
/// in the TypeRegistry. If found and it's a valid entity wrapper component
/// (a tuple struct with a single Entity field), it registers a handler.
///
/// This enables true auto-discovery without compile-time type paths.
pub fn register_entity_wrappers_runtime(
    component_registry: &mut ComponentRegistry,
    type_registry: &AppTypeRegistry,
    discovered_names: &[&str],
) {
    let registry = type_registry.read();

    // Blocklist of internal Bevy types that should NOT be registered as Lua entity wrappers
    // These are internal rendering/ECS relationship types that Bevy manages automatically
    let blocklist = [
        "ChildOf",                         // Bevy hierarchy - managed internally
        "Children",                        // Bevy hierarchy - managed internally
        "RenderEntity",                    // Bevy render world - internal
        "MainEntity",                      // Bevy render world - internal
        "OcclusionCullingSubviewEntities", // Internal render optimization
        "UiCameraView",                    // Internal UI rendering
        "UiViewTarget",                    // Internal UI rendering
        "TargetCamera",                    // Use the specific UiTargetCamera instead
    ];

    for &type_name in discovered_names {
        // Skip blocklisted internal types
        if blocklist.contains(&type_name) {
            debug!(
                "[RUNTIME_ENTITY_WRAPPER] Skipping blocklisted internal type: {}",
                type_name
            );
            continue;
        }

        // Try to find this type by its short name
        let mut found = false;

        for registration in registry.iter() {
            let type_info = registration.type_info();
            let short_path = type_info.type_path_table().short_path();

            // Check if this is the type we're looking for
            if short_path == type_name {
                // Verify it's a component
                if registration.data::<ReflectComponent>().is_none() {
                    debug!(
                        "[RUNTIME_ENTITY_WRAPPER] {} is not a Component, skipping",
                        type_name
                    );
                    continue;
                }

                // Verify it's a tuple struct (entity wrappers are tuple structs)
                if let TypeInfo::TupleStruct(tuple_info) = type_info {
                    // Check if it has exactly one field that could be an Entity
                    if tuple_info.field_len() == 1 {
                        let field_info = tuple_info.field_at(0);
                        if let Some(field) = field_info {
                            let field_type = field.type_path();
                            // Check if it's an Entity wrapper
                            if field_type.contains("Entity") {
                                debug!("[RUNTIME_ENTITY_WRAPPER] ✓ Registering entity wrapper: {} (field: {})", type_name, field_type);

                                // Clone what we need for the closure
                                let type_registry_clone = type_registry.clone();
                                let full_path = type_info.type_path().to_string();
                                let name_for_closure = type_name.to_string();

                                // Create a handler that uses reflection to create the component
                                let handler: Box<
                                    dyn Fn(&LuaValue, &mut EntityCommands) -> LuaResult<()>
                                        + Send
                                        + Sync,
                                > = Box::new(
                                    move |data: &LuaValue, entity_commands: &mut EntityCommands| {
                                        if let LuaValue::Table(table) = data {
                                            if let Ok(entity_id) = table.get::<u64>("entity") {
                                                // entity_spawner pre-resolves temp_ids -> entity bits via resolve_entity()
                                                // so entity_id here is already the full entity bits representation
                                                let target_entity = Entity::from_bits(entity_id);
                                                debug!("[RUNTIME_ENTITY_WRAPPER] {} targeting entity bits {} -> {:?}", 
                                                    name_for_closure, entity_id, target_entity);

                                                // Create a DynamicTupleStruct with the Entity value
                                                // Entity wrappers don't implement Default, so we can't use ReflectDefault
                                                // Instead, we build the tuple struct dynamically with just the Entity field
                                                use bevy::reflect::DynamicTupleStruct;

                                                // Create dynamic tuple struct with the Entity as the only field
                                                let dynamic_tuple = DynamicTupleStruct::from_iter(
                                                    [Box::new(target_entity)
                                                        as Box<dyn bevy::reflect::PartialReflect>],
                                                );

                                                // Insert the component via reflection
                                                let full_path_clone = full_path.clone();
                                                let type_registry_for_cmd =
                                                    type_registry_clone.clone();
                                                entity_commands.queue(
                                                    move |entity: bevy::prelude::EntityWorldMut| {
                                                        let mut entity = entity;
                                                        let reg = type_registry_for_cmd.read();
                                                        if let Some(registration) =
                                                            reg.get_with_type_path(&full_path_clone)
                                                        {
                                                            if let Some(reflect_component) =
                                                                registration
                                                                    .data::<ReflectComponent>()
                                                            {
                                                                reflect_component.insert(
                                                                    &mut entity,
                                                                    &dynamic_tuple,
                                                                    &reg,
                                                                );
                                                            }
                                                        }
                                                    },
                                                );
                                                return Ok(());
                                            }
                                        }
                                        Err(mlua::Error::RuntimeError(format!(
                                            "Failed to create {} from Lua. Expected table with 'entity' field.", 
                                            name_for_closure
                                        )))
                                    },
                                );

                                component_registry
                                    .handlers
                                    .insert(type_name.to_string(), handler);
                                found = true;
                                break;
                            }
                        }
                    }
                }
            }
        }

        if !found {
            // This is expected - many discovered types aren't in TypeRegistry
            // (internal types, types not registered, etc.)
            debug!(
                "[RUNTIME_ENTITY_WRAPPER] Type not found or not usable: {}",
                type_name
            );
        }
    }
}

/// Spawn component using Bevy's reflection system (with serde fallback)
fn spawn_component_via_reflection(
    data: &LuaValue,
    entity: &mut EntityCommands,
    type_path: &str,
    type_registry: &AppTypeRegistry,
    asset_registry: Option<&crate::asset_loading::AssetRegistry>,
) -> LuaResult<()> {
    let registry = type_registry.read();

    // Get type registration
    let Some(registration) = registry.get_with_type_path(type_path) else {
        return Err(LuaError::RuntimeError(format!(
            "Type not found: {}",
            type_path
        )));
    };

    // PATH 1: Try Reflect-based creation (existing system)
    if let Some(reflect_default) = registration.data::<ReflectDefault>() {
        let mut component = reflect_default.default();
        let type_info = registration.type_info();

        // Patch component with Lua data
        match type_info {
            TypeInfo::Struct(struct_info) => {
                // Structs require a table
                let data_table = match data {
                    LuaValue::Table(t) => t,
                    _ => {
                        return Err(LuaError::RuntimeError(format!(
                            "{} requires a table, got {:?}",
                            type_path, data
                        )))
                    }
                };

                // Get mutable reflection
                let reflect_mut = component.reflect_mut();

                // Pattern match to get struct
                if let ReflectMut::Struct(struct_mut) = reflect_mut {
                    // Iterate through struct fields
                    for i in 0..struct_info.field_len() {
                        let field_info = struct_info.field_at(i).unwrap();
                        let field_name = field_info.name();

                        // Try to get value from Lua table
                        if let Ok(lua_value) = data_table.get::<LuaValue>(field_name) {
                            // Get mutable field
                            if let Some(field) = struct_mut.field_at_mut(i) {
                                set_field_from_lua(
                                    field,
                                    &lua_value,
                                    asset_registry,
                                    type_registry,
                                    Some(field_name),
                                )?;
                            }
                        }
                    }
                }
            }

            TypeInfo::TupleStruct(struct_info) => {
                debug!(
                    "[COMPONENT_SPAWN] TupleStruct detected: {} with {} fields",
                    type_path,
                    struct_info.field_len()
                );
                // Special case: single-field tuple struct (newtype wrapper) with scalar value
                // This handles types like GravityScale(f32), Restitution { coefficient: f32, ... }
                // where Lua can just pass a number instead of a table
                if struct_info.field_len() == 1 {
                    // Try to handle as scalar newtype wrapper first
                    if !matches!(data, LuaValue::Table(_)) {
                        let reflect_mut = component.reflect_mut();
                        if let ReflectMut::TupleStruct(tuple_mut) = reflect_mut {
                            if let Some(field) = tuple_mut.field_mut(0) {
                                if set_field_from_lua(
                                    field,
                                    data,
                                    asset_registry,
                                    type_registry,
                                    Some("_0"),
                                )
                                .is_ok()
                                {
                                    // Successfully set the field from scalar value
                                    entity.insert_reflect(component);
                                    return Ok(());
                                }
                            }
                        }
                    }
                }

                // Tuple structs require a table for non-scalar cases
                let data_table = match data {
                    LuaValue::Table(t) => t,
                    _ => {
                        return Err(LuaError::RuntimeError(format!(
                            "{} requires a table, got {:?}",
                            type_path, data
                        )))
                    }
                };

                // Handle single-field tuple structs like Text(String)
                // Try to get the value from common keys
                // Handle single-field tuple structs like Text(String)
                // Try to get the value from common keys
                let lua_value: LuaValue = 'search: {
                    // Check "_0" (reflection-style field name)
                    if let Ok(val) = data_table.raw_get::<LuaValue>("_0") {
                        if !matches!(val, LuaValue::Nil) {
                            debug!(
                                "[COMPONENT_SPAWN] Found _0 field for {}: {:?}",
                                type_path, val
                            );
                            break 'search val;
                        }
                    }

                    // Check "id" (common for handle-wrapping newtypes like Mesh3d, MeshMaterial3d)
                    if let Ok(val) = data_table.raw_get::<LuaValue>("id") {
                        if !matches!(val, LuaValue::Nil) {
                            debug!(
                                "[COMPONENT_SPAWN] Found id field for {}: {:?}",
                                type_path, val
                            );
                            break 'search val;
                        }
                    }

                    // Check "value"
                    if let Ok(val) = data_table.raw_get::<LuaValue>("value") {
                        if !matches!(val, LuaValue::Nil) {
                            break 'search val;
                        }
                    }

                    // Check "0"
                    if let Ok(val) = data_table.raw_get::<LuaValue>("0") {
                        if !matches!(val, LuaValue::Nil) {
                            break 'search val;
                        }
                    }

                    // Check index 1
                    if let Ok(val) = data_table.get::<LuaValue>(1) {
                        if !matches!(val, LuaValue::Nil) {
                            break 'search val;
                        }
                    }

                    // Fallback: if the tuple struct has 1 field, check if we can find a single value
                    if struct_info.field_len() == 1 {
                        let mut pairs = data_table.pairs::<LuaValue, LuaValue>();

                        // Get first pair
                        if let Some(Ok((_, val))) = pairs.next() {
                            // Ensure there is no second pair (to avoid ambiguity)
                            if pairs.next().is_none() {
                                break 'search val;
                            } else {
                                return Err(LuaError::RuntimeError(
                                    "Ambiguous tuple struct data: multiple keys found".to_string(),
                                ));
                            }
                        }
                    }

                    LuaValue::Nil
                };

                if matches!(lua_value, LuaValue::Nil) {
                    debug!(
                        "[COMPONENT_SPAWN] ERROR: Failed to access tuple struct data for {}",
                        type_path
                    );
                    return Err(LuaError::RuntimeError(
                        "Failed to access tuple struct data".to_string(),
                    ));
                }

                let reflect_mut = component.reflect_mut();
                if let ReflectMut::TupleStruct(tuple_mut) = reflect_mut {
                    if let Some(field) = tuple_mut.field_mut(0) {
                        debug!("[COMPONENT_SPAWN] Setting tuple struct field 0 for {} with value: {:?}", type_path, lua_value);
                        set_field_from_lua(
                            field,
                            &lua_value,
                            asset_registry,
                            type_registry,
                            Some("_0"),
                        )?;
                        debug!(
                            "[COMPONENT_SPAWN] Successfully set tuple struct field for {}",
                            type_path
                        );

                        // Debug: Print the actual field value after setting
                        if type_path.contains("Mesh3d") || type_path.contains("MeshMaterial3d") {
                            debug!(
                                "[COMPONENT_SPAWN] DEBUG: Field value after setting: {:?}",
                                field
                            );
                        }
                    }
                }
            }

            TypeInfo::Enum(enum_info) => {
                // Enums can be specified as:
                // 1. Strings for unit variants: "Relative"
                // 2. Tables for tuple/struct variants: { Px = 200 } or { SomeVariant = { field = value } }

                use bevy::reflect::{DynamicEnum, DynamicStruct, DynamicTuple, DynamicVariant};

                let (variant_name, variant_data) = match data {
                    LuaValue::String(s) => {
                        // Unit variant
                        (s.to_str()?.to_string(), None)
                    }
                    LuaValue::Table(t) => {
                        // Tuple or struct variant
                        // The table should have exactly one key (the variant name)
                        let mut pairs = t.pairs::<String, LuaValue>();

                        if let Some(Ok((variant_name, variant_value))) = pairs.next() {
                            // Ensure there's only one key
                            if pairs.next().is_some() {
                                return Err(LuaError::RuntimeError(format!(
                                    "Enum table must have exactly one key (the variant name)"
                                )));
                            }
                            (variant_name, Some(variant_value))
                        } else {
                            return Err(LuaError::RuntimeError(format!(
                                "{} enum requires a variant name",
                                type_path
                            )));
                        }
                    }
                    _ => {
                        return Err(LuaError::RuntimeError(format!(
                            "{} enum requires a string or table, got {:?}",
                            type_path, data
                        )))
                    }
                };

                // Find matching variant
                let variant_info = enum_info.variant(&variant_name).ok_or_else(|| {
                    LuaError::RuntimeError(format!(
                        "Unknown variant '{}' for enum {}. Available variants: {}",
                        variant_name,
                        type_path,
                        enum_info
                            .iter()
                            .map(|v| v.name())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                })?;

                let dynamic_variant = match variant_info {
                    bevy::reflect::VariantInfo::Unit(_) => {
                        // Unit variant
                        if variant_data.is_some() {
                            return Err(LuaError::RuntimeError(format!(
                                "Variant '{}' is a unit variant and does not accept data",
                                variant_name
                            )));
                        }
                        DynamicVariant::Unit
                    }
                    bevy::reflect::VariantInfo::Tuple(tuple_info) => {
                        // Tuple variant like Val::Px(200.0) or RenderTarget::Image(ImageRenderTarget)
                        let data_value = variant_data.ok_or_else(|| {
                            LuaError::RuntimeError(format!(
                                "Variant '{}' is a tuple variant and requires data",
                                variant_name
                            ))
                        })?;

                        let mut dynamic_tuple = DynamicTuple::default();

                        // Handle single-field tuple variants (most common case)
                        if tuple_info.field_len() == 1 {
                            // Check if this is a newtype wrapper using type_info
                            // Handles both TupleStruct newtypes and Struct newtypes (like ImageRenderTarget)
                            // Returns (handle_inner_type, newtype_type_info, field_name, newtype_type_path)
                            let field_info = tuple_info.field_at(0);
                            let newtype_info =
                                field_info.and_then(|f| f.type_info()).and_then(|ti| {
                                    match ti {
                                        bevy::reflect::TypeInfo::TupleStruct(ts) => {
                                            if ts.field_len() == 1 {
                                                let inner = ts
                                                    .field_at(0)
                                                    .map(|inner_f| inner_f.type_path().to_string());
                                                let newtype_path = ti.type_path().to_string();
                                                Some((
                                                    inner,
                                                    Some(ti),
                                                    Some("0".to_string()),
                                                    Some(newtype_path),
                                                ))
                                            } else {
                                                None
                                            }
                                        }
                                        bevy::reflect::TypeInfo::Struct(s) => {
                                            // Check if any field is a Handle<T>
                                            let handle_field_info =
                                                (0..s.field_len()).find_map(|i| {
                                                    s.field_at(i)
                                                        .filter(|f| {
                                                            f.type_path().contains("Handle<")
                                                        })
                                                        .map(|f| {
                                                            (
                                                                f.type_path().to_string(),
                                                                f.name().to_string(),
                                                            )
                                                        })
                                                });

                                            if let Some((inner, field_name)) = handle_field_info {
                                                let newtype_path = ti.type_path().to_string();
                                                Some((
                                                    Some(inner),
                                                    Some(ti),
                                                    Some(field_name),
                                                    Some(newtype_path),
                                                ))
                                            } else {
                                                None
                                            }
                                        }
                                        _ => None,
                                    }
                                });

                            let (
                                newtype_inner_type,
                                newtype_type_info,
                                newtype_field_name,
                                newtype_type_path,
                            ) = newtype_info.unwrap_or((None, None, None, None));
                            let is_newtype_wrapper = newtype_inner_type.is_some();
                            let _ = (newtype_type_info, newtype_field_name); // silence unused warnings

                            // The value can be a scalar or table
                            match &data_value {
                                LuaValue::Number(n) => {
                                    if is_newtype_wrapper {
                                        // This is a newtype-wrapped variant, treat number as asset ID
                                        debug!("[ENUM_GENERIC] Detected newtype with inner type '{:?}', wrapping handle ID {}", 
                                            newtype_inner_type, *n);
                                        if let Some(asset_reg) = asset_registry {
                                            if let Some(untyped_handle) =
                                                asset_reg.get_untyped_handle(*n as u32)
                                            {
                                                // First create typed Handle<T> from UntypedHandle
                                                let handle_box = newtype_inner_type
                                                    .as_ref()
                                                    .and_then(|inner_type| {
                                                        asset_reg.try_create_typed_handle_box(
                                                            inner_type,
                                                            untyped_handle.clone(),
                                                        )
                                                    });

                                                // Then try to wrap in newtype (e.g., Handle<Image> -> ImageRenderTarget)
                                                if let Some(typed_handle) = handle_box {
                                                    // Try to wrap in newtype if a wrapper is registered
                                                    if let Some(ref newtype_path) =
                                                        newtype_type_path
                                                    {
                                                        if let Some(wrapped) = asset_reg
                                                            .try_wrap_in_newtype_with_reflection(
                                                                newtype_path,
                                                                typed_handle,
                                                                type_registry,
                                                            )
                                                        {
                                                            debug!("[ENUM_GENERIC] ✓ Wrapped Handle in newtype '{}'", newtype_path);
                                                            dynamic_tuple.insert_boxed(wrapped);
                                                        } else {
                                                            // Reflection fallback failed too
                                                            debug!("[ENUM_GENERIC] ⚠ Failed to wrap in newtype '{}' even with reflection", newtype_path);
                                                            return Err(LuaError::RuntimeError(format!(
                                                                "Failed to wrap in newtype '{}'. Ensure type has #[derive(Reflect)] with FromReflect", newtype_path
                                                            )));
                                                        }
                                                    } else {
                                                        debug!("[ENUM_GENERIC] ⚠ No newtype path, inserting typed Handle directly");
                                                        return Err(LuaError::RuntimeError(
                                                            "Newtype wrapper detection failed - no type path available".to_string()
                                                        ));
                                                    }
                                                } else {
                                                    debug!("[ENUM_GENERIC] ⚠ Failed to create typed handle");
                                                    return Err(LuaError::RuntimeError(format!(
                                                        "Failed to create typed handle for inner type {:?}", newtype_inner_type
                                                    )));
                                                }
                                            } else {
                                                return Err(LuaError::RuntimeError(format!(
                                                    "Asset ID {} not found",
                                                    *n as u32
                                                )));
                                            }
                                        } else {
                                            return Err(LuaError::RuntimeError(
                                                "Asset registry not available".to_string(),
                                            ));
                                        }
                                    } else {
                                        dynamic_tuple.insert(*n as f32);
                                    }
                                }
                                LuaValue::Integer(i) => {
                                    if is_newtype_wrapper {
                                        // This is a newtype-wrapped variant, treat integer as asset ID
                                        debug!("[ENUM_GENERIC] Detected newtype with inner type '{:?}', wrapping handle ID {}", 
                                            newtype_inner_type, *i);
                                        if let Some(asset_reg) = asset_registry {
                                            if let Some(untyped_handle) =
                                                asset_reg.get_untyped_handle(*i as u32)
                                            {
                                                // First create typed Handle<T> from UntypedHandle
                                                let handle_box = newtype_inner_type
                                                    .as_ref()
                                                    .and_then(|inner_type| {
                                                        asset_reg.try_create_typed_handle_box(
                                                            inner_type,
                                                            untyped_handle.clone(),
                                                        )
                                                    });

                                                // Then try to wrap in newtype (e.g., Handle<Image> -> ImageRenderTarget)
                                                if let Some(typed_handle) = handle_box {
                                                    // Try to wrap in newtype if a wrapper is registered
                                                    if let Some(ref newtype_path) =
                                                        newtype_type_path
                                                    {
                                                        if let Some(wrapped) = asset_reg
                                                            .try_wrap_in_newtype_with_reflection(
                                                                newtype_path,
                                                                typed_handle,
                                                                type_registry,
                                                            )
                                                        {
                                                            debug!("[ENUM_GENERIC] ✓ Wrapped Handle in newtype '{}'", newtype_path);
                                                            dynamic_tuple.insert_boxed(wrapped);
                                                        } else {
                                                            return Err(LuaError::RuntimeError(format!(
                                                                "Failed to wrap in newtype '{}'. Ensure type has #[derive(Reflect)] with FromReflect", newtype_path
                                                            )));
                                                        }
                                                    } else {
                                                        return Err(LuaError::RuntimeError(
                                                            "Newtype wrapper detection failed - no type path available".to_string()
                                                        ));
                                                    }
                                                } else {
                                                    return Err(LuaError::RuntimeError(format!(
                                                        "Failed to create typed handle for inner type {:?}", newtype_inner_type
                                                    )));
                                                }
                                            } else {
                                                return Err(LuaError::RuntimeError(format!(
                                                    "Asset ID {} not found",
                                                    *i as u32
                                                )));
                                            }
                                        } else {
                                            return Err(LuaError::RuntimeError(
                                                "Asset registry not available".to_string(),
                                            ));
                                        }
                                    } else {
                                        // Check if the field type is Uuid (for PointerId::Custom, etc.)
                                        let field_type_path = tuple_info
                                            .field_at(0)
                                            .map(|f| f.type_path().to_string())
                                            .unwrap_or_default();

                                        if field_type_path == "uuid::Uuid" {
                                            // Construct Uuid from integer
                                            let uuid = uuid::Uuid::from_u128(*i as u128);
                                            debug!("[ENUM_GENERIC] Constructed Uuid from integer for variant: {:?}", uuid);
                                            dynamic_tuple.insert_boxed(Box::new(uuid));
                                        } else {
                                            dynamic_tuple.insert(*i as f32);
                                        }
                                    }
                                }
                                LuaValue::Boolean(b) => {
                                    dynamic_tuple.insert(*b);
                                }
                                LuaValue::String(s) => {
                                    dynamic_tuple.insert(s.to_str()?.to_string());
                                }
                                LuaValue::Table(nested_table) => {
                                    // Try to create the inner value from the table
                                    if let Some(inner_value) =
                                        try_create_value_from_table(nested_table, asset_registry)?
                                    {
                                        dynamic_tuple.insert_boxed(inner_value);
                                    } else {
                                        return Err(LuaError::RuntimeError(format!(
                                            "Failed to create value for tuple variant '{}'",
                                            variant_name
                                        )));
                                    }
                                }
                                _ => {
                                    return Err(LuaError::RuntimeError(format!(
                                        "Unsupported value type for tuple variant '{}'",
                                        variant_name
                                    )));
                                }
                            }
                        } else {
                            // Multi-field tuple variant (less common)
                            return Err(LuaError::RuntimeError(format!(
                                "Multi-field tuple variants are not yet supported for '{}'",
                                variant_name
                            )));
                        }

                        DynamicVariant::Tuple(dynamic_tuple)
                    }
                    bevy::reflect::VariantInfo::Struct(struct_info) => {
                        // Struct variant like SomeEnum::StructVariant { field: value }
                        let data_table = match variant_data {
                            Some(LuaValue::Table(t)) => t,
                            _ => {
                                return Err(LuaError::RuntimeError(format!(
                                    "Variant '{}' is a struct variant and requires a table",
                                    variant_name
                                )))
                            }
                        };

                        let mut dynamic_struct = DynamicStruct::default();

                        // Populate struct fields
                        for i in 0..struct_info.field_len() {
                            if let Some(field_info) = struct_info.field_at(i) {
                                let field_name = field_info.name();

                                if let Ok(lua_value) = data_table.get::<LuaValue>(field_name) {
                                    // Convert Lua value to appropriate Rust type
                                    match &lua_value {
                                        LuaValue::Number(n) => {
                                            dynamic_struct
                                                .insert_boxed(field_name, Box::new(*n as f32));
                                        }
                                        LuaValue::Integer(i) => {
                                            dynamic_struct
                                                .insert_boxed(field_name, Box::new(*i as i32));
                                        }
                                        LuaValue::Boolean(b) => {
                                            dynamic_struct.insert_boxed(field_name, Box::new(*b));
                                        }
                                        LuaValue::String(s) => {
                                            dynamic_struct.insert_boxed(
                                                field_name,
                                                Box::new(s.to_str()?.to_string()),
                                            );
                                        }
                                        LuaValue::Table(nested_table) => {
                                            if let Some(inner_value) = try_create_value_from_table(
                                                nested_table,
                                                asset_registry,
                                            )? {
                                                dynamic_struct
                                                    .insert_boxed(field_name, inner_value);
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        DynamicVariant::Struct(dynamic_struct)
                    }
                };

                let dynamic_enum = DynamicEnum::new(&variant_name, dynamic_variant);

                // Convert to concrete type via ReflectFromReflect
                let reflect_from_reflect = registration
                    .data::<bevy::reflect::ReflectFromReflect>()
                    .ok_or_else(|| {
                        LuaError::RuntimeError(format!(
                            "{} doesn't implement FromReflect",
                            type_path
                        ))
                    })?;

                let new_component = reflect_from_reflect
                    .from_reflect(&dynamic_enum)
                    .ok_or_else(|| {
                        LuaError::RuntimeError(format!(
                            "Failed to create {} from reflection.",
                            type_path
                        ))
                    })?;

                component.apply(new_component.as_ref());
            }

            _ => {
                return Err(LuaError::RuntimeError(format!(
                    "Unsupported type: {}",
                    type_path
                )));
            }
        }

        // Insert component via reflection
        debug!(
            "[COMPONENT_SPAWN] ✓ Calling insert_reflect for component: {}",
            type_path
        );
        entity.insert_reflect(component);
        return Ok(());
    }

    // PATH 2: Try Serde deserialization (fallback for non-Reflect types like Collider)
    // Note: This path is for types that ARE in the TypeRegistry (implement Reflect)
    // but use serde for deserialization. Collider is NOT in TypeRegistry, so it
    // will be handled by SerdeComponentRegistry in entity_spawner.rs instead.

    // Neither Reflect nor Serde available
    Err(LuaError::RuntimeError(format!(
        "{} doesn't implement Reflect (with Default). Cannot create from Lua via reflection.",
        type_path
    )))
}

/// Set a reflected field value from a Lua value
/// Set a reflected field value from a Lua value
pub fn set_field_from_lua(
    field: &mut dyn PartialReflect,
    lua_value: &LuaValue,
    asset_registry: Option<&crate::asset_loading::AssetRegistry>,
    type_registry: &bevy::prelude::AppTypeRegistry,
    field_name: Option<&str>,
) -> LuaResult<()> {
    // Fully generic Handle<T> resolution using type-erased handle setters!
    // The AssetRegistry was populated at startup with setters for all asset types in TypeRegistry.
    let type_path = field.reflect_type_path().to_string();
    let field_name_str = field_name.unwrap_or("<unknown>");
    debug!(
        "[FIELD_SET] Setting field '{}' of type: {}, lua_value: {:?}",
        field_name_str, type_path, lua_value
    );
    if type_path.contains("Handle<") {
        debug!("[FIELD_SET] Handle detected: {}", type_path);
        if let LuaValue::Integer(asset_id) = lua_value {
            if let Some(registry) = asset_registry {
                debug!("[FIELD_SET] Looking up asset ID {} in registry", asset_id);
                // Try using the generic handle setter system
                if let Some(untyped_handle) = registry.get_untyped_handle(*asset_id as u32) {
                    debug!(
                        "[FIELD_SET] Found untyped handle for asset ID {}: {:?}",
                        asset_id,
                        untyped_handle.id()
                    );
                    if registry.try_set_handle_field(field, &type_path, untyped_handle.clone()) {
                        debug!(
                            "[FIELD_SET] ✓ Successfully set handle field {} with asset ID {}",
                            type_path, asset_id
                        );
                        return Ok(());
                    } else {
                        debug!(
                            "[FIELD_SET] WARNING: try_set_handle_field failed for {}",
                            type_path
                        );
                    }
                } else {
                    // Handle not found - try loading from registered path
                    debug!(
                        "[FIELD_SET] Asset ID {} not in handle registry, trying path-based loading",
                        asset_id
                    );
                    if let Some(path) = registry.get_path(*asset_id as u32) {
                        debug!("[FIELD_SET] Found path '{}' for asset ID {}", path, asset_id);
                        if let Some(ref asset_server) = registry.asset_server {
                            if let Some(handle) = registry.try_load_from_path(&path, &type_path, asset_server) {
                                debug!("[FIELD_SET] ✓ Loaded from path '{}' -> {:?}", path, handle.id());
                                // Register the handle for future lookups
                                registry.register_handle_with_id(*asset_id as u32, handle.clone());
                                if registry.try_set_handle_field(field, &type_path, handle) {
                                    return Ok(());
                                }
                            } else {
                                debug!("[FIELD_SET] No typed loader for type '{}'", type_path);
                            }
                        } else {
                            debug!("[FIELD_SET] AssetServer not available in registry");
                        }
                    } else {
                        debug!("[FIELD_SET] ERROR: No path registered for asset ID {}", asset_id);
                    }
                }

                // Fallback: image_handles (for loaded images via load_asset)
                if let Some(image_handle) = registry.get_image_handle(*asset_id as u32) {
                    if let Some(handle_field) = field.try_downcast_mut::<Handle<Image>>() {
                        *handle_field = image_handle;
                        return Ok(());
                    }
                }
            }
        }
    }

    // NOTE: Enum types (including RenderTarget) are handled generically via try_construct_enum_variant
    // which detects newtype wrappers and handles asset IDs automatically

    // Try to downcast to common types
    if let Some(f32_field) = field.try_downcast_mut::<f32>() {
        match lua_value {
            LuaValue::Number(n) => {
                *f32_field = *n as f32;
            }
            LuaValue::Integer(i) => {
                *f32_field = *i as f32;
            }
            _ => {}
        }
    } else if let Some(i32_field) = field.try_downcast_mut::<i32>() {
        if let LuaValue::Integer(i) = lua_value {
            *i32_field = *i as i32;
        }
    } else if let Some(isize_field) = field.try_downcast_mut::<isize>() {
        // Camera::order is isize - critical for RTT (needs order = -1)
        if let LuaValue::Integer(i) = lua_value {
            *isize_field = *i as isize;
            debug!("[FIELD_SET] ✓ Set isize field to {}", *i);
        }
    } else if let Some(usize_field) = field.try_downcast_mut::<usize>() {
        if let LuaValue::Integer(i) = lua_value {
            *usize_field = *i as usize;
        }
    } else if let Some(string_field) = field.try_downcast_mut::<String>() {
        if let LuaValue::String(s) = lua_value {
            *string_field = s.to_str()?.to_string();
        }
    } else if let Some(bool_field) = field.try_downcast_mut::<bool>() {
        if let LuaValue::Boolean(b) = lua_value {
            *bool_field = *b;
        }
    } else if let Some(entity_field) = field.try_downcast_mut::<Entity>() {
        // Handle Entity field updates (e.g., UiTargetCamera)
        // Entity is passed as u64 bits from Lua
        match lua_value {
            LuaValue::Integer(i) => {
                let entity = Entity::from_bits(*i as u64);
                *entity_field = entity;
                debug!("[FIELD_SET] ✓ Set Entity field from bits {} -> {:?}", i, entity);
            }
            LuaValue::Number(n) => {
                let entity = Entity::from_bits(*n as u64);
                *entity_field = entity;
                debug!("[FIELD_SET] ✓ Set Entity field from bits {} -> {:?}", n, entity);
            }
            _ => {
                warn!("[FIELD_SET] Entity field expected integer, got: {:?}", lua_value);
            }
        }
    } else if let Some(color_field) = field.try_downcast_mut::<Color>() {
        if let LuaValue::Table(color_table) = lua_value {
            let r: f32 = color_table.get("r").unwrap_or(1.0);
            let g: f32 = color_table.get("g").unwrap_or(1.0);
            let b: f32 = color_table.get("b").unwrap_or(1.0);
            let a: f32 = color_table.get("a").unwrap_or(1.0);
            debug!(
                "[COLOR_SET] Setting Color from table: r={}, g={}, b={}, a={}",
                r, g, b, a
            );
            *color_field = Color::srgba(r, g, b, a);
        } else {
            warn!("[COLOR_SET] Expected Table for Color, got: {:?}", lua_value);
        }
    } else if let Some(vec3_field) = field.try_downcast_mut::<Vec3>() {
        if let LuaValue::Table(vec_table) = lua_value {
            let x: f32 = vec_table.get("x").unwrap_or(0.0);
            let y: f32 = vec_table.get("y").unwrap_or(0.0);
            let z: f32 = vec_table.get("z").unwrap_or(0.0);
            *vec3_field = Vec3::new(x, y, z);
        }
    } else if let Some(vec2_field) = field.try_downcast_mut::<Vec2>() {
        if let LuaValue::Table(vec_table) = lua_value {
            let x: f32 = vec_table.get("x").unwrap_or(0.0);
            let y: f32 = vec_table.get("y").unwrap_or(0.0);
            *vec2_field = Vec2::new(x, y);
        }
    } else if let Some(quat_field) = field.try_downcast_mut::<Quat>() {
        if let LuaValue::Table(quat_table) = lua_value {
            let x: f32 = quat_table.get("x").unwrap_or(0.0);
            let y: f32 = quat_table.get("y").unwrap_or(0.0);
            let z: f32 = quat_table.get("z").unwrap_or(0.0);
            let w: f32 = quat_table.get("w").unwrap_or(1.0);
            *quat_field = Quat::from_xyzw(x, y, z, w);
        }
    } else if let LuaValue::Table(nested_table) = lua_value {
        // Generic nested struct/enum handling using reflection
        if let Err(e) =
            set_nested_field_from_lua(field, nested_table, asset_registry, type_registry)
        {
            // Silently continue if nested field setting fails - might not be a nested struct
            let _ = e;
        }
    } else if let LuaValue::String(variant_name_str) = lua_value {
        // Check if the target field is an enum type - handle string as unit variant name
        // This supports patterns like: flex_direction = "Column", display = "Flex", align_items = "Center"
        use bevy::reflect::{DynamicEnum, DynamicVariant, ReflectMut};
        
        if let ReflectMut::Enum(_) = field.reflect_mut() {
            let variant_name = variant_name_str.to_str()?;
            let type_path = field.reflect_type_path().to_string();
            
            debug!("[FIELD_SET_ENUM] Field '{}' is enum type '{}', trying variant '{}'", 
                   field_name_str, type_path, variant_name);
            
            let registry = type_registry.read();
            if let Some(registration) = registry.get_with_type_path(&type_path) {
                if let Some(reflect_from_reflect) = registration.data::<bevy::reflect::ReflectFromReflect>() {
                    // Create a DynamicEnum with a unit variant
                    let dynamic_enum = DynamicEnum::new(variant_name.to_string(), DynamicVariant::Unit);
                    
                    if let Some(concrete) = reflect_from_reflect.from_reflect(&dynamic_enum) {
                        field.apply(concrete.as_ref());
                        debug!("[FIELD_SET_ENUM] ✓ Applied enum variant '{}' to field '{}'", variant_name, field_name_str);
                        return Ok(());
                    } else {
                        warn!("[FIELD_SET_ENUM] Failed to create enum '{}' with variant '{}' - from_reflect returned None", 
                              type_path, variant_name);
                    }
                } else {
                    debug!("[FIELD_SET_ENUM] Type '{}' doesn't have ReflectFromReflect", type_path);
                }
            } else {
                debug!("[FIELD_SET_ENUM] Type '{}' not found in registry", type_path);
            }
        }
    }

    Ok(())
}

/// Try to create a reflected value from a Lua table generically
/// Uses reflection to automatically populate any struct from Lua tables
/// Supports both array format {1, 2, 3} and object format {x=1, y=2, z=3}
fn try_create_value_from_table(
    table: &LuaTable,
    asset_registry: Option<&crate::asset_loading::AssetRegistry>,
) -> LuaResult<Option<Box<dyn PartialReflect>>> {
    use bevy::reflect::{DynamicStruct, Struct};

    // Create a dynamic struct and populate it from the table
    let mut dynamic_struct = DynamicStruct::default();

    // Try array-style access first (for tuple-like tables: {50, 50})
    let mut array_values = Vec::new();
    for i in 1..=10 {
        // Check up to 10 elements
        match table.get::<LuaValue>(i) {
            Ok(LuaValue::Number(n)) => array_values.push(n as f32),
            Ok(LuaValue::Integer(n)) => array_values.push(n as f32),
            _ => break,
        }
    }

    // If we found array values, try to map them to common struct patterns
    if !array_values.is_empty() {
        match array_values.len() {
            2 => {
                // Could be Vec2, UVec2, IVec2, etc.
                dynamic_struct.insert("x", array_values[0]);
                dynamic_struct.insert("y", array_values[1]);
            }
            3 => {
                // Could be Vec3, UVec3, IVec3, Color (RGB), etc.
                dynamic_struct.insert("x", array_values[0]);
                dynamic_struct.insert("y", array_values[1]);
                dynamic_struct.insert("z", array_values[2]);
            }
            4 => {
                // Could be Vec4, Quat, Color (RGBA), etc.
                dynamic_struct.insert("x", array_values[0]);
                dynamic_struct.insert("y", array_values[1]);
                dynamic_struct.insert("z", array_values[2]);
                dynamic_struct.insert("w", array_values[3]);
            }
            _ => {}
        }
    }

    // Object-style access: populate all named fields from the table
    for pair in table.pairs::<String, LuaValue>() {
        if let Ok((key, value)) = pair {
            match value {
                LuaValue::Number(n) => {
                    dynamic_struct.insert_boxed(&key, Box::new(n as f32));
                }
                LuaValue::Integer(i) => {
                    // Try as both i32 and f32 since we don't know the target type
                    dynamic_struct.insert_boxed(&key, Box::new(i as f32));
                }
                LuaValue::Boolean(b) => {
                    dynamic_struct.insert_boxed(&key, Box::new(b));
                }
                LuaValue::String(s) => {
                    if let Ok(string) = s.to_str() {
                        dynamic_struct.insert_boxed(&key, Box::new(string.to_string()));
                    }
                }
                LuaValue::Table(nested_table) => {
                    // Recursively handle nested tables
                    if let Ok(Some(nested_value)) =
                        try_create_value_from_table(&nested_table, asset_registry)
                    {
                        dynamic_struct.insert_boxed(&key, nested_value);
                    }
                }
                _ => {}
            }
        }
    }

    if dynamic_struct.field_len() > 0 {
        Ok(Some(Box::new(dynamic_struct)))
    } else {
        Ok(None)
    }
}

/// Generic handler for nested structs and enums using reflection
fn set_nested_field_from_lua(
    field: &mut dyn PartialReflect,
    table: &LuaTable,
    asset_registry: Option<&crate::asset_loading::AssetRegistry>,
    type_registry: &bevy::prelude::AppTypeRegistry,
) -> LuaResult<()> {
    use bevy::reflect::{DynamicEnum, DynamicStruct, DynamicTuple, DynamicVariant};

    match field.reflect_mut() {
        ReflectMut::Struct(struct_mut) => {
            // Set each field in the struct from the table
            for i in 0..struct_mut.field_len() {
                if let Some(field_name) = struct_mut.name_at(i) {
                    let field_name_owned = field_name.to_string();
                    if let Ok(lua_value) = table.get::<LuaValue>(field_name) {
                        if let Some(nested_field) = struct_mut.field_at_mut(i) {
                            set_field_from_lua(
                                nested_field,
                                &lua_value,
                                asset_registry,
                                type_registry,
                                Some(&field_name_owned),
                            )?;
                        }
                    }
                }
            }
        }
        ReflectMut::Enum(_enum_mut) => {
            // Get the type info to understand the enum structure
            let type_path = field.reflect_type_path().to_string();

            // For Option types, we want to create Some(inner_value)
            // The table represents the inner value (e.g., Rect { min, max })
            if type_path.contains("Option<") {
                // Try to create the inner value generically from the table
                if let Some(inner_value) = try_create_value_from_table(table, asset_registry)? {
                    // Wrap in Some variant
                    let mut some_tuple = DynamicTuple::default();
                    some_tuple.insert_boxed(inner_value);
                    let some_variant = DynamicVariant::Tuple(some_tuple);
                    let some_enum = DynamicEnum::new("Some", some_variant);
                    field.apply(&some_enum);
                    return Ok(());
                }

                // Fallback to struct-based approach for complex types
                // Create a dynamic struct for the inner value
                let mut inner_struct = DynamicStruct::default();

                // Populate the inner struct from all table fields
                for pair in table.pairs::<String, LuaValue>() {
                    if let Ok((key, value)) = pair {
                        match &value {
                            LuaValue::Integer(i) => {
                                inner_struct.insert_boxed(&key, Box::new(*i as i32));
                            }
                            LuaValue::Number(n) => {
                                inner_struct.insert_boxed(&key, Box::new(*n as f32));
                            }
                            LuaValue::Boolean(b) => {
                                inner_struct.insert_boxed(&key, Box::new(*b));
                            }
                            LuaValue::String(s) => {
                                if let Ok(string) = s.to_str() {
                                    inner_struct.insert_boxed(&key, Box::new(string.to_string()));
                                }
                            }
                            LuaValue::Table(nested_table) => {
                                // Handle nested tables (e.g., min/max in Rect)
                                if key == "min" || key == "max" {
                                    // Try to create Vec2
                                    if let (Ok(x), Ok(y)) =
                                        (nested_table.get::<f32>("x"), nested_table.get::<f32>("y"))
                                    {
                                        inner_struct.insert_boxed(&key, Box::new(Vec2::new(x, y)));
                                    }
                                } else {
                                    // Generic nested struct handling
                                    let mut nested_struct = DynamicStruct::default();
                                    for nested_pair in nested_table.pairs::<String, LuaValue>() {
                                        if let Ok((nested_key, nested_value)) = nested_pair {
                                            match &nested_value {
                                                LuaValue::Number(n) => {
                                                    nested_struct.insert_boxed(
                                                        &nested_key,
                                                        Box::new(*n as f32),
                                                    );
                                                }
                                                LuaValue::Integer(i) => {
                                                    nested_struct.insert_boxed(
                                                        &nested_key,
                                                        Box::new(*i as i32),
                                                    );
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    inner_struct.insert_boxed(&key, Box::new(nested_struct));
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // Wrap in tuple for Some(.0)
                let mut some_tuple = DynamicTuple::default();
                some_tuple.insert_boxed(Box::new(inner_struct));

                let some_variant = DynamicVariant::Tuple(some_tuple);
                let some_enum = DynamicEnum::new("Some", some_variant);

                field.apply(&some_enum);
            } else {
                // Non-Option enum (like Val, RenderTarget) - create the enum variant from table
                // Table should have format: { VariantName = value }
                let mut pairs = table.pairs::<String, LuaValue>();

                if let Some(Ok((variant_name, variant_value))) = pairs.next() {
                    // Ensure there's only one key
                    if pairs.next().is_some() {
                        return Err(LuaError::RuntimeError(
                            "Enum table must have exactly one key (the variant name)".to_string(),
                        ));
                    }
                    // Get enum info to check if this variant's field is a newtype wrapper
                    // Returns (is_newtype, handle_inner_type_path, newtype_type_info, field_name, newtype_type_path)
                    let (
                        variant_is_newtype,
                        newtype_inner_type,
                        newtype_type_info,
                        newtype_field_name,
                        newtype_type_path,
                    ) = if let Some(TypeInfo::Enum(enum_info)) = field.get_represented_type_info() {
                        debug!(
                            "[ENUM_NEWTYPE] Found enum info for type '{}', checking variant '{}'",
                            enum_info.type_path(),
                            variant_name
                        );
                        // Debug: list all available variants
                        let all_variants: Vec<_> =
                            enum_info.iter().map(|v| v.name().to_string()).collect();
                        debug!("[ENUM_NEWTYPE]   Available variants: {:?}", all_variants);

                        // Try direct variant lookup first, fall back to iteration
                        let variant = enum_info.variant(&variant_name).or_else(|| {
                            // Fallback: search through all variants by name (case-insensitive)
                            debug!("[ENUM_NEWTYPE]   Direct lookup failed, trying iteration...");
                            enum_info
                                .iter()
                                .find(|v| v.name().eq_ignore_ascii_case(&variant_name))
                        });

                        variant.and_then(|v| {
                                    debug!("[ENUM_NEWTYPE]   Variant exists");
                                    match v {
                                        bevy::reflect::VariantInfo::Tuple(tv) => {
                                            debug!("[ENUM_NEWTYPE]   Is Tuple variant with {} fields", tv.field_len());
                                            if tv.field_len() == 1 {
                                                let field_type_path = tv.field_at(0).map(|f| f.type_path().to_string());
                                                debug!("[ENUM_NEWTYPE]   Field type path: {:?}", field_type_path);
                                                let type_info = tv.field_at(0).and_then(|f| f.type_info());
                                                debug!("[ENUM_NEWTYPE]   Has type_info: {}", type_info.is_some());
                                                // Check if the field itself is a TupleStruct with 1 field (newtype)
                                                // OR a Struct with 1 field whose type is Handle<T> (Bevy sometimes derives tuple structs as Struct)
                                                type_info.and_then(|ti| {
                                                    match ti {
                                                        TypeInfo::TupleStruct(ts) => {
                                                            debug!("[ENUM_NEWTYPE]   TypeInfo variant: TupleStruct (path: {})", ti.type_path());
                                                            if ts.field_len() == 1 {
                                                                let inner = ts.field_at(0).map(|inner_f| inner_f.type_path().to_string());
                                                                debug!("[ENUM_NEWTYPE]   Inner type path: {:?}", inner);
                                                                let newtype_path = ti.type_path().to_string();
                                                                // For TupleStruct, field name is "0"
                                                                Some((true, inner, Some(ti), Some("0".to_string()), Some(newtype_path)))
                                                            } else {
                                                                Some((false, None, None, None, None))
                                                            }
                                                        }
                                                        TypeInfo::Struct(s) => {
                                                            debug!("[ENUM_NEWTYPE]   TypeInfo variant: Struct (path: {})", ti.type_path());
                                                            debug!("[ENUM_NEWTYPE]   Struct field_len: {}", s.field_len());
                                                            // Find the Handle<T> field and get its name
                                                            let handle_field_info = (0..s.field_len())
                                                                .find_map(|i| s.field_at(i).filter(|f| f.type_path().contains("Handle<"))
                                                                    .map(|f| (f.type_path().to_string(), f.name().to_string())));
                                                            
                                                            if let Some((inner, field_name)) = handle_field_info {
                                                                debug!("[ENUM_NEWTYPE]   ✓ Detected Handle field '{}' with type '{}'", field_name, inner);
                                                                let newtype_path = ti.type_path().to_string();
                                                                Some((true, Some(inner), Some(ti), Some(field_name), Some(newtype_path)))
                                                            } else {
                                                                Some((false, None, None, None, None))
                                                            }
                                                        }
                                                        _ => {
                                                            debug!("[ENUM_NEWTYPE]   TypeInfo variant: {:?} (not TupleStruct or Struct)", ti.type_path());
                                                            Some((false, None, None, None, None))
                                                        }
                                                    }
                                                })
                                            } else {
                                                None
                                            }
                                        }
                                        bevy::reflect::VariantInfo::Struct(sv) => {
                                            // Handle Struct enum variants (e.g., RenderTarget::Image { target: ImageRenderTarget })
                                            debug!("[ENUM_NEWTYPE]   Is Struct variant with {} fields", sv.field_len());
                                            if sv.field_len() == 1 {
                                                let field = sv.field_at(0);
                                                if let Some(f) = field {
                                                    let field_type_path = f.type_path().to_string();
                                                    let field_name = f.name().to_string();
                                                    debug!("[ENUM_NEWTYPE]   Single field: '{}' of type '{}'", field_name, field_type_path);
                                                    
                                                    // Get the type_info of the field to check if it's a newtype
                                                    let type_info = f.type_info();
                                                    if let Some(ti) = type_info {
                                                        // Check if this field type is a struct/tuplestruct with Handle inside
                                                        match ti {
                                                            TypeInfo::TupleStruct(ts) if ts.field_len() == 1 => {
                                                                let inner = ts.field_at(0).map(|inner_f| inner_f.type_path().to_string());
                                                                if inner.as_ref().map(|s| s.contains("Handle<")).unwrap_or(false) {
                                                                    debug!("[ENUM_NEWTYPE]   ✓ Struct variant with TupleStruct newtype");
                                                                    let newtype_path = ti.type_path().to_string();
                                                                    return Some((true, inner, Some(ti), Some("0".to_string()), Some(newtype_path)));
                                                                }
                                                            }
                                                            TypeInfo::Struct(s) => {
                                                                // Find Handle<T> field in the struct
                                                                let handle_field_info = (0..s.field_len())
                                                                    .find_map(|i| s.field_at(i).filter(|inner_f| inner_f.type_path().contains("Handle<"))
                                                                        .map(|inner_f| (inner_f.type_path().to_string(), inner_f.name().to_string())));
                                                                if let Some((inner, inner_field)) = handle_field_info {
                                                                    debug!("[ENUM_NEWTYPE]   ✓ Struct variant with Struct newtype (Handle field: '{}')", inner_field);
                                                                    let newtype_path = ti.type_path().to_string();
                                                                    return Some((true, Some(inner), Some(ti), Some(inner_field), Some(newtype_path)));
                                                                }
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                            }
                                            None
                                        }
                                        _ => {
                                            debug!("[ENUM_NEWTYPE]   Not a Tuple or Struct variant");
                                            None
                                        }
                                    }
                                })
                                .unwrap_or((false, None, None, None, None))
                    } else {
                        debug!("[ENUM_NEWTYPE] No enum info found for field");
                        (false, None, None, None, None)
                    };
                    let _ = (newtype_type_info, newtype_field_name); // silence unused warnings

                    debug!(
                        "[ENUM_NEWTYPE] Result: variant_is_newtype={}, inner_type={:?}",
                        variant_is_newtype, newtype_inner_type
                    );

                    // Create the dynamic enum variant
                    let dynamic_variant = match &variant_value {
                        LuaValue::Nil => {
                            // Unit variant
                            DynamicVariant::Unit
                        }
                        LuaValue::Number(n) => {
                            let mut tuple = DynamicTuple::default();
                            if variant_is_newtype {
                                // This is a newtype-wrapped variant (like RenderTarget::Image(ImageRenderTarget))
                                // Treat number as asset ID and create properly typed Handle wrapped in DynamicStruct
                                debug!("[ENUM_GENERIC] Detected newtype variant '{}', treating {} as asset ID, inner_type={:?}", variant_name, *n, newtype_inner_type);
                                if let Some(registry) = asset_registry {
                                    if let Some(untyped_handle) =
                                        registry.get_untyped_handle(*n as u32)
                                    {
                                        // First create typed Handle<T> from UntypedHandle
                                        let handle_box =
                                            if let Some(ref inner_type) = newtype_inner_type {
                                                registry.try_create_typed_handle_box(
                                                    inner_type,
                                                    untyped_handle.clone(),
                                                )
                                            } else {
                                                None
                                            };

                                        // Then try to wrap in newtype (e.g., Handle<Image> -> ImageRenderTarget)
                                        if let Some(typed_handle) = handle_box {
                                            if let Some(ref newtype_path) = newtype_type_path {
                                                if let Some(wrapped) = registry
                                                    .try_wrap_in_newtype_with_reflection(
                                                        newtype_path,
                                                        typed_handle,
                                                        type_registry,
                                                    )
                                                {
                                                    debug!("[ENUM_GENERIC] ✓ Wrapped Handle in newtype '{}'", newtype_path);
                                                    tuple.insert_boxed(wrapped);
                                                } else {
                                                    return Err(LuaError::RuntimeError(format!(
                                                        "Failed to wrap in newtype '{}'. Ensure type has #[derive(Reflect)] with FromReflect", newtype_path
                                                    )));
                                                }
                                            } else {
                                                return Err(LuaError::RuntimeError(
                                                    "Newtype wrapper detection failed - no type path available".to_string()
                                                ));
                                            }
                                        } else {
                                            return Err(LuaError::RuntimeError(format!(
                                                "Failed to create typed handle for inner type {:?}",
                                                newtype_inner_type
                                            )));
                                        }
                                    } else {
                                        return Err(LuaError::RuntimeError(format!(
                                            "Asset ID {} not found",
                                            *n as u32
                                        )));
                                    }
                                } else {
                                    return Err(LuaError::RuntimeError(
                                        "Asset registry not available".to_string(),
                                    ));
                                }
                            } else {
                                // Regular tuple variant with single f32
                                tuple.insert(*n as f32);
                            }
                            DynamicVariant::Tuple(tuple)
                        }
                        LuaValue::Integer(i) => {
                            let mut tuple = DynamicTuple::default();
                            if variant_is_newtype {
                                // This is a newtype-wrapped variant
                                debug!("[ENUM_GENERIC] Detected newtype variant '{}', treating {} as asset ID, inner_type={:?}", variant_name, *i, newtype_inner_type);
                                if let Some(registry) = asset_registry {
                                    if let Some(untyped_handle) =
                                        registry.get_untyped_handle(*i as u32)
                                    {
                                        // First create typed Handle<T> from UntypedHandle
                                        let handle_box =
                                            if let Some(ref inner_type) = newtype_inner_type {
                                                registry.try_create_typed_handle_box(
                                                    inner_type,
                                                    untyped_handle.clone(),
                                                )
                                            } else {
                                                None
                                            };

                                        // Then try to wrap in newtype (e.g., Handle<Image> -> ImageRenderTarget)
                                        if let Some(typed_handle) = handle_box {
                                            if let Some(ref newtype_path) = newtype_type_path {
                                                if let Some(wrapped) = registry
                                                    .try_wrap_in_newtype_with_reflection(
                                                        newtype_path,
                                                        typed_handle,
                                                        type_registry,
                                                    )
                                                {
                                                    debug!("[ENUM_GENERIC] ✓ Wrapped Handle in newtype '{}'", newtype_path);
                                                    tuple.insert_boxed(wrapped);
                                                } else {
                                                    return Err(LuaError::RuntimeError(format!(
                                                        "Failed to wrap in newtype '{}'. Ensure type has #[derive(Reflect)] with FromReflect", newtype_path
                                                    )));
                                                }
                                            } else {
                                                return Err(LuaError::RuntimeError(
                                                    "Newtype wrapper detection failed - no type path available".to_string()
                                                ));
                                            }
                                        } else {
                                            return Err(LuaError::RuntimeError(format!(
                                                "Failed to create typed handle for inner type {:?}",
                                                newtype_inner_type
                                            )));
                                        }
                                    } else {
                                        return Err(LuaError::RuntimeError(format!(
                                            "Asset ID {} not found",
                                            *i as u32
                                        )));
                                    }
                                } else {
                                    return Err(LuaError::RuntimeError(
                                        "Asset registry not available".to_string(),
                                    ));
                                }
                            } else {
                                // Regular tuple variant with single i32/f32
                                tuple.insert(*i as f32);
                            }
                            DynamicVariant::Tuple(tuple)
                        }
                        LuaValue::Boolean(b) => {
                            // Tuple variant with single bool
                            let mut tuple = DynamicTuple::default();
                            tuple.insert(*b);
                            DynamicVariant::Tuple(tuple)
                        }
                        LuaValue::String(s) => {
                            // Tuple variant with single string
                            let mut tuple = DynamicTuple::default();
                            if let Ok(string) = s.to_str() {
                                tuple.insert(string.to_string());
                            }
                            DynamicVariant::Tuple(tuple)
                        }
                        LuaValue::Table(nested_table) => {
                            // Could be tuple or struct variant
                            if let Some(inner_value) =
                                try_create_value_from_table(nested_table, asset_registry)?
                            {
                                let mut tuple = DynamicTuple::default();
                                tuple.insert_boxed(inner_value);
                                DynamicVariant::Tuple(tuple)
                            } else {
                                DynamicVariant::Unit
                            }
                        }
                        _ => DynamicVariant::Unit,
                    };

                    let mut dynamic_enum = DynamicEnum::new(&variant_name, dynamic_variant);
                    // CRITICAL: Set the represented type so apply() knows the target enum type
                    let has_type_info = field.get_represented_type_info().is_some();
                    if let Some(type_info) = field.get_represented_type_info() {
                        dynamic_enum.set_represented_type(Some(type_info));
                    }
                    debug!("[ENUM_SET] Applying enum variant '{}' to field of type: {} (has_type_info={})", 
                        variant_name, type_path, has_type_info);

                    // Try to use from_reflect to create concrete enum value (preserves handles better)
                    let registry = type_registry.read();
                    let concrete_applied = registry
                        .get_with_type_path(&type_path)
                        .and_then(|reg| reg.data::<bevy::reflect::ReflectFromReflect>())
                        .and_then(|from_reflect| from_reflect.from_reflect(&dynamic_enum))
                        .map(|concrete| {
                            debug!("[ENUM_SET] Created concrete enum via from_reflect");
                            field.apply(concrete.as_ref());
                            true
                        })
                        .unwrap_or(false);
                    drop(registry); // Release read lock

                    if !concrete_applied {
                        // Fallback: try apply directly with DynamicEnum
                        match field.try_apply(&dynamic_enum) {
                            Ok(_) => {
                                debug!("[ENUM_SET] ✓ Applied DynamicEnum directly to {}", type_path)
                            }
                            Err(e) => {
                                warn!(
                                    "[ENUM_SET] ✗ Failed to apply enum variant '{}' to {}: {:?}",
                                    variant_name, type_path, e
                                );
                                field.apply(&dynamic_enum);
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}
