# Network Asset Server System

This document describes the on-demand asset downloading system for the Bevy + Lua project.

## Overview

The network asset system allows Lua scripts to download assets (scripts, images, etc.) from a server on-demand. Key features:
- **Synchronous downloads via `require(path, {network=true})`** - blocks script until downloaded
- **Asynchronous downloads via `require_async()`** - uses callbacks
- **Hash-based update detection** - only downloads changed files
- **Hot reload integration** - downloaded updates trigger script hot reload
- **Chunked transfers with encryption** - supports large files securely

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Lua Scripts                          │
│   require("foo.lua", {network=true, reload=true})           │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                   bevy-lua-ecs Library                       │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ lua_integration.rs: require() wrapper with yields    │    │
│  │ script_cache.rs: dependency tracking & hot reload    │    │
│  │ network_asset_trait.rs: NetworkAssetRequestor trait  │    │
│  └─────────────────────────────────────────────────────┘    │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                      Hello Application                       │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ NetworkAssetPlugin (network_asset_integration.rs)    │    │
│  │   - enable_network_downloads()                       │    │
│  │   - process_download_requests()                      │    │
│  │   - resume_pending_coroutines()                      │    │
│  ├─────────────────────────────────────────────────────┤    │
│  │ PendingAssetRequests (network_asset_client.rs)       │    │
│  │   - Request queue with status tracking               │    │
│  │   - Chunked reassembly                               │    │
│  │   - Encryption/decryption                            │    │
│  ├─────────────────────────────────────────────────────┤    │
│  │ Asset Delivery Systems (asset_server_delivery.rs)    │    │
│  │   - send_asset_requests_global()                     │    │
│  │   - receive_asset_responses_global()                 │    │
│  │   - handle_asset_requests_global() [server]          │    │
│  └─────────────────────────────────────────────────────┘    │
└──────────────────────────┬──────────────────────────────────┘
                           │ Renet UDP (Channel 5)
┌──────────────────────────▼──────────────────────────────────┐
│                      Asset Server                            │
│   - Receives AssetRequestMessage                             │
│   - Compares hashes, sends UpToDate or file content          │
│   - Chunks and encrypts large files                          │
└─────────────────────────────────────────────────────────────┘
```

---

## Module Reference

### 1. `network_asset_client.rs` - Request Tracking

**Location:** `Hello/src/network_asset_client.rs`

Manages pending asset download requests and their status.

#### Key Types

| Type | Description |
|------|-------------|
| `AssetRequestStatus` | Enum: `Pending`, `Requested`, `Downloading`, `Complete`, `UpToDate`, `Error` |
| `AssetType` | Enum: `Script`, `Image`, `Binary` |
| `AssetRequestMessage` | Client→Server request with path, hash, request_id, context_path |
| `AssetResponseMessage` | Server→Client response with data, chunk info, or UpToDate flag |
| `AssetRequest` | Individual request tracking with timestamps & chunks |
| `PendingAssetRequests` | `Resource` tracking all pending requests |

#### `PendingAssetRequests` Methods

```rust
// Queue a new request, returns request ID
fn queue_request(&self, path: String, asset_type: AssetType, context_path: Option<String>) -> u64

// Check request status
fn get_status(&self, path: &str) -> Option<AssetRequestStatus>
fn is_completed(&self, path: &str) -> bool
fn is_up_to_date(&self, path: &str) -> bool

// Retrieve completed data
fn take_completed(&self, path: &str) -> Option<Vec<u8>>

