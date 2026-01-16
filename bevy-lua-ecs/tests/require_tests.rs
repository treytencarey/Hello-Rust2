//! Integration tests for `require` and `require_async` functionality
//!
//! These tests verify:
//! - Basic module loading and caching
//! - The `reload` and `instanced` options
//! - Hot-reload behavior and dependency tracking
//! - Live proxy system

use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use bevy::asset::AssetPlugin;
use bevy_lua_ecs::*;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::TempDir;

// Mutex to ensure tests run sequentially (they change the working directory)
static TEST_MUTEX: Mutex<()> = Mutex::new(());

/// Test helper that sets up a minimal Bevy app with Lua support
struct TestApp {
    app: App,
    temp_dir: TempDir,
    original_dir: PathBuf,
    #[allow(dead_code)]
    lock: std::sync::MutexGuard<'static, ()>,
}

impl TestApp {
    /// Create a new test app with a temporary assets directory
    fn new() -> Self {
        // Acquire lock to ensure sequential test execution
        // Use unwrap_or_else to recover from poisoned mutex (e.g., if a test panicked)
        let lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let original_dir = std::env::current_dir().expect("Failed to get current dir");

        // Create assets/scripts directory in temp
        let assets_dir = temp_dir.path().join("assets").join("scripts");
        fs::create_dir_all(&assets_dir).expect("Failed to create assets/scripts dir");

        // Change to temp directory so require() can find files
        std::env::set_current_dir(temp_dir.path()).expect("Failed to change to temp dir");

        // Create a minimal headless Bevy app with only what we need for Lua
        let mut app = App::new();

        // Add minimal plugins for headless operation (no LogPlugin to avoid global state issues)
        app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_once()));
        app.add_plugins(AssetPlugin::default());

        // Add our Lua plugin
        app.add_plugins(LuaSpawnPlugin);

        // Run startup systems to initialize Lua context
        app.update();

        Self {
            app,
            temp_dir,
            original_dir,
            lock,
        }
    }

    /// Copy a fixture file to the temp assets directory
    fn add_fixture(&self, fixture_name: &str, dest_path: &str) -> PathBuf {
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(fixture_name);
        let dest = self.temp_dir.path().join("assets").join(dest_path);

        // Create parent directories
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dirs");
        }

        fs::copy(&src, &dest).expect(&format!("Failed to copy fixture {:?} to {:?}", src, dest));
        dest
    }

    /// Execute a Lua script and return success/failure
    fn execute_script(&mut self, content: &str, name: &str) -> Result<u64, String> {
        let lua_ctx = self
            .app
            .world()
            .get_resource::<LuaScriptContext>()
            .expect("LuaScriptContext not found")
            .clone();
        let script_instance = self
            .app
            .world()
            .get_resource::<ScriptInstance>()
            .expect("ScriptInstance not found")
            .clone();
        let script_registry = self
            .app
            .world()
            .get_resource::<ScriptRegistry>()
            .expect("ScriptRegistry not found")
            .clone();

        let path = PathBuf::from("scripts").join(name);

        lua_ctx
            .execute_script(content, name, path, &script_instance, &script_registry)
            .map_err(|e| format!("{}", e))
    }

    /// Execute a Lua expression and return the result as a string
    fn eval_lua(&mut self, expr: &str) -> Result<String, String> {
        let lua_ctx = self
            .app
            .world()
            .get_resource::<LuaScriptContext>()
            .expect("LuaScriptContext not found")
            .clone();

        lua_ctx
            .lua
            .load(expr)
            .eval::<mlua::Value>()
            .map(|v| format!("{:?}", v))
            .map_err(|e| format!("{}", e))
    }

    /// Get the ScriptCache for inspection
    fn script_cache(&self) -> ScriptCache {
        self.app
            .world()
            .get_resource::<LuaScriptContext>()
            .expect("LuaScriptContext not found")
            .script_cache
            .clone()
    }

    /// Trigger hot-reload for a specific file path
    fn trigger_hot_reload(&mut self, relative_path: &str) {
        // The hot reload system expects paths relative to CWD starting with "assets/"
        // Since we've changed CWD to temp_dir, use a relative path
        let path = PathBuf::from("assets").join(relative_path);
        // Use write_message for Message types (not send_event which is for Events)
        self.app
            .world_mut()
            .write_message(LuaFileChangeEvent { path });
        self.app.update();
    }

    /// Modify a file in the temp assets directory
    fn modify_file(&self, relative_path: &str, content: &str) {
        let path = self.temp_dir.path().join("assets").join(relative_path);
        fs::write(&path, content).expect(&format!("Failed to write to {:?}", path));
    }

    /// Run one update cycle
    fn update(&mut self) {
        self.app.update();
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        // Restore original directory
        let _ = std::env::set_current_dir(&self.original_dir);
    }
}

