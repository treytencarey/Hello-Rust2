use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use bevy_lua_ecs::*;
use std::sync::Mutex;
use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;

static TEST_MUTEX: Mutex<()> = Mutex::new(());

struct TestApp {
    app: App,
    temp_dir: TempDir,
    original_dir: PathBuf,
    #[allow(dead_code)]
    lock: std::sync::MutexGuard<'static, ()>,
}

impl TestApp {
    fn new() -> Self {
        let lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let original_dir = std::env::current_dir().expect("Failed to get current dir");
        let assets_dir = temp_dir.path().join("assets").join("scripts");
        fs::create_dir_all(&assets_dir).expect("Failed to create assets/scripts dir");
        std::env::set_current_dir(temp_dir.path()).expect("Failed to change to temp dir");

        let mut app = App::new();
        app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_once()));
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(LuaSpawnPlugin);
        app.update();

        Self {
            app,
            temp_dir,
            original_dir,
            lock,
        }
    }

    fn execute_script(&mut self, content: &str) -> Result<u64, String> {
        let lua_ctx = self.app.world().get_resource::<LuaScriptContext>().expect("LuaScriptContext not found").clone();
        let script_instance = self.app.world().get_resource::<ScriptInstance>().expect("ScriptInstance not found").clone();
        let script_registry = self.app.world().get_resource::<ScriptRegistry>().expect("ScriptRegistry not found").clone();
        let path = PathBuf::from("scripts").join("test.lua");

        lua_ctx.execute_script(content, "test.lua", path, &script_instance, &script_registry)
            .map_err(|e| format!("{}", e))
    }

    fn update(&mut self) {
        self.app.update();
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original_dir);
    }
}

