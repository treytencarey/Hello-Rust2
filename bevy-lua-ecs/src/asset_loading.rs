use bevy::prelude::*;
use bevy::asset::{ReflectAsset, UntypedAssetId};
use bevy::reflect::{PartialReflect, Reflect, TypeRegistration};
use mlua::prelude::*;
use mlua::RegistryKey;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

/// Type-erased function that sets a Handle<T> field from an UntypedHandle
type HandleSetter = Box<dyn Fn(&mut dyn PartialReflect, UntypedHandle) -> bool + Send + Sync>;

/// Pending asset to be created via reflection
#[derive(Clone)]
pub struct PendingAsset {
    pub type_name: String,
    pub data: Arc<RegistryKey>,
}

/// Resource that manages loaded and created assets, providing asset IDs to Lua
#[derive(Resource, Clone)]
pub struct AssetRegistry {
    /// Maps asset IDs to Image handles (for load_asset)
    image_handles: Arc<Mutex<HashMap<u32, Handle<Image>>>>,
    
    /// Maps asset IDs to typed asset handles (for created assets - stored as UntypedHandle)
    typed_handles: Arc<Mutex<HashMap<u32, UntypedHandle>>>,
    
    /// Maps asset IDs to (type_name, UntypedAssetId) for created assets
    asset_handles: Arc<Mutex<HashMap<u32, (String, UntypedAssetId)>>>,
    
    /// Pending assets to be created
    pending_assets: Arc<Mutex<HashMap<u32, PendingAsset>>>,
    
    /// Counter for generating unique asset IDs
    next_id: Arc<AtomicU32>,
    
    /// Type-erased handle setters for each asset type (fully generic!)
    handle_setters: Arc<HashMap<String, HandleSetter>>,
}

impl Default for AssetRegistry {
    fn default() -> Self {
        Self {
            image_handles: Default::default(),
            typed_handles: Default::default(),
            asset_handles: Default::default(),
            pending_assets: Default::default(),
            next_id: Default::default(),
            handle_setters: Arc::new(HashMap::new()),
        }
    }
}

/// Macro to register handle setters for asset types
/// Usage: register_handle_setters!(registry, Image, Mesh, StandardMaterial, ...)
/// This allows users to specify which asset types they need in their game
#[macro_export]
macro_rules! register_handle_setters {
    ($registry:expr, $type_registry:expr, $($asset_type:ty),* $(,)?) => {
        {
            let registry_guard = $type_registry.read();
            $(
                // For each asset type, create a setter
                let type_path = std::any::type_name::<$asset_type>();
                if let Some(registration) = registry_guard.get_with_type_path(type_path) {
                    if registration.data::<bevy::asset::ReflectAsset>().is_some() {
                        let handle_type_path = format!("bevy_asset::handle::Handle<{}>", type_path);
                        let setter: Box<dyn Fn(&mut dyn bevy::reflect::PartialReflect, bevy::asset::UntypedHandle) -> bool + Send + Sync> = 
                            Box::new(|field, handle| {
                                if let Some(h) = field.try_downcast_mut::<bevy::asset::Handle<$asset_type>>() {
                                    *h = handle.typed();
                                    true
                                } else {
                                    false
                                }
                            });
                        $registry.insert(handle_type_path, setter);
                    }
                }
            )*
        }
    };
}

impl AssetRegistry {
    /// Create AssetRegistry with empty handle setters (user must register types they need)
    pub fn new() -> Self {
        Self {
            image_handles: Default::default(),
            typed_handles: Default::default(),
            asset_handles: Default::default(),
            pending_assets: Default::default(),
            next_id: Default::default(),
            handle_setters: Arc::new(HashMap::new()),
        }
    }
    