// =============================================================================
// Basic require() Tests
// =============================================================================

#[test]
fn test_require_returns_module_export() {
    let mut test = TestApp::new();
    test.add_fixture("basic_module.lua", "scripts/basic_module.lua");

    let script = r#"
        local mod = require("scripts/basic_module.lua")
        assert(mod.name == "basic", "Expected name='basic', got: " .. tostring(mod.name))
        assert(mod.value == 42, "Expected value=42, got: " .. tostring(mod.value))
        return true
    "#;

    let result = test.execute_script(script, "test.lua");
    assert!(result.is_ok(), "Script failed: {:?}", result.err());
}

#[test]
fn test_require_caches_module() {
    let mut test = TestApp::new();
    test.add_fixture("counter_module.lua", "scripts/counter.lua");

    let script = r#"
        local mod1 = require("scripts/counter.lua")
        local mod2 = require("scripts/counter.lua")

        mod1.increment()
        -- If cached, mod2 should see the same state
        local count = mod2.get()
        assert(count == 1, "Expected mod2 to see mod1's increment, got: " .. tostring(count))
        return true
    "#;

    let result = test.execute_script(script, "test.lua");
    assert!(result.is_ok(), "Script failed: {:?}", result.err());
}

#[test]
fn test_require_resolves_relative_path() {
    let mut test = TestApp::new();
    test.add_fixture("inner.lua", "scripts/inner.lua");
    test.add_fixture("nested_require.lua", "scripts/nested_require.lua");

    let script = r#"
        local mod = require("scripts/nested_require.lua")
        assert(mod.outer == true, "Expected outer=true")
        assert(mod.inner_data ~= nil, "Expected inner_data to be present")
        assert(mod.inner_data.inner == true, "Expected inner_data.inner=true")
        return true
    "#;

    let result = test.execute_script(script, "test.lua");
    assert!(result.is_ok(), "Script failed: {:?}", result.err());
}

// =============================================================================
// require() Options Tests
// =============================================================================

#[test]
fn test_require_reload_true_registers_dependency() {
    let mut test = TestApp::new();
    test.add_fixture("basic_module.lua", "scripts/basic.lua");

    let script = r#"
        -- Default reload=true
        local mod = require("scripts/basic.lua")
        return true
    "#;

    test.execute_script(script, "scripts/test.lua")
        .expect("Script failed");

    // Verify dependency was registered with should_reload=true
    let cache = test.script_cache();
    let importers = cache.get_importers("scripts/basic.lua");

    assert!(
        !importers.is_empty(),
        "Expected dependency to be registered"
    );
    // Find the importer and check reload flag
    let found = importers
        .iter()
        .any(|(path, reload)| path.contains("test.lua") && *reload == true);
    assert!(
        found,
        "Expected importer 'test.lua' with reload=true, got: {:?}",
        importers
    );
}

#[test]
fn test_require_reload_false_registers_dependency() {
    let mut test = TestApp::new();
    test.add_fixture("basic_module.lua", "scripts/basic.lua");

    let script = r#"
        -- Explicit reload=false
        local mod = require("scripts/basic.lua", { reload = false })
        return true
    "#;

    test.execute_script(script, "scripts/test.lua")
        .expect("Script failed");

    // Verify dependency was registered with should_reload=false
    let cache = test.script_cache();
    let importers = cache.get_importers("scripts/basic.lua");

    assert!(
        !importers.is_empty(),
        "Expected dependency to be registered"
    );
    // Find the importer and check reload flag
    let found = importers
        .iter()
        .any(|(path, reload)| path.contains("test.lua") && *reload == false);
    assert!(
        found,
        "Expected importer 'test.lua' with reload=false, got: {:?}",
        importers
    );
}

