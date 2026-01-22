//! Lua-defined resource storage
//!
//! Provides storage for Lua tables that act as resources, scoped by instance_id.
//! Resources are idempotent: calling define_resource with the same name returns
//! the existing table rather than creating a new one.

use bevy::prelude::*;
use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Registry for Lua-defined table resources
/// Key: (resource_name, instance_id) -> LuaRegistryKey pointing to live Lua table
#[derive(Resource, Default, Clone)]
pub struct LuaTableResourceRegistry {
    resources: Arc<Mutex<HashMap<(String, u64), Arc<LuaRegistryKey>>>>,
}

impl LuaTableResourceRegistry {
    /// Define a resource (idempotent)
    /// Returns the existing table if already defined, or creates new from default
    ///
    /// # Arguments
    /// * `lua` - The Lua state
    /// * `name` - Resource name
    /// * `instance_id` - Instance ID for scoping (from __INSTANCE_ID__)
    /// * `default_value` - Default table to use if resource doesn't exist
    ///
    /// When resource already exists, performs deep merge:
    /// - New keys from default are added to existing table
    /// - Existing values are preserved (not overwritten)
    /// - Nested tables are recursively merged
    pub fn define_resource(
        &self,
        lua: &Lua,
        name: &str,
        instance_id: u64,
        default_value: LuaValue,
    ) -> LuaResult<LuaValue> {
        let key = (name.to_string(), instance_id);

        // Check if already exists
        {
            let resources = self.resources.lock().unwrap();
            if let Some(registry_key) = resources.get(&key) {
                // Get existing table
                let existing: LuaValue = lua.registry_value(&**registry_key)?;
                
                // Deep merge default into existing (adds new keys, preserves existing)
                if let (LuaValue::Table(existing_table), LuaValue::Table(default_table)) = 
                    (&existing, &default_value) 
                {
                    Self::deep_merge_tables(existing_table, default_table)?;
                    debug!(
                        "[LUA_RESOURCE] Deep merged defaults into existing resource '{}' for instance {}",
                        name, instance_id
                    );
                } else {
                    debug!(
                        "[LUA_RESOURCE] Returning existing resource '{}' for instance {} (no merge - not tables)",
                        name, instance_id
                    );
                }
                
                return Ok(existing);
            }
        }

        // Create new resource from default
        let registry_key = lua.create_registry_value(default_value.clone())?;
        let arc_key = Arc::new(registry_key);

        {
            let mut resources = self.resources.lock().unwrap();
            resources.insert(key, arc_key.clone());
        }

        debug!(
            "[LUA_RESOURCE] Created new resource '{}' for instance {}",
            name, instance_id
        );

        // Return the value we just stored
        let value: LuaValue = lua.registry_value(&*arc_key)?;
        Ok(value)
    }

    /// Deep merge default table into existing table
    /// - New keys from default are added to existing
    /// - Existing values are preserved (not overwritten)
    /// - If both values are tables, recursively merge
    fn deep_merge_tables(existing: &LuaTable, default: &LuaTable) -> LuaResult<()> {
        for pair in default.pairs::<LuaValue, LuaValue>() {
            let (key, default_val) = pair?;
            
            let existing_val: LuaValue = existing.get(key.clone())?;
            
            match existing_val {
                LuaValue::Nil => {
                    // Key doesn't exist in existing, add it
                    existing.set(key, default_val)?;
                }
                LuaValue::Table(ref existing_nested) => {
                    // Both are tables, recursively merge
                    if let LuaValue::Table(ref default_nested) = default_val {
                        Self::deep_merge_tables(existing_nested, default_nested)?;
                    }
                    // If default is not a table but existing is, keep existing (no change)
                }
                _ => {
                    // Existing value is not nil and not a table, keep it (no change)
                }
            }
        }
        Ok(())
    }

