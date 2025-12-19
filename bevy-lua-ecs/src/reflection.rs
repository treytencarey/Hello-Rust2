use bevy::prelude::*;
#[cfg(feature = "auto-reflection")]
use bevy::reflect::{Reflect, StructInfo, TypeInfo, TypeRegistry};
use mlua::prelude::*;
use std::collections::HashMap;

/// Bundle definition for Lua spawning
pub struct BundleDefinition {
    pub name: String,
    pub spawn_fn: Box<dyn Fn(&LuaValue, &mut EntityCommands) -> LuaResult<()> + Send + Sync>,
}

/// Registry of available bundles for Lua
#[derive(Resource, Default)]
pub struct BundleRegistry {
    bundles: HashMap<String, BundleDefinition>,
}

impl BundleRegistry {
    /// Register a bundle with a spawn function
    pub fn register<F>(&mut self, name: impl Into<String>, spawn_fn: F)
    where
        F: Fn(&LuaValue, &mut EntityCommands) -> LuaResult<()> + Send + Sync + 'static,
    {
        let name = name.into();
        self.bundles.insert(
            name.clone(),
            BundleDefinition {
                name,
                spawn_fn: Box::new(spawn_fn),
            },
        );
    }

    /// Get a bundle definition by name
    pub fn get(&self, name: &str) -> Option<&BundleDefinition> {
        self.bundles.get(name)
    }

    /// Create registry from TypeRegistry using reflection (auto-reflection feature)
    #[cfg(feature = "auto-reflection")]
    pub fn from_type_registry(type_registry: &AppTypeRegistry) -> Self {
        let mut registry = Self::default();
        let type_registry = type_registry.read();

        for registration in type_registry.iter() {
            let type_info = registration.type_info();

            // Only process struct types
            if let TypeInfo::Struct(struct_info) = type_info {
                let type_name = struct_info.type_path_table().short_path();

                // Clone what we need before moving into closure
                let struct_info_clone = struct_info.clone();
                let type_path = registration.type_path().to_string();

                registry.register(
                    type_name,
                    move |data: &LuaValue, entity: &mut EntityCommands| {
                        spawn_from_reflection(data, entity, &struct_info_clone, &type_path)
                    },
                );
            }
        }

        registry
    }
}

/// Spawn entity from reflected type information
#[cfg(feature = "auto-reflection")]
fn spawn_from_reflection(
    data: &LuaValue,
    entity: &mut EntityCommands,
    struct_info: &StructInfo,
    type_path: &str,
) -> LuaResult<()> {
    debug!("Spawning {} via reflection", type_path);

    // For now, we'll use a simplified approach
    // Full implementation would use DynamicStruct and FromReflect

    // This is a placeholder - actual implementation would:
    // 1. Create DynamicStruct from struct_info
    // 2. Populate fields from Lua table
    // 3. Use FromReflect to convert to concrete type
    // 4. Insert into entity

    Err(LuaError::RuntimeError(
        "Full reflection not yet implemented - use manual LuaSpawnable".to_string(),
    ))
}

/// Trait for types that can be spawned from Lua
pub trait LuaSpawnable {
    fn from_lua(data: &LuaTable, entity: &mut EntityCommands) -> LuaResult<()>;
}

/// Macro to register bundles automatically
#[macro_export]
macro_rules! register_lua_bundles {
    ($registry:expr, $($bundle:ty),* $(,)?) => {
        $(
            {
                let bundle_name = std::any::type_name::<$bundle>()
                    .split("::")
                    .last()
                    .unwrap_or(std::any::type_name::<$bundle>());

                $registry.register(bundle_name, |data: &mlua::Table, entity: &mut bevy::ecs::system::EntityCommands| {
                    <$bundle as $crate::reflection::LuaSpawnable>::from_lua(data, entity)
                });
            }
        )*
    };
}

/// Derive macro helper - converts Lua table to reflected value
#[cfg(feature = "auto-reflection")]
pub fn lua_table_to_reflect(
    table: &LuaTable,
    field_name: &str,
    type_info: &TypeInfo,
    type_registry: &TypeRegistry,
) -> LuaResult<Box<dyn Reflect>> {
    match type_info {
        TypeInfo::Struct(struct_info) => {
            // Handle common Bevy types
            match struct_info.type_path() {
                "bevy_color::srgba::Srgba" | "bevy_color::Color" => {
                    let r: f32 = table.get("r").unwrap_or(1.0);
                    let g: f32 = table.get("g").unwrap_or(1.0);
                    let b: f32 = table.get("b").unwrap_or(1.0);
                    let a: f32 = table.get("a").unwrap_or(1.0);
                    Ok(Box::new(Color::srgba(r, g, b, a)))
                }
                "glam::Vec2" => {
                    let x: f32 = table.get("x").unwrap_or(0.0);
                    let y: f32 = table.get("y").unwrap_or(0.0);
                    Ok(Box::new(Vec2::new(x, y)))
                }
                "glam::Vec3" => {
                    let x: f32 = table.get("x").unwrap_or(0.0);
                    let y: f32 = table.get("y").unwrap_or(0.0);
                    let z: f32 = table.get("z").unwrap_or(0.0);
                    Ok(Box::new(Vec3::new(x, y, z)))
                }
                _ => Err(LuaError::RuntimeError(format!(
                    "Unsupported struct type for reflection: {}",
                    struct_info.type_path()
                ))),
            }
        }
        _ => Err(LuaError::RuntimeError(format!(
            "Unsupported type info for field {}: {:?}",
            field_name, type_info
        ))),
    }
}
