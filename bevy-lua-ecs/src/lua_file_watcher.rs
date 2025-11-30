use bevy::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Event emitted when a Lua script file changes
#[derive(Event, Clone, Debug, Reflect)]
#[reflect(Debug)]
pub struct LuaFileChangeEvent {
    pub path: PathBuf,
}

/// Plugin that watches Lua script files for changes
pub struct LuaFileWatcherPlugin;

impl Plugin for LuaFileWatcherPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LuaFileChangeEvent>();
        app.add_systems(Startup, setup_file_watcher);
        app.add_systems(Update, poll_file_changes);
    }
}

#[derive(Resource)]
struct FileWatcherState {
    /// Track last modified times for debouncing
    last_modified: std::collections::HashMap<PathBuf, std::time::SystemTime>,
    /// Debounce duration
    debounce_duration: Duration,
}

impl Default for FileWatcherState {
    fn default() -> Self {
        Self {
            last_modified: std::collections::HashMap::new(),
            debounce_duration: Duration::from_millis(100),
        }
    }
}

fn setup_file_watcher(mut commands: Commands) {
    commands.insert_resource(FileWatcherState::default());
    info!("Lua file watcher initialized (polling mode)");
}

fn poll_file_changes(
    mut state: ResMut<FileWatcherState>,
    mut events: EventWriter<LuaFileChangeEvent>,
) {
    let script_dir = Path::new("assets/scripts");
    
    if !script_dir.exists() {
        return;
    }
    
    // Recursively walk the scripts directory
    if let Ok(entries) = std::fs::read_dir(script_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            // Only check .lua files
            if path.extension().and_then(|s| s.to_str()) != Some("lua") {
                continue;
            }
            
            // Get file metadata
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    // Check if file was recently modified
                    if let Some(last_mod) = state.last_modified.get(&path) {
                        if modified > *last_mod {
                            // Check debounce
                            if let Ok(duration) = modified.duration_since(*last_mod) {
                                if duration >= state.debounce_duration {
                                    info!("Detected change in Lua script: {:?}", path);
                                    events.write(LuaFileChangeEvent {
                                        path: path.clone(),
                                    });
                                    state.last_modified.insert(path.clone(), modified);
                                }
                            }
                        }
                    } else {
                        // First time seeing this file
                        state.last_modified.insert(path.clone(), modified);
                    }
                }
            }
        }
    }
}