    /// Get a resource by name and instance_id
    /// Returns None if resource doesn't exist
    pub fn get_resource(
        &self,
        lua: &Lua,
        name: &str,
        instance_id: u64,
    ) -> LuaResult<Option<LuaValue>> {
        let key = (name.to_string(), instance_id);

        let resources = self.resources.lock().unwrap();
        if let Some(registry_key) = resources.get(&key) {
            let value: LuaValue = lua.registry_value(&**registry_key)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Remove a resource by name and instance_id
    /// Returns true if resource existed and was removed
    pub fn remove_resource(&self, lua: &Lua, name: &str, instance_id: u64) -> bool {
        let key = (name.to_string(), instance_id);

        let mut resources = self.resources.lock().unwrap();
        if let Some(registry_key) = resources.remove(&key) {
            // Clean up the registry key
            if let Ok(key) = Arc::try_unwrap(registry_key) {
                let _ = lua.remove_registry_value(key);
            }
            debug!(
                "[LUA_RESOURCE] Removed resource '{}' for instance {}",
                name, instance_id
            );
            true
        } else {
            false
        }
    }

    /// Remove all resources for a specific instance
    /// Called during script cleanup/hot-reload
    pub fn clear_instance_resources(&self, lua: &Lua, instance_id: u64) {
        let mut resources = self.resources.lock().unwrap();
        let keys_to_remove: Vec<_> = resources
            .keys()
            .filter(|(_, id)| *id == instance_id)
            .cloned()
            .collect();

        for key in keys_to_remove {
            if let Some(registry_key) = resources.remove(&key) {
                if let Ok(key) = Arc::try_unwrap(registry_key) {
                    let _ = lua.remove_registry_value(key);
                }
            }
        }

        debug!(
            "[LUA_RESOURCE] Cleared all resources for instance {}",
            instance_id
        );
    }

    /// Get all resource names for a specific instance
    pub fn get_instance_resource_names(&self, instance_id: u64) -> Vec<String> {
        let resources = self.resources.lock().unwrap();
        resources
            .keys()
            .filter(|(_, id)| *id == instance_id)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_resource_idempotent() {
        let lua = Lua::new();
        let registry = LuaTableResourceRegistry::default();

        // Define a resource
        let default1 = lua.create_table().unwrap();
        default1.set("value", 1).unwrap();
        let result1 = registry
            .define_resource(&lua, "TestResource", 1, LuaValue::Table(default1))
            .unwrap();

        // Define same resource again with different default
        let default2 = lua.create_table().unwrap();
        default2.set("value", 999).unwrap();
        let result2 = registry
            .define_resource(&lua, "TestResource", 1, LuaValue::Table(default2))
            .unwrap();

        // Should return same table (first one)
        if let (LuaValue::Table(t1), LuaValue::Table(t2)) = (result1, result2) {
            assert_eq!(t1.get::<i32>("value").unwrap(), 1);
            assert_eq!(t2.get::<i32>("value").unwrap(), 1);

            // Modify via t1, should be visible in t2
            t1.set("value", 42).unwrap();
            assert_eq!(t2.get::<i32>("value").unwrap(), 42);
        } else {
            panic!("Expected tables");
        }
    }

    #[test]
    fn test_instance_scoped() {
        let lua = Lua::new();
        let registry = LuaTableResourceRegistry::default();

        // Define resource for instance 1
        let default1 = lua.create_table().unwrap();
        default1.set("instance", 1).unwrap();
        let result1 = registry
            .define_resource(&lua, "TestResource", 1, LuaValue::Table(default1))
            .unwrap();

        // Define same resource name for instance 2
        let default2 = lua.create_table().unwrap();
        default2.set("instance", 2).unwrap();
        let result2 = registry
            .define_resource(&lua, "TestResource", 2, LuaValue::Table(default2))
            .unwrap();

        // Should be different tables
        if let (LuaValue::Table(t1), LuaValue::Table(t2)) = (result1, result2) {
            assert_eq!(t1.get::<i32>("instance").unwrap(), 1);
            assert_eq!(t2.get::<i32>("instance").unwrap(), 2);
        } else {
            panic!("Expected tables");
        }
    }

    #[test]
    fn test_remove_resource() {
        let lua = Lua::new();
        let registry = LuaTableResourceRegistry::default();

        // Define a resource
        let default = lua.create_table().unwrap();
        registry
            .define_resource(&lua, "TestResource", 1, LuaValue::Table(default))
            .unwrap();

        // Verify it exists
        assert!(registry.get_resource(&lua, "TestResource", 1).unwrap().is_some());

        // Remove it
        assert!(registry.remove_resource(&lua, "TestResource", 1));

        // Verify it's gone
        assert!(registry.get_resource(&lua, "TestResource", 1).unwrap().is_none());

        // Remove again should return false
        assert!(!registry.remove_resource(&lua, "TestResource", 1));
    }

    #[test]
    fn test_deep_merge_adds_new_keys() {
        let lua = Lua::new();
        let registry = LuaTableResourceRegistry::default();

        // Define resource with initial keys
        let default1 = lua.create_table().unwrap();
        default1.set("foo", 1).unwrap();
        let result1 = registry
            .define_resource(&lua, "TestResource", 1, LuaValue::Table(default1))
            .unwrap();

        // Define again with additional key
        let default2 = lua.create_table().unwrap();
        default2.set("foo", 1).unwrap();
        default2.set("bar", 2).unwrap(); // New key
        let result2 = registry
            .define_resource(&lua, "TestResource", 1, LuaValue::Table(default2))
            .unwrap();

        // Should have both keys now
        if let LuaValue::Table(t) = result2 {
            assert_eq!(t.get::<i32>("foo").unwrap(), 1);
            assert_eq!(t.get::<i32>("bar").unwrap(), 2); // New key added
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_deep_merge_preserves_existing_values() {
        let lua = Lua::new();
        let registry = LuaTableResourceRegistry::default();

        // Define resource with initial value
        let default1 = lua.create_table().unwrap();
        default1.set("counter", 0).unwrap();
        registry
            .define_resource(&lua, "TestResource", 1, LuaValue::Table(default1))
            .unwrap();

        // Simulate runtime modification
        let state = registry.get_resource(&lua, "TestResource", 1).unwrap().unwrap();
        if let LuaValue::Table(t) = state {
            t.set("counter", 42).unwrap(); // Runtime sets it to 42
        }

        // Define again with different default
        let default2 = lua.create_table().unwrap();
        default2.set("counter", 0).unwrap(); // Default is still 0
        default2.set("new_field", "hello").unwrap();
        let result = registry
            .define_resource(&lua, "TestResource", 1, LuaValue::Table(default2))
            .unwrap();

        // Runtime value should be preserved, NOT overwritten
        if let LuaValue::Table(t) = result {
            assert_eq!(t.get::<i32>("counter").unwrap(), 42); // Still 42!
            assert_eq!(t.get::<String>("new_field").unwrap(), "hello"); // New field added
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_deep_merge_nested_tables() {
        let lua = Lua::new();
        let registry = LuaTableResourceRegistry::default();

        // Define resource with nested table
        let inner1 = lua.create_table().unwrap();
        inner1.set("x", 10).unwrap();
        let default1 = lua.create_table().unwrap();
        default1.set("nested", inner1).unwrap();
        registry
            .define_resource(&lua, "TestResource", 1, LuaValue::Table(default1))
            .unwrap();

        // Modify nested value at runtime
        let state = registry.get_resource(&lua, "TestResource", 1).unwrap().unwrap();
        if let LuaValue::Table(t) = state {
            let nested: LuaTable = t.get("nested").unwrap();
            nested.set("x", 100).unwrap(); // Runtime changes x to 100
        }

        // Define again with new nested key
        let inner2 = lua.create_table().unwrap();
        inner2.set("x", 10).unwrap(); // Default x
        inner2.set("y", 20).unwrap(); // New key y
        let default2 = lua.create_table().unwrap();
        default2.set("nested", inner2).unwrap();
        default2.set("top_level_new", true).unwrap();
        let result = registry
            .define_resource(&lua, "TestResource", 1, LuaValue::Table(default2))
            .unwrap();

        // Verify nested merge
        if let LuaValue::Table(t) = result {
            let nested: LuaTable = t.get("nested").unwrap();
            assert_eq!(nested.get::<i32>("x").unwrap(), 100); // Runtime value preserved
            assert_eq!(nested.get::<i32>("y").unwrap(), 20);  // New key added
            assert!(t.get::<bool>("top_level_new").unwrap()); // Top-level new key
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_deep_merge_runtime_value_not_lost_when_default_changes() {
        let lua = Lua::new();
        let registry = LuaTableResourceRegistry::default();

        // Initial definition
        let default1 = lua.create_table().unwrap();
        default1.set("timeout", 10).unwrap();
        default1.set("name", "initial").unwrap();
        registry
            .define_resource(&lua, "Config", 1, LuaValue::Table(default1))
            .unwrap();

        // User changes 'name' at runtime
        let state = registry.get_resource(&lua, "Config", 1).unwrap().unwrap();
        if let LuaValue::Table(t) = state {
            t.set("name", "user_chosen_name").unwrap();
        }

        // Hot-reload: developer changes default 'name' to "updated_default"
        let default2 = lua.create_table().unwrap();
        default2.set("timeout", 10).unwrap();
        default2.set("name", "updated_default").unwrap(); // Changed default
        default2.set("new_option", true).unwrap(); // Also added new field
        let result = registry
            .define_resource(&lua, "Config", 1, LuaValue::Table(default2))
            .unwrap();

        // Runtime-set 'name' should NOT be overwritten by new default
        if let LuaValue::Table(t) = result {
            assert_eq!(t.get::<String>("name").unwrap(), "user_chosen_name"); // Preserved!
            assert_eq!(t.get::<i32>("timeout").unwrap(), 10);
            assert!(t.get::<bool>("new_option").unwrap()); // New field added
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_reset_resource_then_define_gets_fresh_state() {
        let lua = Lua::new();
        let registry = LuaTableResourceRegistry::default();

        // Define and modify
        let default1 = lua.create_table().unwrap();
        default1.set("value", 0).unwrap();
        registry
            .define_resource(&lua, "TestResource", 1, LuaValue::Table(default1))
            .unwrap();

        let state = registry.get_resource(&lua, "TestResource", 1).unwrap().unwrap();
        if let LuaValue::Table(t) = state {
            t.set("value", 999).unwrap();
        }

        // Reset (remove)
        registry.remove_resource(&lua, "TestResource", 1);

        // Define again - should get fresh default
        let default2 = lua.create_table().unwrap();
        default2.set("value", 0).unwrap();
        let result = registry
            .define_resource(&lua, "TestResource", 1, LuaValue::Table(default2))
            .unwrap();

        // Should be fresh default, NOT the old runtime value
        if let LuaValue::Table(t) = result {
            assert_eq!(t.get::<i32>("value").unwrap(), 0); // Fresh!
        } else {
            panic!("Expected table");
        }
    }
}