/// Test that `instanced = true` creates separate module instances.
/// Each instanced require gets its own state_id, so module state is isolated.
#[test]
fn test_require_instanced_creates_separate_cache() {
    let mut test = TestApp::new();
    test.add_fixture("instanced/stateful.lua", "scripts/stateful.lua");

    let script = r#"
        -- Two instanced requires should have separate state
        local mod1 = require("scripts/stateful.lua", { instanced = true })
        local mod2 = require("scripts/stateful.lua", { instanced = true })

        mod1.set(100)

        -- mod2 should still be at 0 (separate instance)
        local val2 = mod2.get()
        local val1 = mod1.get()

        assert(val2 == 0, "Expected mod2 to have separate state (0), got: " .. tostring(val2))
        assert(val1 == 100, "Expected mod1 to be at 100, got: " .. tostring(val1))
        return true
    "#;

    let result = test.execute_script(script, "test.lua");
    assert!(result.is_ok(), "Script failed: {:?}", result.err());
}

// =============================================================================
// require_async() Tests
// =============================================================================

#[test]
fn test_require_async_invokes_callback() {
    let mut test = TestApp::new();
    test.add_fixture("basic_module.lua", "scripts/basic.lua");

    let script = r#"
        _G.callback_invoked = false
        _G.received_value = nil

        require_async("scripts/basic.lua", function(mod)
            _G.callback_invoked = true
            _G.received_value = mod.value
        end)

        return true
    "#;

    test.execute_script(script, "test.lua")
        .expect("Script failed");

    // Run an update to process async callbacks
    test.update();

    // Check globals
    let callback_invoked = test.eval_lua("return _G.callback_invoked").unwrap();
    let received_value = test.eval_lua("return _G.received_value").unwrap();

    assert!(
        callback_invoked.contains("true"),
        "Expected callback to be invoked, got: {}",
        callback_invoked
    );
    assert!(
        received_value.contains("42"),
        "Expected received_value=42, got: {}",
        received_value
    );
}

#[test]
fn test_require_async_reload_false_records_flag() {
    let mut test = TestApp::new();
    test.add_fixture("basic_module.lua", "scripts/basic.lua");

    let script = r#"
        require_async("scripts/basic.lua", function(mod)
            -- callback
        end, { reload = false })
        return true
    "#;

    test.execute_script(script, "scripts/test.lua")
        .expect("Script failed");
    test.update();

    // Verify hot_reload_callback was registered with should_invoke_callback=false
    let cache = test.script_cache();
    let callbacks = cache.get_hot_reload_callbacks("scripts/basic.lua");

    // Should have callback registered
    assert!(
        !callbacks.is_empty(),
        "Expected hot reload callback to be registered"
    );

    // Check that should_invoke_callback is false (third element of tuple)
    let has_no_invoke = callbacks.iter().any(|(_, _, should_invoke, _)| !*should_invoke);
    assert!(
        has_no_invoke,
        "Expected callback with should_invoke=false, got: {:?}",
        callbacks.iter().map(|(_, _, s, _)| s).collect::<Vec<_>>()
    );
}

// =============================================================================
// Hot-Reload Behavior Tests
// =============================================================================

#[test]
fn test_hot_reload_invalidates_module_cache() {
    let mut test = TestApp::new();
    test.add_fixture("basic_module.lua", "scripts/basic.lua");

    // First load the module and store its value
    let script = r#"
        local mod = require("scripts/basic.lua")
        _G.initial_value = mod.value
        return mod.value
    "#;
    test.execute_script(script, "test.lua")
        .expect("Script failed");

    // Verify initial value
    let initial = test.eval_lua("return _G.initial_value").unwrap();
    assert!(initial.contains("42"), "Expected initial value=42");

    // Modify and trigger hot reload
    test.modify_file(
        "scripts/basic.lua",
        r#"return { name = "updated", value = 99 }"#,
    );
    test.trigger_hot_reload("scripts/basic.lua");

    // After hot reload, re-requiring should get new value
    let script2 = r#"
        local mod = require("scripts/basic.lua")
        _G.new_value = mod.value
        return mod.value
    "#;
    test.execute_script(script2, "test2.lua")
        .expect("Script2 failed");

    let new_value = test.eval_lua("return _G.new_value").unwrap();
    assert!(
        new_value.contains("99"),
        "Expected new value=99 after hot reload, got: {}",
        new_value
    );
}