    /// Create AssetRegistry and populate with common Bevy asset types
    /// This is a convenience method - for full Zero Rust, use new() + register_asset_types!
    pub fn from_type_registry(type_registry: &AppTypeRegistry) -> Self {
        use bevy::prelude::*;
        
        let mut handle_setters: HashMap<String, HandleSetter> = HashMap::new();
        
        // Register common Bevy asset types using the macro
        // Users can customize this list in their own code
        register_handle_setters!(
            handle_setters,
            type_registry,
            Image,
            Mesh,
            StandardMaterial,
            Scene,
            AnimationClip,
            AudioSource,
            Font,
        );
        
        debug!("✓ Registered {} handle setters for asset types", handle_setters.len());
        debug!("📋 Registered asset type paths:");
        for type_path in handle_setters.keys() {
            debug!("  - {}", type_path);
        }
        
        Self {
            image_handles: Default::default(),
            typed_handles: Default::default(),
            asset_handles: Default::default(),
            pending_assets: Default::default(),
            next_id: Default::default(),
            handle_setters: Arc::new(handle_setters),
        }
    }
    
    /// Try to set a handle field using the registered handle setters
    pub fn try_set_handle_field(
        &self,
        field: &mut dyn PartialReflect,
        field_type_path: &str,
        untyped_handle: UntypedHandle,
    ) -> bool {
        if let Some(setter) = self.handle_setters.get(field_type_path) {
            setter(field, untyped_handle)
        } else {
            false
        }
    }
    
    /// Register an image handle (for load_asset)
    pub fn register_image(&self, handle: Handle<Image>) -> u32 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.image_handles.lock().unwrap().insert(id, handle);
        id
    }
    
    /// Get an image handle by ID
    pub fn get_image_handle(&self, id: u32) -> Option<Handle<Image>> {
        self.image_handles.lock().unwrap().get(&id).cloned()
    }
    
    /// Register a typed asset handle (stores as UntypedHandle for any asset type)
    pub fn register_typed_handle(&self, id: u32, handle: UntypedHandle) {
        self.typed_handles.lock().unwrap().insert(id, handle);
    }
    
    /// Get a typed asset handle by ID and convert to specific type
    pub fn get_typed_handle<T: bevy::asset::Asset>(&self, id: u32) -> Option<Handle<T>> {
        let handles = self.typed_handles.lock().unwrap();
        let untyped_handle = handles.get(&id)?;
        Some(untyped_handle.clone().typed())
    }
    
    /// Get an untyped handle by ID (most generic - works for any asset type)
    pub fn get_untyped_handle(&self, id: u32) -> Option<UntypedHandle> {
        self.typed_handles.lock().unwrap().get(&id).cloned()
    }
    
    /// Register a pending asset for creation
    pub fn register_pending_asset(&self, pending: PendingAsset) -> u32 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.pending_assets.lock().unwrap().insert(id, pending);
        id
    }
    
    /// Register a created asset handle
    pub fn register_asset_handle(&self, id: u32, type_name: String, asset_id: UntypedAssetId) {
        self.asset_handles.lock().unwrap().insert(id, (type_name, asset_id));
    }
    
    /// Get an asset handle by ID
    pub fn get_asset_handle(&self, id: u32) -> Option<(String, UntypedAssetId)> {
        self.asset_handles.lock().unwrap().get(&id).cloned()
    }
    
    /// Drain pending assets
    pub fn drain_pending_assets(&self) -> Vec<(u32, PendingAsset)> {
        let mut pending = self.pending_assets.lock().unwrap();
        pending.drain().collect()
    }
}

