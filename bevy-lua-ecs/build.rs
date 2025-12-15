// Build script for bevy-lua-ecs
// Automatically generates Lua bindings for resource types specified in dependent crates

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{File, Item, ItemImpl, ImplItem, Visibility, FnArg, ReturnType};
use quote::quote;
use serde::{Serialize, Deserialize};

fn main() {
    // Only rerun if these files change (not on every compile)
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=../Hello/Cargo.toml");
    println!("cargo:rerun-if-changed=../Hello/Cargo.lock");
    
    let pkg_name = env::var("CARGO_PKG_NAME").unwrap_or_default();
    println!("cargo:warning=Build script: PKG={}", pkg_name);
    
    // Read our own Cargo.toml for event types
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let own_manifest = PathBuf::from(manifest_dir).join("Cargo.toml");
        if let Ok(content) = fs::read_to_string(&own_manifest) {
            if let Ok(manifest) = toml::from_str::<toml::Value>(&content) {
                let event_types = generate_event_registrations(&manifest);
                
                // Try to find parent manifest with lua_resources metadata
                if let Ok(out_dir) = env::var("OUT_DIR") {
                    let build_dir = PathBuf::from(out_dir);
                    if let Some(parent_manifest) = find_parent_manifest(&build_dir) {
                        println!("cargo:warning=Found parent manifest: {:?}", parent_manifest);
                        generate_bindings_for_manifest(&parent_manifest);
                        return;
                    }
                }
                
                write_empty_bindings_with_events(event_types);
                return;
            }
        }
    }
    write_empty_bindings_with_events(Vec::new());
}

/// Convert PascalCase to snake_case (e.g., TextureUsages -> texture_usages)
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

#[allow(dead_code)]
fn find_parent_manifest(build_dir: &Path) -> Option<PathBuf> {
    // Strategy 1: Navigate up from build directory to find workspace root
    let mut current = build_dir.to_path_buf();
    
    // Go up several levels to escape the build directory structure
    for _ in 0..10 {
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
            
            // Look for Cargo.toml files
            let cargo_toml = current.join("Cargo.toml");
            if cargo_toml.exists() {
                // Read and check if it has our metadata
                if let Ok(content) = fs::read_to_string(&cargo_toml) {
                    if content.contains("[package.metadata.lua_resources]") {
                        eprintln!("Found parent manifest: {:?}", cargo_toml);
                        return Some(cargo_toml);
                    }
                }
            }
        } else {
            break;
        }
    }
    
    // Strategy 2: Look for Hello/Cargo.toml relative to workspace
    // Try to find it by looking for the workspace root
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        // Go up from bevy-lua-ecs to workspace root
        let workspace_root = PathBuf::from(&manifest_dir).parent()?.to_path_buf();
        
        // Check Hello/Cargo.toml
        let hello_manifest = workspace_root.join("Hello").join("Cargo.toml");
        if hello_manifest.exists() {
            if let Ok(content) = fs::read_to_string(&hello_manifest) {
                if content.contains("[package.metadata.lua_resources]") {
                    eprintln!("Found Hello manifest: {:?}", hello_manifest);
                    return Some(hello_manifest);
                }
            }
        }
    }
    
    eprintln!("No parent manifest found with lua_resources metadata");
    None
}

#[allow(dead_code)]
fn generate_bindings_for_manifest(manifest_path: &Path) {
    // Read parent's Cargo.toml
    let manifest_content = match fs::read_to_string(manifest_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read manifest: {}", e);
            write_empty_bindings_with_events(Vec::new());
            return;
        }
    };
    
    let manifest: toml::Value = match toml::from_str(&manifest_content) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to parse manifest: {}", e);
            write_empty_bindings_with_events(Vec::new());
            return;
        }
    };
    
    // Get types from metadata
    let types_to_expose = get_types_from_metadata(&manifest);
    
    if types_to_expose.is_empty() {
        println!("cargo:warning=No types specified in [package.metadata.lua_resources]");
        write_empty_bindings_with_events(Vec::new());
        return;
    }
    
    // Generate bindings for each type
    let mut all_bindings = Vec::new();
    for type_spec in types_to_expose {
        match generate_bindings_for_type(&type_spec) {
            Ok(bindings) => {
                println!("cargo:warning=✓ Generated bindings for {}", type_spec.full_path);
                all_bindings.push(bindings);
            }
            Err(e) => {
                println!("cargo:warning=⚠ Failed to generate bindings for {}: {}", type_spec.full_path, e);
            }
        }
    }
    
    // Process constructors
    let constructors_to_expose = get_constructors_from_metadata(&manifest);
    let mut all_constructor_bindings = Vec::new();
    
    for constructor_spec in constructors_to_expose {
        match generate_bindings_for_constructor(&constructor_spec) {
            Ok(bindings) => {
                println!("cargo:warning=✓ Generated constructor binding for {}", constructor_spec.full_path);
                all_constructor_bindings.push(bindings);
            }
            Err(e) => {
                println!("cargo:warning=⚠ Failed to generate constructor binding for {}: {}", constructor_spec.full_path, e);
            }
        }
    }
    
    // Auto-discover asset types by scanning bevy crates and workspace members
    // Pattern: impl Asset for Type or #[derive(Asset)]
    // 
    // RUNTIME-BASED: We collect TYPE NAMES only (not compile-time paths)
    // The runtime will look up each name in TypeRegistry and register if found
    let discovered_assets = discover_asset_types();
    
    // Collect just the type names for runtime lookup (no compile-time paths)
    let asset_type_names: Vec<String> = discovered_assets.iter()
        .map(|a| a.type_name.clone())
        .collect();
    
    println!("cargo:warning=  ✓ Collected {} asset type names for runtime registration", asset_type_names.len());
    
    // Auto-discover constructors for asset types (new_*, from_*, etc.)
    // These are used for opaque types that can't be created via reflection
    let discovered_constructors = discover_asset_constructors(&discovered_assets);
    
    let all_discovered_bitflags: Vec<DiscoveredBitflags> = Vec::new();
    
    // Get parent crate's src directory
    let parent_src_dir = manifest_path.parent().unwrap().join("src");
    
    // Auto-discover entity wrapper components by scanning bevy crates and workspace members
    // Pattern: pub struct Foo(pub Entity) with #[derive(Component)]
    // 
    // RUNTIME-BASED: We collect TYPE NAMES (not compile-time paths) for runtime registration
    // The runtime will look up each name in TypeRegistry and register if found
    let discovered_entity_wrappers = discover_entity_wrapper_components();
    
    // Collect just the type names for runtime lookup (no compile-time paths)
    let entity_wrapper_names: Vec<String> = discovered_entity_wrappers
        .iter()
        .map(|w| w.type_name.clone())
        .collect();
    
    println!("cargo:warning=  ✓ Collected {} entity wrapper type names for runtime registration", entity_wrapper_names.len());
    
    // Generate event registrations from our own manifest (not parent's)
    let event_types = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let own_manifest = PathBuf::from(manifest_dir).join("Cargo.toml");
        if let Ok(content) = fs::read_to_string(&own_manifest) {
            if let Ok(own_manifest_toml) = toml::from_str::<toml::Value>(&content) {
                generate_event_registrations(&own_manifest_toml)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    
    
    // Auto-discover Handle<T> newtype wrappers (e.g., ImageRenderTarget)
    // No manual metadata configuration needed!
    let discovered_newtypes = discover_handle_newtype_wrappers();
    
    // Convert to NewtypeSpec format for code generation
    let newtypes: Vec<NewtypeSpec> = discovered_newtypes.into_iter()
        .map(|dn| NewtypeSpec {
            newtype_path: dn.newtype_path,
            // Use inner asset name as path placeholder - runtime will resolve via TypeRegistry
            inner_asset_path: dn.inner_asset_name,
        })
        .collect();
    
    // Auto-discover SystemParam types and methods (with caching)
    // Uses JSON cache file in OUT_DIR, invalidated when Cargo.lock changes
    let (discovered_systemparams, discovered_systemparam_methods) = get_discovered_systemparams_and_methods();
    
    // Parse bitflags from metadata [package.metadata.lua_bitflags]
    let metadata_bitflags = get_bitflags_from_metadata(&manifest);
    
    // Convert metadata bitflags to DiscoveredBitflags format
    let mut all_bitflags = all_discovered_bitflags;
    for bf in metadata_bitflags {
        // Only add if not already discovered (metadata takes precedence for names)
        if !all_bitflags.iter().any(|d| d.name == bf.name) {
            all_bitflags.push(DiscoveredBitflags {
                name: bf.name,
                _full_path: String::new(),
                flags: bf.flags.iter().map(|(n, _)| n.clone()).collect(),
            });
        }
    }
    
    // Write generated code to parent crate's src directory
    // Now with simplified signature - asset_type_names for runtime registration
    write_bindings_to_parent_crate(
        all_bindings, 
        all_constructor_bindings, 
        entity_wrapper_names, 
        asset_type_names,
        discovered_assets,
        discovered_constructors,
        all_bitflags, 
        newtypes,
        discovered_systemparams,
        discovered_systemparam_methods,
        &parent_src_dir
    );
    
    //Write events to our own auto_bindings.rs
    write_empty_bindings_with_events(event_types);
}

fn generate_event_registrations(manifest: &toml::Value) -> Vec<String> {
    let events_array = manifest
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("lua_events"))
        .and_then(|le| le.get("types"))
        .and_then(|t| t.as_array());
    
    let Some(events) = events_array else {
        return Vec::new();
    };
    
    events.iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect()
}

#[derive(Debug, Clone)]
struct TypeSpec {
    full_path: String,
    crate_name: String,
    module_path: Vec<String>,
    type_name: String,
}

#[derive(Debug, Clone)]
struct ConstructorSpec {
    full_path: String,  // e.g., "renet::RenetClient::new"
    type_spec: TypeSpec,
    function_name: String,  // e.g., "new"
}

fn get_types_from_metadata(manifest: &toml::Value) -> Vec<TypeSpec> {
    let types_array = manifest
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("lua_resources"))
        .and_then(|lr| lr.get("types"))
        .and_then(|t| t.as_array());
    
    let Some(types) = types_array else {
        return Vec::new();
    };
    
    types.iter()
        .filter_map(|v| v.as_str())
        .filter_map(|s| parse_type_spec(s))
        .collect()
}

/// Asset constructor specification - includes type path for generated function naming
#[derive(Debug, Clone)]
struct AssetConstructorSpec {
    full_path: String,  // e.g., "bevy_image::image::Image::new_fill"
    type_path: String,  // e.g., "bevy_image::image::Image"
    type_spec: TypeSpec,
    function_name: String,  // e.g., "new_fill"
}

/// Get asset types from [package.metadata.lua_assets.types] in Cargo.toml
/// Unlike the old constructors approach, this just specifies the type -
/// constructors are auto-discovered via syn parsing
fn get_asset_types_from_metadata(manifest: &toml::Value) -> Vec<TypeSpec> {
    let types_array = manifest
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("lua_assets"))
        .and_then(|la| la.get("types"))
        .and_then(|t| t.as_array());
    
    let Some(types) = types_array else {
        return Vec::new();
    };
    
    types.iter()
        .filter_map(|v| v.as_str())
        .filter_map(|s| parse_type_spec(s))
        .collect()
}

/// Bitflags specification from Cargo.toml metadata
#[derive(Debug, Clone)]
struct BitflagsSpec {
    name: String,
    flags: Vec<(String, u32)>,  // (flag_name, bit_value)
}

/// Parse bitflags from [package.metadata.lua_bitflags] in Cargo.toml
/// Format:
/// [package.metadata.lua_bitflags]
/// TextureUsages = ["COPY_SRC", "COPY_DST", "TEXTURE_BINDING", "STORAGE_BINDING", "RENDER_ATTACHMENT"]
/// RenderAssetUsages = ["MAIN_WORLD", "RENDER_WORLD"]
fn get_bitflags_from_metadata(manifest: &toml::Value) -> Vec<BitflagsSpec> {
    let bitflags_table = manifest
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("lua_bitflags"))
        .and_then(|bf| bf.as_table());
    
    let Some(table) = bitflags_table else {
        return Vec::new();
    };
    
    table.iter()
        .filter_map(|(name, value)| {
            let flags = value.as_array()?;
            let flag_tuples: Vec<(String, u32)> = flags.iter()
                .enumerate()
                .filter_map(|(idx, v)| {
                    let flag_name = v.as_str()?.to_string();
                    Some((flag_name, 1u32 << idx))
                })
                .collect();
            
            if flag_tuples.is_empty() {
                None
            } else {
                println!("cargo:warning=  ✓ Found metadata bitflags {} with {} flags", name, flag_tuples.len());
                Some(BitflagsSpec {
                    name: name.clone(),
                    flags: flag_tuples,
                })
            }
        })
        .collect()
}

fn parse_type_spec(full_path: &str) -> Option<TypeSpec> {
    let parts: Vec<&str> = full_path.split("::").collect();
    if parts.len() < 2 {
        return None;
    }
    
    let crate_name = parts[0].to_string();
    let type_name = parts.last()?.to_string();
    let module_path = parts[1..parts.len()-1].iter()
        .map(|s| s.to_string())
        .collect();
    
    Some(TypeSpec {
        full_path: full_path.to_string(),
        crate_name,
        module_path,
        type_name,
    })
}


fn get_constructors_from_metadata(manifest: &toml::Value) -> Vec<ConstructorSpec> {
    let constructors_array = manifest
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("lua_resources"))
        .and_then(|lr| lr.get("constructors"))
        .and_then(|c| c.as_array());
    
    constructors_array
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str())
            .filter_map(|s| parse_constructor_spec(s))
            .collect())
        .unwrap_or_default()
}

fn parse_constructor_spec(full_path: &str) -> Option<ConstructorSpec> {
    let parts: Vec<&str> = full_path.split("::").collect();
    if parts.len() < 3 { return None; }
    let function_name = parts.last()?.to_string();
    let type_path = parts[..parts.len()-1].join("::");
    let type_spec = parse_type_spec(&type_path)?;
    Some(ConstructorSpec { 
        full_path: full_path.to_string(), 
        type_spec, 
        function_name 
    })
}

/// Get entity_components from TOML metadata - these are newtypes wrapping Entity
/// Example: entity_components = ["bevy_ui::ui_node::UiTargetCamera"]
fn get_entity_components_from_metadata(manifest: &toml::Value) -> Vec<String> {
    let entity_components_array = manifest
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("lua_resources"))
        .and_then(|lr| lr.get("entity_components"))
        .and_then(|ec| ec.as_array());
    
    entity_components_array
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect())
        .unwrap_or_default()
}

// =============================================================================
// AUTO-DISCOVERY FUNCTIONS
// These scan bevy source code to find entity wrappers and asset types
// =============================================================================

/// Discovered entity wrapper component (newtype around Entity)
#[derive(Debug, Clone)]
struct DiscoveredEntityWrapper {
    /// Full type path (e.g., "bevy_ui::ui_node::UiTargetCamera")
    full_path: String,
    /// Just the type name (e.g., "UiTargetCamera")
    type_name: String,
}

/// Discovered asset type (implements Asset trait)
#[derive(Debug, Clone)]
struct DiscoveredAssetType {
    /// Full type path (e.g., "bevy_mesh::mesh::Mesh")
    full_path: String,
    /// Crate name (e.g., "bevy_mesh")
    crate_name: String,
    /// Module path (e.g., ["mesh"])
    module_path: Vec<String>,
    /// Type name (e.g., "Mesh")
    type_name: String,
    /// Whether the type implements Clone (detected via #[derive(Clone)] or impl Clone)
    has_clone: bool,
    /// Whether the struct has generic type parameters (can't be instantiated without concrete types)
    is_generic: bool,
}

/// A constructor parameter with name and type
#[derive(Debug, Clone)]
struct ConstructorParam {
    /// Parameter name (e.g., "width")
    name: String,
    /// Parameter type as string (e.g., "u32", "TextureFormat")
    type_str: String,
}

/// Discovered asset constructor method
#[derive(Debug, Clone)]
struct DiscoveredAssetConstructor {
    /// Full type path (e.g., "bevy_image::image::Image")
    type_path: String,
    /// Type name only (e.g., "Image")
    type_name: String,
    /// Constructor method name (e.g., "new_target_texture")
    method_name: String,
    /// Parameters with names and types
    params: Vec<ConstructorParam>,
}


/// Discovered Handle<T> newtype wrapper (e.g., ImageRenderTarget wraps Handle<Image>)
#[derive(Debug, Clone)]
struct DiscoveredHandleNewtype {
    /// Full newtype path (e.g., "bevy_render::camera::ImageRenderTarget")
    newtype_path: String,
    /// Just the type name (e.g., "ImageRenderTarget")
    type_name: String,
    /// Inner asset type name extracted from Handle<T> (e.g., "Image")
    inner_asset_name: String,
    /// Full inner asset path if determinable (e.g., "bevy_image::image::Image")
    inner_asset_path: Option<String>,
}

/// Discovered SystemParam type (uses #[derive(SystemParam)])
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveredSystemParam {
    /// Full type path (e.g., "bevy_picking::mesh_picking::ray_cast::MeshRayCast")
    full_path: String,
    /// Just the type name (e.g., "MeshRayCast")
    type_name: String,
    /// Crate name (e.g., "bevy_picking")
    crate_name: String,
}

/// A method parameter with name and type for SystemParam methods
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemParamMethodParam {
    /// Parameter name (e.g., "ray")
    name: String,
    /// Parameter type as string (e.g., "Ray3d", "&MeshRayCastSettings")
    type_str: String,
    /// Whether this is a reference type
    is_reference: bool,
    /// Whether this type likely implements Reflect (based on common patterns)
    is_reflectable: bool,
}

/// Discovered method on a SystemParam type
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveredSystemParamMethod {
    /// SystemParam type name (e.g., "MeshRayCast")
    param_type: String,
    /// Method name (e.g., "cast_ray")
    method_name: String,
    /// Parameters (excluding &self/&mut self)
    params: Vec<SystemParamMethodParam>,
    /// Return type as string
    return_type: String,
    /// Whether return type is an iterator (needs .collect())
    returns_iterator: bool,
}

/// Cache for discovered SystemParam types and methods
/// Stored as JSON in target directory to speed up subsequent builds
#[derive(Debug, Serialize, Deserialize)]
struct SystemParamCache {
    /// Hash of Cargo.lock to detect dependency changes
    cargo_lock_hash: String,
    /// Discovered SystemParam types
    systemparams: Vec<DiscoveredSystemParam>,
    /// Discovered methods on those types
    methods: Vec<DiscoveredSystemParamMethod>,
}

/// Get cache file path in the target directory
fn get_systemparam_cache_path() -> Option<PathBuf> {
    let out_dir = env::var("OUT_DIR").ok()?;
    Some(PathBuf::from(out_dir).join("systemparam_cache.json"))
}

