use bevy::prelude::*;
use bevy::asset::{ReflectAsset, UntypedAssetId};
use bevy::reflect::PartialReflect;
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
        
        info!("✓ Registered {} handle setters for asset types", handle_setters.len());
        
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
                // Try to create via ReflectDefault
                if let Some(reflect_default) = registration.data::<ReflectDefault>() {
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
                    
                    // Add the asset using ReflectAsset - this returns UntypedHandle
                    let untyped_handle = reflect_asset.add(world, asset.as_partial_reflect());
                    
                    // Register both the untyped handle (for generic access) and ID
                    asset_registry.register_typed_handle(id, untyped_handle.clone());
                    asset_registry.register_asset_handle(id, pending_asset.type_name.clone(), untyped_handle.id());
                    
                    info!("✓ Created asset {} with ID {}", pending_asset.type_name, id);
                } else {
                    error!("Asset type {} doesn't implement Default", pending_asset.type_name);
                }
            } else {
                error!("Type {} is not an asset (missing ReflectAsset)", pending_asset.type_name);
            }
        } else {
            error!("Asset type not found: {}", pending_asset.type_name);
        }
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