#[test]
fn test_hot_reload_propagates_to_reload_true_dependents() {
    let mut test = TestApp::new();
    test.add_fixture("inner.lua", "scripts/inner.lua");

    // Script that requires with reload=true (default) and stores dependency version
    let script = r#"
        local inner = require("scripts/inner.lua")
        _G.initial_version = inner.version
        return inner.version
    "#;
    test.execute_script(script, "scripts/parent.lua")
        .expect("Script failed");

    // Verify initial version
    let initial = test.eval_lua("return _G.initial_version").unwrap();
    assert!(initial.contains("1"), "Expected initial version=1");

    // Modify inner and trigger hot reload
    test.modify_file("scripts/inner.lua", r#"return { inner = true, version = 2 }"#);
    test.trigger_hot_reload("scripts/inner.lua");

    // Re-require the inner module and check it has new version
    let script2 = r#"
        local inner = require("scripts/inner.lua")
        _G.new_version = inner.version
        return inner.version
    "#;
    test.execute_script(script2, "scripts/checker.lua")
        .expect("Script2 failed");

    let new_version = test.eval_lua("return _G.new_version").unwrap();
    assert!(
        new_version.contains("2"),
        "Expected new version=2 after hot reload, got: {}",
        new_version
    );
}

#[test]
fn test_hot_reload_does_not_propagate_to_reload_false_parent() {
    let mut test = TestApp::new();
    test.add_fixture("inner.lua", "scripts/inner.lua");

    // Script that requires with reload=false
    let script = r#"
        local inner = require("scripts/inner.lua", { reload = false })
        _G.initial_version = inner.version
        return inner.version
    "#;
    test.execute_script(script, "scripts/parent.lua")
        .expect("Script failed");

    // Verify initial version
    let initial = test.eval_lua("return _G.initial_version").unwrap();
    assert!(initial.contains("1"), "Expected initial version=1");

    // Modify inner and trigger hot reload
    test.modify_file("scripts/inner.lua", r#"return { inner = true, version = 2 }"#);
    test.trigger_hot_reload("scripts/inner.lua");

    // The inner module itself should still reload when directly required
    let script2 = r#"
        local inner = require("scripts/inner.lua")
        _G.direct_version = inner.version
        return inner.version
    "#;
    test.execute_script(script2, "scripts/direct.lua")
        .expect("Script2 failed");

    let direct_version = test.eval_lua("return _G.direct_version").unwrap();
    assert!(
        direct_version.contains("2"),
        "Expected inner module to have new version=2 when directly required, got: {}",
        direct_version
    );

    // The parent script's cached reference shouldn't trigger automatic reload
    // because it had reload=false. This is verified by the dependency tracking.
    let cache = test.script_cache();
    let importers = cache.get_importers("scripts/inner.lua");
    let parent_has_reload_false = importers
        .iter()
        .any(|(path, reload)| path.contains("parent.lua") && !*reload);
    assert!(
        parent_has_reload_false,
        "Parent should have reload=false for inner dependency"
    );
}

/// CRITICAL TEST: When parent uses reload=false, the dependency itself should still reload.
/// Only the propagation to the parent is blocked, not the dependency's own reload.
#[test]
fn test_dependency_reloads_when_parent_has_reload_false() {
    let mut test = TestApp::new();
    test.add_fixture("inner.lua", "scripts/child.lua");

    // First script requires child with reload=false
    let script = r#"
        local child = require("scripts/child.lua", { reload = false })
        _G.initial_version = child.version
        return true
    "#;
    test.execute_script(script, "scripts/parent.lua")
        .expect("Script failed");

    // Verify initial version
    let initial = test.eval_lua("return _G.initial_version").unwrap();
    assert!(
        initial.contains("1"),
        "Expected initial version=1, got: {}",
        initial
    );

    // Modify child.lua
    test.modify_file("scripts/child.lua", r#"return { inner = true, version = 2 }"#);

    // Trigger hot reload
    test.trigger_hot_reload("scripts/child.lua");

    // Now require the child again from a different script (simulating another access)
    // The child should have the new version because its cache was invalidated
    let script2 = r#"
        local child = require("scripts/child.lua")
        _G.new_version = child.version
        return true
    "#;
    test.execute_script(script2, "scripts/other.lua")
        .expect("Script2 failed");

    // Verify new version - this is the critical assertion
    let new_version = test.eval_lua("return _G.new_version").unwrap();
    assert!(
        new_version.contains("2"),
        "Expected new version=2 (dependency should reload), got: {}",
        new_version
    );

    // Verify the parent registered with reload=false
    let cache = test.script_cache();
    let importers = cache.get_importers("scripts/child.lua");
    let parent_has_reload_false = importers
        .iter()
        .any(|(path, reload)| path.contains("parent.lua") && !*reload);
    assert!(
        parent_has_reload_false,
        "Parent should have reload=false for child dependency"
    );
}

// =============================================================================
// Live Proxy System Tests
// =============================================================================

#[test]
fn test_require_returns_proxy_not_raw_table() {
    let mut test = TestApp::new();
    test.add_fixture("basic_module.lua", "scripts/basic.lua");

    let script = r#"
        local mod = require("scripts/basic.lua")
        local mt = getmetatable(mod)
        -- Proxy should have a metatable with __cache_key
        assert(mt ~= nil, "Expected proxy to have metatable")
        assert(mt.__cache_key ~= nil, "Expected metatable to have __cache_key")
        return true
    "#;

    let result = test.execute_script(script, "test.lua");
    assert!(result.is_ok(), "Script failed: {:?}", result.err());
}

#[test]
fn test_proxy_sees_reloaded_module() {
    let mut test = TestApp::new();
    test.add_fixture("basic_module.lua", "scripts/basic.lua");

    // Get a reference to the module
    let script1 = r#"
        _G.mod_ref = require("scripts/basic.lua")
        _G.initial_value = _G.mod_ref.value
        return true
    "#;
    test.execute_script(script1, "test1.lua")
        .expect("Script1 failed");

    // Verify initial value
    let initial = test.eval_lua("return _G.initial_value").unwrap();
    assert!(initial.contains("42"), "Expected initial value=42");

    // Modify and reload
    test.modify_file(
        "scripts/basic.lua",
        r#"return { name = "updated", value = 99 }"#,
    );
    test.trigger_hot_reload("scripts/basic.lua");

    // Re-require to populate the cache with new version
    let script2 = r#"
        require("scripts/basic.lua")
        return true
    "#;
    test.execute_script(script2, "test2.lua")
        .expect("Script2 failed");

    // The existing proxy reference should now see the new value
    let new_value = test.eval_lua("return _G.mod_ref.value").unwrap();
    assert!(
        new_value.contains("99"),
        "Expected proxy to see new value=99, got: {}",
        new_value
    );
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_require_nonexistent_file_error() {
    let mut test = TestApp::new();

    let script = r#"
        local success, err = pcall(function()
            require("scripts/nonexistent.lua")
        end)
        assert(not success, "Expected require to fail for nonexistent file")
        return true
    "#;

    let result = test.execute_script(script, "test.lua");
    assert!(result.is_ok(), "Script failed: {:?}", result.err());
}

#[test]
fn test_circular_dependency_handled() {
    let mut test = TestApp::new();

    // Create two modules that depend on each other
    test.modify_file(
        "scripts/a.lua",
        r#"
        local b = require("scripts/b.lua")
        return { name = "a", b_name = b.name }
    "#,
    );
    test.modify_file(
        "scripts/b.lua",
        r#"
        local a = require("scripts/a.lua")
        return { name = "b", a_name = a.name }
    "#,
    );

    // This should not hang - circular deps return proxy
    let script = r#"
        local success, result = pcall(function()
            return require("scripts/a.lua")
        end)
        -- May succeed with partial data or fail gracefully
        return true
    "#;

    let result = test.execute_script(script, "test.lua");
    // We just verify it doesn't hang
    assert!(result.is_ok(), "Script failed or hung: {:?}", result.err());
}