/// Compute a simple hash of Cargo.lock for cache invalidation
fn compute_cargo_lock_hash() -> String {
    // Try multiple locations for Cargo.lock
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let paths_to_try = [
        PathBuf::from(&manifest_dir).join("../Hello/Cargo.lock"),
        PathBuf::from(&manifest_dir).join("Cargo.lock"),
        PathBuf::from(&manifest_dir).parent().map(|p| p.join("Cargo.lock")).unwrap_or_default(),
    ];
    
    for path in &paths_to_try {
        if let Ok(content) = fs::read_to_string(path) {
            // Simple hash: use length and first/last bytes
            let len = content.len();
            let first = content.bytes().next().unwrap_or(0) as u64;
            let last = content.bytes().last().unwrap_or(0) as u64;
            return format!("{}-{}-{}", len, first, last);
        }
    }
    
    // No Cargo.lock found, use timestamp to invalidate frequently
    format!("no-lock-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0))
}

/// Try to load SystemParam cache from disk
fn load_systemparam_cache() -> Option<SystemParamCache> {
    let cache_path = get_systemparam_cache_path()?;
    let content = fs::read_to_string(&cache_path).ok()?;
    let cache: SystemParamCache = serde_json::from_str(&content).ok()?;
    
    // Check if cache is still valid (Cargo.lock hasn't changed)
    let current_hash = compute_cargo_lock_hash();
    if cache.cargo_lock_hash == current_hash {
        println!("cargo:warning=[CACHE] Loaded {} SystemParams and {} methods from cache", 
            cache.systemparams.len(), cache.methods.len());
        Some(cache)
    } else {
        println!("cargo:warning=[CACHE] Cache invalidated (Cargo.lock changed)");
        None
    }
}

/// Save SystemParam cache to disk
fn save_systemparam_cache(systemparams: &[DiscoveredSystemParam], methods: &[DiscoveredSystemParamMethod]) {
    let Some(cache_path) = get_systemparam_cache_path() else { return };
    
    let cache = SystemParamCache {
        cargo_lock_hash: compute_cargo_lock_hash(),
        systemparams: systemparams.to_vec(),
        methods: methods.to_vec(),
    };
    
    if let Ok(json) = serde_json::to_string_pretty(&cache) {
        if fs::write(&cache_path, json).is_ok() {
            println!("cargo:warning=[CACHE] Saved {} SystemParams and {} methods to cache",
                systemparams.len(), methods.len());
        }
    }
}

/// Get discovered SystemParams and methods (from cache or by discovery)
fn get_discovered_systemparams_and_methods() -> (Vec<DiscoveredSystemParam>, Vec<DiscoveredSystemParamMethod>) {
    // Try cache first
    if let Some(cache) = load_systemparam_cache() {
        return (cache.systemparams, cache.methods);
    }
    
    // Cache miss - run discovery
    println!("cargo:warning=[CACHE] Cache miss - running SystemParam discovery...");
    let systemparams = discover_systemparam_types();
    let methods = discover_systemparam_methods(&systemparams);
    
    // Save to cache for next time
    save_systemparam_cache(&systemparams, &methods);
    
    (systemparams, methods)
}

/// Resolve a short type name to its full Bevy path
/// For compile-time code generation, we need fully qualified paths
fn resolve_short_type_to_full_path(short_name: &str) -> Option<String> {
    // Map of short names to full Bevy paths
    // These are the most common types found in SystemParam method signatures
    let mappings: &[(&str, &str)] = &[
        // Math types
        ("Ray3d", "bevy::math::Ray3d"),
        ("Dir3", "bevy::math::Dir3"),
        ("Dir3A", "bevy::math::Dir3A"),
        ("Vec2", "bevy::math::Vec2"),
        ("Vec3", "bevy::math::Vec3"),
        ("Vec3A", "bevy::math::Vec3A"),
        ("Vec4", "bevy::math::Vec4"),
        ("Quat", "bevy::math::Quat"),
        ("Mat4", "bevy::math::Mat4"),
        
        // Picking types
        ("MeshRayCastSettings", "bevy::picking::mesh_picking::ray_cast::MeshRayCastSettings"),
        ("RayCastSettings", "bevy::picking::mesh_picking::ray_cast::RayCastSettings"),
        
        // ECS types
        ("Entity", "bevy::ecs::entity::Entity"),
        
        // Common types - these are primitives and don't need full paths
        // Just return them as-is if they parse ok
    ];
    
    for (short, full) in mappings {
        if short_name == *short {
            return Some(full.to_string());
        }
    }
    
    // If it already contains ::, assume it's a full path
    if short_name.contains("::") {
        return Some(short_name.to_string());
    }
    
    // Primitive types don't need resolution
    if is_primitive_type(short_name) {
        return Some(short_name.to_string());
    }
    
    // Unknown type - can't resolve
    None
}

/// Auto-discover entity wrapper components from bevy crates and workspace members
/// Pattern: pub struct Foo(pub Entity) with #[derive(Component)]
/// 
/// RUNTIME-BASED DISCOVERY: This function discovers ALL entity wrapper type names.
/// No filtering is applied - runtime TypeRegistry lookup will determine which types
/// are actually available and usable.
fn discover_entity_wrapper_components() -> Vec<DiscoveredEntityWrapper> {
    let mut wrappers = Vec::new();
    
    // Get cargo registry path
    let cargo_home = env::var("CARGO_HOME")
        .or_else(|_| env::var("HOME").map(|h| format!("{}/.cargo", h)))
        .or_else(|_| env::var("USERPROFILE").map(|h| format!("{}/.cargo", h)))
        .unwrap_or_default();
    
    if cargo_home.is_empty() {
        println!("cargo:warning=  ⚠ Cannot find CARGO_HOME for entity wrapper discovery");
        return wrappers;
    }
    
    let registry_src = PathBuf::from(&cargo_home).join("registry").join("src");
    
    if !registry_src.exists() {
        println!("cargo:warning=  ⚠ Registry source not found for entity wrapper discovery");
        return wrappers;
    }
    
    // Read ALL bevy_* dependencies from Cargo.lock - no filtering
    let dependencies = get_bevy_dependencies_from_lock();
    println!("cargo:warning=  📦 Found {} bevy_* dependencies in Cargo.lock", dependencies.len());
    
    // Scan ALL bevy_* dependency crates for entity wrappers
    for index_entry in fs::read_dir(&registry_src).into_iter().flatten().flatten() {
        let index_dir = index_entry.path();
        if !index_dir.is_dir() { continue; }
        
        for crate_entry in fs::read_dir(&index_dir).into_iter().flatten().flatten() {
            let crate_dir = crate_entry.path();
            let dir_name = crate_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // Extract base crate name (e.g., "bevy_ui" from "bevy_ui-0.17.2")
            let base_crate = dir_name.split('-').next().unwrap_or(dir_name);
            
            // Only scan crates that are in our dependencies
            if !dependencies.contains(&base_crate.to_string()) { continue; }
            
            let src_dir = crate_dir.join("src");
            if src_dir.exists() {
                scan_directory_for_entity_wrappers(&src_dir, base_crate, &mut wrappers);
            }
        }
    }
    
    // Also scan workspace members
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        if let Some(workspace_root) = PathBuf::from(&manifest_dir).parent() {
            let workspace_toml = workspace_root.join("Cargo.toml");
            if let Ok(content) = fs::read_to_string(&workspace_toml) {
                if let Ok(manifest) = toml::from_str::<toml::Value>(&content) {
                    if let Some(members) = manifest.get("workspace")
                        .and_then(|w| w.get("members"))
                        .and_then(|m| m.as_array()) 
                    {
                        for member in members {
                            if let Some(member_name) = member.as_str() {
                                let member_src = workspace_root.join(member_name).join("src");
                                if member_src.exists() {
                                    scan_directory_for_entity_wrappers(&member_src, member_name, &mut wrappers);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Deduplicate by type_name - we only care about the short name for runtime lookup
    let mut seen = std::collections::HashSet::new();
    wrappers.retain(|w| seen.insert(w.type_name.clone()));
    
    println!("cargo:warning=  ✓ Auto-discovered {} entity wrapper type names (for runtime registration)", wrappers.len());
    for wrapper in &wrappers {
        println!("cargo:warning=    - {} (from {})", wrapper.type_name, wrapper.full_path);
    }
    
    wrappers
}

/// Read Cargo.lock to get list of bevy_* crate names that are actual dependencies
fn get_bevy_dependencies_from_lock() -> Vec<String> {
    let mut deps = Vec::new();
    
    // Try to find Cargo.lock in workspace root or parent manifest dir
    let lock_paths = [
        env::var("CARGO_MANIFEST_DIR").ok().map(|p| PathBuf::from(p).parent().map(|p| p.join("Cargo.lock"))).flatten(),
        env::var("CARGO_MANIFEST_DIR").ok().map(|p| PathBuf::from(p).join("Cargo.lock")),
    ];
    
    for lock_path in lock_paths.into_iter().flatten() {
        if let Ok(content) = fs::read_to_string(&lock_path) {
            // Simple parsing of Cargo.lock - look for [[package]] sections with bevy_* names
            for line in content.lines() {
                if line.starts_with("name = \"bevy_") {
                    // Extract crate name from: name = "bevy_something"
                    if let Some(start) = line.find('"') {
                        if let Some(end) = line.rfind('"') {
                            if start < end {
                                let name = &line[start+1..end];
                                if !deps.contains(&name.to_string()) {
                                    deps.push(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
            if !deps.is_empty() {
                break; // Found deps, stop looking
            }
        }
    }
    
    deps
}

/// Scan a directory recursively for entity wrapper components
fn scan_directory_for_entity_wrappers(dir: &Path, crate_name: &str, results: &mut Vec<DiscoveredEntityWrapper>) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        
        if path.is_dir() {
            scan_directory_for_entity_wrappers(&path, crate_name, results);
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            if let Ok(source) = fs::read_to_string(&path) {
                parse_entity_wrappers_from_source(&source, crate_name, &path, results);
            }
        }
    }
}

/// Parse a source file for entity wrapper patterns
/// Pattern: pub struct Foo(pub Entity) with #[derive(Component)]
fn parse_entity_wrappers_from_source(source: &str, crate_name: &str, file_path: &Path, results: &mut Vec<DiscoveredEntityWrapper>) {
    let Ok(file) = syn::parse_file(source) else { return; };
    
    for item in file.items {
        if let syn::Item::Struct(item_struct) = item {
            // Must be public
            if !matches!(item_struct.vis, syn::Visibility::Public(_)) {
                continue;
            }
            
            // Check if it's a tuple struct with single Entity field
            if let syn::Fields::Unnamed(fields) = &item_struct.fields {
                if fields.unnamed.len() != 1 {
                    continue;
                }
                
                let field = &fields.unnamed[0];
                
                // Check if field type is Entity
                let field_type_str = quote::quote!(#field).to_string();
                if !field_type_str.contains("Entity") {
                    continue;
                }
                
                // Check for Component derive
                let has_component = item_struct.attrs.iter().any(|attr| {
                    if attr.path().is_ident("derive") {
                        if let Ok(meta) = attr.parse_args::<proc_macro2::TokenStream>() {
                            return meta.to_string().contains("Component");
                        }
                    }
                    false
                });
                
                if !has_component {
                    continue;
                }
                
                let type_name = item_struct.ident.to_string();
                
                // Build full path from file path
                let module_path = build_module_path_from_file(file_path, crate_name);
                let full_path = if module_path.is_empty() {
                    format!("{}::{}", crate_name, type_name)
                } else {
                    format!("{}::{}::{}", crate_name, module_path, type_name)
                };
                
                // Convert underscore crate name to bevy:: path for bevy crates
                // Skip if path cannot be normalized (internal modules)
                let Some(full_path) = normalize_bevy_path(&full_path) else {
                    continue;
                };
                
                results.push(DiscoveredEntityWrapper {
                    full_path,
                    type_name,
                });
            }
        }
    }
}

/// Auto-discover Handle<T> newtype wrappers from bevy crates
/// Pattern: pub struct TypeName(pub Handle<AssetType>) or pub struct TypeName { handle: Handle<AssetType> }
/// Examples: ImageRenderTarget, etc.
fn discover_handle_newtype_wrappers() -> Vec<DiscoveredHandleNewtype> {
    let mut wrappers = Vec::new();
    
    println!("cargo:warning=[NEWTYPE_DISCOVERY] Starting Handle<T> newtype wrapper discovery...");
    
    // Get cargo home
    let cargo_home = env::var("CARGO_HOME")
        .or_else(|_| env::var("HOME").map(|h| format!("{}/.cargo", h)))
        .or_else(|_| env::var("USERPROFILE").map(|h| format!("{}/.cargo", h)))
        .unwrap_or_default();
    
    if cargo_home.is_empty() {
        println!("cargo:warning=  ⚠ Cannot find CARGO_HOME for newtype wrapper discovery");
        return wrappers;
    }
    
    let registry_src = PathBuf::from(&cargo_home).join("registry").join("src");
    
    if !registry_src.exists() {
        println!("cargo:warning=  ⚠ Registry source not found for newtype wrapper discovery");
        return wrappers;
    }
    
    // Read ALL bevy_* dependencies from Cargo.lock - no filtering
    let dependencies = get_bevy_dependencies_from_lock();
    println!("cargo:warning=[NEWTYPE_DISCOVERY] Scanning {} bevy_* dependencies", dependencies.len());
    
    // Scan ALL bevy_* dependency crates for Handle newtypes
    for index_entry in fs::read_dir(&registry_src).into_iter().flatten().flatten() {
        let index_dir = index_entry.path();
        if !index_dir.is_dir() { continue; }
        
        for crate_entry in fs::read_dir(&index_dir).into_iter().flatten().flatten() {
            let crate_dir = crate_entry.path();
            let dir_name = crate_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // Extract base crate name (e.g., "bevy_render" from "bevy_render-0.17.2")
            let base_crate = dir_name.split('-').next().unwrap_or(dir_name);
            
            // Only scan crates that are in our dependencies
            if !dependencies.contains(&base_crate.to_string()) { continue; }
            
            let src_dir = crate_dir.join("src");
            if src_dir.exists() {
                scan_directory_for_handle_newtypes(&src_dir, base_crate, &mut wrappers);
            }
        }
    }
    
    // Deduplicate by type_name (keep first occurrence)
    let mut seen_names = std::collections::HashSet::new();
    wrappers.retain(|w| seen_names.insert(w.type_name.clone()));
    
    println!("cargo:warning=  ✓ Auto-discovered {} Handle<T> newtype wrappers", wrappers.len());
    for wrapper in &wrappers {
        println!("cargo:warning=    - {} (wraps Handle<{}>)", wrapper.type_name, wrapper.inner_asset_name);
    }
    
    wrappers
}

/// Scan a directory recursively for Handle<T> newtype wrappers
fn scan_directory_for_handle_newtypes(dir: &Path, crate_name: &str, results: &mut Vec<DiscoveredHandleNewtype>) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        
        if path.is_dir() {
            scan_directory_for_handle_newtypes(&path, crate_name, results);
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            if let Ok(source) = fs::read_to_string(&path) {
                parse_handle_newtypes_from_source(&source, crate_name, &path, results);
            }
        }
    }
}

/// Parse a source file for Handle<T> newtype wrapper patterns
/// Pattern: pub struct TypeName(pub Handle<AssetType>) with any derive macros
fn parse_handle_newtypes_from_source(source: &str, crate_name: &str, file_path: &Path, results: &mut Vec<DiscoveredHandleNewtype>) {
    let Ok(file) = syn::parse_file(source) else { return; };
    
    for item in file.items {
        if let syn::Item::Struct(item_struct) = item {
            // Must be public
            if !matches!(item_struct.vis, syn::Visibility::Public(_)) {
                continue;
            }
            
            // Skip generic structs (they need type parameters)
            if !item_struct.generics.params.is_empty() {
                continue;
            }
            
            let type_name = item_struct.ident.to_string();
            
            // Check for tuple struct pattern: TypeName(Handle<T>)
            if let syn::Fields::Unnamed(fields) = &item_struct.fields {
                if fields.unnamed.len() != 1 {
                    continue;
                }
                
                let field = &fields.unnamed[0];
                let field_type_str = quote::quote!(#field).to_string();
                
                // Check if this is a Handle<T> wrapper
                if let Some(inner_asset) = extract_handle_inner_type(&field_type_str) {
                    // Build full path
                    let module_path = build_module_path_from_file(file_path, crate_name);
                    let full_path = if module_path.is_empty() {
                        format!("{}::{}", crate_name, type_name)
                    } else {
                        format!("{}::{}::{}", crate_name, module_path, type_name)
                    };
                    
                    // Skip if path cannot be normalized (internal modules)
                    let Some(newtype_path) = normalize_bevy_path(&full_path) else {
                        continue;
                    };
                    
                    results.push(DiscoveredHandleNewtype {
                        newtype_path,
                        type_name: type_name.clone(),
                        inner_asset_name: inner_asset,
                        inner_asset_path: None, // Will be resolved at runtime via TypeRegistry
                    });
                }
            }
            
            // Also check for struct pattern: TypeName { handle: Handle<T> } (single field)
            if let syn::Fields::Named(fields) = &item_struct.fields {
                if fields.named.len() != 1 {
                    continue;
                }
                
                let field = &fields.named[0];
                let field_type_str = quote::quote!(#field).to_string();
                
                // Check if this is a Handle<T> wrapper
                if let Some(inner_asset) = extract_handle_inner_type(&field_type_str) {
                    // Build full path
                    let module_path = build_module_path_from_file(file_path, crate_name);
                    let full_path = if module_path.is_empty() {
                        format!("{}::{}", crate_name, type_name)
                    } else {
                        format!("{}::{}::{}", crate_name, module_path, type_name)
                    };
                    
                    // Skip if path cannot be normalized (internal modules)
                    let Some(newtype_path) = normalize_bevy_path(&full_path) else {
                        continue;
                    };
                    
                    results.push(DiscoveredHandleNewtype {
                        newtype_path,
                        type_name: type_name.clone(),
                        inner_asset_name: inner_asset,
                        inner_asset_path: None,
                    });
                }
            }
        }
    }
}

/// Extract the inner type name from a Handle<T> type string
/// Returns Some("Image") for "Handle<Image>" or "Handle < Image >"
fn extract_handle_inner_type(type_str: &str) -> Option<String> {
    // Look for Handle<...>
    if !type_str.contains("Handle") {
        return None;
    }
    
    // Find the angle bracket content
    let start = type_str.find('<')?;
    let end = type_str.rfind('>')?;
    
    if start >= end {
        return None;
    }
    
    // Extract and clean the inner type
    let inner = type_str[start+1..end].trim();
    
    // Get just the type name (last segment if it's a path)
    let type_name = inner.split("::").last().unwrap_or(inner).trim();
    
    // Skip if empty or looks like a generic parameter
    if type_name.is_empty() || type_name.len() == 1 {
        return None;
    }
    
    Some(type_name.to_string())
}

/// Auto-discover asset types from bevy crates and workspace members

/// Scans bevy_* crates in Cargo registry for types implementing Asset
/// Auto-discover asset types from bevy crates and workspace members
/// Pattern: impl Asset for Type OR #[derive(Asset)] struct Type
/// 
/// RUNTIME-BASED DISCOVERY: This function discovers ALL asset type names.
/// No filtering is applied - runtime TypeRegistry lookup will determine which types
/// are actually available and usable.
fn discover_asset_types() -> Vec<DiscoveredAssetType> {
    let mut assets = Vec::new();
    
    println!("cargo:warning=[ASSET_DISCOVERY] Starting asset type discovery (no filtering)...");
    
    // Read ALL bevy_* dependencies from Cargo.lock - no filtering
    let dependencies = get_bevy_dependencies_from_lock();
    println!("cargo:warning=[ASSET_DISCOVERY] Found {} bevy_* dependencies in Cargo.lock", dependencies.len());
    
    // Scan ALL bevy_* crates in cargo registry  
    if let Ok(home) = env::var("CARGO_HOME").or_else(|_| env::var("USERPROFILE").map(|p| format!("{}/.cargo", p))) {
        let registry_src = PathBuf::from(&home).join("registry").join("src");
        
        if registry_src.exists() {
            // Find the registry index directory
            if let Ok(entries) = fs::read_dir(&registry_src) {
                for entry in entries.flatten() {
                    let index_dir = entry.path();
                    if !index_dir.is_dir() { continue; }
                    
                    // Scan bevy_* crates
                    if let Ok(crate_entries) = fs::read_dir(&index_dir) {
                        for crate_entry in crate_entries.flatten() {
                            let crate_name = crate_entry.file_name().to_string_lossy().to_string();
                            
                            // Extract base crate name (e.g., "bevy_mesh" from "bevy_mesh-0.17.2")
                            let base_crate = crate_name.split('-').next().unwrap_or(&crate_name);
                            
                            // Only scan crates that are actual dependencies
                            if !dependencies.contains(&base_crate.to_string()) { continue; }
                            
                            let crate_src = crate_entry.path().join("src");
                            if crate_src.exists() {
                                scan_directory_for_asset_types(&crate_src, base_crate, &mut assets);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Also scan workspace members for custom asset types
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        if let Some(workspace_root) = PathBuf::from(&manifest_dir).parent() {
            let workspace_toml = workspace_root.join("Cargo.toml");
            if let Ok(content) = fs::read_to_string(&workspace_toml) {
                if let Ok(manifest) = toml::from_str::<toml::Value>(&content) {
                    if let Some(members) = manifest.get("workspace")
                        .and_then(|w| w.get("members"))
                        .and_then(|m| m.as_array()) 
                    {
                        for member in members {
                            if let Some(member_name) = member.as_str() {
                                let member_src = workspace_root.join(member_name).join("src");
                                if member_src.exists() {
                                    scan_directory_for_asset_types(&member_src, member_name, &mut assets);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Only filter GENERIC types (contain angle brackets) - these are syntactically invalid
    assets.retain(|asset| {
        let type_name = &asset.type_name;
        
        // Skip generic types (contain angle brackets in name) - can't be used in const array
        if type_name.contains('<') || type_name.contains('>') {
            return false;
        }
        
        true
    });
    
    // Deduplicate by type_name (keep first occurrence)
    let mut seen_names = std::collections::HashSet::new();
    assets.retain(|a| seen_names.insert(a.type_name.clone()));
    
    println!("cargo:warning=  ✓ Auto-discovered {} asset type names (for runtime registration)", assets.len());
    for asset in &assets {
        println!("cargo:warning=    - {} (from {})", asset.type_name, asset.full_path);
    }
    
    assets
}

/// Scan a directory recursively for asset types
fn scan_directory_for_asset_types(dir: &Path, crate_name: &str, results: &mut Vec<DiscoveredAssetType>) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        
        if path.is_dir() {
            scan_directory_for_asset_types(&path, crate_name, results);
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            if let Ok(source) = fs::read_to_string(&path) {
                parse_asset_types_from_source(&source, crate_name, &path, results);
            }
        }
    }
}

/// Parse a source file for Asset implementations
/// Pattern: impl Asset for Type OR #[derive(Asset)] struct Type
/// Also detects Clone implementation via #[derive(Clone)] or impl Clone
fn parse_asset_types_from_source(source: &str, crate_name: &str, file_path: &Path, results: &mut Vec<DiscoveredAssetType>) {
    let Ok(file) = syn::parse_file(source) else { return; };
    
    // Track types that derive Asset and whether they also derive Clone
    // Key: type name, Value: (has_asset, has_clone, is_generic)
    let mut type_info: std::collections::HashMap<String, (bool, bool, bool)> = std::collections::HashMap::new();
    
    // First pass: find #[derive(Asset)] and #[derive(Clone)] structs
    for item in &file.items {
        if let syn::Item::Struct(item_struct) = item {
            // Must be public
            if !matches!(item_struct.vis, syn::Visibility::Public(_)) {
                continue;
            }
            
            let type_name = item_struct.ident.to_string();
            
            // Check if struct has generic type parameters (can't instantiate without concrete types)
            let is_generic = !item_struct.generics.params.is_empty();
            
            // Check for Asset and Clone derives in all attributes
            for attr in &item_struct.attrs {
                if attr.path().is_ident("derive") {
                    if let Ok(meta) = attr.parse_args::<proc_macro2::TokenStream>() {
                        let derive_str = meta.to_string();
                        let entry = type_info.entry(type_name.clone()).or_insert((false, false, is_generic));
                        if derive_str.contains("Asset") {
                            entry.0 = true;
                        }
                        if derive_str.contains("Clone") {
                            entry.1 = true;
                        }
                    }
                }
            }
        }
    }
    
    // Second pass: find impl Asset for Type and impl Clone for Type
    for item in &file.items {
        if let syn::Item::Impl(item_impl) = item {
            // Check if implementing Asset or Clone trait
            if let Some((_, trait_path, _)) = &item_impl.trait_ {
                let trait_name = trait_path.segments.last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default();
                
                // Get the type being implemented
                if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                    if let Some(seg) = type_path.path.segments.last() {
                        let type_name = seg.ident.to_string();
                        // Check if impl has generic parameters (e.g., impl<B, E> Asset for ExtendedMaterial<B, E>)
                        let is_generic = !item_impl.generics.params.is_empty();
                        let entry = type_info.entry(type_name).or_insert((false, false, is_generic));
                        
                        if trait_name == "Asset" {
                            entry.0 = true;
                        }
                        if trait_name == "Clone" {
                            entry.1 = true;
                        }
                    }
                }
            }
        }
    }
    
    // Build full paths for discovered assets (only types that implement Asset)
    for (type_name, (has_asset, has_clone, is_generic)) in type_info {
        if !has_asset {
            continue;
        }
        
        let module_path_str = build_module_path_from_file(file_path, crate_name);
        let module_path: Vec<String> = if module_path_str.is_empty() {
            Vec::new()
        } else {
            module_path_str.split("::").map(|s| s.to_string()).collect()
        };
        
        let full_path = if module_path_str.is_empty() {
            format!("{}::{}", crate_name, type_name)
        } else {
            format!("{}::{}::{}", crate_name, module_path_str, type_name)
        };
        
        results.push(DiscoveredAssetType {
            full_path,
            crate_name: crate_name.to_string(),
            module_path,
            type_name,
            has_clone,
            is_generic,
        });
    }
}

/// Build module path from file path relative to src/
fn build_module_path_from_file(file_path: &Path, _crate_name: &str) -> String {
    let file_name = file_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    
    // For lib.rs and mod.rs, module is parent directory
    if file_name == "lib" || file_name == "mod" {
        // Get parent directory name
        if let Some(parent) = file_path.parent() {
            if let Some(parent_name) = parent.file_name().and_then(|s| s.to_str()) {
                if parent_name != "src" {
                    return parent_name.to_string();
                }
            }
        }
        return String::new();
    }
    
    // Otherwise module is the file name
    file_name.to_string()
}

/// Auto-discover constructors for asset types
/// Scans for `impl TypeName` blocks with methods like new_*, from_*, etc. that return Self
fn discover_asset_constructors(asset_types: &[DiscoveredAssetType]) -> Vec<DiscoveredAssetConstructor> {
    let mut constructors = Vec::new();
    
    println!("cargo:warning=[CONSTRUCTOR_DISCOVERY] Scanning for asset constructors...");
    
    // Get cargo home
    let cargo_home = env::var("CARGO_HOME")
        .or_else(|_| env::var("HOME").map(|h| format!("{}/.cargo", h)))
        .or_else(|_| env::var("USERPROFILE").map(|h| format!("{}/.cargo", h)))
        .unwrap_or_default();
    
    if cargo_home.is_empty() {
        return constructors;
    }
    
    let registry_src = PathBuf::from(&cargo_home).join("registry").join("src");
    if !registry_src.exists() {
        return constructors;
    }
    
    let dependencies = get_bevy_dependencies_from_lock();
    
    // Scan bevy_* crates for constructors
    for index_entry in fs::read_dir(&registry_src).into_iter().flatten().flatten() {
        let index_dir = index_entry.path();
        if !index_dir.is_dir() { continue; }
        
        for crate_entry in fs::read_dir(&index_dir).into_iter().flatten().flatten() {
            let crate_dir = crate_entry.path();
            let dir_name = crate_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            let base_crate = dir_name.split('-').next().unwrap_or(dir_name);
            if !dependencies.contains(&base_crate.to_string()) { continue; }
            
            let src_dir = crate_dir.join("src");
            if src_dir.exists() {
                scan_directory_for_constructors(&src_dir, base_crate, asset_types, &mut constructors);
            }
        }
    }
    
    // Prioritize certain constructor names for RTT use cases
    // new_target_texture > new_fill > new > default
    let priority = |name: &str| -> i32 {
        match name {
            "new_target_texture" => 0,
            n if n.starts_with("new_") => 1,
            "new" => 2,
            n if n.starts_with("from_") => 3,
            _ => 4,
        }
    };
    
    constructors.sort_by(|a, b| priority(&a.method_name).cmp(&priority(&b.method_name)));
    
    // Deduplicate by type_path (keep highest priority constructor)
    let mut seen = std::collections::HashSet::new();
    constructors.retain(|c| seen.insert(c.type_path.clone()));
    
    println!("cargo:warning=  ✓ Discovered {} asset constructors", constructors.len());
    for c in &constructors {
        let params: Vec<String> = c.params.iter().map(|p| format!("{}: {}", p.name, p.type_str)).collect();
        println!("cargo:warning=    - {}::{}({})", c.type_name, c.method_name, params.join(", "));
    }
    
    constructors
}

/// Scan directory for constructor methods
fn scan_directory_for_constructors(
    dir: &Path, 
    crate_name: &str, 
    asset_types: &[DiscoveredAssetType],
    results: &mut Vec<DiscoveredAssetConstructor>
) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        
        if path.is_dir() {
            scan_directory_for_constructors(&path, crate_name, asset_types, results);
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            if let Ok(source) = fs::read_to_string(&path) {
                parse_constructors_from_source(&source, crate_name, &path, asset_types, results);
            }
        }
    }
}

/// Parse source file for constructor methods in `impl TypeName` blocks
fn parse_constructors_from_source(
    source: &str,
    crate_name: &str,
    file_path: &Path,
    asset_types: &[DiscoveredAssetType],
    results: &mut Vec<DiscoveredAssetConstructor>,
) {
    let Ok(file) = syn::parse_file(source) else { return; };
    
    for item in file.items {
        if let syn::Item::Impl(item_impl) = item {
            // Only look at inherent impls (not trait impls)
            if item_impl.trait_.is_some() { continue; }
            
            // Get the type being implemented
            let type_name = match &*item_impl.self_ty {
                syn::Type::Path(type_path) => {
                    type_path.path.segments.last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_default()
                }
                _ => continue,
            };
            
            // Check if this is an asset type we care about
            let asset_type = asset_types.iter().find(|a| a.type_name == type_name);
            if asset_type.is_none() { continue; }
            let asset_type = asset_type.unwrap();
            
            // Look for constructor methods
            for impl_item in &item_impl.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    // Must be public
                    if !matches!(method.vis, syn::Visibility::Public(_)) { continue; }
                    
                    let method_name = method.sig.ident.to_string();
                    
                    // Check if it looks like a constructor (new*, from*, etc.)
                    if !is_constructor_name(&method_name) { continue; }
                    
                    // Check if it returns Self (or the type name)
                    if !returns_self_or_type(&method.sig.output, &type_name) { continue; }
                    
                    // Parse parameters (skip &self, &mut self)
                    let params = parse_method_params(&method.sig);
                    
                    // Must have at least one parameter
                    if params.is_empty() { continue; }
                    
                    results.push(DiscoveredAssetConstructor {
                        type_path: asset_type.full_path.clone(),
                        type_name: type_name.clone(),
                        method_name,
                        params,
                    });
                }
            }
        }
    }
}

/// Check if method name looks like a constructor
fn is_constructor_name(name: &str) -> bool {
    name.starts_with("new") || name.starts_with("from_") || name == "default"
}

/// Check if return type is Self or the type name
fn returns_self_or_type(output: &syn::ReturnType, type_name: &str) -> bool {
    match output {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => {
            let ty_str = quote::quote!(#ty).to_string();
            ty_str == "Self" || ty_str.contains(type_name)
        }
    }
}

/// Parse method parameters, skipping self parameters
fn parse_method_params(sig: &syn::Signature) -> Vec<ConstructorParam> {
    let mut params = Vec::new();
    
    for arg in &sig.inputs {
        match arg {
            syn::FnArg::Receiver(_) => continue, // Skip &self, &mut self
            syn::FnArg::Typed(pat_type) => {
                // Get parameter name
                let name = match &*pat_type.pat {
                    syn::Pat::Ident(ident) => ident.ident.to_string(),
                    _ => continue,
                };
                
                // Get type as string - need to extract just the type, not the whole pattern
                let ty = &pat_type.ty;
                let type_str = quote::quote!(#ty).to_string()
                    .replace(" ", "")
                    .replace("&", "");
                
                params.push(ConstructorParam { name, type_str });
            }
        }
    }
    
    params
}

// =============================================================================
// SYSTEMPARAM AUTO-DISCOVERY
// Scan bevy crates for #[derive(SystemParam)] types and their methods
// =============================================================================

/// Auto-discover SystemParam types from bevy crates
/// Pattern: #[derive(SystemParam)] pub struct TypeName
fn discover_systemparam_types() -> Vec<DiscoveredSystemParam> {
    let mut params = Vec::new();
    
    println!("cargo:warning=[SYSTEMPARAM_DISCOVERY] Starting SystemParam type discovery...");
    
    // Get cargo home
    let cargo_home = env::var("CARGO_HOME")
        .or_else(|_| env::var("HOME").map(|h| format!("{}/.cargo", h)))
        .or_else(|_| env::var("USERPROFILE").map(|h| format!("{}/.cargo", h)))
        .unwrap_or_default();
    
    if cargo_home.is_empty() {
        println!("cargo:warning=  ⚠ Cannot find CARGO_HOME for SystemParam discovery");
        return params;
    }
    
    let registry_src = PathBuf::from(&cargo_home).join("registry").join("src");
    
    if !registry_src.exists() {
        println!("cargo:warning=  ⚠ Registry source not found for SystemParam discovery");
        return params;
    }
    
    // Read ALL bevy_* dependencies from Cargo.lock
    let dependencies = get_bevy_dependencies_from_lock();
    println!("cargo:warning=[SYSTEMPARAM_DISCOVERY] Scanning {} bevy_* dependencies", dependencies.len());
    
    // Scan bevy_* crates for SystemParam types
    for index_entry in fs::read_dir(&registry_src).into_iter().flatten().flatten() {
        let index_dir = index_entry.path();
        if !index_dir.is_dir() { continue; }
        
        for crate_entry in fs::read_dir(&index_dir).into_iter().flatten().flatten() {
            let crate_dir = crate_entry.path();
            let dir_name = crate_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // Extract base crate name (e.g., "bevy_picking" from "bevy_picking-0.17.2")
            let base_crate = dir_name.split('-').next().unwrap_or(dir_name);
            
            // Only scan crates that are in our dependencies
            if !dependencies.contains(&base_crate.to_string()) { continue; }
            
            let src_dir = crate_dir.join("src");
            if src_dir.exists() {
                scan_directory_for_systemparams(&src_dir, base_crate, &mut params);
            }
        }
    }
    
    // Deduplicate by type_name
    let mut seen = std::collections::HashSet::new();
    params.retain(|p| seen.insert(p.type_name.clone()));
    
    println!("cargo:warning=  ✓ Auto-discovered {} SystemParam types", params.len());
    for param in &params {
        println!("cargo:warning=    - {} (from {})", param.type_name, param.crate_name);
    }
    
    params
}

/// Scan a directory recursively for SystemParam types
fn scan_directory_for_systemparams(dir: &Path, crate_name: &str, results: &mut Vec<DiscoveredSystemParam>) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        
        if path.is_dir() {
            scan_directory_for_systemparams(&path, crate_name, results);
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            if let Ok(source) = fs::read_to_string(&path) {
                parse_systemparams_from_source(&source, crate_name, &path, results);
            }
        }
    }
}

/// Parse a source file for #[derive(SystemParam)] patterns
fn parse_systemparams_from_source(source: &str, crate_name: &str, file_path: &Path, results: &mut Vec<DiscoveredSystemParam>) {
    let Ok(file) = syn::parse_file(source) else { return; };
    
    for item in file.items {
        if let syn::Item::Struct(item_struct) = item {
            // Must be public
            if !matches!(item_struct.vis, syn::Visibility::Public(_)) {
                continue;
            }
            
            // Check for #[derive(SystemParam)]
            let has_systemparam = item_struct.attrs.iter().any(|attr| {
                if attr.path().is_ident("derive") {
                    if let Ok(meta) = attr.parse_args::<proc_macro2::TokenStream>() {
                        return meta.to_string().contains("SystemParam");
                    }
                }
                false
            });
            
            if !has_systemparam {
                continue;
            }
            
            let type_name = item_struct.ident.to_string();
            
            // Build full path
            let module_path = build_module_path_from_file(file_path, crate_name);
            let full_path = if module_path.is_empty() {
                format!("{}::{}", crate_name, type_name)
            } else {
                format!("{}::{}::{}", crate_name, module_path, type_name)
            };
            
            // Normalize to bevy:: path
            let Some(full_path) = normalize_bevy_path(&full_path) else {
                continue;
            };
            
            results.push(DiscoveredSystemParam {
                full_path,
                type_name,
                crate_name: crate_name.to_string(),
            });
        }
    }
}

/// Discover methods on SystemParam types
/// Scans for `impl TypeName` blocks with public methods that take &self or &mut self
fn discover_systemparam_methods(param_types: &[DiscoveredSystemParam]) -> Vec<DiscoveredSystemParamMethod> {
    let mut methods = Vec::new();
    
    println!("cargo:warning=[SYSTEMPARAM_DISCOVERY] Scanning for SystemParam methods...");
    
    // Get cargo home
    let cargo_home = env::var("CARGO_HOME")
        .or_else(|_| env::var("HOME").map(|h| format!("{}/.cargo", h)))
        .or_else(|_| env::var("USERPROFILE").map(|h| format!("{}/.cargo", h)))
        .unwrap_or_default();
    
    if cargo_home.is_empty() {
        return methods;
    }
    
    let registry_src = PathBuf::from(&cargo_home).join("registry").join("src");
    if !registry_src.exists() {
        return methods;
    }
    
    let dependencies = get_bevy_dependencies_from_lock();
    
    // Scan for methods
    for index_entry in fs::read_dir(&registry_src).into_iter().flatten().flatten() {
        let index_dir = index_entry.path();
        if !index_dir.is_dir() { continue; }
        
        for crate_entry in fs::read_dir(&index_dir).into_iter().flatten().flatten() {
            let crate_dir = crate_entry.path();
            let dir_name = crate_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            let base_crate = dir_name.split('-').next().unwrap_or(dir_name);
            if !dependencies.contains(&base_crate.to_string()) { continue; }
            
            let src_dir = crate_dir.join("src");
            if src_dir.exists() {
                scan_directory_for_systemparam_methods(&src_dir, param_types, &mut methods);
            }
        }
    }
    
    println!("cargo:warning=  ✓ Discovered {} SystemParam methods", methods.len());
    for method in &methods {
        let params_str: Vec<String> = method.params.iter()
            .map(|p| format!("{}: {}", p.name, p.type_str))
            .collect();
        println!("cargo:warning=    - {}::{}({})", method.param_type, method.method_name, params_str.join(", "));
    }
    
    methods
}

/// Scan directory for SystemParam method implementations
fn scan_directory_for_systemparam_methods(
    dir: &Path,
    param_types: &[DiscoveredSystemParam],
    results: &mut Vec<DiscoveredSystemParamMethod>,
) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        
        if path.is_dir() {
            scan_directory_for_systemparam_methods(&path, param_types, results);
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            if let Ok(source) = fs::read_to_string(&path) {
                parse_systemparam_methods_from_source(&source, param_types, results);
            }
        }
    }
}

/// Parse source for impl blocks of SystemParam types
fn parse_systemparam_methods_from_source(
    source: &str,
    param_types: &[DiscoveredSystemParam],
    results: &mut Vec<DiscoveredSystemParamMethod>,
) {
    let Ok(file) = syn::parse_file(source) else { return; };
    
    for item in file.items {
        if let syn::Item::Impl(item_impl) = item {
            // Only inherent impls (not trait impls)
            if item_impl.trait_.is_some() { continue; }
            
            // Get type name being implemented
            let type_name = match &*item_impl.self_ty {
                syn::Type::Path(type_path) => {
                    type_path.path.segments.last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_default()
                }
                _ => continue,
            };
            
            // Check if this is a SystemParam type we discovered
            if !param_types.iter().any(|p| p.type_name == type_name) {
                continue;
            }
            
            // Find public methods that take &self or &mut self
            for impl_item in &item_impl.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    // Must be public
                    if !matches!(method.vis, syn::Visibility::Public(_)) { continue; }
                    
                    // Must have &self or &mut self receiver
                    let has_self = method.sig.inputs.iter().any(|arg| {
                        matches!(arg, syn::FnArg::Receiver(_))
                    });
                    if !has_self { continue; }
                    
                    let method_name = method.sig.ident.to_string();
                    
                    // Parse parameters
                    let params = parse_systemparam_method_params(&method.sig);
                    
                    // Get return type
                    let (return_type, returns_iterator) = parse_return_type(&method.sig.output);
                    
                    results.push(DiscoveredSystemParamMethod {
                        param_type: type_name.clone(),
                        method_name,
                        params,
                        return_type,
                        returns_iterator,
                    });
                }
            }
        }
    }
}

/// Parse method parameters for SystemParam methods
fn parse_systemparam_method_params(sig: &syn::Signature) -> Vec<SystemParamMethodParam> {
    let mut params = Vec::new();
    
    for arg in &sig.inputs {
        match arg {
            syn::FnArg::Receiver(_) => continue, // Skip &self
            syn::FnArg::Typed(pat_type) => {
                let name = match &*pat_type.pat {
                    syn::Pat::Ident(ident) => ident.ident.to_string(),
                    _ => continue,
                };
                
                let ty = &pat_type.ty;
                let type_str = quote::quote!(#ty).to_string()
                    .replace(" ", "");
                
                // Check if it's a reference
                let is_reference = type_str.starts_with("&");
                
                // Check if this type implements Reflect (discovered from source)
                let is_reflectable = is_type_reflectable(&type_str);
                
                params.push(SystemParamMethodParam {
                    name,
                    type_str: type_str.replace("&", "").replace("mut", "").trim().to_string(),
                    is_reference,
                    is_reflectable,
                });
            }
        }
    }
    
    params
}

/// Cached set of types that implement Reflect (discovered by scanning sources)
/// This is populated by discover_reflect_types() and used by is_type_reflectable()
static REFLECT_TYPES: std::sync::OnceLock<std::collections::HashSet<String>> = std::sync::OnceLock::new();

/// Cached set of types that implement Debug (discovered by scanning sources)
/// This is populated by discover_debug_types() and used by is_type_debuggable()
static DEBUG_TYPES: std::sync::OnceLock<std::collections::HashSet<String>> = std::sync::OnceLock::new();

/// Check if a type implements Debug by looking it up in the discovered set
fn is_type_debuggable(type_str: &str) -> bool {
    let clean = type_str
        .replace("&", "")
        .replace("mut", "")
        .replace("'_", "")
        .replace("'static", "")
        .trim()
        .to_string();
    
    // Slice types like [(Entity,RayMeshHit)] - assume debuggable if they parse
    // (Entity and most Bevy types implement Debug)
    if clean.starts_with('[') && clean.ends_with(']') {
        return true;
    }
    
    // Tuple types like (Entity, RayMeshHit) - assume debuggable
    if clean.starts_with('(') && clean.ends_with(')') {
        return true;
    }
    
    // Primitive types always implement Debug
    let type_name_base = clean.split("::").last().unwrap_or(&clean);
    if is_primitive_type(type_name_base) {
        return true;
    }
    
    // Common std types implement Debug
    let always_debug = ["Vec", "Option", "Result", "String", "Box", "Arc", "Rc", 
                        "Entity", "RayMeshHit", "Camera", "UiCameraConfig"];
    for t in always_debug {
        if type_name_base.starts_with(t) {
            return true;
        }
    }
    
    // Check the discovered Debug types
    let debug_types = DEBUG_TYPES.get_or_init(|| {
        discover_debug_types()
    });
    
    // Extract just the type name (handle generics)
    let type_name = clean.split('<').next().unwrap_or(&clean);
    let type_name = type_name.split("::").last().unwrap_or(type_name);
    
    debug_types.contains(type_name)
}

/// Discover all types that implement Debug by scanning bevy crates
/// Similar to discover_reflect_types but for Debug
fn discover_debug_types() -> std::collections::HashSet<String> {
    let mut debug_types = std::collections::HashSet::new();
    
    // Get cargo home
    let cargo_home = env::var("CARGO_HOME")
        .or_else(|_| env::var("HOME").map(|h| format!("{}/.cargo", h)))
        .or_else(|_| env::var("USERPROFILE").map(|h| format!("{}/.cargo", h)))
        .unwrap_or_default();
    
    if cargo_home.is_empty() {
        return debug_types;
    }
    
    let registry_src = PathBuf::from(&cargo_home).join("registry").join("src");
    
    if !registry_src.exists() {
        return debug_types;
    }
    
    let dependencies = get_bevy_dependencies_from_lock();
    
    // Scan bevy crates for Debug implementations
    for index_entry in fs::read_dir(&registry_src).into_iter().flatten().flatten() {
        let index_dir = index_entry.path();
        if !index_dir.is_dir() { continue; }
        
        for crate_entry in fs::read_dir(&index_dir).into_iter().flatten().flatten() {
            let crate_dir = crate_entry.path();
            let dir_name = crate_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            let base_crate = dir_name.split('-').next().unwrap_or(dir_name);
            if !dependencies.contains(&base_crate.to_string()) { continue; }
            
            let src_dir = crate_dir.join("src");
            if src_dir.exists() {
                scan_directory_for_debug_types(&src_dir, &mut debug_types);
            }
        }
    }
    
    println!("cargo:warning=  ✓ Discovered {} Debug types", debug_types.len());
    
    debug_types
}

/// Scan directory for types with #[derive(Debug)]
fn scan_directory_for_debug_types(dir: &Path, results: &mut std::collections::HashSet<String>) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        
        if path.is_dir() {
            scan_directory_for_debug_types(&path, results);
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            if let Ok(source) = fs::read_to_string(&path) {
                parse_debug_types_from_source(&source, results);
            }
        }
    }
}

/// Parse source file for #[derive(Debug)] and impl Debug patterns
fn parse_debug_types_from_source(source: &str, results: &mut std::collections::HashSet<String>) {
    let Ok(file) = syn::parse_file(source) else { return; };
    
    for item in file.items {
        match item {
            syn::Item::Struct(item_struct) => {
                let has_debug = item_struct.attrs.iter().any(|attr| {
                    if attr.path().is_ident("derive") {
                        if let Ok(meta) = attr.parse_args::<proc_macro2::TokenStream>() {
                            return meta.to_string().contains("Debug");
                        }
                    }
                    false
                });
                
                if has_debug {
                    results.insert(item_struct.ident.to_string());
                }
            }
            syn::Item::Enum(item_enum) => {
                let has_debug = item_enum.attrs.iter().any(|attr| {
                    if attr.path().is_ident("derive") {
                        if let Ok(meta) = attr.parse_args::<proc_macro2::TokenStream>() {
                            return meta.to_string().contains("Debug");
                        }
                    }
                    false
                });
                
                if has_debug {
                    results.insert(item_enum.ident.to_string());
                }
            }
            syn::Item::Impl(item_impl) => {
                // Check for impl Debug for Type
                if let Some((_, trait_path, _)) = &item_impl.trait_ {
                    let trait_name = trait_path.segments.last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_default();
                    
                    if trait_name == "Debug" {
                        if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                            if let Some(seg) = type_path.path.segments.last() {
                                results.insert(seg.ident.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Check if a type implements Reflect by looking it up in the discovered set
fn is_type_reflectable(type_str: &str) -> bool {
    let clean = type_str
        .replace("&", "")
        .replace("mut", "")
        .replace("'_", "")
        .replace("'static", "")
        .trim()
        .to_string();
    
    // Types containing closures or complex generics are never reflectable
    if clean.contains("Fn(") || clean.contains("dyn ") || clean.contains("impl ") {
        return false;
    }
    
    // Primitive types are always reflectable
    let type_name_base = clean.split("::").last().unwrap_or(&clean);
    if is_primitive_type(type_name_base) {
        return true;
    }
    
    // Check the discovered Reflect types
    let reflect_types = REFLECT_TYPES.get_or_init(|| {
        discover_reflect_types()
    });
    
    // Extract just the type name (handle generics like Vec<T> -> Vec)
    let type_name = clean.split('<').next().unwrap_or(&clean);
    let type_name = type_name.split("::").last().unwrap_or(type_name);
    
    reflect_types.contains(type_name)
}

/// Discover all types that implement Reflect by scanning bevy crates
/// Pattern: #[derive(Reflect)] or impl Reflect for Type
fn discover_reflect_types() -> std::collections::HashSet<String> {
    let mut reflect_types = std::collections::HashSet::new();
    
    println!("cargo:warning=[REFLECT_DISCOVERY] Scanning for Reflect types...");
    
    // Get cargo home
    let cargo_home = env::var("CARGO_HOME")
        .or_else(|_| env::var("HOME").map(|h| format!("{}/.cargo", h)))
        .or_else(|_| env::var("USERPROFILE").map(|h| format!("{}/.cargo", h)))
        .unwrap_or_default();
    
    if cargo_home.is_empty() {
        println!("cargo:warning=  ⚠ Cannot find CARGO_HOME for Reflect discovery");
        return reflect_types;
    }
    
    let registry_src = PathBuf::from(&cargo_home).join("registry").join("src");
    
    if !registry_src.exists() {
        return reflect_types;
    }
    
    let dependencies = get_bevy_dependencies_from_lock();
    
    // Scan bevy crates for Reflect implementations
    for index_entry in fs::read_dir(&registry_src).into_iter().flatten().flatten() {
        let index_dir = index_entry.path();
        if !index_dir.is_dir() { continue; }
        
        for crate_entry in fs::read_dir(&index_dir).into_iter().flatten().flatten() {
            let crate_dir = crate_entry.path();
            let dir_name = crate_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            let base_crate = dir_name.split('-').next().unwrap_or(dir_name);
            if !dependencies.contains(&base_crate.to_string()) { continue; }
            
            let src_dir = crate_dir.join("src");
            if src_dir.exists() {
                scan_directory_for_reflect_types(&src_dir, &mut reflect_types);
            }
        }
    }
    
    println!("cargo:warning=  ✓ Discovered {} Reflect types", reflect_types.len());
    
    reflect_types
}

/// Scan directory for types with #[derive(Reflect)]
fn scan_directory_for_reflect_types(dir: &Path, results: &mut std::collections::HashSet<String>) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        
        if path.is_dir() {
            scan_directory_for_reflect_types(&path, results);
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            if let Ok(source) = fs::read_to_string(&path) {
                parse_reflect_types_from_source(&source, results);
            }
        }
    }
}

/// Parse source file for #[derive(Reflect)] and impl Reflect patterns
fn parse_reflect_types_from_source(source: &str, results: &mut std::collections::HashSet<String>) {
    let Ok(file) = syn::parse_file(source) else { return; };
    
    for item in file.items {
        match item {
            syn::Item::Struct(item_struct) => {
                // Check for #[derive(Reflect)]
                let has_reflect = item_struct.attrs.iter().any(|attr| {
                    if attr.path().is_ident("derive") {
                        if let Ok(meta) = attr.parse_args::<proc_macro2::TokenStream>() {
                            return meta.to_string().contains("Reflect");
                        }
                    }
                    false
                });
                
                if has_reflect {
                    results.insert(item_struct.ident.to_string());
                }
            }
            syn::Item::Enum(item_enum) => {
                let has_reflect = item_enum.attrs.iter().any(|attr| {
                    if attr.path().is_ident("derive") {
                        if let Ok(meta) = attr.parse_args::<proc_macro2::TokenStream>() {
                            return meta.to_string().contains("Reflect");
                        }
                    }
                    false
                });
                
                if has_reflect {
                    results.insert(item_enum.ident.to_string());
                }
            }
            syn::Item::Impl(item_impl) => {
                // Check for impl Reflect for Type
                if let Some((_, trait_path, _)) = &item_impl.trait_ {
                    let trait_name = trait_path.segments.last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_default();
                    
                    if trait_name == "Reflect" || trait_name == "PartialReflect" {
                        if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                            if let Some(seg) = type_path.path.segments.last() {
                                results.insert(seg.ident.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Parse return type from method signature
fn parse_return_type(output: &syn::ReturnType) -> (String, bool) {
    match output {
        syn::ReturnType::Default => ("()".to_string(), false),
        syn::ReturnType::Type(_, ty) => {
            let type_str = quote::quote!(#ty).to_string().replace(" ", "");
            let returns_iterator = type_str.contains("impl Iterator") || 
                                   type_str.contains("impl IntoIterator") ||
                                   type_str.contains("Iter<");
            (type_str, returns_iterator)
        }
    }
}


/// Used for auto-registering handle creators and newtype wrappers
#[derive(Debug, Clone)]
struct NewtypeSpec {
    /// Full path to the newtype (e.g., "bevy::camera::camera::ImageRenderTarget")
    newtype_path: String,
    /// Full path to the inner asset type (e.g., "bevy_image::image::Image")
    inner_asset_path: String,
}

/// Get newtype wrappers from [package.metadata.lua_newtypes.wrappers] in Cargo.toml
/// Format: wrappers = [{ newtype = "path::Newtype", inner = "path::AssetType" }, ...]
fn get_newtypes_from_metadata(manifest: &toml::Value) -> Vec<NewtypeSpec> {
    let wrappers_array = manifest
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("lua_newtypes"))
        .and_then(|ln| ln.get("wrappers"))
        .and_then(|w| w.as_array());
    
    let Some(wrappers) = wrappers_array else {
        return Vec::new();
    };
    
    wrappers.iter()
        .filter_map(|entry| {
            let newtype = entry.get("newtype")?.as_str()?.to_string();
            let inner = entry.get("inner")?.as_str()?.to_string();
            println!("cargo:warning=  ✓ Found newtype wrapper: {} wraps Handle<{}>", newtype, inner);
            Some(NewtypeSpec {
                newtype_path: newtype,
                inner_asset_path: inner,
            })
        })
        .collect()
}

/// Specification for an observable event (for Lua observer callbacks)
#[derive(Debug, Clone)]
struct ObservableEventSpec {
    /// Lua-facing name (e.g., "Pointer<Over>", "Pointer<Down>")
    lua_name: String,
    /// Lua suffix for function naming (e.g., "down", "up", "over")
    lua_suffix: String,
    /// Rust event type name (e.g., "Over", "Press", "Release")  
    event_type: String,
    /// Whether this event has position data
    has_position: bool,
}

/// Specification for a Bevy Event type (for Lua read_events)
#[derive(Debug, Clone)]
struct BevyEventSpec {
    /// Short type name (e.g., "CursorMoved", "MouseButtonInput")
    type_name: String,
    /// Full type path (e.g., "bevy_window::event::CursorMoved")
    full_path: String,
    /// Crate name (e.g., "bevy_window")
    crate_name: String,
}

/// Specification for a Bevy Message type (for Lua write_message via MessageWriter)
/// Messages differ from Events in that they use MessageWriter<T> instead of EventWriter<T>
#[derive(Debug, Clone)]
struct BevyMessageSpec {
    /// Short type name (e.g., "PointerInput")
    type_name: String,
    /// Full type path (e.g., "bevy_picking::pointer::PointerInput")
    full_path: String,
    /// Bevy re-export path (e.g., "bevy::picking::pointer::PointerInput")
    bevy_path: String,
    /// Crate name (e.g., "bevy_picking")
    crate_name: String,
}

/// Discover message types by scanning bevy_picking crate
/// Message types are those that use MessageWriter<T> for dispatch (e.g., PointerInput)
fn discover_bevy_messages() -> Vec<BevyMessageSpec> {
    let mut messages = Vec::new();
    
    // Find cargo home
    let cargo_home = env::var("CARGO_HOME")
        .or_else(|_| env::var("HOME").map(|h| format!("{}/.cargo", h)))
        .or_else(|_| env::var("USERPROFILE").map(|h| format!("{}/.cargo", h)))
        .unwrap_or_else(|_| String::new());
    
    if cargo_home.is_empty() {
        println!("cargo:warning=  ⚠ Cannot find CARGO_HOME for message discovery");
        return messages;
    }
    
    let registry_src = PathBuf::from(&cargo_home).join("registry").join("src");
    
    if !registry_src.exists() {
        println!("cargo:warning=  ⚠ Registry source not found for message discovery");
        return messages;
    }
    
    // Message types to discover and their locations
    // These types use MessageWriter<T> instead of EventWriter<T>
    let message_types = [
        ("bevy_picking", "pointer.rs", "PointerInput"),
    ];
    
    'type_loop: for (crate_prefix, file_name, type_name) in &message_types {
        // Check if we already found this type
        if messages.iter().any(|m| m.type_name == *type_name) {
            continue;
        }
        
        // Iterate through registry index directories
        for index_entry in fs::read_dir(&registry_src).into_iter().flatten().flatten() {
            let index_dir = index_entry.path();
            if !index_dir.is_dir() { continue; }
            
            // Look for crate directories (e.g., bevy_picking-0.3.0)
            for crate_entry in fs::read_dir(&index_dir).into_iter().flatten().flatten() {
                let crate_dir = crate_entry.path();
                if !crate_dir.is_dir() { continue; }
                
                let crate_name = crate_dir.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                // Check if this is the target crate
                if !crate_name.starts_with(crate_prefix) {
                    continue;
                }
                
                // Found the crate - look for the type
                let source_file = crate_dir.join("src").join(file_name);
                if source_file.exists() {
                    if let Ok(source) = fs::read_to_string(&source_file) {
                        // Verify the type exists in the file with Event derive
                        if source.contains(&format!("pub struct {}", type_name)) 
                            && source.contains("#[derive(") 
                            && source.contains("Event") 
                        {
                            let module_name = file_name.trim_end_matches(".rs");
                            let full_path = format!("{}::{}::{}", crate_prefix, module_name, type_name);
                            let bevy_path = full_path.replace("bevy_picking::", "bevy::picking::");
                            
                            messages.push(BevyMessageSpec {
                                type_name: type_name.to_string(),
                                full_path,
                                bevy_path,
                                crate_name: crate_prefix.to_string(),
                            });
                            
                            println!("cargo:warning=    - Message: {} ({})", type_name, crate_prefix);
                            continue 'type_loop; // Found it, move to next type
                        }
                    }
                }
            }
        }
    }
    
    // Deduplicate by type_name (in case of multiple crate versions)
    messages.sort_by(|a, b| a.type_name.cmp(&b.type_name));
    messages.dedup_by(|a, b| a.type_name == b.type_name);
    
    println!("cargo:warning=  ✓ Discovered {} Message types for Lua write_message()", messages.len());
    
    messages
}


/// Discover Bevy Event types by scanning bevy_window and bevy_input crates
fn discover_bevy_events() -> Vec<BevyEventSpec> {
    let mut events = Vec::new();
    
    // Find cargo home
    let cargo_home = env::var("CARGO_HOME")
        .or_else(|_| env::var("HOME").map(|h| format!("{}/.cargo", h)))
        .or_else(|_| env::var("USERPROFILE").map(|h| format!("{}/.cargo", h)))
        .unwrap_or_else(|_| String::new());
    
    if cargo_home.is_empty() {
        println!("cargo:warning=  ⚠ Cannot find CARGO_HOME for event discovery");
        return events;
    }
    
    let registry_src = PathBuf::from(&cargo_home).join("registry").join("src");
    
    if !registry_src.exists() {
        println!("cargo:warning=  ⚠ Registry source not found for event discovery");
        return events;
    }
    
    // Crates to scan for Event derives
    let crates_to_scan = [
        ("bevy_window", vec!["event.rs", "cursor.rs"]),
        ("bevy_input", vec!["keyboard.rs", "mouse.rs"]),
        ("bevy_picking", vec!["pointer.rs"]),  // For PointerInput
    ];
    
    for (crate_prefix, files) in &crates_to_scan {
        events.extend(scan_crate_for_events(&registry_src, crate_prefix, files));
    }
    
    println!("cargo:warning=  ✓ Discovered {} Bevy Event types for Lua read_events()", events.len());
    for event in &events {
        println!("cargo:warning=    - {} ({})", event.type_name, event.full_path);
    }
    
    events
}

/// Scan a specific crate for Event types (types with #[derive(Event)])
fn scan_crate_for_events(registry_src: &Path, crate_prefix: &str, files: &[&str]) -> Vec<BevyEventSpec> {
    let mut events = Vec::new();
    
    // Iterate through registry index directories
    'outer: for index_entry in fs::read_dir(registry_src).into_iter().flatten().flatten() {
        let index_dir = index_entry.path();
        if !index_dir.is_dir() { continue; }
        
        for crate_entry in fs::read_dir(&index_dir).into_iter().flatten().flatten() {
            let crate_dir = crate_entry.path();
            let dir_name = crate_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // Match crate name with version suffix (e.g., "bevy_window-0.15.0")
            if !dir_name.starts_with(&format!("{}-", crate_prefix)) {
                continue;
            }
            
            // Found the crate, scan specified files
            let src_dir = crate_dir.join("src");
            for file_name in files {
                let file_path = src_dir.join(file_name);
                if file_path.exists() {
                    if let Ok(source) = fs::read_to_string(&file_path) {
                        let file_events = parse_events_from_source(&source, crate_prefix, file_name);
                        events.extend(file_events);
                    }
                }
            }
            
            // Found the crate, stop searching (avoid duplicates from multiple versions)
            if !events.is_empty() {
                break 'outer;
            }
        }
    }
    
    events
}

/// Parse a source file for structs with #[derive(Event)]
fn parse_events_from_source(source: &str, crate_name: &str, file_name: &str) -> Vec<BevyEventSpec> {
    let mut events = Vec::new();
    
    let Ok(file) = syn::parse_file(source) else {
        return events;
    };
    
    // Derive the module path from file name (e.g., "event.rs" -> "event", "cursor.rs" -> "cursor")
    let module_name = file_name.trim_end_matches(".rs");
    
    for item in file.items {
        if let syn::Item::Struct(item_struct) = item {
            // Check if public
            if !matches!(item_struct.vis, syn::Visibility::Public(_)) {
                continue;
            }
            
            let struct_name = item_struct.ident.to_string();
            
            // Check if it has #[derive(...Event...)]
            let has_event_derive = item_struct.attrs.iter().any(|attr| {
                if attr.path().is_ident("derive") {
                    if let Ok(meta) = attr.parse_args::<proc_macro2::TokenStream>() {
                        // Check for Event in derives (handles Event, bevy_ecs::event::Event, etc.)
                        return meta.to_string().contains("Event");
                    }
                }
                false
            });
            
            if has_event_derive {
                events.push(BevyEventSpec {
                    type_name: struct_name.clone(),
                    full_path: format!("{}::{}::{}", crate_name, module_name, struct_name),
                    crate_name: crate_name.to_string(),
                });
            }
        }
    }
    
    events
}

/// Discover all observable events by scanning crate sources
/// This scans bevy_picking for Pointer events and other crates for EntityEvent types
fn discover_observable_events() -> Vec<ObservableEventSpec> {
    let mut events = Vec::new();
    
    // Find cargo home
    let cargo_home = env::var("CARGO_HOME")
        .or_else(|_| env::var("HOME").map(|h| format!("{}/.cargo", h)))
        .or_else(|_| env::var("USERPROFILE").map(|h| format!("{}/.cargo", h)))
        .unwrap_or_else(|_| String::new());
    
    if cargo_home.is_empty() {
        println!("cargo:warning=  ⚠ Cannot find CARGO_HOME for observer discovery");
        return events;
    }
    
    let registry_src = PathBuf::from(&cargo_home).join("registry").join("src");
    
    if !registry_src.exists() {
        println!("cargo:warning=  ⚠ Registry source not found for observer discovery");
        return events;
    }
    
    // Scan bevy_picking crate for Pointer events
    events.extend(scan_bevy_picking_events(&registry_src));
    
    println!("cargo:warning=  ✓ Discovered {} observable events", events.len());
    for event in &events {
        println!("cargo:warning=    - {} (rust_type: {}, has_position: {})", event.lua_name, event.event_type, event.has_position);
    }
    
    events
}

/// Scan bevy_picking crate for Pointer event types
fn scan_bevy_picking_events(registry_src: &Path) -> Vec<ObservableEventSpec> {
    let mut events = Vec::new();
    
    // Find bevy_picking crate directory
    'outer: for index_entry in fs::read_dir(registry_src).into_iter().flatten().flatten() {
        let index_dir = index_entry.path();
        if !index_dir.is_dir() { continue; }
        
        for crate_entry in fs::read_dir(&index_dir).into_iter().flatten().flatten() {
            let crate_dir = crate_entry.path();
            let dir_name = crate_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            if !dir_name.starts_with("bevy_picking-") {
                continue;
            }
            
            // Found bevy_picking, scan for events
            let events_file = crate_dir.join("src").join("events.rs");
            if events_file.exists() {
                if let Ok(source) = fs::read_to_string(&events_file) {
                    events.extend(parse_picking_events(&source));
                    // Found events, stop searching (avoid picking up same events from multiple versions)
                    if !events.is_empty() {
                        break 'outer;
                    }
                }
            }
            
            // Also check src/pointer/events.rs in case of different structure
            let alt_events_file = crate_dir.join("src").join("pointer").join("events.rs");
            if alt_events_file.exists() {
                if let Ok(source) = fs::read_to_string(&alt_events_file) {
                    events.extend(parse_picking_events(&source));
                    // Found events, stop searching
                    if !events.is_empty() {
                        break 'outer;
                    }
                }
            }
        }
    }
    
    // Deduplicate by lua suffix (the user-facing name)
    let mut seen = std::collections::HashSet::new();
    events.retain(|e| seen.insert(e.lua_suffix.clone()));
    
    events
}

/// Parse bevy_picking events source to find all event structs dynamically
/// This parses the actual source code to discover event types without hardcoding
fn parse_picking_events(source: &str) -> Vec<ObservableEventSpec> {
    let mut events = Vec::new();
    
    // Parse the source file with syn
    let Ok(file) = syn::parse_file(source) else {
        println!("cargo:warning=  ⚠ Could not parse bevy_picking events source");
        return events;
    };
    
    // Find all public structs
    for item in file.items {
        if let syn::Item::Struct(item_struct) = item {
            // Check if public
            if !matches!(item_struct.vis, syn::Visibility::Public(_)) {
                continue;
            }
            
            let struct_name = item_struct.ident.to_string();
            
            // Check if it has #[derive(...)] with Reflect
            let has_reflect = item_struct.attrs.iter().any(|attr| {
                if attr.path().is_ident("derive") {
                    if let Ok(meta) = attr.parse_args::<proc_macro2::TokenStream>() {
                        return meta.to_string().contains("Reflect");
                    }
                }
                false
            });
            
            // Skip structs that don't have Reflect derive (not events)
            if !has_reflect {
                continue;
            }
            
            // Skip certain known non-event types
            if struct_name == "Pointer" || struct_name == "Location" || struct_name == "PointerHits" {
                continue;
            }
            
            // Determine if this event has position data by checking for common position-related fields
            // Events with position typically have delta, distance, or are Move/Drag related
            let has_position = struct_name.contains("Move") 
                || struct_name.contains("Drag") 
                || struct_name.contains("Scroll");
            
            // Create the event spec
            // bevy_picking source has Down/Up, but bevy::prelude exports Press/Release
            // So we need to map the source struct names to bevy::prelude types:
            // - source: Down -> bevy::prelude::Press (lua name stays "Down")
            // - source: Up -> bevy::prelude::Release (lua name stays "Up")
            let (lua_suffix, rust_type) = match struct_name.as_str() {
                "Down" => ("Down".to_string(), "Press".to_string()),
                "Up" => ("Up".to_string(), "Release".to_string()),
                _ => (struct_name.clone(), struct_name.clone()),
            };
            
            events.push(ObservableEventSpec {
                lua_name: format!("Pointer<{}>", lua_suffix),
                lua_suffix: lua_suffix.to_lowercase(),
                event_type: rust_type,
                has_position,
            });
        }
    }
    
    println!("cargo:warning=  ✓ Dynamically discovered {} events from source", events.len());
    
    events
}

/// Generate observer handler code for all discovered events
fn generate_observer_handlers(events: &[ObservableEventSpec]) -> proc_macro2::TokenStream {
    let handlers: Vec<proc_macro2::TokenStream> = events.iter().map(|event| {
        let event_type_ident: syn::Ident = syn::parse_str(&event.event_type)
            .unwrap_or_else(|_| syn::parse_str("Over").unwrap());
        let fn_name = format!("on_pointer_{}_lua", event.lua_suffix);
        let fn_ident: syn::Ident = syn::parse_str(&fn_name).unwrap();
        let lua_name = &event.lua_name;
        
        if event.has_position {
            quote::quote! {
                fn #fn_ident(
                    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::#event_type_ident>>,
                    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
                    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
                    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
                ) {
                    let pos = event.event().pointer_location.position;
                    dispatch_lua_observer(&lua_ctx, &observer_registry, &update_queue, event.event().entity, #lua_name, Some(pos));
                }
            }
        } else {
            quote::quote! {
                fn #fn_ident(
                    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::#event_type_ident>>,
                    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
                    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
                    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
                ) {
                    dispatch_lua_observer(&lua_ctx, &observer_registry, &update_queue, event.event().entity, #lua_name, None);
                }
            }
        }
    }).collect();
    
    quote::quote! {
        #(#handlers)*
    }
}

/// Generate the match arms for attach_observer_by_name function
fn generate_observer_match_arms(events: &[ObservableEventSpec]) -> proc_macro2::TokenStream {
    let arms: Vec<proc_macro2::TokenStream> = events.iter().map(|event| {
        let lua_name = &event.lua_name;
        let fn_name = format!("on_pointer_{}_lua", event.lua_suffix);
        let fn_ident: syn::Ident = syn::parse_str(&fn_name).unwrap();
        
        quote::quote! {
            #lua_name => { commands.entity(entity).observe(#fn_ident); }
        }
    }).collect();
    
    quote::quote! {
        #(#arms)*
    }
}

/// Generate a component handler for a newtype that wraps an Entity
/// These components have the pattern: pub struct ComponentName(pub Entity);
/// The Lua table should have format: { entity = entity_id }
fn generate_entity_component_binding(full_path: &str) -> Result<proc_macro2::TokenStream, String> {
    // Parse the type path to get the component name and module path
    let parts: Vec<&str> = full_path.split("::").collect();
    if parts.len() < 2 {
        return Err(format!("Invalid type path: {}", full_path));
    }
    
    let component_name = *parts.last().unwrap();
    let component_name_literal = component_name;
    let full_path_ident: syn::Path = syn::parse_str(full_path)
        .map_err(|e| format!("Failed to parse type path '{}': {}", full_path, e))?;
    
    // Generate call with constructor - tuple structs act as functions: TypeName(entity)
    let tokens = quote! {
        registry.register_entity_component::<#full_path_ident, _>(#component_name_literal, #full_path_ident);
    };
    
    Ok(tokens)
}

#[allow(dead_code)]
fn generate_bindings_for_type(spec: &TypeSpec) -> Result<proc_macro2::TokenStream, String> {
    // Find source file
    let source_path = find_source_file(spec)?;
    
    // Parse source
    let source_code = fs::read_to_string(&source_path)
        .map_err(|e| format!("Failed to read source: {}", e))?;
    
    let syntax_tree: File = syn::parse_file(&source_code)
        .map_err(|e| format!("Failed to parse source: {}", e))?;
    
    // Extract methods
    let methods = extract_methods_for_type(&syntax_tree, &spec.type_name)?;
    
    if methods.is_empty() {
        return Err("No public methods found".to_string());
    }
    
    // Generate binding code
    generate_registration_code(spec, &methods)
}

fn find_source_file(spec: &TypeSpec) -> Result<PathBuf, String> {
    // Find in cargo registry cache
    let cargo_home = env::var("CARGO_HOME")
        .or_else(|_| env::var("HOME").map(|h| format!("{}/.cargo", h)))
        .or_else(|_| env::var("USERPROFILE").map(|h| format!("{}/.cargo", h)))
        .map_err(|_| "Cannot find CARGO_HOME")?;
    
    let registry_src = PathBuf::from(cargo_home)
        .join("registry")
        .join("src");
    
    if !registry_src.exists() {
        return Err(format!("Registry source directory not found: {:?}", registry_src));
    }
    
    // Find the crate directory
    for entry in fs::read_dir(&registry_src)
        .map_err(|e| format!("Cannot read registry: {}", e))? 
    {
        let entry = entry.map_err(|e| format!("Cannot read entry: {}", e))?;
        let index_dir = entry.path();
        
        if !index_dir.is_dir() {
            continue;
        }
        
        for crate_entry in fs::read_dir(&index_dir)
            .map_err(|e| format!("Cannot read index dir: {}", e))? 
        {
            let crate_entry = crate_entry.map_err(|e| format!("Cannot read crate entry: {}", e))?;
            let crate_dir = crate_entry.path();
            
            if !crate_dir.is_dir() {
                continue;
            }
            
            // Try both original name and hyphenated version
            // bevy_image stays as bevy_image-, but wgpu_types becomes wgpu-types-
            let crate_name_original = &spec.crate_name;
            let crate_name_hyphenated = spec.crate_name.replace('_', "-");
            
            let dir_name = crate_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            let matches = dir_name.starts_with(&format!("{}-", crate_name_original))
                || dir_name.starts_with(&format!("{}-", crate_name_hyphenated));
            
            if matches
            {
                let src_dir = crate_dir.join("src");
                if src_dir.exists() {
                    let file_path = find_module_file(&src_dir, &spec.module_path)?;
                    if file_path.exists() {
                        return Ok(file_path);
                    }
                }
            }
        }
    }
    
    Err(format!("Source not found for {}", spec.crate_name))
}

fn find_module_file(src_dir: &Path, module_path: &[String]) -> Result<PathBuf, String> {
    if module_path.is_empty() {
        return Ok(src_dir.join("lib.rs"));
    }
    
    let mut current_path = src_dir.to_path_buf();
    
    for (i, module) in module_path.iter().enumerate() {
        let is_last = i == module_path.len() - 1;
        
        // Try module.rs
        let module_file = current_path.join(format!("{}.rs", module));
        if module_file.exists() && is_last {
            return Ok(module_file);
        }
        
        // Try module/mod.rs
        let mod_dir = current_path.join(module);
        let mod_file = mod_dir.join("mod.rs");
        if mod_file.exists() {
            if is_last {
                return Ok(mod_file);
            }
            current_path = mod_dir;
            continue;
        }
        
        return Err(format!("Module {} not found", module));
    }
    
    Ok(current_path.join("mod.rs"))
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MethodInfo {
    name: String,
    #[allow(dead_code)]
    is_mut: bool,
    args: Vec<(String, String)>,
    #[allow(dead_code)]
    return_type: Option<String>,
}

#[allow(dead_code)]

fn extract_associated_function(syntax_tree: &File, type_name: &str, function_name: &str) -> Result<MethodInfo, String> {
    for item in &syntax_tree.items {
        if let Item::Impl(impl_block) = item {
            if !is_impl_for_type(impl_block, type_name) {
                continue;
            }
            
            for impl_item in &impl_block.items {
                if let ImplItem::Fn(method) = impl_item {
                    if method.sig.ident.to_string() != function_name {
                        continue;
                    }
                    
                    // Check if it's public
                    if !matches!(method.vis, Visibility::Public(_)) {
                        continue;
                    }
                    
                    // Extract arguments (skip self parameter if present)
                    let args: Vec<_> = method.sig.inputs.iter()
                        .filter_map(|arg| {
                            if let FnArg::Typed(pat_type) = arg {
                                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                                    let name = pat_ident.ident.to_string();
                                    let ty = quote::quote!(#pat_type.ty).to_string();
                                    return Some((name, ty));
                                }
                            }
                            None
                        })
                        .collect();
                    
                    let return_type = match &method.sig.output {
                        ReturnType::Type(_, ty) => Some(quote::quote!(#ty).to_string()),
                        _ => None,
                    };
                    
                    return Ok(MethodInfo {
                        name: function_name.to_string(),
                        is_mut: false,  // Constructors don't have &mut self
                        args,
                        return_type,
                    });
                }
            }
        }
    }
    
    Err(format!("Function '{}' not found for type '{}'", function_name, type_name))
}

fn extract_methods_for_type(syntax_tree: &File, type_name: &str) -> Result<Vec<MethodInfo>, String> {
    let mut methods = Vec::new();
    
    for item in &syntax_tree.items {
        if let Item::Impl(impl_block) = item {
            if !is_impl_for_type(impl_block, type_name) {
                continue;
            }
            
            for impl_item in &impl_block.items {
                if let ImplItem::Fn(method) = impl_item {
                    if !matches!(method.vis, Visibility::Public(_)) {
                        continue;
                    }
                    
                    let mut has_self = false;
                    let mut is_mut = false;
                    
                    for arg in &method.sig.inputs {
                        if let FnArg::Receiver(receiver) = arg {
                            has_self = true;
                            is_mut = receiver.mutability.is_some();
                            break;
                        }
                    }
                    
                    if !has_self {
                        continue;
                    }
                    
                    let args: Vec<_> = method.sig.inputs.iter()
                        .filter_map(|arg| {
                            if let FnArg::Typed(pat_type) = arg {
                                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                                    let name = pat_ident.ident.to_string();
                                    let ty = quote!(#pat_type.ty).to_string();
                                    return Some((name, ty));
                                }
                            }
                            None
                        })
                        .collect();
                    
                    let return_type = match &method.sig.output {
                        ReturnType::Type(_, ty) => Some(quote!(#ty).to_string()),
                        _ => None,
                    };
                    
                    methods.push(MethodInfo {
                        name: method.sig.ident.to_string(),
                        is_mut,
                        args,
                        return_type,
                    });
                }
            }
        }
    }
    
    Ok(methods)
}

#[allow(dead_code)]
fn is_impl_for_type(impl_block: &ItemImpl, type_name: &str) -> bool {
    if let syn::Type::Path(type_path) = &*impl_block.self_ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == type_name;
        }
    }
    false
}

#[allow(dead_code)]

fn generate_constructor_binding(spec: &ConstructorSpec, method_info: &MethodInfo) -> Result<proc_macro2::TokenStream, String> {
    let type_path = syn::parse_str::<syn::Path>(&spec.type_spec.full_path)
        .map_err(|e| format!("Invalid type path: {}", e))?;
    let function_name = &spec.function_name;
    let function_ident = syn::Ident::new(function_name, proc_macro2::Span::call_site());
    let lua_function_name = format!("create_{}", spec.type_spec.type_name.to_lowercase());
    
    // Generate based on argument count
    let binding = match method_info.args.len() {
        0 => {
            // No arguments - simple call
            quote::quote! {
                lua.globals().set(#lua_function_name, lua.create_function(|_lua, ()| {
                    let result = #type_path::#function_ident();
                    Ok(result)
                })?)?;
            }
        }
        1 => {
            // Single argument
            quote::quote! {
                lua.globals().set(#lua_function_name, lua.create_function(|_lua, arg: mlua::Value| {
                    let result = #type_path::#function_ident(arg);
                    Ok(result)
                })?)?;
            }
        }
        _ => {
            // Multiple arguments - use MultiValue
            quote::quote! {
                lua.globals().set(#lua_function_name, lua.create_function(|_lua, args: mlua::MultiValue| {
                    // TODO: Proper multi-arg handling
                    let result = #type_path::#function_ident();
                    Ok(result)
                })?)?;
            }
        }
    };
    
    Ok(binding)
}

fn generate_bindings_for_constructor(spec: &ConstructorSpec) -> Result<proc_macro2::TokenStream, String> {
    // Find source file
    let source_path = find_source_file(&spec.type_spec)?;
    
    // Parse source
    let source_code = fs::read_to_string(&source_path)
        .map_err(|e| format!("Failed to read source: {}", e))?;
    
    let syntax_tree: File = syn::parse_file(&source_code)
        .map_err(|e| format!("Failed to parse source: {}", e))?;
    
    // Extract the associated function
    let method_info = extract_associated_function(&syntax_tree, &spec.type_spec.type_name, &spec.function_name)?;
    
    // Generate binding code
    generate_constructor_binding(spec, &method_info)
}

fn generate_registration_code(
    spec: &TypeSpec,
    methods: &[MethodInfo],
) -> Result<proc_macro2::TokenStream, String> {
    let type_path = syn::parse_str::<syn::Path>(&spec.full_path)
        .map_err(|e| format!("Invalid type path: {}", e))?;
    let type_name_str = &spec.type_name;
    
    let method_registrations: Vec<_> = methods.iter()
        .map(|method| {
            let method_name = &method.name;
            let method_ident = syn::Ident::new(method_name, proc_macro2::Span::call_site());
            
            // Generate based on argument count - using mlua's automatic type conversion
            match method.args.len() {
                0 => {
                    // No arguments - simple call
                    quote! {
                        methods.add(#method_name, |resource, _lua, _args: ()| {
                            let result = resource.#method_ident();
                            Ok(result)
                        });
                    }
                }
                1 => {
                    // Single argument - mlua will handle conversion
                    quote! {
                        methods.add(#method_name, |resource, _lua, arg: mlua::Value| {
                            let result = resource.#method_ident(arg);
                            Ok(result)
                        });
                    }
                }
                _ => {
                    // Multiple arguments - use mlua::MultiValue
                    quote! {
                        methods.add(#method_name, |resource, _lua, args: mlua::MultiValue| {
                            // For now, just call with first arg
                            // TODO: Proper multi-arg handling
                            let result = resource.#method_ident();
                            Ok(result)
                        });
                    }
                }
            }
        })
        .collect();
    
    Ok(quote! {
        registry.register_resource::<#type_path>(#type_name_str, |methods| {
            #(#method_registrations)*
        });
    })
}

fn write_bindings_to_parent_crate(
    method_bindings: Vec<proc_macro2::TokenStream>, 
    constructor_bindings: Vec<proc_macro2::TokenStream>,
    entity_wrapper_names: Vec<String>,  // Type names for runtime TypeRegistry lookup
    asset_type_names: Vec<String>,      // Asset type names for runtime TypeRegistry lookup
    discovered_assets: Vec<DiscoveredAssetType>,  // Full asset info for cloner generation
    discovered_constructors: Vec<DiscoveredAssetConstructor>,  // Auto-discovered constructors for opaque types
    discovered_bitflags: Vec<DiscoveredBitflags>,
    newtypes: Vec<NewtypeSpec>,
    discovered_systemparams: Vec<DiscoveredSystemParam>,  // Auto-discovered SystemParam types
    discovered_systemparam_methods: Vec<DiscoveredSystemParamMethod>,  // Methods on SystemParam types
    parent_src_dir: &Path
) {

    let generated_file = parent_src_dir.join("auto_resource_bindings.rs");
    
    // Generate bitflags registration code from discovered bitflags
    let bitflags_registrations: Vec<_> = discovered_bitflags.iter().map(|bf| {
        let name = &bf.name;
        // For each flag, generate a tuple of (name, value)
        // The value is determined by flag position (bit index)
        let flag_tuples: Vec<_> = bf.flags.iter().enumerate().map(|(idx, flag_name)| {
            let bit_value = 1u32 << idx;
            quote::quote! { (#flag_name, #bit_value) }
        }).collect();
        
        println!("cargo:warning=  ✓ Generating bitflags registration for {} with {} flags", name, bf.flags.len());
        
        quote::quote! {
            // Auto-discovered #name bitflags
            registry.register(#name, &[
                #(#flag_tuples),*
            ]);
        }
    }).collect();
    
    // Generate asset type name literals for runtime registration const array
    // No compile-time handle setters, asset adders, or handle creators needed
    // All asset type registration happens at runtime via TypeRegistry lookup
    let asset_type_name_literals: Vec<_> = asset_type_names.iter()
        .map(|name| quote::quote! { #name })
        .collect();
    
    println!("cargo:warning=  ✓ Generating {} asset type names for runtime registration", asset_type_names.len());
    
    // Generate cloner registrations ONLY for asset types that implement Clone
    // This is detected at compile time by parsing #[derive(Clone)] or impl Clone
    let cloner_registrations: Vec<_> = discovered_assets.iter().filter_map(|asset| {
        // Only generate cloner for types that implement Clone
        if !asset.has_clone {
            return None;
        }
        
        // Skip generic types - can't instantiate without concrete type parameters
        if asset.is_generic {
            println!("cargo:warning=    - Skipping generic type {} (requires type parameters)", asset.type_name);
            return None;
        }
        
        // Also skip if full_path contains angle brackets (backup filter)
        if asset.full_path.contains('<') || asset.type_name.contains('<') {
            println!("cargo:warning=    - Skipping generic type {} (has angle brackets)", asset.type_name);
            return None;
        }
        
        let normalized_path = normalize_bevy_path(&asset.full_path)?;
        let type_path: syn::Path = syn::parse_str(&normalized_path).ok()?;
        
        println!("cargo:warning=    - Registering cloner for {} (full_path: {})", asset.type_name, asset.full_path);
        
        Some(quote::quote! {
            bevy_lua_ecs::register_cloner_if_clone::<#type_path>(&mut cloners);
        })
    }).collect();
    
    let clone_count = cloner_registrations.len();
    let total_count = discovered_assets.len();
    println!("cargo:warning=  ✓ Generated {} cloner registrations (out of {} total assets)", clone_count, total_count);

    // Generate asset constructor registrations for discovered constructors
    // These allow opaque types like Image to be created from Lua using their actual constructors
    let constructor_registrations: Vec<_> = discovered_constructors.iter().filter_map(|ctor| {
        let normalized_path = normalize_bevy_path(&ctor.type_path)?;
        let type_path: syn::Path = syn::parse_str(&normalized_path).ok()?;
        let type_path_str = &ctor.type_path;
        let method_name = &ctor.method_name;
        
        // Generate parameter extraction code for each parameter
        let mut param_extractions = Vec::new();
        let mut param_names = Vec::new();
        let mut all_params_supported = true;
        
        for param in &ctor.params {
            let param_name = &param.name;
            let param_type = &param.type_str;
            let param_ident = syn::Ident::new(param_name, proc_macro2::Span::call_site());
            
            // Generate extraction code based on parameter type
            let extraction = match param_type.as_str() {
                // Primitive types - direct extraction with defaults
                "u32" => quote::quote! { let #param_ident: u32 = table.get(#param_name).unwrap_or(0); },
                "i32" => quote::quote! { let #param_ident: i32 = table.get(#param_name).unwrap_or(0); },
                "u64" => quote::quote! { let #param_ident: u64 = table.get(#param_name).unwrap_or(0); },
                "i64" => quote::quote! { let #param_ident: i64 = table.get(#param_name).unwrap_or(0); },
                "f32" => quote::quote! { let #param_ident: f32 = table.get(#param_name).unwrap_or(0.0); },
                "f64" => quote::quote! { let #param_ident: f64 = table.get(#param_name).unwrap_or(0.0); },
                "usize" => quote::quote! { let #param_ident: usize = table.get::<u64>(#param_name).unwrap_or(0) as usize; },
                "bool" => quote::quote! { let #param_ident: bool = table.get(#param_name).unwrap_or(false); },
                "String" | "&str" => quote::quote! { let #param_ident: String = table.get(#param_name).unwrap_or_else(|_| String::new()); },
                
                // Check if it looks like a type name (PascalCase) - treat as enum/struct
                // Try to resolve the full type path and use reflection-based parsing
                other => {
                    // Check if it's a simple PascalCase name (potential enum)
                    let first_char = other.chars().next().unwrap_or('a');
                    if first_char.is_uppercase() && !other.contains("::") {
                        // Generate direct match for known wgpu/bevy types
                        match other {
                            "TextureFormat" => {
                                quote::quote! {
                                    let #param_ident = {
                                        use bevy::render::render_resource::TextureFormat;
                                        let format_str: String = table.get(#param_name).unwrap_or_else(|_| "Bgra8UnormSrgb".to_string());
                                        match format_str.as_str() {
                                            "Rgba8UnormSrgb" => TextureFormat::Rgba8UnormSrgb,
                                            "Bgra8UnormSrgb" => TextureFormat::Bgra8UnormSrgb,
                                            "Rgba8Unorm" => TextureFormat::Rgba8Unorm,
                                            "Bgra8Unorm" => TextureFormat::Bgra8Unorm,
                                            "Rgba16Float" => TextureFormat::Rgba16Float,
                                            "Rgba32Float" => TextureFormat::Rgba32Float,
                                            "R8Unorm" => TextureFormat::R8Unorm,
                                            "Rg8Unorm" => TextureFormat::Rg8Unorm,
                                            _ => TextureFormat::Bgra8UnormSrgb,
                                        }
                                    };
                                }
                            },
                            "TextureDimension" => {
                                quote::quote! {
                                    let #param_ident = {
                                        use bevy::render::render_resource::TextureDimension;
                                        let dim_str: String = table.get(#param_name).unwrap_or_else(|_| "D2".to_string());
                                        match dim_str.as_str() {
                                            "D1" => TextureDimension::D1,
                                            "D2" => TextureDimension::D2,
                                            "D3" => TextureDimension::D3,
                                            _ => TextureDimension::D2,
                                        }
                                    };
                                }
                            },
                            _ => {
                                // Unknown type - can't generate code
                                println!("cargo:warning=      ⚠ Unknown enum type: {} for {} (add to build.rs mapping)", other, param_name);
                                all_params_supported = false;
                                continue;
                            }
                        }
                    } else {
                        // Complex type path - can't handle generically
                        println!("cargo:warning=      ⚠ Unsupported parameter type: {} for {}", other, param_name);
                        all_params_supported = false;
                        continue;
                    }
                }
            };
            
            param_extractions.push(extraction);
            param_names.push(param_ident);
        }
        
        if !all_params_supported || param_names.is_empty() {
            println!("cargo:warning=    - Skipping constructor {}::{} (unsupported parameter types)", ctor.type_name, method_name);
            return None;
        }
        
        // Generate the constructor call
        let method_ident = syn::Ident::new(method_name, proc_macro2::Span::call_site());
        
        println!("cargo:warning=    - Registering constructor {}::{}({} params)", ctor.type_name, method_name, param_names.len());
        
        Some(quote::quote! {
            asset_registry.register_asset_constructor(#type_path_str, |table| {
                #(#param_extractions)*
                
                bevy::log::debug!("[AUTO_CONSTRUCTOR] Calling {}::{}", stringify!(#type_path), stringify!(#method_ident));
                Ok(Box::new(#type_path::#method_ident(#(#param_names),*)) as Box<dyn bevy::reflect::Reflect>)
            });
        })
    }).collect();
    
    println!("cargo:warning=  ✓ Generated {} constructor registrations", constructor_registrations.len());


    // Generate RUNTIME-BASED newtype wrapper names for TypeRegistry lookup
    // No compile-time type paths needed - runtime will discover via reflection
    let newtype_wrapper_tuples: Vec<_> = newtypes.iter()
        .filter_map(|nt| {
            // Filter out newtypes wrapping complex types (tuples, Arc, Mutex, etc.)
            let inner = &nt.inner_asset_path;
            if inner.contains("(") || inner.contains(")") || 
               inner.contains("Arc") || inner.contains("Mutex") ||
               inner.contains("Vec") || inner.len() <= 1 {
                return None;
            }
            
            // Extract just the newtype name from the path
            let newtype_name = nt.newtype_path.split("::").last()?;
            
            println!("cargo:warning=  ✓ Newtype wrapper: {} wraps Handle<{}>", newtype_name, inner);
            
            Some(quote::quote! { (#newtype_name, #inner) })
        })
        .collect();
    
    println!("cargo:warning=  ✓ Generated {} newtype wrapper names for runtime lookup", newtype_wrapper_tuples.len());
    
    
    // Discover observable events and generate observer handlers
    let observable_events = discover_observable_events();
    let observer_handlers = generate_observer_handlers(&observable_events);
    let observer_match_arms = generate_observer_match_arms(&observable_events);
    
    // Discover Bevy Event types for Lua read_events()
    let bevy_events = discover_bevy_events();
    
    // Discover Bevy Message types for Lua write_message() (uses MessageWriter<T>)
    let bevy_messages = discover_bevy_messages();
    
    // Deprecated/removed types to skip
    let deprecated_types = ["ReceivedCharacter", "Ime"];
    
    // Generate event registration code AND event dispatch match arms (for both read AND write)
    // We use bevy:: re-export paths because internal crates (bevy_window) aren't accessible
    // Instead of reflection, we generate dispatch_read_events and dispatch_write_events functions with match arms
    let mut event_match_arms = Vec::new();      // For reading events
    let mut event_write_match_arms = Vec::new(); // For writing events
    
    // Generate message write match arms from discovered message types
    let message_write_match_arms: Vec<_> = bevy_messages.iter().filter_map(|msg| {
        let short_name = &msg.type_name;
        let bevy_path_str = &msg.bevy_path;
        let full_path_str = &msg.full_path; // Original crate path (e.g., bevy_picking::pointer::PointerInput)
        let type_path: syn::Path = syn::parse_str(bevy_path_str).ok()?;
        
        Some(quote::quote! {
            #short_name | #bevy_path_str | #full_path_str => {
                // Get TypeRegistry for reflection
                let type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>().clone();
                let registry = type_registry.read();
                
                // Find the type registration for this message
                // Try both the original crate path and the bevy re-export path
                let type_registration = registry.get_with_type_path(#full_path_str)
                    .or_else(|| registry.get_with_type_path(#bevy_path_str));
                
                if let Some(type_registration) = type_registration {
                    let type_info = type_registration.type_info();
                    
                    // Get AssetRegistry for handle ID lookup during reflection
                    let asset_registry = world.get_resource::<bevy_lua_ecs::AssetRegistry>().cloned();
                    let dynamic = bevy_lua_ecs::lua_table_to_dynamic_with_assets(
                        lua, data, type_info, &type_registry, asset_registry.as_ref()
                    ).map_err(|e| format!("Failed to build message '{}': {}", #bevy_path_str, e))?;
                    
                    // Debug: Log fields in the DynamicStruct before conversion
                    use bevy::reflect::Struct;
                    for i in 0..dynamic.field_len() {
                        let field_name = dynamic.name_at(i).unwrap_or("unknown");
                        let field_value = dynamic.field_at(i).map(|f| {
                            format!("{} (kind: {:?})", f.reflect_type_path(), f.reflect_kind())
                        }).unwrap_or("None".to_string());
                        bevy::log::debug!("[MESSAGE_CONSTRUCT] Field '{}': {}", field_name, field_value);
                    }
                    
                    // Strategy 1: Create default instance and apply entire dynamic struct
                    if let Some(reflect_default) = type_registration.data::<bevy::prelude::ReflectDefault>() {
                        let mut concrete_instance = reflect_default.default();
                        
                        // Try to apply the entire dynamic struct to the default instance
                        match concrete_instance.try_apply(&dynamic) {
                            Ok(()) => {
                                // Downcast and send
                                if let Ok(concrete_message) = concrete_instance.take::<#type_path>() {
                                    drop(registry); // Release read lock before getting mutable access
                                    let mut system_state = bevy::ecs::system::SystemState::<bevy::prelude::MessageWriter<#type_path>>::new(world);
                                    let mut message_writer = system_state.get_mut(world);
                                    message_writer.write(concrete_message);
                                    bevy::log::debug!("[MESSAGE_WRITE] Sent message via try_apply: {}", #bevy_path_str);
                                    return Ok(());
                                } else {
                                    bevy::log::warn!("[MESSAGE_WRITE] try_apply succeeded but downcast failed for '{}'", #bevy_path_str);
                                }
                            }
                            Err(e) => {
                                bevy::log::debug!("[MESSAGE_WRITE] try_apply failed for '{}': {:?}", #bevy_path_str, e);
                            }
                        }
                    }
                    
                    // Strategy 2: Try ReflectFromReflect (handles most cases but may fail with complex nested types)
                    if let Some(reflect_from_reflect) = type_registration.data::<bevy::reflect::ReflectFromReflect>() {
                        if let Some(concrete_value) = reflect_from_reflect.from_reflect(&dynamic) {
                            // Downcast the reflected value to the concrete message type using take
                            if let Ok(concrete_message) = concrete_value.take::<#type_path>() {
                                // Use SystemState to get MessageWriter
                                drop(registry); // Release read lock before getting mutable access
                                let mut system_state = bevy::ecs::system::SystemState::<bevy::prelude::MessageWriter<#type_path>>::new(world);
                                let mut message_writer = system_state.get_mut(world);
                                message_writer.write(concrete_message);
                                bevy::log::debug!("[MESSAGE_WRITE] Sent message via ReflectFromReflect: {}", #bevy_path_str);
                                return Ok(());
                            } else {
                                bevy::log::warn!("[MESSAGE_WRITE] ReflectFromReflect succeeded but downcast failed for '{}'", #bevy_path_str);
                            }
                        } else {
                            bevy::log::debug!("[MESSAGE_WRITE] ReflectFromReflect::from_reflect returned None for '{}'", #bevy_path_str);
                        }
                    }
                    
                    // Strategy 3: Try FromReflect trait directly
                    if let Some(concrete_value) = <#type_path as bevy::reflect::FromReflect>::from_reflect(&dynamic) {
                        drop(registry); // Release read lock before getting mutable access
                        let mut system_state = bevy::ecs::system::SystemState::<bevy::prelude::MessageWriter<#type_path>>::new(world);
                        let mut message_writer = system_state.get_mut(world);
                        message_writer.write(concrete_value);
                        bevy::log::debug!("[MESSAGE_WRITE] Sent message via FromReflect trait: {}", #bevy_path_str);
                        return Ok(());
                    }
                    
                    return Err(format!("Failed to construct message '{}' - all conversion strategies failed. This usually means a nested type doesn't implement FromReflect properly or a newtype wrapper is causing issues.", #bevy_path_str));
                } else {
                    return Err(format!("Message type '{}' not found in TypeRegistry", #bevy_path_str));
                }
            }
        })
    }).collect();
    
    println!("cargo:warning=  ✓ Generated {} message dispatch match arms", message_write_match_arms.len());
    
    // Generate message type registrations (for TypeRegistry)
    let message_registrations: Vec<_> = bevy_messages.iter().filter_map(|msg| {
        let bevy_path_str = &msg.bevy_path;
        let type_path: syn::Path = syn::parse_str(bevy_path_str).ok()?;
        
        Some(quote::quote! {
            app.register_type::<#type_path>();
            bevy::log::debug!("[REGISTER_MESSAGES] Adding message type: {}", #bevy_path_str);
        })
    }).collect();
    
    println!("cargo:warning=  ✓ Generated {} message type registrations", message_registrations.len());
    
    let event_registrations: Vec<_> = bevy_events.iter().filter_map(|event| {
        // Skip deprecated types
        if deprecated_types.contains(&event.type_name.as_str()) {
            println!("cargo:warning=    ⚠ Skipping deprecated event: {}", event.type_name);
            return None;
        }
        
        // Convert to bevy:: re-export paths
        let bevy_path = if event.crate_name == "bevy_window" {
            // Window events are re-exported directly to bevy::window
            format!("bevy::window::{}", event.type_name)
        } else if event.crate_name == "bevy_input" {
            // Input events keep their submodule
            event.full_path.replace("bevy_input::", "bevy::input::")
        } else if event.crate_name == "bevy_picking" {
            // Picking events are re-exported to bevy::picking
            event.full_path.replace("bevy_picking::", "bevy::picking::")
        } else {
            event.full_path.clone()
        };
        
        let type_path: syn::Path = match syn::parse_str(&bevy_path) {
            Ok(p) => p,
            Err(_) => {
                println!("cargo:warning=  ⚠ Could not parse event path: {}", bevy_path);
                return None;
            }
        };
        
        // Generate match arm for event dispatch - use both short name and full internal path
        let short_name = &event.type_name;
        let full_internal_path = &event.full_path;
        
        event_match_arms.push(quote::quote! {
            #short_name | #full_internal_path | #bevy_path => {
                // Read events using EventReader
                let mut system_state = bevy::ecs::system::SystemState::<bevy::prelude::EventReader<#type_path>>::new(world);
                let mut event_reader = system_state.get_mut(world);
                
                let results = lua.create_table()?;
                let mut index = 1;
                
                for event in event_reader.read() {
                    // Convert event to Lua via reflection
                    // reflection_to_lua returns Result<LuaValue, LuaError>
                    if let Ok(event_value) = bevy_lua_ecs::reflection_to_lua(lua, event as &dyn bevy::reflect::PartialReflect, &type_registry) {
                        results.set(index, event_value)?;
                        index += 1;
                    }
                }
                
                Ok(mlua::Value::Table(results))
            }
        });
        
        // Generate match arm for EVENT WRITING - use EventWriter<T> and lua_table_to_dynamic
        event_write_match_arms.push(quote::quote! {
            #short_name | #full_internal_path | #bevy_path => {
                // Get TypeRegistry for reflection
                let type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>().clone();
                let registry = type_registry.read();
                
                // Find the type registration for this event
                if let Some(type_registration) = registry.get_with_type_path(#bevy_path) {
                    let type_info = type_registration.type_info();
                    
                    // Build DynamicStruct from the Lua table using type info
                    let dynamic = bevy_lua_ecs::lua_table_to_dynamic(lua, data, type_info, &type_registry)
                        .map_err(|e| format!("Failed to build event '{}': {}", #bevy_path, e))?;
                    
                    // Convert via FromReflect - T::from_reflect returns Option<T> directly
                    if let Some(concrete_event) = <#type_path as bevy::reflect::FromReflect>::from_reflect(&dynamic) {
                        // Use SystemState to get EventWriter
                        drop(registry); // Release read lock before getting mutable access
                        let mut system_state = bevy::ecs::system::SystemState::<bevy::prelude::EventWriter<#type_path>>::new(world);
                        let mut event_writer = system_state.get_mut(world);
                        event_writer.write(concrete_event);
                        bevy::log::debug!("[EVENT_WRITE] Sent event: {}", #bevy_path);
                        return Ok(());
                    }
                    return Err(format!("Failed to construct event '{}' via FromReflect", #bevy_path));
                } else {
                    return Err(format!("Event type '{}' not found in TypeRegistry", #bevy_path));
                }
            }
        });
        
        // Generate a string literal for debug logging
        let bevy_path_str = bevy_path.clone();
        
        Some(quote::quote! {
            bevy::log::debug!("[REGISTER_EVENTS] Adding event type: {}", #bevy_path_str);
            // Use add_event instead of register_type - this properly creates the Events<T> resource
            // and registers it in the app. register_type alone doesn't work for Events.
            app.add_event::<#type_path>();
            
            // Also register the type for reflection
            app.register_type::<#type_path>();
        })
    }).collect();
    println!("cargo:warning=  ✓ Generated {} event dispatch match arms", event_match_arms.len());
    
    // Convert entity wrapper names to quote literals for const array
    let entity_wrapper_name_literals: Vec<_> = entity_wrapper_names.iter()
        .map(|name| quote::quote! { #name })
        .collect();
    
    // Generate SystemParam type name literals for const array
    let systemparam_type_name_literals: Vec<_> = discovered_systemparams.iter()
        .map(|p| {
            let name = &p.type_name;
            let full_path = &p.full_path;
            quote::quote! { (#name, #full_path) }
        })
        .collect();
    
    println!("cargo:warning=  ✓ Generated {} SystemParam type names for runtime lookup", systemparam_type_name_literals.len());
    
    // Generate SystemParam method tuples: (param_type, method_name, return_type, is_iterator)
    let systemparam_method_literals: Vec<_> = discovered_systemparam_methods.iter()
        .filter(|m| {
            // Only include methods where all params are likely reflectable
            m.params.iter().all(|p| p.is_reflectable)
        })
        .map(|m| {
            let param_type = &m.param_type;
            let method_name = &m.method_name;
            let return_type = &m.return_type;
            let returns_iterator = m.returns_iterator;
            quote::quote! { (#param_type, #method_name, #return_type, #returns_iterator) }
        })
        .collect();
    
    println!("cargo:warning=  ✓ Generated {} SystemParam method entries for runtime lookup", systemparam_method_literals.len());
    
    // Generate dispatch match arms for SystemParam methods
    // Note: We no longer filter on is_reflectable since we use runtime TypeRegistry
    // Methods with non-registered params will fail at runtime with a clear error
    let systemparam_dispatch_arms: Vec<_> = discovered_systemparam_methods.iter()
        .filter_map(|m| {
            // Find the full path for this SystemParam type
            let param_info = discovered_systemparams.iter()
                .find(|p| p.type_name == m.param_type)?;
            
            // Normalize discovered path to actual bevy re-export path
            // Some discovered paths are simplified and need correction
            let path = match param_info.full_path.as_str() {
                // MeshRayCast is in a nested module
                p if p.contains("MeshRayCast") => 
                    "bevy::picking::mesh_picking::ray_cast::MeshRayCast".to_string(),
                // DefaultUiCamera also needs full path if used
                p if p.contains("DefaultUiCamera") =>
                    "bevy::ui::ui_node::DefaultUiCamera".to_string(),
                // Otherwise use as-is
                other => other.to_string(),
            };
            let path = &path;
            
            // Skip types with generic parameters (they require type arguments)
            if path.contains('<') || path.contains('>') {
                return None;
            }
            
            // Skip methods whose return type doesn't implement Debug (required for format!("{:?}") in dispatch)
            if !is_type_debuggable(&m.return_type) {
                if m.param_type == "MeshRayCast" {
                    println!("cargo:warning=  [FILTER] {}::{} filtered: return type '{}' not debuggable", 
                        m.param_type, m.method_name, m.return_type);
                }
                return None;
            }
            
            // TEMPORARY: Focused allowlist for fast builds - only include most useful SystemParam types
            // TODO: Make this more generic once build caching is implemented
            let allowed_prefixes = [
                "bevy::picking::",  // MeshRayCast
            ];
            if !allowed_prefixes.iter().any(|prefix| path.starts_with(prefix)) {
                return None;
            }
            
            let param_name = &m.param_type;
            let method_name = &m.method_name;
            
            // Debug: Log which methods pass the allowlist
            println!("cargo:warning=  [DISPATCH] Processing {}::{} (path: {}, {} params)", 
                param_name, method_name, path, m.params.len());
            
            // Try to parse the type path
            let type_path: syn::Path = syn::parse_str(path).ok()?;
            let method_ident = syn::Ident::new(method_name, proc_macro2::Span::call_site());
            
            // Build parameter handling code
            let mut param_extractions = Vec::new();
            let mut param_names = Vec::new();
            
            for (idx, p) in m.params.iter().enumerate() {
                // Skip reference params with lifetimes (like &MeshRayCastSettings)
                if p.is_reference && p.type_str.contains("Settings") {
                    // Use default for settings types
                    param_names.push(quote::quote! { &Default::default() });
                    continue;
                }
                
                // Clean up the parameter type string
                let param_type_str = p.type_str.replace("'_", "'static").replace("& ", "").replace("&", "");
                let type_name_cleaned = param_type_str.trim();
                
                // Resolve short type names to full paths using our mapping
                let Some(full_type_path) = resolve_short_type_to_full_path(type_name_cleaned) else {
                    println!("cargo:warning=      ⚠ Skipping {}::{}: can't resolve param type '{}'", 
                        param_name, method_name, type_name_cleaned);
                    return None;
                };
                
                // Try to parse the resolved full path as syn::Path
                let Ok(param_type_path) = syn::parse_str::<syn::Path>(&full_type_path) else {
                    println!("cargo:warning=      ⚠ Skipping {}::{}: can't parse resolved type '{}'", 
                        param_name, method_name, full_type_path);
                    return None;
                };
                
                let typed_param_ident = syn::Ident::new(&format!("typed_arg{}", idx), proc_macro2::Span::call_site());
                // Use short name for TypeRegistry lookup (it uses short_type_path)
                let type_name_lit = type_name_cleaned.to_string();
                
                // Generate extraction code using runtime reflection (TypeRegistry + ReflectDefault/FromReflect)
                // Then downcast to the concrete type for the method call
                param_extractions.push(quote::quote! {
                    let #typed_param_ident: #param_type_path = {
                        // Try to construct parameter via reflection
                        // First try ReflectDefault, then try constructing from Lua table directly via FromReflect
                        let type_reg = type_registry.get_with_short_type_path(#type_name_lit)
                            .or_else(|| type_registry.get_with_type_path(#type_name_lit));
                        
                        let param_result: Option<Box<dyn bevy::reflect::Reflect>> = type_reg
                            .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>())
                            .map(|rd| rd.default());
                        
                        let used_default = param_result.is_some();
                        
                        // If Default wasn't available, try to construct via FromReflect from Lua data
                        let mut param_instance = if let Some(inst) = param_result {
                            inst
                        } else {
                            // Try FromReflect - construct a DynamicStruct from Lua and convert
                            if let Some(arg_val) = args.front() {
                                if let mlua::Value::Table(t) = arg_val {
                                    // Get the type info and FromReflect trait data
                                    if let Some(type_registration) = type_reg {
                                        if let Some(from_reflect_data) = type_registration.data::<bevy::reflect::ReflectFromReflect>() {
                                            // Build DynamicStruct from Lua table using type info
                                            let type_info = type_registration.type_info();
                                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(lua, t, type_info, &app_type_registry)
                                                .map_err(|e| mlua::Error::RuntimeError(format!(
                                                    "Failed to build DynamicStruct for '{}': {}", #type_name_lit, e
                                                )))?;
                                            
                                            // Convert via FromReflect
                                            if let Some(reflected) = from_reflect_data.from_reflect(&dynamic) {
                                                // Remove the arg since we consumed it
                                                args.pop_front();
                                                reflected
                                            } else {
                                                return Err(mlua::Error::RuntimeError(format!(
                                                    "Cannot construct parameter type '{}' - FromReflect conversion failed. Check that all fields are provided.",
                                                    #type_name_lit
                                                )));
                                            }
                                        } else {
                                            return Err(mlua::Error::RuntimeError(format!(
                                                "Cannot construct parameter type '{}' - doesn't implement FromReflect",
                                                #type_name_lit
                                            )));
                                        }
                                    } else {
                                        return Err(mlua::Error::RuntimeError(format!(
                                            "Cannot construct parameter type '{}' - not found in TypeRegistry",
                                            #type_name_lit
                                        )));
                                    }
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Cannot construct parameter type '{}' - expected table argument",
                                        #type_name_lit
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Cannot construct parameter type '{}' - no argument provided and no Default",
                                    #type_name_lit
                                )));
                            }
                        };
                        
                        // If we used Default, populate from Lua arg if available
                        if used_default {
                            if let Some(arg_val) = args.pop_front() {
                                if let mlua::Value::Table(t) = arg_val {
                                    let _ = bevy_lua_ecs::lua_to_reflection(lua, &mlua::Value::Table(t), param_instance.as_partial_reflect_mut(), &app_type_registry);
                                }
                            }
                        }
                        
                        // Downcast to concrete type
                        param_instance.downcast_ref::<#param_type_path>()
                            .cloned()
                            .ok_or_else(|| mlua::Error::RuntimeError(format!(
                                "Failed to downcast parameter to '{}'", #type_name_lit
                            )))?
                    };
                });
                
                // If this is a reference parameter, pass a reference
                if p.is_reference {
                    param_names.push(quote::quote! { &#typed_param_ident });
                } else {
                    param_names.push(quote::quote! { #typed_param_ident });
                }
            }
            
            println!("cargo:warning=    + Generating dispatch for {}::{} ({} params)", param_name, method_name, m.params.len());
            
            // Generate method call with proper param passing
            let method_call = if param_names.is_empty() {
                quote::quote! { param.#method_ident() }
            } else {
                quote::quote! { param.#method_ident(#(#param_names),*) }
            };
            
            Some(quote::quote! {
                (#param_name, #method_name) => {
                    let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>().clone();
                    let type_registry = app_type_registry.read();
                    let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
                    
                    #(#param_extractions)*
                    
                    let mut state = bevy::ecs::system::SystemState::<#type_path>::new(world);
                    let mut param = state.get_mut(world);
                    let result = #method_call;
                    Ok(mlua::Value::String(lua.create_string(&format!("{:?}", result))?))
                }
            })
        })
        .collect();
    
    println!("cargo:warning=  ✓ Generated {} SystemParam dispatch arms", systemparam_dispatch_arms.len());
    
    let full_code = quote! {

        // Auto-generated Lua resource and component method bindings
        // Generated by bevy-lua-ecs build script
        
        pub fn register_auto_resource_bindings(registry: &bevy_lua_ecs::LuaResourceRegistry) {
            #(#method_bindings)*
        }
        
        /// Auto-discovered entity wrapper type names (for runtime TypeRegistry lookup)
        /// These are type names discovered by scanning bevy_* crates for:
        /// `pub struct TypeName(pub Entity)` with `#[derive(Component)]`
        pub const DISCOVERED_ENTITY_WRAPPERS: &[&str] = &[#(#entity_wrapper_name_literals),*];
        
        /// Register entity wrapper components at runtime using TypeRegistry
        /// This looks up each discovered type name in the registry and registers
        /// a handler if it's a valid entity wrapper component
        pub fn register_entity_wrappers_from_registry(
            component_registry: &mut bevy_lua_ecs::ComponentRegistry,
            type_registry: &bevy::ecs::reflect::AppTypeRegistry,
        ) {
            bevy_lua_ecs::register_entity_wrappers_runtime(
                component_registry,
                type_registry,
                DISCOVERED_ENTITY_WRAPPERS,
            );
        }
        
        pub fn register_auto_constructors(lua: &mlua::Lua) -> Result<(), mlua::Error> {
            #(#constructor_bindings)*
            Ok(())
        }
        
        /// Register all discovered bitflags types with the BitflagsRegistry
        /// Call this in your app's Startup systems to enable generic bitflags handling
        /// Generated from types discovered during asset constructor parsing
        pub fn register_auto_bitflags(registry: &bevy_lua_ecs::BitflagsRegistry) {
            #(#bitflags_registrations)*
        }
        
        /// Auto-discovered asset type names (for runtime TypeRegistry lookup)
        /// These are type names discovered by scanning bevy_* crates for:
        /// `impl Asset for TypeName` or `#[derive(Asset)] struct TypeName`
        pub const DISCOVERED_ASSET_TYPES: &[&str] = &[#(#asset_type_name_literals),*];
        
        /// Register asset types at runtime using TypeRegistry
        /// This looks up each discovered type name in the registry and registers
        /// handlers for valid Asset types (handle setters, asset adders, etc.)
        pub fn register_asset_types_from_registry(
            asset_registry: &bevy_lua_ecs::AssetRegistry,
            type_registry: &bevy::ecs::reflect::AppTypeRegistry,
        ) {
            bevy_lua_ecs::register_asset_types_runtime(
                asset_registry,
                type_registry,
                DISCOVERED_ASSET_TYPES,
            );
        }
        
        /// Auto-discovered Handle<T> newtype wrappers
        /// Format: (newtype_name, inner_asset_name) - runtime will resolve via TypeRegistry
        /// Examples: ("ImageRenderTarget", "Image"), ("Mesh3d", "Mesh")
        pub const DISCOVERED_NEWTYPE_WRAPPERS: &[(&str, &str)] = &[#(#newtype_wrapper_tuples),*];
        
        /// Register newtype wrappers at runtime using TypeRegistry discovery
        /// Enables wrapping Handle<T> in newtypes like ImageRenderTarget
        pub fn register_auto_newtype_wrappers(
            newtype_wrappers: &std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, bevy_lua_ecs::NewtypeWrapperCreator>>>,
        ) {
            // Runtime registration via TypeRegistry is done in asset_loading.rs
            // This const array is used for discovery
            bevy::log::debug!("[NEWTYPE_WRAPPERS] Discovered {} newtype wrappers for runtime lookup", DISCOVERED_NEWTYPE_WRAPPERS.len());
            for (newtype_name, inner_name) in DISCOVERED_NEWTYPE_WRAPPERS {
                bevy::log::debug!("[NEWTYPE_WRAPPERS]   - {} wraps Handle<{}>", newtype_name, inner_name);
            }
        }
        
        
        // ========================================
        // Auto-discovered SystemParam Types
        // ========================================
        
        /// Auto-discovered SystemParam type names and their full paths
        /// Format: (type_name, full_path) - for runtime lookup
        /// Examples: ("MeshRayCast", "bevy::picking::mesh_picking::ray_cast::MeshRayCast")
        pub const DISCOVERED_SYSTEMPARAMS: &[(&str, &str)] = &[#(#systemparam_type_name_literals),*];
        
        /// Auto-discovered SystemParam methods that use Reflect-compatible parameters
        /// Format: (param_type, method_name, return_type, returns_iterator)
        pub const DISCOVERED_SYSTEMPARAM_METHODS: &[(&str, &str, &str, bool)] = &[#(#systemparam_method_literals),*];
        
        /// Dispatch a SystemParam method call from Lua
        /// This uses SystemState to access SystemParams from World
        /// Currently supports no-arg methods; parameterized methods need reflection-based arg parsing
        pub fn dispatch_systemparam_method(
            lua: &mlua::Lua,
            world: &mut bevy::prelude::World,
            param_name: &str,
            method_name: &str,
            args: mlua::MultiValue,
        ) -> mlua::Result<mlua::Value> {
            match (param_name, method_name) {
                #(#systemparam_dispatch_arms),*
                _ => Err(mlua::Error::RuntimeError(format!(
                    "Unknown or unsupported SystemParam method: {}::{}", param_name, method_name
                )))
            }
        }
        
        /// Dispatch read_events call for a specific event type
        /// Returns a Lua table of events converted via reflection
        pub fn dispatch_read_events(
            lua: &mlua::Lua,
            world: &mut bevy::prelude::World,
            event_type: &str,
        ) -> mlua::Result<mlua::Value> {
            let type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>().clone();
            
            match event_type {
                #(#event_match_arms),*
                _ => Err(mlua::Error::RuntimeError(format!(
                    "Unknown event type: '{}'. Available events are discovered from bevy_window and bevy_input.", event_type
                )))
            }
        }
        
        /// Dispatch write_events call for a specific event type
        /// Constructs the event from a Lua table using reflection and sends via EventWriter
        pub fn dispatch_write_events(
            lua: &mlua::Lua,
            world: &mut bevy::prelude::World,
            event_type: &str,
            data: &mlua::Table,
        ) -> Result<(), String> {
            match event_type {
                #(#event_write_match_arms),*
                _ => Err(format!(
                    "Unknown event type: '{}'. Available events are discovered from bevy_window and bevy_input.", event_type
                ))
            }
        }
        
        /// Dispatch write_message call for a specific message type
        /// Uses MessageWriter<T> and lua_table_to_dynamic for reflection-based construction
        pub fn dispatch_write_message(
            lua: &mlua::Lua,
            world: &mut bevy::prelude::World,
            message_type: &str,
            data: &mlua::Table,
        ) -> Result<(), String> {
            match message_type {
                #(#message_write_match_arms),*
                _ => Err(format!(
                    "Unknown message type: '{}'. Discovered message types are auto-generated.", message_type
                ))
            }
        }

        // ========================================
        // Auto-generated Lua Observer Handlers
        // ========================================
        
        /// Dispatch a Lua observer callback for an entity
        fn dispatch_lua_observer(
            lua_ctx: &bevy_lua_ecs::LuaScriptContext,
            observer_registry: &bevy_lua_ecs::LuaObserverRegistry,
            update_queue: &bevy_lua_ecs::ComponentUpdateQueue,
            entity: bevy::prelude::Entity,
            event_type: &str,
            position: Option<bevy::math::Vec2>,
        ) {
            let callbacks = observer_registry.callbacks().lock().unwrap();
            
            if let Some(observers) = callbacks.get(&entity) {
                for (ev_type, callback_key) in observers {
                    if ev_type == event_type {
                        if let Ok(callback) = lua_ctx.lua.registry_value::<mlua::Function>(callback_key) {
                            let entity_snapshot = bevy_lua_ecs::LuaEntitySnapshot {
                                entity,
                                component_data: std::collections::HashMap::new(),
                                lua_components: std::collections::HashMap::new(),
                                update_queue: update_queue.clone(),
                            };
                            
                            let event_table = lua_ctx.lua.create_table().unwrap();
                            if let Some(pos) = position {
                                let _ = event_table.set("x", pos.x);
                                let _ = event_table.set("y", pos.y);
                            }
                            
                            if let Err(e) = callback.call::<()>((entity_snapshot, event_table)) {
                                bevy::log::error!("[LUA_OBSERVER] Error calling {} callback: {}", event_type, e);
                            }
                        }
                    }
                }
            }
        }
        
        // Auto-generated observer handler functions
        #observer_handlers
        
        /// Attach a Lua observer to an entity by event type name
        /// This function is generated with match arms for all discovered observable events
        pub fn attach_observer_by_name(
            commands: &mut bevy::prelude::Commands,
            entity: bevy::prelude::Entity,
            event_type: &str,
        ) {
            match event_type {
                #observer_match_arms
                _ => bevy::log::warn!("[LUA_OBSERVER] Unknown observer type: {}", event_type),
            }
        }
        
        // ========================================
        // Auto-generated LuaBindingsPlugin
        // ========================================
        
        /// Plugin that wraps LuaSpawnPlugin and automatically registers all auto-generated bindings.
        /// Use this instead of LuaSpawnPlugin directly to get automatic bitflags, component bindings,
        /// handle setters, and asset adders registration.
        pub struct LuaBindingsPlugin;
        
        impl bevy::prelude::Plugin for LuaBindingsPlugin {
            fn build(&self, app: &mut bevy::prelude::App) {
                // Add the core LuaSpawnPlugin
                app.add_plugins(bevy_lua_ecs::LuaSpawnPlugin);
                
                // Register the observer attacher - this connects the generated
                // attach_observer_by_name function to the library's observer system
                bevy_lua_ecs::set_observer_attacher(attach_observer_by_name);
                
                // Register the SystemParam method dispatcher - this connects the generated
                // dispatch_systemparam_method function to the library's call_systemparam_method
                bevy_lua_ecs::set_systemparam_dispatcher(dispatch_systemparam_method);
                
                // Register the event reader dispatcher - this connects the generated
                // dispatch_read_events function to the library's read_events
                bevy_lua_ecs::set_event_dispatcher(dispatch_read_events);
                
                // Register the event writer dispatcher - this connects the generated
                // dispatch_write_events function to the library's send_event
                bevy_lua_ecs::set_event_write_dispatcher(dispatch_write_events);
                
                // Register the message writer dispatcher - this connects the generated
                // dispatch_write_message function to the library's write_message
                bevy_lua_ecs::set_message_write_dispatcher(dispatch_write_message);
                
                // Register Bevy Event types for Lua read_events()
                // This registers Events<T> for auto-discovered event types
                register_bevy_events(app);
                
                // Initialize BitflagsRegistry
                app.init_resource::<bevy_lua_ecs::BitflagsRegistry>();
                
                // Add registration systems
                app.add_systems(bevy::prelude::Startup, setup_bitflags);
                app.add_systems(bevy::prelude::Startup, log_registered_events);
                app.add_systems(bevy::prelude::PostStartup, register_asset_constructors);
                
                // Add Lua message dispatch system (handles world:write_message in Lua scripts)
                app.add_systems(bevy::prelude::Update, bevy_lua_ecs::dispatch_lua_messages);
            }
        }
        
        /// Debug system to log all registered Events<T> types in the TypeRegistry
        fn log_registered_events(type_registry: bevy::prelude::Res<bevy::ecs::reflect::AppTypeRegistry>) {
            let registry = type_registry.read();
            bevy::log::info!("[DEBUG_EVENTS] === Scanning TypeRegistry for Events<*> types ===");
            
            let mut found_count = 0;
            for registration in registry.iter() {
                let type_path = registration.type_info().type_path();
                if type_path.contains("Events<") {
                    bevy::log::info!("[DEBUG_EVENTS] Found: '{}'", type_path);
                    found_count += 1;
                }
            }
            
            bevy::log::info!("[DEBUG_EVENTS] Total Events<*> types found: {}", found_count);
        }
        
        /// Register auto-discovered Bevy Event and Message types for Lua events/messages
        fn register_bevy_events(app: &mut bevy::prelude::App) {
            #(#event_registrations)*
            
            // Register message types (e.g., PointerInput for MessageWriter)
            #(#message_registrations)*
            
            bevy::log::debug!("Auto-discovered Bevy Events and Messages registered for Lua");
        }
        
        /// System to register auto-generated bitflags types
        fn setup_bitflags(registry: bevy::prelude::Res<bevy_lua_ecs::BitflagsRegistry>) {
            register_auto_bitflags(&registry);
            bevy::log::debug!("Auto-generated bitflags registered");
        }
        
        /// System to register auto-generated asset constructors, handle setters, and component bindings
        fn register_asset_constructors(
            asset_registry: bevy::prelude::Res<bevy_lua_ecs::AssetRegistry>,
            type_registry: bevy::prelude::Res<bevy::ecs::reflect::AppTypeRegistry>,
            mut component_registry: bevy::prelude::ResMut<bevy_lua_ecs::ComponentRegistry>,
        ) {
            // Register entity wrapper components using runtime TypeRegistry lookup
            // This looks up discovered type names and registers handlers for valid entity wrappers
            register_entity_wrappers_from_registry(&mut component_registry, &type_registry);
            
            // Register asset types using runtime TypeRegistry lookup
            // This discovers and registers handle setters, asset adders, and handle creators
            // for all asset types found in the TypeRegistry based on discovered type names
            register_asset_types_from_registry(&asset_registry, &type_registry);
            
            // Register newtype wrappers (still compile-time based)
            register_auto_newtype_wrappers(
                &asset_registry.newtype_wrappers,
            );
            
            // Register asset cloners for types that implement Clone
            // Only types detected via #[derive(Clone)] or impl Clone at compile time get cloners
            register_asset_cloners(&asset_registry);
            
            // Register auto-discovered asset constructors (for opaque types like Image)
            register_asset_constructor_bindings(&asset_registry);
            
            bevy::log::debug!("Auto-generated asset constructors, component bindings, and newtype wrappers registered");
        }
        
        /// Register asset cloners for types that implement Clone
        /// This is auto-generated based on compile-time detection of Clone derives/impls
        fn register_asset_cloners(asset_registry: &bevy_lua_ecs::AssetRegistry) {
            let mut cloners = asset_registry.asset_cloners_by_typeid.lock().unwrap();
            
            // Auto-generated cloner registrations (only for types with Clone at compile time)
            #(#cloner_registrations)*
            
            bevy::log::debug!("[ASSET_CLONER] Registered {} asset cloners (types with Clone impl)", cloners.len());
        }
        
        /// Register asset constructors for opaque types that need explicit constructors
        /// This is auto-generated based on discovered constructor methods
        fn register_asset_constructor_bindings(asset_registry: &bevy_lua_ecs::AssetRegistry) {
            // Auto-generated constructor registrations for opaque types
            #(#constructor_registrations)*
            
            bevy::log::debug!("[ASSET_CONSTRUCTOR] Registered auto-discovered asset constructors for opaque types");
        }
    };
    
    fs::write(&generated_file, full_code.to_string())
        .expect("Failed to write auto_resource_bindings.rs");
    
    println!("cargo:warning=✓ Wrote bindings to {:?}", generated_file);
}

fn write_empty_bindings_with_events(event_types: Vec<String>) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let generated_file = out_dir.join("auto_bindings.rs");
    
    // Generate event registration code
    let event_registrations: Vec<_> = event_types.iter().map(|event_type| {
        // Replace bevy_lua_ecs:: with crate:: for internal types
        let adjusted_type = event_type.replace("bevy_lua_ecs::", "crate::");
        let type_path = syn::parse_str::<syn::Path>(&adjusted_type).unwrap();
        quote::quote! {
            app.register_type::<#type_path>();
            #[allow(deprecated)]
            app.register_type::<bevy::prelude::Events<#type_path>>();
        }
    }).collect();
    
    // These were for a match-based approach, unused now
    // Using dispatch_stmts with named EventWriters instead
    
    // Build dispatch statements with proper variable names
    // NOTE: serde_json::from_value requires Deserialize, which Bevy events don't implement.
    // For now, we generate a logging-only dispatch. Full implementation needs reflection.
    let _dispatch_stmts: Vec<_> = event_types.iter().map(|event_type| {
        let adjusted_type = event_type.replace("bevy_lua_ecs::", "crate::");
        let type_path = syn::parse_str::<syn::Path>(&adjusted_type).unwrap();
        let type_name = event_type.clone();
        let writer_ident = syn::Ident::new(
            &adjusted_type.replace("::", "_").to_lowercase(),
            proc_macro2::Span::call_site()
        );
        quote::quote! {
            if type_name == #type_name {
                // TODO: Use reflection to construct event from JSON data
                bevy::log::debug!("[LUA_EVENT] Would dispatch {} (reflection-based dispatch not yet implemented)", type_name);
                let _ = (#writer_ident, #type_path::default); // Suppress unused warnings
                continue;
            }
        }
    }).collect();
    
    // Build EventWriter params with unique names
    let _writer_params_named: Vec<_> = event_types.iter().map(|event_type| {
        let adjusted_type = event_type.replace("bevy_lua_ecs::", "crate::");
        let type_path = syn::parse_str::<syn::Path>(&adjusted_type).unwrap();
        let writer_ident = syn::Ident::new(
            &adjusted_type.replace("::", "_").to_lowercase(),
            proc_macro2::Span::call_site()
        );
        quote::quote! {
            mut #writer_ident: bevy::prelude::EventWriter<#type_path>
        }
    }).collect();
    
    // Discover message types and generate match arms for MessageWriter<T> dispatch
    let bevy_messages = discover_bevy_messages();
    let message_write_match_arms: Vec<_> = bevy_messages.iter().filter_map(|msg| {
        let short_name = &msg.type_name;
        let bevy_path_str = &msg.bevy_path;
        let type_path: syn::Path = syn::parse_str(bevy_path_str).ok()?;
        
        Some(quote::quote! {
            #short_name | #bevy_path_str => {
                let type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>().clone();
                let registry = type_registry.read();
                
                if let Some(type_registration) = registry.get_with_type_path(#bevy_path_str) {
                    let type_info = type_registration.type_info();
                    let dynamic = crate::lua_table_to_dynamic(lua, data, type_info, &type_registry)
                        .map_err(|e| format!("Failed to build message '{}': {}", #bevy_path_str, e))?;
                    
                    if let Some(concrete_message) = <#type_path as bevy::reflect::FromReflect>::from_reflect(&dynamic) {
                        drop(registry);
                        let mut system_state = bevy::ecs::system::SystemState::<bevy::prelude::MessageWriter<#type_path>>::new(world);
                        let mut message_writer = system_state.get_mut(world);
                        message_writer.write(concrete_message);
                        bevy::log::debug!("[MESSAGE_WRITE] Sent message: {}", #bevy_path_str);
                        return Ok(());
                    }
                    return Err(format!("Failed to construct message '{}' via FromReflect", #bevy_path_str));
                } else {
                    return Err(format!("Message type '{}' not found in TypeRegistry", #bevy_path_str));
                }
            }
        })
    }).collect();
    
    // Generate message type registrations (for TypeRegistry)
    let message_registrations: Vec<_> = bevy_messages.iter().filter_map(|msg| {
        let bevy_path_str = &msg.bevy_path;
        let type_path: syn::Path = syn::parse_str(bevy_path_str).ok()?;
        
        Some(quote::quote! {
            app.register_type::<#type_path>();
            bevy::log::debug!("[REGISTER_MESSAGES] Adding message type: {}", #bevy_path_str);
        })
    }).collect();
    
    let full_code = quote! {
        /// Auto-generated Lua resource bindings
        pub fn register_auto_bindings(_registry: &crate::resource_lua_trait::LuaResourceRegistry) {
            // No resource bindings generated
        }
        
        /// Auto-generated event and message type registrations
        pub fn register_auto_events(app: &mut bevy::prelude::App) {
            #(#event_registrations)*
            
            // Register message types for reflection (e.g., PointerInput)
            #(#message_registrations)*
        }
        
        /// Auto-generated event dispatch system
        /// This system drains PendingLuaEvents and logs them.
        /// TODO: Implement reflection-based event construction and dispatch.
        pub fn dispatch_lua_events(
            pending: bevy::prelude::Res<crate::event_sender::PendingLuaEvents>,
        ) {
            let events = pending.drain_events();
            for (type_name, data) in events {
                bevy::log::debug!("[LUA_EVENT] Received event '{}': {:?}", type_name, data);
            }
        }
        
        /// Auto-generated message dispatch system
        /// This system drains PendingLuaMessages and dispatches them via MessageWriter.
        /// Uses the same reflection pattern as event dispatch.
        pub fn dispatch_lua_messages(
            world: &mut bevy::prelude::World,
        ) {
            // Get pending messages
            let pending = world.resource::<crate::event_sender::PendingLuaMessages>().clone();
            let messages = pending.drain_messages();
            
            if messages.is_empty() {
                return;
            }
            
            // Get Lua context for reflection
            let lua_ctx = world.resource::<crate::LuaScriptContext>().clone();
            
            for (type_name, data) in messages {
                bevy::log::debug!("[LUA_MESSAGE] Processing message '{}': {:?}", type_name, data);
                
                // Convert JSON to Lua table for reflection
                match json_to_lua_table(&lua_ctx.lua, &data) {
                    Ok(lua_table) => {
                        // Use the global message dispatch function (set by parent crate's generated code)
                        if let Err(e) = crate::call_write_messages_global(&lua_ctx.lua, world, &type_name, &lua_table) {
                            bevy::log::warn!("[LUA_MESSAGE] Failed to dispatch '{}': {}", type_name, e);
                        }
                    }
                    Err(e) => {
                        bevy::log::warn!("[LUA_MESSAGE] Failed to convert JSON to Lua table: {}", e);
                    }
                }
            }
        }
        
        /// Convert serde_json::Value to mlua::Table
        fn json_to_lua_table(lua: &mlua::Lua, value: &serde_json::Value) -> mlua::Result<mlua::Table> {
            fn json_to_lua_value(lua: &mlua::Lua, value: &serde_json::Value) -> mlua::Result<mlua::Value> {
                match value {
                    serde_json::Value::Null => Ok(mlua::Value::Nil),
                    serde_json::Value::Bool(b) => Ok(mlua::Value::Boolean(*b)),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            Ok(mlua::Value::Integer(i))
                        } else if let Some(f) = n.as_f64() {
                            Ok(mlua::Value::Number(f))
                        } else {
                            Ok(mlua::Value::Nil)
                        }
                    }
                    serde_json::Value::String(s) => Ok(mlua::Value::String(lua.create_string(s)?)),
                    serde_json::Value::Array(arr) => {
                        let table = lua.create_table()?;
                        for (i, v) in arr.iter().enumerate() {
                            table.set(i + 1, json_to_lua_value(lua, v)?)?;
                        }
                        Ok(mlua::Value::Table(table))
                    }
                    serde_json::Value::Object(obj) => {
                        let table = lua.create_table()?;
                        for (k, v) in obj {
                            table.set(k.as_str(), json_to_lua_value(lua, v)?)?;
                        }
                        Ok(mlua::Value::Table(table))
                    }
                }
            }
            
            if let serde_json::Value::Object(_) = value {
                if let mlua::Value::Table(t) = json_to_lua_value(lua, value)? {
                    return Ok(t);
                }
            }
            Err(mlua::Error::RuntimeError("Expected JSON object".into()))
        }
        
        /// Dispatch write_message call for a specific message type
        /// Uses MessageWriter<T> and lua_table_to_dynamic for reflection-based construction
        pub fn dispatch_write_message(
            lua: &mlua::Lua,
            world: &mut bevy::prelude::World,
            message_type: &str,
            data: &mlua::Table,
        ) -> Result<(), String> {
            match message_type {
                #(#message_write_match_arms),*
                _ => Err(format!(
                    "Unknown message type: '{}'. Discovered message types are auto-generated.", message_type
                ))
            }
        }
        
        /// Dispatch a SystemParam method call from Lua (stub when no parent manifest)
        /// The full implementation is generated when building from a parent crate with lua_resources
        pub fn dispatch_systemparam_method(
            _lua: &mlua::Lua,
            _world: &mut bevy::prelude::World,
            param_name: &str,
            method_name: &str,
            _args: mlua::MultiValue,
        ) -> mlua::Result<mlua::Value> {
            Err(mlua::Error::RuntimeError(format!(
                "SystemParam method dispatch not available: {}::{}. Build from parent crate to enable.",
                param_name, method_name
            )))
        }
    };
    
    fs::write(generated_file, full_code.to_string())
        .expect("Failed to write auto_bindings.rs");
}

// =============================================================================
// ASSET CONSTRUCTOR DISCOVERY (Recursive Type Discovery)
// =============================================================================

/// Represents a discovered type definition
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum TypeDefinition {
    /// Primitive type (u32, f32, i32, bool, String, etc.)
    Primitive { name: String },
    /// Struct with fields
    Struct { 
        name: String,
        full_path: String,
        fields: Vec<FieldDef>,
    },
    /// Enum with variants
    Enum {
        name: String,
        full_path: String,
        variants: Vec<String>,
    },
    /// Bitflags type (has CONST flags that can be ORed)
    Bitflags {
        name: String,
        full_path: String,
        flags: Vec<String>,
    },
    /// Reference type (e.g., &[u8])
    Reference { inner: String },
    /// Unknown type - treat as opaque
    Unknown { name: String },
}

/// Map of short type names to full paths, extracted from use statements
type ImportMap = std::collections::HashMap<String, String>;

/// Parse use statements from a source file to build an import map
#[allow(dead_code)]
fn parse_use_statements(syntax_tree: &File) -> ImportMap {
    let mut imports = ImportMap::new();
    
    for item in &syntax_tree.items {
        if let Item::Use(use_item) = item {
            collect_use_paths(&use_item.tree, String::new(), &mut imports);
        }
    }
    
    imports
}

/// Recursively collect paths from use trees
fn collect_use_paths(tree: &syn::UseTree, prefix: String, imports: &mut ImportMap) {
    match tree {
        syn::UseTree::Path(use_path) => {
            let new_prefix = if prefix.is_empty() {
                use_path.ident.to_string()
            } else {
                format!("{}::{}", prefix, use_path.ident)
            };
            collect_use_paths(&use_path.tree, new_prefix, imports);
        }
        syn::UseTree::Name(use_name) => {
            let full_path = if prefix.is_empty() {
                use_name.ident.to_string()
            } else {
                format!("{}::{}", prefix, use_name.ident)
            };
            imports.insert(use_name.ident.to_string(), full_path);
        }
        syn::UseTree::Rename(use_rename) => {
            let full_path = if prefix.is_empty() {
                use_rename.ident.to_string()
            } else {
                format!("{}::{}", prefix, use_rename.ident)
            };
            imports.insert(use_rename.rename.to_string(), full_path);
        }
        syn::UseTree::Glob(_) => {
            // Can't resolve glob imports statically
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_use_paths(item, prefix.clone(), imports);
            }
        }
    }
}

/// Known Bevy type paths for common types that might not be in source file's imports
/// Maps to the actual source crate where the type is defined (not re-export paths)
#[allow(dead_code)]
fn get_known_bevy_type_path(short_name: &str) -> Option<&'static str> {
    match short_name {
        // wgpu types - these are in wgpu-types crate
        "Extent3d" => Some("wgpu_types::Extent3d"),
        "TextureDimension" => Some("wgpu_types::TextureDimension"),
        "TextureFormat" => Some("wgpu_types::TextureFormat"),
        "TextureUsages" => Some("wgpu_types::TextureUsages"),
        
        // Bevy render asset types
        "RenderAssetUsages" => Some("bevy_render::render_asset::RenderAssetUsages"),
        
        // Math types
        "Vec2" => Some("glam::f32::vec2::Vec2"),
        "Vec3" => Some("glam::f32::vec3::Vec3"),
        "Vec4" => Some("glam::f32::vec4::Vec4"),
        "Quat" => Some("glam::f32::quat::Quat"),
        
        // Color
        "Color" => Some("bevy_color::color::Color"),
        
        _ => None,
    }
}

/// Resolve a short type name to its full path using imports and known types
#[allow(dead_code)]
fn resolve_type_path(short_name: &str, imports: &ImportMap) -> String {
    // First check imports
    if let Some(full_path) = imports.get(short_name) {
        return full_path.clone();
    }
    
    // Then check known Bevy types
    if let Some(known_path) = get_known_bevy_type_path(short_name) {
        return known_path.to_string();
    }
    
    // Return as-is if not found
    short_name.to_string()
}

/// Get a hardcoded type definition for common types that are hard to discover
/// (e.g., wgpu types, external crate types)
/// Uses Bevy re-export paths so they're accessible from user crates
#[allow(dead_code)]
fn get_known_type_definition(type_name: &str) -> Option<TypeDefinition> {
    match type_name {
        "Extent3d" => Some(TypeDefinition::Struct {
            name: "Extent3d".to_string(),
            full_path: "bevy::render::render_resource::Extent3d".to_string(),
            fields: vec![
                FieldDef { name: "width".to_string(), type_str: "u32".to_string(), type_def: None },
                FieldDef { name: "height".to_string(), type_str: "u32".to_string(), type_def: None },
                FieldDef { name: "depth_or_array_layers".to_string(), type_str: "u32".to_string(), type_def: None },
            ],
        }),
        "TextureDimension" => Some(TypeDefinition::Enum {
            name: "TextureDimension".to_string(),
            full_path: "bevy::render::render_resource::TextureDimension".to_string(),
            variants: vec!["D1".to_string(), "D2".to_string(), "D3".to_string()],
        }),
        "TextureFormat" => Some(TypeDefinition::Enum {
            name: "TextureFormat".to_string(),
            full_path: "bevy::render::render_resource::TextureFormat".to_string(),
            variants: vec![
                "R8Unorm".to_string(), "R8Snorm".to_string(), "R8Uint".to_string(), "R8Sint".to_string(),
                "R16Uint".to_string(), "R16Sint".to_string(), "R16Float".to_string(),
                "Rg8Unorm".to_string(), "Rg8Snorm".to_string(), "Rg8Uint".to_string(), "Rg8Sint".to_string(),
                "R32Uint".to_string(), "R32Sint".to_string(), "R32Float".to_string(),
                "Rg16Uint".to_string(), "Rg16Sint".to_string(), "Rg16Float".to_string(),
                "Rgba8Unorm".to_string(), "Rgba8UnormSrgb".to_string(), "Rgba8Snorm".to_string(),
                "Rgba8Uint".to_string(), "Rgba8Sint".to_string(),
                "Bgra8Unorm".to_string(), "Bgra8UnormSrgb".to_string(),
                "Rgba16Uint".to_string(), "Rgba16Sint".to_string(), "Rgba16Float".to_string(),
                "Rgba32Uint".to_string(), "Rgba32Sint".to_string(), "Rgba32Float".to_string(),
                "Depth32Float".to_string(), "Depth24Plus".to_string(), "Depth24PlusStencil8".to_string(),
            ],
        }),
        "TextureUsages" => Some(TypeDefinition::Bitflags {
            name: "TextureUsages".to_string(),
            full_path: "bevy::render::render_resource::TextureUsages".to_string(),
            flags: vec![
                "COPY_SRC".to_string(), "COPY_DST".to_string(), 
                "TEXTURE_BINDING".to_string(), "STORAGE_BINDING".to_string(), 
                "RENDER_ATTACHMENT".to_string(),
            ],
        }),
        "RenderAssetUsages" => Some(TypeDefinition::Bitflags {
            name: "RenderAssetUsages".to_string(),
            full_path: "bevy::asset::RenderAssetUsages".to_string(),
            flags: vec![
                "MAIN_WORLD".to_string(), "RENDER_WORLD".to_string(),
            ],
        }),
        _ => None,
    }
}

/// Normalize Bevy crate paths to use bevy:: re-exports accessible from user crates
/// Returns None if the path cannot be safely normalized (internal modules, etc.)
/// 
/// e.g., bevy_image::image::Image -> bevy::prelude::Image
///       bevy_mesh::mesh::Mesh -> bevy::prelude::Mesh
/// 
/// Types from internal modules (::forward::, ::prepare::, ::extract::, etc.) are rejected.
fn normalize_bevy_path(path: &str) -> Option<String> {
    // Reject non-core bevy crates (not part of bevy umbrella)
    let non_core_crates = [
        "bevy_ecs_tilemap", "bevy_egui", "bevy_rapier2d", "bevy_rapier3d",
        "bevy_xpbd", "bevy_hanabi", "bevy_kira_audio",
    ];
    for crate_name in &non_core_crates {
        if path.starts_with(&format!("{}::", crate_name)) {
            return None;
        }
    }
    
    // Reject paths from internal-looking modules  
    let internal_patterns = [
        "::forward::", "::prepare::", "::extract::", "::render::", 
        "::internal::", "::private::", "::asset::", "::skinning::",
        "::compensation_curve::", "::gpu_", "_systems::", "::tilemap",
        "::wireframe", "::pitch::", "::audio_output", "::gizmos::",
        "::storage::", "::buffer::", "::line_gizmo::",
        "ColorMaterial", "TextureAtlas", // These types don't exist at simple paths
        "LineGizmo", "AnimationGraph", "Shader", // Private or inaccessible types
    ];
    for pattern in &internal_patterns {
        if path.contains(pattern) {
            return None;
        }
    }
    
    // wgpu_types::X -> bevy::render::render_resource::X
    if path.starts_with("wgpu_types::") {
        let type_name = path.strip_prefix("wgpu_types::")?;
        return Some(format!("bevy::render::render_resource::{}", type_name));
    }
    
    // Special case: Mesh is re-exported in bevy::prelude
    if path == "bevy_mesh::mesh::Mesh" || path == "bevy_mesh::Mesh" {
        return Some("bevy::prelude::Mesh".to_string());
    }
    
    // Special case: StandardMaterial is re-exported in bevy::prelude
    if path == "bevy_pbr::pbr_material::StandardMaterial" || path == "bevy_pbr::StandardMaterial" {
        return Some("bevy::prelude::StandardMaterial".to_string());
    }
    
    // Special case: Image is re-exported in bevy::prelude
    if path == "bevy_image::image::Image" || path == "bevy_image::Image" {
        return Some("bevy::prelude::Image".to_string());
    }
    
    // bevy_ui types -> bevy::ui
    if path.contains("UiTargetCamera") {
        return Some("bevy::ui::UiTargetCamera".to_string());
    }
    if path.starts_with("bevy_ui::") {
        if let Some(type_name) = path.split("::").last() {
            return Some(format!("bevy::ui::{}", type_name));
        }
    }
    
    // TextureAtlasLayout -> bevy::prelude
    if path.contains("TextureAtlasLayout") {
        return Some("bevy::prelude::TextureAtlasLayout".to_string());
    }
    
    // bevy_sprite types -> bevy::sprite (only simple paths)
    if path.starts_with("bevy_sprite::") {
        if let Some(type_name) = path.split("::").last() {
            if path.matches("::").count() <= 2 {
                return Some(format!("bevy::sprite::{}", type_name));
            }
        }
        return None; // Reject complex paths
    }
    
    // bevy_text types -> bevy::text (only simple paths)
    if path.starts_with("bevy_text::") {
        if let Some(type_name) = path.split("::").last() {
            if path.matches("::").count() <= 2 {
                return Some(format!("bevy::text::{}", type_name));
            }
        }
        return None;
    }
    
    // For other bevy_* crates, be conservative - only allow simple direct paths
    if path.starts_with("bevy_") {
        let parts: Vec<&str> = path.split("::").collect();
        // Only transform if path has 2-3 segments (crate::Type or crate::module::Type)
        if parts.len() >= 2 && parts.len() <= 3 {
            let crate_part = parts[0].strip_prefix("bevy_")?;
            let type_name = *parts.last()?;
            return Some(format!("bevy::{}::{}", crate_part, type_name));
        }
        // Paths with more segments are likely internal, reject them
        return None;
    }
    
    // Non-bevy crate paths (custom crates) pass through unchanged
    Some(path.to_string())
}

/// A field in a struct
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FieldDef {
    name: String,
    type_str: String,
    type_def: Option<Box<TypeDefinition>>,
}

/// A parameter in a constructor
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ParamDef {
    name: String,
    type_str: String,
    type_def: Option<TypeDefinition>,
}

/// Discover a type definition recursively by parsing source files
#[allow(dead_code)]
fn discover_type_recursive(
    type_path: &str,
    discovered: &mut std::collections::HashMap<String, TypeDefinition>,
) -> Result<TypeDefinition, String> {
    // Check if already discovered
    if let Some(def) = discovered.get(type_path) {
        return Ok(def.clone());
    }
    
    // Check for primitives first
    if is_primitive_type(type_path) {
        let def = TypeDefinition::Primitive { name: type_path.to_string() };
        discovered.insert(type_path.to_string(), def.clone());
        return Ok(def);
    }
    
    // Check for reference types
    if type_path.starts_with("&") {
        let inner = type_path.trim_start_matches("&").trim();
        let def = TypeDefinition::Reference { inner: inner.to_string() };
        discovered.insert(type_path.to_string(), def.clone());
        return Ok(def);
    }
    
    // Parse the type spec
    let type_spec = parse_type_spec(type_path)
        .ok_or_else(|| format!("Could not parse type path: {}", type_path))?;
    
    // Find and parse the source file
    let source_path = find_source_file(&type_spec)?;
    let source_code = fs::read_to_string(&source_path)
        .map_err(|e| format!("Failed to read source: {}", e))?;
    
    let syntax_tree: File = syn::parse_file(&source_code)
        .map_err(|e| format!("Failed to parse source: {}", e))?;
    
    // Find the type definition in the syntax tree
    let type_def = find_type_definition(&syntax_tree, &type_spec.type_name, type_path)?;
    
    discovered.insert(type_path.to_string(), type_def.clone());
    
    Ok(type_def)
}

/// Check if a type is a primitive
fn is_primitive_type(type_str: &str) -> bool {
    matches!(type_str, 
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
        "f32" | "f64" | "bool" | "String" | "str" | "()"
    )
}

/// Find type definition in a syntax tree
#[allow(dead_code)]
fn find_type_definition(
    syntax_tree: &File,
    type_name: &str,
    full_path: &str,
) -> Result<TypeDefinition, String> {
    for item in &syntax_tree.items {
        match item {
            Item::Struct(item_struct) if item_struct.ident == type_name => {
                return parse_struct_definition(item_struct, full_path);
            }
            Item::Enum(item_enum) if item_enum.ident == type_name => {
                return parse_enum_definition(item_enum, full_path);
            }
            // Check for bitflags macro
            Item::Macro(item_macro) => {
                if let Some(def) = try_parse_bitflags_macro(item_macro, type_name, full_path) {
                    return Ok(def);
                }
            }
            _ => {}
        }
    }
    
    Err(format!("Type {} not found in source file", type_name))
}

/// Parse a struct definition
#[allow(dead_code)]
fn parse_struct_definition(item_struct: &syn::ItemStruct, full_path: &str) -> Result<TypeDefinition, String> {
    let mut fields = Vec::new();
    
    if let syn::Fields::Named(named_fields) = &item_struct.fields {
        for field in &named_fields.named {
            if let Some(name) = &field.ident {
                let type_str = quote::quote!(#field.ty).to_string()
                    .replace(" ", "");
                
                fields.push(FieldDef {
                    name: name.to_string(),
                    type_str,
                    type_def: None, // Will be filled by recursive discovery
                });
            }
        }
    }
    
    Ok(TypeDefinition::Struct {
        name: item_struct.ident.to_string(),
        full_path: full_path.to_string(),
        fields,
    })
}

/// Parse an enum definition - only collects unit variants (no fields)
#[allow(dead_code)]
fn parse_enum_definition(item_enum: &syn::ItemEnum, full_path: &str) -> Result<TypeDefinition, String> {
    let mut variants = Vec::new();
    
    // Variants that are in wgpu-types but have different names or don't exist in Bevy
    let wgpu_bevy_incompatible: std::collections::HashSet<&str> = [
        "Rg11b10Float",  // wgpu uses this, Bevy uses Rg11b10Ufloat
        "Rgb10a2Uint",   // May not exist in all Bevy versions
        "Rg11b10Ufloat", // Bevy name, not in wgpu source
    ].iter().copied().collect();
    
    for variant in &item_enum.variants {
        // Only collect unit variants (no fields) - skip tuple/struct variants
        if matches!(variant.fields, syn::Fields::Unit) {
            let variant_name = variant.ident.to_string();
            
            // Skip variants known to have wgpu/Bevy incompatibilities
            if wgpu_bevy_incompatible.contains(variant_name.as_str()) {
                continue;
            }
            
            variants.push(variant_name);
        }
    }
    
    Ok(TypeDefinition::Enum {
        name: item_enum.ident.to_string(),
        full_path: full_path.to_string(),
        variants,
    })
}

/// Try to parse a bitflags! macro invocation
#[allow(dead_code)]
fn try_parse_bitflags_macro(
    item_macro: &syn::ItemMacro,
    type_name: &str,
    full_path: &str,
) -> Option<TypeDefinition> {
    // Check if this is a bitflags! macro
    let macro_name = item_macro.mac.path.segments.last()?.ident.to_string();
    if macro_name != "bitflags" {
        return None;
    }
    
    // Parse the macro content to find the type and flags
    let tokens_str = item_macro.mac.tokens.to_string();
    
    // Simple heuristic: check if this macro defines our type
    if !tokens_str.contains(type_name) {
        return None;
    }
    
    // Extract flag names (const NAME = ...)
    let mut flags = Vec::new();
    for line in tokens_str.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("const ") {
            if let Some(name) = trimmed
                .strip_prefix("const ")?
                .split(&['=', ':'][..])
                .next()
            {
                let flag_name = name.trim();
                if !flag_name.is_empty() && flag_name.chars().all(|c| c.is_uppercase() || c == '_') {
                    flags.push(flag_name.to_string());
                }
            }
        }
    }
    
    if flags.is_empty() {
        return None;
    }
    
    Some(TypeDefinition::Bitflags {
        name: type_name.to_string(),
        full_path: full_path.to_string(),
        flags,
    })
}

/// Parse a constructor function signature and return parameter definitions
#[allow(dead_code)]
fn parse_constructor_signature(
    syntax_tree: &File,
    type_name: &str,
    fn_name: &str,
) -> Result<Vec<ParamDef>, String> {
    for item in &syntax_tree.items {
        if let Item::Impl(item_impl) = item {
            // Check if this is an impl for our type
            if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                let impl_type_name = type_path.path.segments.last()
                    .map(|s| s.ident.to_string());
                
                if impl_type_name.as_deref() != Some(type_name) {
                    continue;
                }
            }
            
            // Look for the function
            for impl_item in &item_impl.items {
                if let ImplItem::Fn(impl_fn) = impl_item {
                    if impl_fn.sig.ident == fn_name {
                        return extract_fn_params(&impl_fn.sig);
                    }
                }
            }
        }
    }
    
    Err(format!("Constructor {}::{} not found", type_name, fn_name))
}

/// Extract parameters from a function signature
#[allow(dead_code)]
fn extract_fn_params(sig: &syn::Signature) -> Result<Vec<ParamDef>, String> {
    let mut params = Vec::new();
    
    for arg in &sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            let name = match &*pat_type.pat {
                syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                _ => "_".to_string(),
            };
            
            // Extract the type properly
            let ty = &*pat_type.ty;
            let type_str = quote::quote!(#ty).to_string()
                .replace(" ", "");
            
            params.push(ParamDef {
                name,
                type_str,
                type_def: None,
            });
        }
    }
    
    Ok(params)
}

/// Discovered bitflags info for auto-registration
#[derive(Clone, Debug)]
struct DiscoveredBitflags {
    name: String,
    _full_path: String,
    flags: Vec<String>,
}

/// Discover all constructors for a type by parsing its impl blocks
/// Returns a list of function names that are likely constructors (return Self or the type)
fn discover_constructors_for_type(
    syntax_tree: &File,
    type_name: &str,
) -> Vec<String> {
    let mut constructors = Vec::new();
    
    for item in &syntax_tree.items {
        if let Item::Impl(impl_block) = item {
            // Check if this impl is for our type
            if !is_impl_for_type(impl_block, type_name) {
                continue;
            }
            
            for impl_item in &impl_block.items {
                if let ImplItem::Fn(method) = impl_item {
                    // Check if public
                    if !matches!(method.vis, Visibility::Public(_)) {
                        continue;
                    }
                    
                    let fn_name = method.sig.ident.to_string();
                    
                    // Skip methods that take &self or &mut self (not constructors)
                    let has_self_receiver = method.sig.inputs.iter().any(|arg| {
                        matches!(arg, FnArg::Receiver(_))
                    });
                    if has_self_receiver {
                        continue;
                    }
                    
                    // Check if return type indicates a constructor
                    // Look for Self, TypeName, Result<Self>, Option<Self>, etc.
                    let is_constructor = match &method.sig.output {
                        ReturnType::Default => false,
                        ReturnType::Type(_, ty) => {
                            let type_str = quote::quote!(#ty).to_string();
                            type_str.contains("Self") || 
                            type_str.contains(type_name) ||
                            // Common constructor patterns
                            fn_name.starts_with("new") ||
                            fn_name.starts_with("from_") ||
                            fn_name == "default" ||
                            fn_name == "empty" ||
                            fn_name == "create"
                        }
                    };
                    
                    if is_constructor {
                        println!("cargo:warning=    ✓ Discovered constructor: {}::{}", type_name, fn_name);
                        constructors.push(fn_name);
                    }
                }
            }
        }
    }
    
    constructors
}

/// Generate Lua binding code for an asset constructor
/// Returns the TokenStream and any discovered bitflags types
#[allow(dead_code)]
fn generate_asset_constructor_binding(
    spec: &AssetConstructorSpec,
    global_generated_names: &mut std::collections::HashSet<String>,
) -> Result<(proc_macro2::TokenStream, Vec<DiscoveredBitflags>), String> {
    // Find source file
    let source_path = find_source_file(&spec.type_spec)?;
    let source_code = fs::read_to_string(&source_path)
        .map_err(|e| format!("Failed to read source: {}", e))?;
    
    let syntax_tree: File = syn::parse_file(&source_code)
        .map_err(|e| format!("Failed to parse source: {}", e))?;
    
    // Parse constructor signature
    let params = parse_constructor_signature(&syntax_tree, &spec.type_spec.type_name, &spec.function_name)?;
    
    println!("cargo:warning=  Found {} parameters for {}::{}", 
        params.len(), spec.type_spec.type_name, spec.function_name);
    
    for param in &params {
        println!("cargo:warning=    - {}: {}", param.name, param.type_str);
    }
    
    // Discover all parameter types recursively
    let mut discovered_types = std::collections::HashMap::new();
    for param in &params {
        if let Err(e) = try_discover_param_type(&param.type_str, &mut discovered_types) {
            println!("cargo:warning=    ⚠ Could not discover type {}: {}", param.type_str, e);
        }
    }
    
    println!("cargo:warning=  Discovered {} types total", discovered_types.len());
    
    // Generate conversion functions for each discovered type
    // Pass global_generated_names to avoid duplicates across multiple constructors
    let type_converters = generate_type_converters(&discovered_types, global_generated_names);
    
    // Generate the main constructor function
    let fn_name_ident = syn::Ident::new(
        &format!("create_{}_from_lua", spec.type_spec.type_name.to_lowercase()),
        proc_macro2::Span::call_site()
    );
    
    // Normalize crate paths to use bevy re-exports
    let normalized_path = normalize_bevy_path(&spec.type_path)
        .ok_or_else(|| format!("Cannot normalize path: {} (internal module)", spec.type_path))?;
    let type_path: syn::Path = syn::parse_str(&normalized_path)
        .map_err(|e| format!("Invalid type path: {}", e))?;
    
    // Build parameter extraction code
    let param_extractions: Vec<_> = params.iter().map(|p| {
        let name = syn::Ident::new(&p.name, proc_macro2::Span::call_site());
        let type_str = &p.type_str;
        let name_str = &p.name;
        
        // Generate extraction based on type
        if is_primitive_type(type_str) {
            quote::quote! {
                let #name = table.get::<u32>(#name_str).unwrap_or_default();
            }
        } else if type_str.starts_with("&") {
            // Reference type like &[u8] - use "pixel" key from table as vec
            // Use default [0, 0, 0, 255] (RGBA black) if empty, since Image::new_fill panics on empty slice
            quote::quote! {
                let pixel_vec: Vec<u8> = table.get::<Option<Vec<u8>>>(#name_str)
                    .unwrap_or(None).unwrap_or_default();
                let #name = if pixel_vec.is_empty() { &[0u8, 0, 0, 255][..] } else { pixel_vec.as_slice() };
            }
        } else if let Some(type_def) = get_known_type_definition(type_str) {
            // Use generated converter based on type definition kind
            match type_def {
                TypeDefinition::Struct { name: type_name, .. } => {
                    let converter_fn = syn::Ident::new(
                        &format!("lua_to_{}", type_name.to_lowercase()),
                        proc_macro2::Span::call_site()
                    );
                    quote::quote! {
                        let #name = {
                            let sub_table: mlua::prelude::LuaTable = table.get(#name_str)?;
                            #converter_fn(&sub_table)?
                        };
                    }
                }
                TypeDefinition::Enum { name: type_name, .. } => {
                    let converter_fn = syn::Ident::new(
                        &format!("lua_to_{}", type_name.to_lowercase()),
                        proc_macro2::Span::call_site()
                    );
                    quote::quote! {
                        let #name = {
                            let value_str: String = table.get(#name_str)?;
                            #converter_fn(&value_str)?
                        };
                    }
                }
                TypeDefinition::Bitflags { name: type_name, .. } => {
                    let converter_fn = syn::Ident::new(
                        &format!("lua_to_{}", type_name.to_lowercase()),
                        proc_macro2::Span::call_site()
                    );
                    quote::quote! {
                        let #name = {
                            let value_str: String = table.get(#name_str)?;
                            #converter_fn(&value_str)
                        };
                    }
                }
                _ => {
                    quote::quote! {
                        let #name = Default::default();  // Unknown type
                    }
                }
            }
        } else {
            // Unknown type - use default
            quote::quote! {
                let #name = Default::default();  // TODO: Parse complex type
            }
        }
    }).collect();
    
    let param_names: Vec<_> = params.iter().map(|p| {
        syn::Ident::new(&p.name, proc_macro2::Span::call_site())
    }).collect();
    
    let constructor_fn = syn::Ident::new(&spec.function_name, proc_macro2::Span::call_site());
    
    // Post-construction bitflags handling (like texture_descriptor.usage) is now done
    // via runtime reflection in asset_loading.rs - no type-specific code here!
    
    // For Image assets, we also need to handle texture_usages which sets
    // image.texture_descriptor.usage (wgpu type, not Reflect-enabled)
    let is_image = spec.type_spec.type_name == "Image";
    
    let post_construction = if is_image {
        quote::quote! {
            // Apply texture_usages if provided (sets texture_descriptor.usage)
            if let Ok(texture_usages_str) = table.get::<String>("texture_usages") {
                use bevy::render::render_resource::TextureUsages;
                let mut usage = TextureUsages::empty();
                for flag in texture_usages_str.split('|') {
                    match flag.trim() {
                        "COPY_SRC" => usage |= TextureUsages::COPY_SRC,
                        "COPY_DST" => usage |= TextureUsages::COPY_DST,
                        "TEXTURE_BINDING" => usage |= TextureUsages::TEXTURE_BINDING,
                        "STORAGE_BINDING" => usage |= TextureUsages::STORAGE_BINDING,
                        "RENDER_ATTACHMENT" => usage |= TextureUsages::RENDER_ATTACHMENT,
                        _ => {}
                    }
                }
                result.texture_descriptor.usage = usage;
            }
        }
    } else {
        quote::quote! {}
    };
    
    let tokens = quote::quote! {
        #type_converters
        
        /// Auto-generated Lua binding for #type_path::#constructor_fn
        pub fn #fn_name_ident(
            table: &mlua::prelude::LuaTable,
        ) -> mlua::prelude::LuaResult<#type_path> {
            #(#param_extractions)*
            
            let mut result = #type_path::#constructor_fn(#(#param_names),*);
            #post_construction
            Ok(result)
        }
    };
    
    // Extract discovered bitflags for auto-registration
    let bitflags: Vec<DiscoveredBitflags> = discovered_types.iter()
        .filter_map(|(name, def)| {
            if let TypeDefinition::Bitflags { name: n, full_path, flags } = def {
                Some(DiscoveredBitflags {
                    name: n.clone(),
                    _full_path: full_path.clone(),
                    flags: flags.clone(),
                })
            } else {
                None
            }
        })
        .collect();
    
    Ok((tokens, bitflags))
}

/// Try to discover a parameter type - prefers source discovery, falls back to hardcoded
#[allow(dead_code)]
fn try_discover_param_type(
    type_str: &str,
    discovered: &mut std::collections::HashMap<String, TypeDefinition>,
) -> Result<(), String> {
    // Skip primitives and references
    if is_primitive_type(type_str) || type_str.starts_with("&") {
        return Ok(());
    }
    
    // Already discovered?
    if discovered.contains_key(type_str) {
        return Ok(());
    }
    
    // Resolve short name to full path using known types (this just provides the path, not definition)
    let resolved_path = if let Some(known_path) = get_known_bevy_type_path(type_str) {
        println!("cargo:warning=    → Resolved {} to {}", type_str, known_path);
        known_path.to_string()
    } else {
        type_str.to_string()
    };
    
    // Try to discover the type from source FIRST
    match discover_type_recursive(&resolved_path, discovered) {
        Ok(def) => {
            println!("cargo:warning=    ✓ Auto-discovered {} from source", type_str);
            // Store under original short name for param extraction code generation
            if type_str != resolved_path {
                discovered.insert(type_str.to_string(), def);
            }
            return Ok(());
        }
        Err(e) => {
            println!("cargo:warning=    ⚠ Source discovery failed for {}: {}", type_str, e);
        }
    }
    
    // Fall back to hardcoded type definitions (for complex types that can't be auto-discovered)
    if let Some(known_def) = get_known_type_definition(type_str) {
        println!("cargo:warning=    ✓ Using hardcoded fallback definition for {}", type_str);
        discovered.insert(type_str.to_string(), known_def);
        return Ok(());
    }
    
    // Add as Unknown type to allow partial code generation
    println!("cargo:warning=    ⚠ Type {} could not be discovered or hardcoded", type_str);
    discovered.insert(type_str.to_string(), TypeDefinition::Unknown { name: type_str.to_string() });
    Ok(())
}

/// Generate conversion functions for all discovered types
#[allow(dead_code)]
fn generate_type_converters(
    discovered: &std::collections::HashMap<String, TypeDefinition>,
    generated_names: &mut std::collections::HashSet<String>,
) -> proc_macro2::TokenStream {
    let mut converters: Vec<proc_macro2::TokenStream> = Vec::new();
    
    for (_type_path, type_def) in discovered {
        match type_def {
            TypeDefinition::Struct { name, fields, full_path } => {
                // Skip if already generated
                let fn_name_str = format!("lua_to_{}", name.to_lowercase());
                if generated_names.contains(&fn_name_str) {
                    continue;
                }
                generated_names.insert(fn_name_str.clone());
                
                let fn_name = syn::Ident::new(&fn_name_str, proc_macro2::Span::call_site());
                
                // Normalize path for user crate access
                let Some(normalized_path) = normalize_bevy_path(full_path) else {
                    continue;
                };
                
                // Parse the normalized path as a type
                let type_path_syn: syn::Path = match syn::parse_str(&normalized_path) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                
                // Generate field extractions
                let field_extracts: Vec<_> = fields.iter().map(|f| {
                    let field_name = syn::Ident::new(&f.name, proc_macro2::Span::call_site());
                    let field_str = &f.name;
                    quote::quote! {
                        #field_name: table.get(#field_str).unwrap_or_default()
                    }
                }).collect();
                
                let converter = quote::quote! {
                    /// Convert a Lua table to #type_path_syn
                    fn #fn_name(table: &mlua::prelude::LuaTable) -> mlua::prelude::LuaResult<#type_path_syn> {
                        Ok(#type_path_syn {
                            #(#field_extracts),*
                        })
                    }
                };
                
                converters.push(converter);
                println!("cargo:warning=    ✓ Generated converter for Struct {}", name);
            }
            TypeDefinition::Enum { name, variants, full_path } => {
                // Skip if already generated
                let fn_name_str = format!("lua_to_{}", name.to_lowercase());
                if generated_names.contains(&fn_name_str) {
                    continue;
                }
                generated_names.insert(fn_name_str.clone());
                
                let fn_name = syn::Ident::new(&fn_name_str, proc_macro2::Span::call_site());
                
                // Normalize path for user crate access
                let Some(normalized_path) = normalize_bevy_path(full_path) else {
                    continue;
                };
                
                // Parse the normalized path as a type
                let type_path_syn: syn::Path = match syn::parse_str(&normalized_path) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                
                // Filter out variants that have data (struct variants like Astc { ... })
                // These can't be constructed from a simple string
                let simple_variants: Vec<_> = variants.iter()
                    .filter(|v| !v.contains('(') && !v.contains('{'))
                    .collect();
                
                // Generate variant matches for simple (unit) variants only
                let variant_matches: Vec<_> = simple_variants.iter().map(|v| {
                    let v_str = v.as_str();
                    let v_ident = syn::Ident::new(v, proc_macro2::Span::call_site());
                    quote::quote! {
                        #v_str => Ok(#type_path_syn::#v_ident)
                    }
                }).collect();
                
                let converter = quote::quote! {
                    /// Convert a Lua string to #type_path_syn enum
                    fn #fn_name(value: &str) -> mlua::prelude::LuaResult<#type_path_syn> {
                        match value {
                            #(#variant_matches),*,
                            _ => Err(mlua::prelude::LuaError::RuntimeError(
                                format!("Unknown {} variant: {}", stringify!(#type_path_syn), value)
                            ))
                        }
                    }
                };
                
                converters.push(converter);
                println!("cargo:warning=    ✓ Generated converter for Enum {} ({} variants)", name, simple_variants.len());
            }
            TypeDefinition::Bitflags { name, flags, full_path } => {
                // Skip if already generated
                let fn_name_str = format!("lua_to_{}", name.to_lowercase());
                if generated_names.contains(&fn_name_str) {
                    continue;
                }
                generated_names.insert(fn_name_str.clone());
                
                let fn_name = syn::Ident::new(&fn_name_str, proc_macro2::Span::call_site());
                
                // Normalize path for user crate access
                let Some(normalized_path) = normalize_bevy_path(full_path) else {
                    continue;
                };
                
                // Parse the normalized path as a type
                let type_path_syn: syn::Path = match syn::parse_str(&normalized_path) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                
                // Generate flag matches
                let flag_matches: Vec<_> = flags.iter().map(|f| {
                    let f_str = f.as_str();
                    let f_ident = syn::Ident::new(f, proc_macro2::Span::call_site());
                    quote::quote! {
                        #f_str => result |= #type_path_syn::#f_ident
                    }
                }).collect();
                
                let converter = quote::quote! {
                    /// Convert a Lua pipe-separated string to #type_path_syn bitflags
                    fn #fn_name(value: &str) -> #type_path_syn {
                        let mut result = #type_path_syn::empty();
                        for flag in value.split('|') {
                            let flag = flag.trim();
                            match flag {
                                #(#flag_matches),*,
                                _ => {}
                            }
                        }
                        result
                    }
                };
                
                converters.push(converter);
                println!("cargo:warning=    ✓ Generated converter for Bitflags {}", name);
            }
            _ => {}
        }
    }
    
    quote::quote! {
        #(#converters)*
    }
}


