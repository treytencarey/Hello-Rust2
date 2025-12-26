//! Text Input Lua Binding
//! 
//! Provides Reflect-enabled components for bevy_ui_text_input.
//! 
//! ```lua
//! -- Spawn text input:
//! local input_id = spawn({
//!     LuaTextInput = { placeholder = "Enter text..." },
//!     Node = { width = {Px = 200}, height = {Px = 30} },
//! }):id()
//! 
//! -- Later, read the text:
//! local value = get_entity(input_id):get("LuaTextInputValue")
//! print(value.text)  -- The current text content
//! ```

use bevy::prelude::*;
use bevy_ui_text_input::TextInputBuffer;

/// Reflect-enabled marker component that triggers TextInputNode insertion.
/// This allows Lua's generic spawn system to create text inputs.
#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct LuaTextInput {
    /// Placeholder text shown when empty
    #[reflect(default)]
    pub placeholder: String,
    /// Initial text value (pre-filled)
    #[reflect(default)]
    pub initial_value: String,
    /// Whether to clear text on submit
    #[reflect(default)]
    pub clear_on_submit: bool,
    /// Whether to auto-focus this input when spawned
    #[reflect(default)]
    pub auto_focus: bool,
}

/// Reflect-enabled component that mirrors TextInputBuffer's text content.
/// This allows Lua to read the current text via entity:get("LuaTextInputValue").
#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct LuaTextInputValue {
    /// The current text content of the input
    pub text: String,
}

/// Plugin that handles LuaTextInput -> TextInputNode conversion
pub struct LuaTextInputPlugin;

impl Plugin for LuaTextInputPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<LuaTextInput>();
        app.register_type::<LuaTextInputValue>();
        app.add_systems(Update, convert_lua_text_inputs);
        app.add_systems(Update, sync_text_input_value);
    }
}

/// System that converts LuaTextInput marker components to actual TextInputNode components
fn convert_lua_text_inputs(
    mut commands: Commands,
    query: Query<(Entity, &LuaTextInput), Added<LuaTextInput>>,
    mut input_focus: ResMut<bevy::input_focus::InputFocus>,
) {
    for (entity, lua_input) in query.iter() {
        // Create the actual TextInputNode with settings
        let text_input = bevy_ui_text_input::TextInputNode {
            clear_on_submit: lua_input.clear_on_submit,
            ..default()
        };
        
        // Insert the real component, the value mirror, and remove the marker
        commands.entity(entity)
            .insert(text_input)
            .insert(LuaTextInputValue { 
                text: lua_input.initial_value.clone() 
            })
            .remove::<LuaTextInput>();
        
        // Set placeholder if provided
        if !lua_input.placeholder.is_empty() {
            commands.entity(entity).insert(
                bevy_ui_text_input::TextInputPrompt {
                    text: lua_input.placeholder.clone(),
                    font: None,
                    color: None,
                }
            );
        }
        
        // Queue initial text if provided (using Paste action to insert string)
        if !lua_input.initial_value.is_empty() {
            use bevy_ui_text_input::actions::{TextInputAction, TextInputEdit};
            commands.entity(entity).insert(bevy_ui_text_input::TextInputQueue {
                actions: vec![
                    TextInputAction::Edit(TextInputEdit::Paste(lua_input.initial_value.clone()))
                ].into(),
            });
            info!("Set initial text for entity {:?}: {}", entity, lua_input.initial_value);
        }
        
        // Auto-focus if requested
        if lua_input.auto_focus {
            input_focus.0 = Some(entity);
            info!("Auto-focused text input entity {:?}", entity);
        }
        
        info!("Converted LuaTextInput to TextInputNode for entity {:?}", entity);
    }
}

/// System that syncs LuaTextInputValue with the actual TextInputBuffer content
/// Runs on all entities with both components to ensure initial sync happens
fn sync_text_input_value(
    mut query: Query<(&TextInputBuffer, &mut LuaTextInputValue)>,
) {
    for (buffer, mut value) in query.iter_mut() {
        let current_text = buffer.get_text();
        if value.text != current_text {
            value.text = current_text;
        }
    }
}


