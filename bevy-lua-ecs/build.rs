// Build script for bevy-lua-ecs
// Automatically generates Lua bindings for resource types specified in dependent crates

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{File, Item, ItemImpl, ImplItem, Visibility, FnArg, ReturnType};
use quote::quote;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
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
    
    // Get parent crate's src directory
    let parent_src_dir = manifest_path.parent().unwrap().join("src");
    
    // Component bindings are registered manually by game developers
    // No auto-detection to keep bevy-lua-ecs generic
    let component_bindings = Vec::new();
    
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
    
    // Write generated code to parent crate's src directory
    write_bindings_to_parent_crate(all_bindings, all_constructor_bindings, component_bindings, &parent_src_dir);
    
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

#[derive(Debug)]
struct TypeSpec {
    full_path: String,
    crate_name: String,
    module_path: Vec<String>,
    type_name: String,
}

#[derive(Debug)]
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
            
            if crate_dir.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(&format!("{}-", spec.crate_name)))
                .unwrap_or(false)
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
    component_bindings: Vec<proc_macro2::TokenStream>,
    parent_src_dir: &Path
) {
    let generated_file = parent_src_dir.join("auto_resource_bindings.rs");
    
    let full_code = quote! {
        // Auto-generated Lua resource and component method bindings
        // Generated by bevy-lua-ecs build script
        
        pub fn register_auto_resource_bindings(registry: &bevy_lua_ecs::LuaResourceRegistry) {
            #(#method_bindings)*
        }
        
        pub fn register_auto_component_bindings(registry: &bevy_lua_ecs::LuaComponentRegistry) {
            #(#component_bindings)*
        }
        
        pub fn register_auto_constructors(lua: &mlua::Lua) -> Result<(), mlua::Error> {
            #(#constructor_bindings)*
            Ok(())
        }
    };
    
    fs::write(&generated_file, full_code.to_string())
        .expect("Failed to write auto_resource_bindings.rs");
    
    println!("cargo:warning=✓ Wrote bindings to {:?}", generated_file);
}

fn write_generated_bindings(bindings: Vec<proc_macro2::TokenStream>, event_types: Vec<String>) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let generated_file = out_dir.join("auto_bindings.rs");
    
    // Generate event registration code
    let event_registrations: Vec<_> = event_types.iter().map(|event_type| {
        // Replace bevy_lua_ecs:: with crate:: for internal types
        let adjusted_type = event_type.replace("bevy_lua_ecs::", "crate::");
        let type_path = syn::parse_str::<syn::Path>(&adjusted_type).unwrap();
        quote::quote! {
            app.register_type::<#type_path>();
            app.register_type::<bevy::prelude::Events<#type_path>>();
        }
    }).collect();
    
    let full_code = quote! {
        /// Auto-generated Lua resource bindings
        pub fn register_auto_bindings(registry: &crate::resource_lua_trait::LuaResourceRegistry) {
            #(#bindings)*
        }
        
        /// Auto-generated event registrations
        pub fn register_auto_events(app: &mut bevy::prelude::App) {
            #(#event_registrations)*
        }
    };
    
    fs::write(generated_file, full_code.to_string())
        .expect("Failed to write generated bindings");
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
            app.register_type::<bevy::prelude::Events<#type_path>>();
        }
    }).collect();
    
    let full_code = quote! {
        /// Auto-generated Lua resource bindings
        pub fn register_auto_bindings(_registry: &crate::resource_lua_trait::LuaResourceRegistry) {
            // No resource bindings generated
        }
        
        /// Auto-generated event registrations
        pub fn register_auto_events(app: &mut bevy::prelude::App) {
            #(#event_registrations)*
        }
    };
    
    fs::write(generated_file, full_code.to_string())
        .expect("Failed to write auto_bindings.rs");
}






