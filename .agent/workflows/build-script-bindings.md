---
description: Understanding and updating the bevy-lua-ecs build.rs binding generation system
---

# Build Script Binding Generation System

This document explains how `bevy-lua-ecs/build.rs` automatically generates Lua bindings by scanning Bevy source code and workspace metadata.

## Overview

The build script generates code that:
1. Registers entity wrapper components (newtypes around Entity)
2. Registers asset types with handle setters and cloners
3. Discovers and registers asset constructors for opaque types
4. Registers Handle<T> newtype wrappers
5. Registers bitflags for string-based parsing
6. Generates event registrations

## Key Concepts

### Parent Manifest Discovery

The build script runs in the context of `bevy-lua-ecs` but needs to read metadata from the **parent crate** (e.g., `Hello`):

```rust
fn find_parent_manifest(build_dir: &Path) -> Option<PathBuf>
```

- Searches upward from `OUT_DIR` for a `Cargo.toml` containing `[package.metadata.lua_resources]`
- Falls back to looking for `Hello/Cargo.toml` relative to workspace root

### Generated Output Files

1. **`Hello/src/auto_resource_bindings.rs`** - Written to parent crate
   - Contains `LuaBindingsPlugin` with all registrations
   - Must be included via `mod auto_resource_bindings;`

2. **`bevy-lua-ecs/target/.../auto_bindings.rs`** - Internal library use
   - Contains event registrations from `[package.metadata.lua_events]`

## Auto-Discovery Systems

### 1. Entity Wrapper Components

**Pattern detected:** `pub struct Foo(pub Entity)` with `#[derive(Component)]`

```rust
fn discover_entity_wrapper_components() -> Vec<DiscoveredEntityWrapper>
fn parse_entity_wrappers_from_source(...) // Uses syn to parse AST
```

**Traits looked for:**
- `syn::Item::Struct` with `Visibility::Public`
- `syn::Fields::Unnamed` with single field
- Field type containing `Entity`
- Attributes containing `#[derive(...Component...)]`

**Generated code:** Registers handlers via `ComponentRegistry` at runtime.

### 2. Asset Type Discovery

**Pattern detected:** Types implementing `Asset` trait

```rust
fn discover_asset_types() -> Vec<DiscoveredAssetType>
fn parse_asset_types_from_source(...) // Two-pass parsing
```

**Traits looked for:**
- `impl Asset for Type` blocks
- `#[derive(Asset)]` on structs
- Also detects `#[derive(Clone)]` or `impl Clone` for cloner generation

**Generated code:**
- `register_asset_types_from_registry()` - Runtime TypeRegistry lookup
- `register_asset_cloners()` - For types with Clone

### 3. Asset Constructor Discovery (NEW)

**Pattern detected:** `pub fn new_*()` methods returning `Self`

```rust
fn discover_asset_constructors(asset_types: &[DiscoveredAssetType]) -> Vec<DiscoveredAssetConstructor>
fn parse_constructors_from_source(...) // Scans impl blocks
fn parse_method_params(sig: &syn::Signature) -> Vec<ConstructorParam>
```

**Traits/patterns looked for:**
- `syn::Item::Impl` without trait (inherent impl)
- Methods with `Visibility::Public`
- Method name matching `new*`, `from_*`, or `default`
- Return type is `Self` or contains type name
- Parameters extracted with name and type

**Supported parameter types:**
- Primitives: `u32`, `i32`, `f32`, `f64`, `usize`, `bool`, `String`
- Enums: `TextureFormat`, `TextureDimension` (see enum handling below)

**Enum handling:** Since wgpu types like `TextureFormat` don't have Bevy reflection, we generate direct `match` statements:

```rust
// Generated code for TextureFormat parameter
match format_str.as_str() {
    "Rgba8UnormSrgb" => TextureFormat::Rgba8UnormSrgb,
    "Bgra8UnormSrgb" => TextureFormat::Bgra8UnormSrgb,
    // ... more variants
    _ => TextureFormat::Bgra8UnormSrgb,
}
```

**To add new enum types:** Update the match in `write_bindings_to_parent_crate`:
```rust
"MyEnumType" => {
    quote::quote! { ... generate match statement ... }
}
```

