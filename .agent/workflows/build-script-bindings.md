# Build Script Auto-Generation (build.rs)

**Purpose:** Scan Bevy source + metadata → generate Lua bindings in `Hello/src/auto_resource_bindings.rs`

**Generated Registrations:**
1. Entity wrapper components (newtypes around Entity)
2. Asset types (handle setters/cloners)
3. Asset constructors (opaque types)
4. Handle<T> newtype wrappers
5. Bitflags (string parsing)
6. Events (from metadata)

**Output:** `LuaBindingsPlugin` struct implementing `Plugin`, called at runtime

## Discovery Systems

**1. Entity Wrappers:** `pub struct Foo(pub Entity)` with `#[derive(Component)]`

**2. Asset Types:** `impl Asset for T` or `#[derive(Asset)]`, detects Clone for cloners

**3. Asset Constructors:** `pub fn new_*/from_*/default()` returning Self
- Params: u32, i32, f32, f64, usize, bool, String, TextureFormat, TextureDimension
- Enum params → generated match statements (add new enums in write_bindings_to_parent_crate)
- Registered in `AssetRegistry` for `create_asset()` calls

**4. Handle Newtype Wrappers:** `pub struct T(Handle<Asset>)` or `pub struct T { handle: Handle<Asset>, ... }`
- Generates `DISCOVERED_NEWTYPE_WRAPPERS` array for runtime lookup
- Creates `NewtypeWrapperCreator` closures via TypeRegistry

**5. Bitflags:** From `Cargo.toml` metadata → string parsing (`"FLAG_A|FLAG_B"`)
```toml
[package.metadata.lua_bitflags]
TextureUsages = ["COPY_SRC", "TEXTURE_BINDING", ...]
```

## Runtime Flow

1. `main()` → `generate_bindings_for_manifest()` → discovers all patterns
2. `write_bindings_to_parent_crate()` → generates `LuaBindingsPlugin`
3. Plugin adds systems: `PostStartup` (register_asset_constructors), `Startup` (setup_bitflags)
4. `process_pending_assets` (asset_loading.rs) checks constructors first, falls back to ReflectDefault

## Extending

**New enum param type:** Update match in write_bindings_to_parent_crate (~line 2145):
```rust
"MyEnum" => quote::quote! { match s.as_str() { "Var1" => MyEnum::Var1, ... } }
```

**New discovery pattern:** Create `Discovered*` struct → scanning fn → parsing fn (syn) → quote::quote! → add to output

## Runtime Enum/Newtype Handling

**Enum Variant Detection** (`components.rs::get_newtype_from_enum_variant`):
1. Lua passes `{Image = asset_id}` → lookup VariantInfo for "Image"
2. Check if variant contains newtype (not raw Handle<T>)
3. Extract newtype's inner Handle<T> type via TypeInfo
4. Handles both Tuple and Struct variants

**Newtype Construction** (`asset_loading.rs::try_wrap_in_newtype_with_reflection`):
- TupleStruct: `DynamicTupleStruct` with handle at index 0
- Struct: `DynamicStruct` inserting handle + ReflectDefault/FromReflect for other fields
- Returns `from_reflect(&dynamic_value)` → concrete type

**DynamicEnum Application** (CRITICAL for Handle preservation):
```rust
// Must set represented type + use from_reflect
dynamic_enum.set_represented_type(Some(type_info));
let concrete = from_reflect.from_reflect(&dynamic_enum);
field.apply(concrete.as_ref());  // NOT field.apply(&dynamic_enum)
```

**Debug Markers:** `[ENUM_NEWTYPE]`, `[NEWTYPE_WRAP_REFLECT]`, `[ENUM_SET]`, `[HANDLE_CREATE]`

**Common Issues:**
- Variant not found → spelling/case
- Newtype not detected → verify `#[derive(Reflect)]` with FromReflect
- Handle not preserved → must use from_reflect, not direct apply()

**Files:** build.rs (discovery), asset_loading.rs (construction), components.rs (application)