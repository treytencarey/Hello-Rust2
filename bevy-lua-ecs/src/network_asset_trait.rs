// Network asset loading trait
//
// This module provides a generic interface for network-based asset loading.
// The concrete implementation lives in the application crate (Hello),
// while bevy-lua-ecs only uses the trait.

use bevy::prelude::*;
use mlua::RegistryKey;
use std::sync::Arc;

/// Status of an asset download request
#[derive(Clone, Debug, PartialEq)]
pub enum AssetDownloadStatus {
    /// File exists locally, no download needed
    LocalAvailable,
    /// Request queued but not yet sent
    Pending,
    /// Download in progress with progress (0.0-1.0)
    Downloading(f32),
    /// Download complete, data available
    Complete,
    /// Download failed with error
    Failed(String),
}

/// Trait for requesting network asset downloads
///
/// This trait is implemented by the application crate to provide
/// network asset downloading functionality. The bevy-lua-ecs library
/// calls this trait without knowing the concrete networking implementation.
pub trait NetworkAssetRequestor: Send + Sync + 'static {
    /// Check if an asset is available locally
    fn is_available_locally(&self, path: &str) -> bool;

    /// Queue a request to download an asset
    /// Returns a request ID that can be used to check status
    fn queue_download(&self, path: &str) -> u64;

    /// Check if download is complete for a path
    fn is_download_complete(&self, path: &str) -> bool;

    /// Take downloaded data (removes from cache)
    fn take_downloaded_data(&self, path: &str) -> Option<Vec<u8>>;

    /// Get download status
    fn get_status(&self, path: &str) -> AssetDownloadStatus;

    /// Register a coroutine to be resumed when download completes
    fn register_pending_coroutine(
        &self,
        path: &str,
        coroutine_key: Arc<RegistryKey>,
        instance_id: u64,
    );

    /// Check if any coroutines are pending for a path
    fn has_pending_coroutines(&self, path: &str) -> bool;
}

/// Resource that holds the network asset requestor implementation
///
/// This resource is optional - if not present, scripts will only load local files.
/// The Hello project inserts this resource with a concrete implementation.
#[derive(Resource, Clone)]
pub struct NetworkAssetLoader {
    /// The actual requestor implementation (boxed trait object)
    requestor: Arc<dyn NetworkAssetRequestor>,
}

impl NetworkAssetLoader {
    /// Create a new NetworkAssetLoader from an implementation
    pub fn new<T: NetworkAssetRequestor>(requestor: T) -> Self {
        Self {
            requestor: Arc::new(requestor),
        }
    }

    /// Check if an asset is available locally
    pub fn is_available_locally(&self, path: &str) -> bool {
        self.requestor.is_available_locally(path)
    }

    /// Queue a download request
    pub fn queue_download(&self, path: &str) -> u64 {
        self.requestor.queue_download(path)
    }

    /// Check if download is complete
    pub fn is_download_complete(&self, path: &str) -> bool {
        self.requestor.is_download_complete(path)
    }

    /// Take downloaded data
    pub fn take_downloaded_data(&self, path: &str) -> Option<Vec<u8>> {
        self.requestor.take_downloaded_data(path)
    }

    /// Get download status
    pub fn get_status(&self, path: &str) -> AssetDownloadStatus {
        self.requestor.get_status(path)
    }

    /// Register a pending coroutine
    pub fn register_pending_coroutine(
        &self,
        path: &str,
        coroutine_key: Arc<RegistryKey>,
        instance_id: u64,
    ) {
        self.requestor
            .register_pending_coroutine(path, coroutine_key, instance_id);
    }

    /// Check for pending coroutines
    pub fn has_pending_coroutines(&self, path: &str) -> bool {
        self.requestor.has_pending_coroutines(path)
    }
}
