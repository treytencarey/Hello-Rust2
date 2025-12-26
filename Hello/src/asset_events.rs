// Asset events for Lua consumption via world:read_events()
//
// These events are emitted by the network asset system and can be read
// by Lua scripts to build UI components like file browsers, upload progress bars,
// and conflict resolution modals.
//
// Note: In Bevy 0.17, events use the Message trait and are registered with add_message()

use bevy::prelude::*;

// ============================================================================
// Directory Listing Event
// ============================================================================

/// Event emitted when a directory listing response is received from the server
#[derive(Message, Clone, Debug, Reflect)]
pub struct AssetDirectoryListingEvent {
    /// Directory path that was listed
    pub path: String,
    /// Files/directories in this page
    pub files: Vec<AssetFileInfo>,
    /// Total count of items in directory
    pub total_count: u32,
    /// Offset used for this response
    pub offset: u32,
    /// Whether there are more items after this page
    pub has_more: bool,
    /// Error message if listing failed
    pub error: Option<String>,
}

/// File information for directory listing (reflected for Lua)
#[derive(Clone, Debug, Reflect, Default)]
pub struct AssetFileInfo {
    /// File name (basename only)
    pub name: String,
    /// Full relative path from assets/
    pub path: String,
    /// File size in bytes (0 for directories)
    pub size: u64,
    /// Last modified time as Unix timestamp
    pub modified: u64,
    /// Whether this is a directory
    pub is_directory: bool,
}

impl From<&crate::network_asset_client::FileInfo> for AssetFileInfo {
    fn from(info: &crate::network_asset_client::FileInfo) -> Self {
        Self {
            name: info.name.clone(),
            path: info.path.clone(),
            size: info.size,
            modified: info.modified,
            is_directory: info.is_directory,
        }
    }
}

// ============================================================================
// Upload Progress Event
// ============================================================================

/// Event emitted when upload progress changes or upload completes
#[derive(Message, Clone, Debug, Reflect)]
pub struct AssetUploadProgressEvent {
    /// Path being uploaded (destination on server)
    pub path: String,
    /// Number of chunks sent to server
    pub chunks_sent: u32,
    /// Total number of chunks
    pub total_chunks: u32,
    /// Progress as a value from 0.0 to 1.0
    pub progress: f32,
    /// Status: "uploading", "complete", "conflict", "error"
    pub status: String,
    /// Error message if status is "error"
    pub error: Option<String>,
    /// Server's hash of existing file (for conflict resolution)
    pub server_hash: Option<String>,
}

// ============================================================================
// LocalNewer Event
// ============================================================================

/// Event emitted when a local file is detected as newer than the server version
/// 
/// This happens when:
/// 1. A subscribed file changes locally (detected by file watcher)
/// 2. The local hash differs from the last-known server hash
/// 
/// Lua scripts can listen for this event to show an upload prompt modal.
#[derive(Message, Clone, Debug, Reflect)]
pub struct AssetLocalNewerEvent {
    /// Path of the file that is newer locally (relative to assets/)
    pub path: String,
    /// Hash of the local file
    pub local_hash: String,
    /// Last-known hash of the server file
    pub server_hash: String,
    /// Local file's modification time as Unix timestamp
    pub local_modified: u64,
}

// ============================================================================
// Rename/Delete Events
// ============================================================================

/// Event emitted when a rename/move operation completes
#[derive(Message, Clone, Debug, Reflect)]
pub struct AssetRenameEvent {
    /// Old path
    pub old_path: String,
    /// New path
    pub new_path: String,
    /// Whether operation succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Event emitted when a delete operation completes
#[derive(Message, Clone, Debug, Reflect)]
pub struct AssetDeleteEvent {
    /// Deleted path
    pub path: String,
    /// Whether operation succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

// ============================================================================
// Plugin Registration
// ============================================================================

/// Plugin that registers asset events for Lua consumption
pub struct AssetEventsPlugin;

impl Plugin for AssetEventsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register message types (Bevy 0.17 uses Message instead of Event)
            .add_message::<AssetDirectoryListingEvent>()
            .add_message::<AssetUploadProgressEvent>()
            .add_message::<AssetLocalNewerEvent>()
            .add_message::<AssetRenameEvent>()
            .add_message::<AssetDeleteEvent>()
            // Register types for reflection (so Lua can read_events)
            .register_type::<AssetDirectoryListingEvent>()
            .register_type::<AssetUploadProgressEvent>()
            .register_type::<AssetLocalNewerEvent>()
            .register_type::<AssetRenameEvent>()
            .register_type::<AssetDeleteEvent>()
            .register_type::<AssetFileInfo>()
            .register_type::<Vec<AssetFileInfo>>();
    }
}
