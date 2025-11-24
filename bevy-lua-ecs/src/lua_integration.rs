use bevy::prelude::*;
use mlua::prelude::*;
use crate::spawn_queue::SpawnQueue;
use crate::lua_systems::LuaSystemRegistry;
use std::sync::Arc;

/// Resource that holds the Lua context
#[derive(Resource, Clone)]
pub struct LuaScriptContext {
    pub lua: Arc<Lua>,
}
impl LuaScriptContext {
    /// Create a new Lua context with component-based spawn function
    pub fn new(queue: SpawnQueue, system_registry: LuaSystemRegistry) -> Result<Self, LuaError> {
        let lua = Lua::new();
        
        // Clone what we need for the closure
        let queue_clone = queue.clone();
        let lua_clone = Arc::new(lua);
        let lua_for_closure = lua_clone.clone();
        
        // Create component-based spawn function
        let spawn = lua_clone.create_function(move |_lua_ctx, components: LuaTable| {
            let mut all_components = Vec::new();
            
            // Iterate over components table
            for pair in components.pairs::<String, LuaValue>() {
                let (component_name, component_value) = pair?;
                
                // Store everything as registry value
                let registry_key = lua_for_closure.create_registry_value(component_value)?;
                all_components.push((component_name, registry_key));
            }
            
            // Pass empty list for lua_components, we'll sort it out in the spawner
            queue_clone.clone().queue_spawn(all_components, Vec::new());
            Ok(())
        })?;
        
        // Create register_system function
        let system_reg = system_registry.clone();
        let register_system = lua_clone.create_function(move |lua_ctx, (_schedule, func): (String, LuaFunction)| {
            let registry_key = lua_ctx.create_registry_value(func)?;
            system_reg.update_systems.lock().unwrap().push(Arc::new(registry_key));
            Ok(())
        })?;
        
        // Inject into globals
        lua_clone.globals().set("spawn", spawn)?;
        lua_clone.globals().set("register_system", register_system)?;
        
        Ok(Self {
            lua: lua_clone,
        })
    }
    
    /// Execute a Lua script from a string
    pub fn execute_script(&self, script_content: &str, script_name: &str) -> Result<(), LuaError> {
        self.lua.load(script_content).set_name(script_name).exec()?;
        Ok(())
    }
}

/// Plugin that sets up Lua scripting with component-based spawn function
pub struct LuaSpawnPlugin;

impl Plugin for LuaSpawnPlugin {
    fn build(&self, app: &mut App) {
        // Initialize SerdeComponentRegistry if not already present
        if !app.world().contains_resource::<crate::serde_components::SerdeComponentRegistry>() {
            app.init_resource::<crate::serde_components::SerdeComponentRegistry>();
        }
        
        app.add_systems(Startup, setup_lua_context);
    }
}

/// System to initialize Lua context
fn setup_lua_context(
    mut commands: Commands,
    queue: Res<SpawnQueue>,
) {
    let system_registry = LuaSystemRegistry::default();
    match LuaScriptContext::new(queue.clone(), system_registry.clone()) {
        Ok(ctx) => {
            commands.insert_resource(ctx);
            commands.insert_resource(system_registry);
        }
        Err(e) => {
            error!("Failed to initialize Lua context: {}", e);
        }
    }
}