/// System to process pending assets and create them via reflection
pub fn process_pending_assets(world: &mut World) {
    // Get resources we need
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let asset_registry = world.resource::<AssetRegistry>().clone();
    let lua_ctx = world.resource::<crate::lua_integration::LuaScriptContext>().clone();
    
    let pending = asset_registry.drain_pending_assets();
    
    if pending.is_empty() {
        return;
    }
    
    for (id, pending_asset) in pending {
        let registry = type_registry.read();
        
        // Find the type registration
        if let Some(registration) = registry.get_with_type_path(&pending_asset.type_name) {
            // Check if it has ReflectAsset
            if let Some(reflect_asset) = registration.data::<ReflectAsset>() {
                // Special handling for Mesh - convert primitives to mesh
                let asset_result = if pending_asset.type_name == "bevy_mesh::mesh::Mesh" {
                    debug!("[ASSET_PROCESS] Processing Mesh asset, ID {}", id);
                    match create_mesh_from_primitive(&pending_asset.data, &lua_ctx.lua, &type_registry) {
                        Ok(mesh) => {
                            if let Some(mesh_ref) = mesh.as_reflect().downcast_ref::<Mesh>() {
                                debug!("[ASSET_PROCESS] Mesh created successfully, vertices: {}", mesh_ref.count_vertices());
                            } else {
                                debug!("[ASSET_PROCESS] Mesh created, but downcast to Mesh failed!");
                            }
                            Some(mesh)
                        },
                        Err(e) => {
                            error!("Failed to create mesh from primitive: {}", e);
                            None
                        }
                    }
                } else if let Some(reflect_default) = registration.data::<ReflectDefault>() {
                    // Use Default if available
                    let mut asset = reflect_default.default();
                    
                    // Populate fields from Lua data
                    if let Err(e) = populate_asset_from_lua(
                        asset.as_partial_reflect_mut(),
                        &pending_asset.data,
                        &lua_ctx.lua,
                    ) {
                        error!("Failed to populate asset {}: {}", pending_asset.type_name, e);
                        continue;
                    }
                    
                    Some(asset)
                } else {
                    // Try to construct from Lua data directly using reflection
                    match construct_asset_from_lua(
                        registration,
                        &pending_asset.data,
                        &lua_ctx.lua,
                        &type_registry,
                    ) {
                        Ok(asset) => Some(asset),
                        Err(e) => {
                            error!("Failed to construct asset {}: {}", pending_asset.type_name, e);
                            None
                        }
                    }
                };
                
                if let Some(asset) = asset_result {
                    // Special handling for Mesh: Add directly to Assets<Mesh> to avoid reflection lossiness
                    // ReflectAsset for Mesh might not correctly handle all vertex attributes (like Tangents)
                    let untyped_handle = if let Some(mesh) = asset.as_any().downcast_ref::<Mesh>() {
                        let mut meshes = world.resource_mut::<Assets<Mesh>>();
                        let handle = meshes.add(mesh.clone());
                        handle.untyped()
                    } else {
                        // Fallback for other assets: Add using ReflectAsset
                        reflect_asset.add(world, asset.as_partial_reflect())
                    };
                    
                    debug!("[ASSET_PROCESS] Asset {} added, handle ID: {:?}", pending_asset.type_name, untyped_handle.id());
                    
                    // Register both the untyped handle (for generic access) and ID
                    asset_registry.register_typed_handle(id, untyped_handle.clone());
                    asset_registry.register_asset_handle(id, pending_asset.type_name.clone(), untyped_handle.id());
                    
                    debug!("✓ Created asset {} with ID {} -> Handle {:?}", pending_asset.type_name, id, untyped_handle.id());
                }
            } else {
                error!("Type {} is not an asset (missing ReflectAsset)", pending_asset.type_name);
            }
        } else {
            error!("Asset type not found: {}", pending_asset.type_name);
            error!("Available asset types in registry: {:?}", 
                registry.iter()
                    .filter(|r| r.data::<ReflectAsset>().is_some())
                    .map(|r| r.type_info().type_path())
                    .collect::<Vec<_>>()
            );
        }
    }
}