**Generated code:**
- `register_asset_constructor_bindings()` - Registers with `AssetRegistry`

### 4. Handle Newtype Wrappers

**Pattern detected:** `pub struct TypeName(pub Handle<Asset>)`

```rust
fn discover_handle_newtype_wrappers() -> Vec<DiscoveredHandleNewtype>
fn parse_handle_newtypes_from_source(...)
```

**Traits looked for:**
- `syn::Fields::Unnamed` with single field
- Field type containing `Handle<`
- Extracts inner asset name from `Handle<AssetName>`

**Generated code:**
- `register_auto_newtype_wrappers()` - Creates `NewtypeWrapperCreator` closures

### 5. Bitflags from Metadata

**Specified via Cargo.toml:**
```toml
[package.metadata.lua_bitflags]
TextureUsages = ["COPY_SRC", "COPY_DST", "TEXTURE_BINDING", "RENDER_ATTACHMENT"]
RenderAssetUsages = ["MAIN_WORLD", "RENDER_WORLD"]
```

```rust
fn get_bitflags_from_metadata(manifest: &toml::Value) -> Vec<BitflagsSpec>
```

**Generated code:**
- `register_auto_bitflags()` - Registers flag names and bit values
- Used by `set_basic_field()` to parse strings like `"FLAG_A|FLAG_B"`

## Code Generation Flow

1. `main()` calls `generate_bindings_for_manifest()`
2. Discovers all entity wrappers, asset types, constructors, newtypes, bitflags
3. Generates TokenStream for each registration
4. Calls `write_bindings_to_parent_crate()` which assembles:
   - `LuaBindingsPlugin` struct implementing `Plugin`
   - All helper functions (`register_auto_*`)
   - Asset type name constants

## Runtime Registration

The generated `LuaBindingsPlugin` adds systems:

```rust
app.add_systems(PostStartup, register_asset_constructors);
app.add_systems(Startup, setup_bitflags);
```

`register_asset_constructors` calls:
1. `register_entity_wrappers_from_registry()` - Uses TypeRegistry lookup
2. `register_asset_types_from_registry()` - Discovers handle setters
3. `register_auto_newtype_wrappers()` - Adds newtype creators
4. `register_asset_cloners()` - For Clone types
5. `register_asset_constructor_bindings()` - For opaque type constructors

## Integration with Asset Loading

In `bevy-lua-ecs/src/asset_loading.rs`:

```rust
// AssetRegistry stores constructors
pub type AssetConstructor = Box<dyn Fn(&mlua::Table) -> LuaResult<Box<dyn Reflect>> + Send + Sync>;

// process_pending_assets checks for constructors first
if let Some(result) = asset_registry.try_construct_asset(type_name, &table) {
    // Use constructor result
} else {
    // Fall back to ReflectDefault + field population
}
```

## Common Modifications

### Adding a new discoverable pattern

1. Create `Discovered*` struct in build.rs
2. Add scanning function following existing patterns
3. Add parsing function using `syn`
4. Generate registration code with `quote::quote!`
5. Add to `write_bindings_to_parent_crate()` output

### Adding support for a new enum parameter type

1. Update the match in constructor generation (lines ~2145-2185 in build.rs):
```rust
"MyEnumType" => {
    quote::quote! {
        let #param_ident = {
            use my_crate::MyEnumType;
            let s: String = table.get(#param_name).unwrap_or_else(|_| "Default".to_string());
            match s.as_str() {
                "Variant1" => MyEnumType::Variant1,
                "Variant2" => MyEnumType::Variant2,
                _ => MyEnumType::Default,
            }
        };
    }
}
```

### Adding constructor discovery for a new asset type

The system auto-discovers constructors for any asset type. Just ensure:
1. The type is discovered by `discover_asset_types()` (implements Asset)
2. Constructor is `pub fn new_*()` returning `Self`
3. All parameters are supported types

## Files Reference

- `bevy-lua-ecs/build.rs` - All discovery and generation logic
- `bevy-lua-ecs/src/asset_loading.rs` - AssetRegistry and runtime usage
- `bevy-lua-ecs/src/lib.rs` - Exports for generated code
- `Hello/src/auto_resource_bindings.rs` - Generated output (DO NOT EDIT)
