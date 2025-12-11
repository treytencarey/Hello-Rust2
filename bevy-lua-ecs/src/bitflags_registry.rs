//! Bitflags Registry - runtime registration of bitflags types for generic handling
//! 
//! This module provides a registry that maps bitflags type paths to their variant
//! name->value mappings. Build scripts generate registration code that populates
//! this registry at startup, allowing asset_loading.rs to apply bitflags generically.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use bevy::prelude::*;

/// A registered bitflags type with its variants
#[derive(Clone, Debug)]
pub struct BitflagsEntry {
    /// Full type path (e.g., "wgpu_types::TextureUsages")
    pub type_path: String,
    /// Variant name -> bit value mapping
    pub variants: HashMap<String, u32>,
}

/// Runtime registry of bitflags types
/// Populated by auto-generated code from dependent crates
#[derive(Resource, Clone, Default)]
pub struct BitflagsRegistry {
    entries: Arc<RwLock<HashMap<String, BitflagsEntry>>>,
}

impl BitflagsRegistry {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a bitflags type with its variants
    /// Called by generated code from build.rs
    pub fn register(&self, type_path: impl Into<String>, variants: &[(&str, u32)]) {
        let type_path = type_path.into();
        let entry = BitflagsEntry {
            type_path: type_path.clone(),
            variants: variants.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        };
        self.entries.write().unwrap().insert(type_path, entry);
    }
    
    /// Look up a bitflags type by checking if any registered type path is contained
    /// in the given type_path string (partial matching)
    pub fn find_by_type_path(&self, type_path: &str) -> Option<BitflagsEntry> {
        let entries = self.entries.read().unwrap();
        for (registered_path, entry) in entries.iter() {
            if type_path.contains(registered_path) || type_path.contains(&entry.type_path) {
                return Some(entry.clone());
            }
        }
        None
    }
    
    /// Parse flag names to a u32 value using the registered variants
    pub fn parse_flags(&self, type_path: &str, flag_names: &[&str]) -> Option<u32> {
        let entry = self.find_by_type_path(type_path)?;
        let mut value: u32 = 0;
        for name in flag_names {
            if let Some(v) = entry.variants.get(*name) {
                value |= v;
            }
        }
        Some(value)
    }
}