/// Create a Mesh from primitive shape data using reflection (generic for any primitive type)
fn create_mesh_from_primitive(
    registry_key: &RegistryKey,
    lua: &Lua,
    type_registry: &AppTypeRegistry,
) -> LuaResult<Box<dyn Reflect>> {
    use bevy::prelude::Mesh;
    use bevy::reflect::{ReflectFromReflect, DynamicStruct};
    use bevy::prelude::ReflectDefault;
    
    // Get the Lua value
    let lua_value: LuaValue = lua.registry_value(registry_key)?;
    let data_table = match lua_value {
        LuaValue::Table(t) => t,
        _ => return Err(LuaError::RuntimeError("Mesh data must be a table".to_string())),
    };
    
    // Look for "primitive" field with shape data
    let primitive_value: LuaValue = data_table.get("primitive")?;
    let primitive_table = match primitive_value {
        LuaValue::Table(t) => t,
        _ => return Err(LuaError::RuntimeError("Mesh must have 'primitive' field".to_string())),
    };
    
    // Get the shape variant (e.g., "Cuboid", "Sphere", "Circle")
    let pairs: Vec<(String, LuaValue)> = primitive_table
        .pairs()
        .collect::<LuaResult<_>>()?;
    
    if pairs.is_empty() {
        return Err(LuaError::RuntimeError("Primitive must specify a shape".to_string()));
    }
    
    let (shape_name, shape_data) = &pairs[0];
    
    // Build the full type path for the primitive
    let primitive_type_path = format!("bevy_math::primitives::dim3::{}", shape_name);
    
    debug!("[MESH_GEN] Looking up primitive type: {}", primitive_type_path);
    
    // Get type registration from registry
    let registry = type_registry.read();
    let registration = registry.get_with_type_path(&primitive_type_path)
        .ok_or_else(|| LuaError::RuntimeError(
            format!("Primitive type not found: {}. Available primitives in registry: {:?}", 
                primitive_type_path,
                registry.iter()
                    .filter(|r| r.type_info().type_path().contains("bevy_math::primitives"))
                    .map(|r| r.type_info().type_path())
                    .collect::<Vec<_>>()
            )
        ))?;
    
    debug!("[MESH_GEN] Found primitive type registration");
    
    // Create instance using reflection
    let mut primitive_instance = if let Some(reflect_default) = registration.data::<ReflectDefault>() {
        // Use default constructor if available
        debug!("[MESH_GEN] Creating primitive using ReflectDefault");
        reflect_default.default()
    } else if let Some(reflect_from_reflect) = registration.data::<ReflectFromReflect>() {
        // Try to construct from a dynamic struct
        debug!("[MESH_GEN] Creating primitive using ReflectFromReflect");
        let mut dynamic_struct = DynamicStruct::default();
        dynamic_struct.set_represented_type(Some(registration.type_info()));
        
        reflect_from_reflect.from_reflect(&dynamic_struct)
            .ok_or_else(|| LuaError::RuntimeError(
                format!("Failed to create primitive {} from reflection", shape_name)
            ))?
    } else {
        return Err(LuaError::RuntimeError(
            format!("Primitive type {} doesn't support Default or FromReflect", shape_name)
        ));
    };
    
    // Populate fields from Lua data using generic reflection
    if let LuaValue::Table(ref data_table) = shape_data {
        println!("[MESH_GEN] Populating primitive fields from Lua data");
        
        if let bevy::reflect::ReflectMut::Struct(struct_mut) = primitive_instance.reflect_mut() {
            // Iterate through all fields in the struct
            for i in 0..struct_mut.field_len() {
                // Clone the name to avoid holding an immutable borrow while we need a mutable one later
                let field_name = struct_mut.name_at(i).map(|s| s.to_string());
                
                if let Some(name) = field_name {
                    // Try to get value from Lua table
                    if let Ok(lua_val) = data_table.get::<LuaValue>(name.clone()) {
                        if let Some(field) = struct_mut.field_at_mut(i) {
                            debug!("[MESH_GEN]   Setting field '{}' from Lua", name);
                            set_basic_field(field, &lua_val)?;
                        }
                    }
                }
            }
        }
    }
    
    debug!("[MESH_GEN] Converting primitive {} to Mesh", shape_name);
    
    // Convert the primitive to a Mesh using Into<Mesh>
    // We need to downcast to the concrete primitive type and call .into()
    // Since we can't do this generically, we'll use the type path to match
    let mesh: Mesh = match shape_name.as_str() {
        "Cuboid" => {
            use bevy::math::primitives::Cuboid;
            if let Some(cuboid) = primitive_instance.downcast_ref::<Cuboid>() {
                (*cuboid).into()
            } else {
                return Err(LuaError::RuntimeError("Failed to downcast to Cuboid".to_string()));
            }
        }
        "Sphere" => {
            use bevy::math::primitives::Sphere;
            if let Some(sphere) = primitive_instance.downcast_ref::<Sphere>() {
                (*sphere).into()
            } else {
                return Err(LuaError::RuntimeError("Failed to downcast to Sphere".to_string()));
            }
        }
        "Cylinder" => {
            use bevy::math::primitives::Cylinder;
            if let Some(cylinder) = primitive_instance.downcast_ref::<Cylinder>() {
                (*cylinder).into()
            } else {
                return Err(LuaError::RuntimeError("Failed to downcast to Cylinder".to_string()));
            }
        }
        "Capsule3d" => {
            use bevy::math::primitives::Capsule3d;
            if let Some(capsule) = primitive_instance.downcast_ref::<Capsule3d>() {
                (*capsule).into()
            } else {
                return Err(LuaError::RuntimeError("Failed to downcast to Capsule3d".to_string()));
            }
        }
        "Torus" => {
            use bevy::math::primitives::Torus;
            if let Some(torus) = primitive_instance.downcast_ref::<Torus>() {
                (*torus).into()
            } else {
                return Err(LuaError::RuntimeError("Failed to downcast to Torus".to_string()));
            }
        }
        _ => {
            return Err(LuaError::RuntimeError(
                format!("Primitive type {} doesn't support conversion to Mesh. Supported: Cuboid, Sphere, Cylinder, Capsule3d, Torus", shape_name)
            ));
        }
    };
    
    // Log what attributes the mesh has
    debug!("[MESH_GEN] Created {} mesh with {} vertices", shape_name, mesh.count_vertices());
    debug!("[MESH_GEN] Topology: {:?}", mesh.primitive_topology());
    debug!("[MESH_GEN] Has POSITION: {}", mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
    debug!("[MESH_GEN] Has NORMAL: {}", mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
    debug!("[MESH_GEN] Has UV_0: {}", mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
    debug!("[MESH_GEN] Has TANGENT: {}", mesh.attribute(Mesh::ATTRIBUTE_TANGENT).is_some());
    
    // Generate tangents if missing (required for PBR prepass with shadows)
    // Tangent generation requires: TriangleList topology, POSITION, NORMAL, UV_0
    let mut mesh = mesh;
    if mesh.attribute(Mesh::ATTRIBUTE_TANGENT).is_none() {
        debug!("[MESH_GEN] Tangents missing, attempting to generate...");
        
        // Check prerequisites for tangent generation
        let has_prerequisites = mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some()
            && mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some()
            && mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some();
        
        if !has_prerequisites {
            debug!("[MESH_GEN] WARNING: Cannot generate tangents - missing required attributes");
            debug!("[MESH_GEN]   Required: POSITION, NORMAL, UV_0");
            debug!("[MESH_GEN]   This mesh may not render correctly with PBR materials that use normal maps");
        } else {
            // Attempt to generate tangents
            // generate_tangents() will return an error if prerequisites aren't met (e.g. wrong topology)
            match mesh.generate_tangents() {
                Ok(()) => {
                    debug!("[MESH_GEN] ✓ Tangents generated successfully");
                }
                Err(e) => {
                    debug!("[MESH_GEN] ERROR: Failed to generate tangents: {:?}", e);
                    debug!("[MESH_GEN]   The mesh may not render correctly with shadows or normal maps");
                    debug!("[MESH_GEN]   Consider using an unlit material or disabling shadows");
                }
            }
        }
    } else {
        debug!("[MESH_GEN] Tangents already present");
    }
    
    Ok(Box::new(mesh))
}

/// Construct an asset from Lua data without Default (using FromReflect)
fn construct_asset_from_lua(
    registration: &TypeRegistration,
    registry_key: &RegistryKey,
    lua: &Lua,
    _type_registry: &AppTypeRegistry,
) -> LuaResult<Box<dyn Reflect>> {
    use bevy::reflect::{DynamicStruct, ReflectFromReflect};
    
    // Get the Lua value from registry
    let lua_value: LuaValue = lua.registry_value(registry_key)?;
    
    // Must be a table
    let data_table = match lua_value {
        LuaValue::Table(t) => t,
        _ => return Err(LuaError::RuntimeError("Asset data must be a table".to_string())),
    };
    
    // Get type info
    let type_info = registration.type_info();
    
    // Create a dynamic struct
    let mut dynamic_struct = DynamicStruct::default();
    dynamic_struct.set_represented_type(Some(type_info));
    
    // Get the AssetRegistry for handle conversion
    let asset_registry = lua.globals().get::<LuaTable>("__asset_registry")
        .ok()
        .and_then(|t| t.raw_get::<mlua::AnyUserData>("ptr").ok())
        .and_then(|ud| ud.borrow::<Arc<AssetRegistry>>().ok().map(|r| r.clone()));
    
    // Populate fields from Lua table
    for pair in data_table.pairs::<String, LuaValue>() {
        let (field_name, field_value) = pair?;
        
        // Try to create the field value
        if let Some(value) = try_create_value_from_lua(field_value, asset_registry.as_deref())? {
            dynamic_struct.insert_boxed(&field_name, value);
        }
    }
    
    // Convert to concrete type via FromReflect
    let reflect_from_reflect = registration.data::<ReflectFromReflect>()
        .ok_or_else(|| LuaError::RuntimeError(
            format!("Asset type {} doesn't implement FromReflect", registration.type_info().type_path())
        ))?;
    
    let asset = reflect_from_reflect.from_reflect(&dynamic_struct)
        .ok_or_else(|| LuaError::RuntimeError(
            format!("Failed to create asset {} from reflection", registration.type_info().type_path())
        ))?;
    
    Ok(asset)
}

/// Helper to convert Lua values to reflection values
fn try_create_value_from_lua(
    lua_value: LuaValue,
    asset_registry: Option<&AssetRegistry>,
) -> LuaResult<Option<Box<dyn PartialReflect>>> {
    use bevy::reflect::{DynamicStruct, DynamicEnum, DynamicVariant};
    
    match lua_value {
        LuaValue::Boolean(b) => Ok(Some(Box::new(b))),
        LuaValue::Integer(i) => Ok(Some(Box::new(i as i32))),
        LuaValue::Number(n) => Ok(Some(Box::new(n as f32))),
        LuaValue::String(s) => Ok(Some(Box::new(s.to_str()?.to_string()))),
        LuaValue::Table(t) => {
            // Check if it's an enum (single key-value where value is a table)
            let pairs: Vec<_> = t.clone().pairs::<String, LuaValue>().collect::<LuaResult<_>>()?;
            
            if pairs.len() == 1 {
                let (key, value) = &pairs[0];
                
                // Check for asset handle (numeric ID) - skip for now, needs type parameter
                if key == "asset_id" {
                    // Can't create typed handles without knowing the asset type at compile time
                    // This would need special handling or a different approach
                }
                
                // Try as enum variant
                if let LuaValue::Table(variant_data) = value {
                    let mut dynamic_struct = DynamicStruct::default();
                    
                    for pair in variant_data.pairs::<String, LuaValue>() {
                        let (field_name, field_value) = pair?;
                        if let Some(nested_value) = try_create_value_from_lua(field_value, asset_registry)? {
                            dynamic_struct.insert_boxed(&field_name, nested_value);
                        }
                    }
                    
                    let dynamic_enum = DynamicEnum::new(key, DynamicVariant::Struct(dynamic_struct));
                    return Ok(Some(Box::new(dynamic_enum)));
                }
            }
            
            // Regular struct
            let mut dynamic_struct = DynamicStruct::default();
            
            for pair in t.pairs::<String, LuaValue>() {
                let (field_name, field_value) = pair?;
                if let Some(nested_value) = try_create_value_from_lua(field_value, asset_registry)? {
                    dynamic_struct.insert_boxed(&field_name, nested_value);
                }
            }
            
            Ok(Some(Box::new(dynamic_struct)))
        }
        _ => Ok(None),
    }
}

/// Populate asset fields from Lua data
fn populate_asset_from_lua(
    asset: &mut dyn PartialReflect,
    registry_key: &RegistryKey,
    lua: &Lua,
) -> LuaResult<()> {
    // Get the Lua value from registry
    let lua_value: LuaValue = lua.registry_value(registry_key)?;
    
    // Must be a table
    let data_table = match lua_value {
        LuaValue::Table(t) => t,
        _ => return Err(LuaError::RuntimeError("Asset data must be a table".to_string())),
    };
    
    // Get struct reflection
    if let bevy::reflect::ReflectMut::Struct(struct_mut) = asset.reflect_mut() {
        // Iterate through fields
        for i in 0..struct_mut.field_len() {
            if let Some(field_name) = struct_mut.name_at(i) {
                // Try to get value from Lua table
                if let Ok(lua_val) = data_table.get::<LuaValue>(field_name) {
                    if let Some(field) = struct_mut.field_at_mut(i) {
                        // Simple field population - handle basic types
                        set_basic_field(field, &lua_val)?;
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Set field types from Lua values using pure reflection (fully generic)
fn set_basic_field(field: &mut dyn PartialReflect, lua_value: &LuaValue) -> LuaResult<()> {
    use bevy::reflect::ReflectMut;
    
    match lua_value {
        LuaValue::Integer(v) => {
            // Try to apply integer to any numeric type via reflection
            // Use try_downcast_mut for value types (primitives)
            if let Some(f32_field) = field.try_downcast_mut::<f32>() {
                *f32_field = *v as f32;
            } else if let Some(u32_field) = field.try_downcast_mut::<u32>() {
                *u32_field = *v as u32;
            } else if let Some(i32_field) = field.try_downcast_mut::<i32>() {
                *i32_field = *v as i32;
            } else if let Some(usize_field) = field.try_downcast_mut::<usize>() {
                *usize_field = *v as usize;
            }
        }
        LuaValue::Number(v) => {
            // Try to apply number to any numeric type
            if let Some(f32_field) = field.try_downcast_mut::<f32>() {
                *f32_field = *v as f32;
            } else if let Some(f64_field) = field.try_downcast_mut::<f64>() {
                *f64_field = *v;
            }
        }
        LuaValue::Boolean(v) => {
            if let Some(bool_field) = field.try_downcast_mut::<bool>() {
                *bool_field = *v;
            }
        }
        LuaValue::String(s) => {
            if let Ok(string) = s.to_str() {
                if let Some(string_field) = field.try_downcast_mut::<String>() {
                    *string_field = string.to_string();
                }
            }
        }
        LuaValue::Table(table) => {
            // Generic table handling using reflection - works for any struct
            match field.reflect_mut() {
                ReflectMut::Struct(struct_mut) => {
                    // Iterate through struct fields and populate from table
                    for i in 0..struct_mut.field_len() {
                        if let Some(field_name) = struct_mut.name_at(i) {
                            if let Ok(nested_value) = table.get::<LuaValue>(field_name) {
                                if let Some(nested_field) = struct_mut.field_at_mut(i) {
                                    set_basic_field(nested_field, &nested_value)?;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    
    Ok(())
}

/// Add asset loading and creation capabilities to Lua context
pub fn add_asset_loading_to_lua(
    lua_ctx: &crate::lua_integration::LuaScriptContext,
    asset_server: AssetServer,
    asset_registry: AssetRegistry,
) -> Result<(), LuaError> {
    let lua = &lua_ctx.lua;
    
    // load_asset - for loading image files
    let asset_registry_clone = asset_registry.clone();
    let load_asset = lua.create_function(move |_lua_ctx, path: String| {
        let handle: Handle<Image> = asset_server.load(&path);
        let id = asset_registry_clone.register_image(handle);
        Ok(id)
    })?;
    
    // create_asset - generic asset creation via reflection
    let registry_clone = asset_registry.clone();
    let create_asset = lua.create_function(move |lua_ctx, (type_name, data): (String, LuaTable)| {
        // Store the Lua table in registry
        let registry_key = lua_ctx.create_registry_value(data)?;
        
        let pending = PendingAsset {
            type_name,
            data: Arc::new(registry_key),
        };
        
        let id = registry_clone.register_pending_asset(pending);
        Ok(id)
    })?;
    
    // Inject into globals
    lua.globals().set("load_asset", load_asset)?;
    lua.globals().set("create_asset", create_asset)?;
    
    Ok(())
}