// Chunk handling
fn add_chunk(&self, request_id: u64, chunk_index: u32, total_chunks: u32, total_size: usize, data: Vec<u8>) -> Option<String>
```

#### Encryption

Files are encrypted with XOR before transmission:
```rust
pub fn chunk_and_encrypt(data: &[u8], request_id: u64) -> Vec<AssetResponseMessage>
pub fn decrypt_data(encrypted_data: &[u8]) -> Option<Vec<u8>>
```

---

### 2. `asset_server_delivery.rs` - Network I/O

**Location:** `Hello/src/asset_server_delivery.rs`

Bevy systems for sending/receiving asset messages over Renet.

#### Systems

| System | Role | Runs On |
|--------|------|---------|
| `handle_asset_requests_global` | Process incoming requests, send file or UpToDate | Server |
| `send_asset_requests_global` | Send pending requests to server | Client |
| `receive_asset_responses_global` | Receive and process server responses | Client |
| `check_request_timeouts` | Mark timed-out requests as failed | Client |

#### Path Resolution

The server resolves asset paths using context:
```rust
fn resolve_asset_path(requested_path: &str, context_path: Option<&str>) -> PathBuf
```

If `context_path` is provided (e.g., the script calling `require()`), relative paths are resolved from that script's directory first.

---

### 3. `network_asset_integration.rs` - Bevy Plugin

**Location:** `Hello/src/network_asset_integration.rs`

The `NetworkAssetPlugin` integrates network downloading with Lua coroutines.

#### Plugin Systems (Update Schedule)

```rust
fn build(&self, app: &mut App) {
    app.init_resource::<PendingAssetRequests>()
       .init_resource::<PendingCoroutines>()
       .add_systems(Update, (
           enable_network_downloads,
           process_download_requests,
           crate::asset_server_delivery::send_asset_requests_global,
           crate::asset_server_delivery::receive_asset_responses_global,
           resume_pending_coroutines,
           crate::asset_server_delivery::check_request_timeouts,
       ).chain());
}
```

#### Key Systems

**`enable_network_downloads`**
- Sets `__NETWORK_DOWNLOAD_ENABLED__ = true` in Lua globals
- Signals to `require()` that network downloads are available

**`process_download_requests`**
- Reads pending download paths from `script_cache.take_pending_download_coroutines()`
- Queues them in `PendingAssetRequests` for network transmission

**`resume_pending_coroutines`**
- Checks for completed/up-to-date downloads
- Resumes Lua coroutines waiting for those paths
- Writes downloaded scripts to disk
- Emits `LuaFileChangeEvent` to trigger hot reload

---

### 4. `network_asset_trait.rs` - Interface

**Location:** `bevy-lua-ecs/src/network_asset_trait.rs`

Defines the trait for network asset loading (currently unused in favor of direct integration).

```rust
pub trait NetworkAssetRequestor: Send + Sync + 'static {
    fn is_available_locally(&self, path: &str) -> bool;
    fn queue_download(&self, path: &str) -> u64;
    fn is_download_complete(&self, path: &str) -> bool;
    fn take_downloaded_data(&self, path: &str) -> Option<Vec<u8>>;
    fn get_status(&self, path: &str) -> AssetDownloadStatus;
}
```

---

### 5. `script_cache.rs` - Hot Reload & Dependencies

**Location:** `bevy-lua-ecs/src/script_cache.rs`

Manages module caching, dependency tracking, and hot reload.

#### Dependency Tracking

```rust
// Track that importer_path imports imported_path
fn add_dependency(&self, imported_path: String, importer_path: String, should_reload: bool)

// Get all scripts that import a module
fn get_importers(&self, module_path: &str) -> Vec<(String, bool)>

// Invalidate module and all dependents (returns paths to reload)
fn invalidate_module(&self, path: &str) -> Vec<String>
```

#### Pending Download Coroutines

When `require()` needs to download a file, it registers a pending coroutine:

```rust
fn register_pending_download_coroutine(
    &self, 
    path: String, 
    coroutine_key: Arc<LuaRegistryKey>,
    instance_id: u64,
    is_binary: bool,
    context_path: Option<String>
)