#[test]
fn test_query_precise_change_detection() {
    let mut test = TestApp::new();

    // 1. Setup entities
    test.execute_script(r#"
        spawn({
            Transform = { translation = {x=0, y=0, z=0} },
            Sprite = { color = {r=1, g=1, b=1, a=1} },
            MyMarker = {}
        })
    "#).unwrap();
    test.update();

    // 3. Register CheckChanges system
    test.execute_script(r#"
        register_system("CheckChanges", function(world)
            local entities = world:query({
                with = {"MyMarker"},
                any_of = {"Transform", "Sprite"},
                ["or"] = {
                    changed = {"Transform", "Sprite"}
                }
            })
            
            if not _G.warmed_up then
                _G.warmed_up = true
                return
            end
            
            assert(#entities == 1, "Expected 1 entity, got " .. #entities)
            local e = entities[1]
            
            local changed = e:changed_components()
            local changed_map = {}
            for _, name in ipairs(changed) do changed_map[name] = true end
            
            assert(changed_map["Transform"] == true, "Expected Transform to be changed")
            assert(changed_map["Sprite"] == nil, "Expected Sprite to NOT be changed")
            assert(e:is_changed("Transform") == true, "is_changed(Transform) failed")
            assert(e:is_changed("Sprite") == false, "is_changed(Sprite) failed")
            
            _G.test_success = true
        end)
    "#).unwrap();
    test.update(); // Warmup: runs once with last_run=0, sets warmed_up=true

    // 4. Modify one component
    test.execute_script(r#"
        register_system("UpdateOnce", function(world)
            local entities = world:query({"Transform", "MyMarker"})
            for _, e in ipairs(entities) do
                e:set({ Transform = { translation = {x=1, y=0, z=0} } })
            end
            return true -- One-shot
        end)
    "#).unwrap();
    test.update(); // Runs UpdateOnce (Transform changed) and CheckChanges (last_run was warmup tick)

    // 5. Final update to ensure CheckChanges sees the update from previous frame
    // Wait, in Step 4, UpdateOnce runs, then CheckChanges runs. 
    // Does CheckChanges see the change from UpdateOnce in the SAME frame?
    // Usually yes if it runs after. But let's do another update to be sure.
    test.update(); 

    let success: bool = test.app.world().get_resource::<LuaScriptContext>().unwrap().lua.globals().get("test_success").unwrap_or(false);
    assert!(success, "Test script did not set test_success to true");
}

#[test]
fn test_query_added_detection() {
    let mut test = TestApp::new();

    // 1. Spawn entity with only Transform
    test.execute_script(r#"
        spawn({
            Transform = { translation = {x=0, y=0, z=0} },
            MyMarker = {}
        })
    "#).unwrap();
    test.update();

    // 3. Register CheckAdded system
    test.execute_script(r#"
        register_system("CheckAdded", function(world)
            local entities = world:query({
                with = {"MyMarker"},
                ["or"] = {
                    added = {"Sprite", "Transform"}
                }
            })
            
            if not _G.added_warmed_up then
                _G.added_warmed_up = true
                return
            end
            
            assert(#entities == 1, "Expected 1 entity, got " .. #entities)
            local e = entities[1]
            
            assert(e:is_added("Sprite") == true, "Expected Sprite to be added")
            assert(e:is_added("Transform") == false, "Expected Transform to NOT be added")
            
            _G.added_success = true
        end)
    "#).unwrap();
    test.update(); // Warmup

    // 4. Add Sprite component via one-shot system
    test.execute_script(r#"
        register_system("AddSpriteOnce", function(world)
            local entities = world:query({"MyMarker"})
            for _, e in ipairs(entities) do
                e:set({ Sprite = { color = {r=1, g=1, b=1, a=1} } })
            end
            return true
        end)
    "#).unwrap();
    test.update(); // Runs AddSpriteOnce and CheckAdded
    test.update(); // Just to be safe

    let success: bool = test.app.world().get_resource::<LuaScriptContext>().unwrap().lua.globals().get("added_success").unwrap_or(false);
    assert!(success, "Test script did not set added_success to true");
}

#[test]
fn test_query_only_returns_requested_components() {
    let mut test = TestApp::new();

    // 1. Spawn entity with three Lua components: CompA, CompB, CompC
    test.execute_script(r#"
        spawn({
            CompA = { value = "A" },
            CompB = { value = "B" },
            CompC = { value = "C" },
            MyMarker = {}
        })
    "#).unwrap();
    test.update();

    // 2. Query for only CompA and CompB, NOT CompC
    test.execute_script(r#"
        register_system("CheckSerialize", function(world)
            local entities = world:query({
                with = {"CompA", "CompB", "MyMarker"}
            })

            assert(#entities == 1, "Expected 1 entity, got " .. #entities)
            local e = entities[1]

            -- CompA and CompB should be accessible
            local a = e:get("CompA")
            local b = e:get("CompB")
            assert(a ~= nil, "Expected CompA to be available")
            assert(a.value == "A", "Expected CompA.value to be 'A'")
            assert(b ~= nil, "Expected CompB to be available")
            assert(b.value == "B", "Expected CompB.value to be 'B'")

            -- CompC should NOT be serialized since we didn't query for it
            local c = e:get("CompC")
            assert(c == nil, "Expected CompC to be nil (not serialized), but got: " .. tostring(c))

            _G.serialize_test_success = true
            return true -- One-shot
        end)
    "#).unwrap();
    test.update();

    let success: bool = test.app.world().get_resource::<LuaScriptContext>().unwrap().lua.globals().get("serialize_test_success").unwrap_or(false);
    assert!(success, "Test script did not set serialize_test_success to true - query may have over-serialized components");
}

#[test]
fn test_query_changed_components_auto_serialized() {
    let mut test = TestApp::new();

    // 1. Spawn entity with components A, B, C
    test.execute_script(r#"
        spawn({
            CompA = { value = "A" },
            CompB = { value = "B" },
            CompC = { value = "C" },
            MyMarker = {}
        })
    "#).unwrap();
    test.update();

    // 2. Register system that queries with changed filter but without any_of
    test.execute_script(r#"
        register_system("UpdateOnce", function(world)
            if _G.update_done then return true end

            local entities = world:query({"CompA", "MyMarker"})
            for _, e in ipairs(entities) do
                -- Modify CompB (not in the with filter)
                e:set({ CompB = { value = "B_modified" } })
            end
            _G.update_done = true
            return true
        end)

        register_system("CheckChanged", function(world)
            if not _G.update_done then return end
            if _G.check_done then return end

            -- Query for entities where CompB changed, but DON'T include CompB in 'with' or 'any_of'
            -- The component should still be serialized because it's in the 'changed' filter
            local entities = world:query({
                with = {"MyMarker"},
                ["or"] = {
                    changed = {"CompB"}
                }
            })

            if #entities == 1 then
                local e = entities[1]
                -- CompB should be accessible even though we didn't specify it in 'with' or 'any_of'
                local b = e:get("CompB")
                assert(b ~= nil, "Expected CompB to be auto-serialized from changed filter")
                assert(b.value == "B_modified", "Expected CompB.value to be 'B_modified', got: " .. tostring(b.value))
                _G.check_done = true
            end
        end)
    "#).unwrap();

    // Run a few updates
    for _ in 0..5 {
        test.update();
        let success: bool = test.app.world().get_resource::<LuaScriptContext>().unwrap().lua.globals().get("check_done").unwrap_or(false);
        if success {
            break;
        }
    }

    let success: bool = test.app.world().get_resource::<LuaScriptContext>().unwrap().lua.globals().get("check_done").unwrap_or(false);
    assert!(success, "Test failed - components from changed filter should be auto-serialized");
}

#[test]
fn test_query_removed_filter() {
    let mut test = TestApp::new();

    // 1. Spawn entity with a component and capture its real ID
    test.execute_script(r#"
        spawn({
            Transform = { translation = {x=0, y=0, z=0} },
            MyRemovableComp = { data = "test" }
        })
    "#).unwrap();
    test.update(); // Entity exists

    // 2. Register system to check for removed components and to despawn
    test.execute_script(r#"
        register_system("DespawnOnce", function(world)
            if _G.despawn_done then return true end

            local entities = world:query({"MyRemovableComp"})
            if #entities > 0 then
                -- Despawn the first entity with the component
                despawn(entities[1])
                _G.despawn_done = true
            end
            return _G.despawn_done or false
        end)

        register_system("CheckRemoved", function(world)
            if _G.removed_test_success then return end

            local removed = world:query({
                removed = {"MyRemovableComp"}
            })

            if #removed > 0 then
                _G.removed_test_success = true
                _G.removed_entity_bits = removed[1]:id()
            end
        end)
    "#).unwrap();

    // Run updates:
    // Frame 1: DespawnOnce despawns the entity
    // Frame 2: Entity is removed, tracker sees previous state had component
    // Frame 3+: query({ removed = {...} }) should return the entity bits
    for _ in 0..6 {
        test.update();
        let success: bool = test.app.world().get_resource::<LuaScriptContext>().unwrap().lua.globals().get("removed_test_success").unwrap_or(false);
        if success {
            break;
        }
    }

    let success: bool = test.app.world().get_resource::<LuaScriptContext>().unwrap().lua.globals().get("removed_test_success").unwrap_or(false);
    assert!(success, "Test script did not detect the removed component via query({{ removed = {{...}} }})");
}