fn take_pending_download_coroutines(&self, path: &str) -> Vec<(Arc<LuaRegistryKey>, u64)>
fn has_pending_download_coroutines(&self, path: &str) -> bool
```

#### Hot Reload Callbacks

For `require_async()` with `reload=true`:

```rust
fn register_hot_reload_callback(&self, path: String, callback: Arc<LuaRegistryKey>, parent_instance_id: u64)
fn get_hot_reload_callbacks(&self, path: &str) -> Vec<(Arc<LuaRegistryKey>, u64)>
```

---

## Request Flow

### Synchronous `require()` with Network

```
1. Lua: require("foo.lua", {network=true, reload=true})
2. lua_integration.rs: Check local file
3. If local exists:
   - Queue background server check (non-blocking)
   - Return local content immediately
   - Server check triggers hot reload if update found
4. If local missing:
   - Return __PENDING_DOWNLOAD__ table
   - Lua wrapper yields coroutine
5. process_download_requests: Sees pending path, queues request
6. send_asset_requests_global: Sends to server with hash
7. Server: Compares hash, returns UpToDate or file content
8. receive_asset_responses_global: Stores response
9. resume_pending_coroutines:
   - Writes file to disk
   - Emits LuaFileChangeEvent
   - Resumes coroutine
10. Module executes and returns
```

### Hot Reload When Server Has Update

```
1. Background server check finds hash mismatch
2. Server sends updated content
3. resume_pending_coroutines:
   - Writes new file to disk
   - Calls resume_coroutines_with_source() to invoke callbacks
   - Emits LuaFileChangeEvent
4. auto_reload_changed_scripts system:
   - Receives LuaFileChangeEvent
   - Calls invalidate_module() to get affected scripts
   - Cleans up old instances
   - Re-executes affected scripts
```

---

## Examples

### Asset Server (`examples/asset_server.rs`)

```bash
cargo run --example asset_server --features networking
```

- Listens on `127.0.0.1:5000`
- Serves files from `assets/` directory
- Compares client hashes, sends `UpToDate` or file content
- Supports chunked transfers for large files

### Asset Client (`examples/asset_client.rs`)

```bash
cargo run --example asset_client --features networking
```

- Connects to asset server
- Downloads and executes `scripts/examples/network_test_module.lua`
- Supports hot reload when server files change
- Demonstrates nested `require()` with network downloads

---

## Lua API

### `require(path, options)`

Synchronous module loading with optional network download.

```lua
-- Local only
local m = require("utils/helper.lua")

-- With network download (blocking)
local m = require("module.lua", {network = true})

-- With network + hot reload on updates
local m = require("module.lua", {network = true, reload = true})
```

### `require_async(path, callback, options)`

Asynchronous module loading with callback.

```lua
require_async("module.lua", function(module)
    print("Module loaded:", module.name)
end, {network = true, reload = true})
```

### `load_asset(path, options)`

Synchronous asset loading (images, etc.).

```lua
local img = load_asset("images/sprite.png", {network = true})
```

### `load_asset_async(path, callback, options)`

Asynchronous asset loading with callback.

```lua
load_asset_async("images/sprite.png", function(asset_id)
    -- Use asset_id in spawn
end, {network = true, reload = true})
```

---

## Configuration

### Channel Configuration

The asset delivery uses Renet Channel 5 with unreliable sequenced delivery:

```rust
ChannelConfig {
    channel_id: ASSET_CHANNEL, // 5
    max_memory_usage_bytes: 50 * 1024 * 1024, // 50MB
    send_type: SendType::ReliableOrdered {
        resend_time: Duration::from_millis(100),
    },
}
```

### Timeouts

- Request timeout: 30 seconds (`REQUEST_TIMEOUT_SECS`)
- Chunk size: 64KB (`CHUNK_SIZE`)

### Encryption

Simple XOR encryption with magic header (`ASET`):

```rust
const ENCRYPTION_MAGIC: [u8; 4] = [0xAE, 0x53, 0x45, 0x54];
const ENCRYPTION_KEY: [u8; 32] = [...]; // Must match server/client
```
