# Network Asset Server System

This document describes the on-demand asset downloading system for the Bevy + Lua project.

## Overview

The network asset system allows Lua scripts to download assets (scripts, images, etc.) from a server on-demand. Key features:
- **Synchronous downloads via `require(path, {network=true})`** - blocks script until downloaded
- **Asynchronous downloads via `require_async()`** - uses callbacks
- **Hash-based update detection** - only downloads changed files
- **File sync subscriptions** - server pushes updates when files change
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
│  │ script_cache.rs: dependency tracking, subscriptions  │    │
│  │ network_asset_trait.rs: NetworkAssetRequestor trait  │    │
│  └─────────────────────────────────────────────────────┘    │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                      Hello Application                       │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ NetworkAssetPlugin (network_asset_integration.rs)    │    │
│  │   - process_download_requests()                      │    │
│  │   - resume_pending_coroutines()                      │    │
│  │   - send_subscription_messages()                     │    │
│  │   - receive_asset_updates()                          │    │
│  ├─────────────────────────────────────────────────────┤    │
│  │ PendingAssetRequests (network_asset_client.rs)       │    │
│  │   - Request queue with status tracking               │    │
│  │   - Message types with wrapper enums                 │    │
│  │   - Chunked reassembly, encryption/decryption        │    │
│  ├─────────────────────────────────────────────────────┤    │
│  │ Asset Delivery Systems (asset_server_delivery.rs)    │    │
│  │   - send_asset_requests_global() [client]            │    │
│  │   - receive_asset_responses_global() [client]        │    │
│  │   - handle_asset_requests_global() [server]          │    │
│  │   - broadcast_file_updates() [server]                │    │
│  │   - cleanup_disconnected_clients() [server]          │    │
│  ├─────────────────────────────────────────────────────┤    │
│  │ Subscription Registry (subscription_registry.rs)     │    │
│  │   - AssetSubscriptionRegistry: client→path tracking  │    │
│  │   - FileWatcherResource: notify crate integration    │    │
│  └─────────────────────────────────────────────────────┘    │
└──────────────────────────┬──────────────────────────────────┘
                           │ Renet UDP (Channel 5)
┌──────────────────────────▼──────────────────────────────────┐
│                      Asset Server                            │
│   - Receives ClientToServerMessage (Request/Subscription)    │
│   - Sends ServerToClientMessage (Response/Update)            │
│   - Watches files, broadcasts changes to subscribers         │
└─────────────────────────────────────────────────────────────┘
```

---

## Message Types

### Wrapper Enums

Bincode greedily deserializes similar structs. Without explicit tagging, messages can be misparsed. Wrapper enums solve this:

#### Client → Server

```rust
#[derive(Serialize, Deserialize)]
pub enum ClientToServerMessage {
    /// Initial file download request
    Request(AssetRequestMessage),
    /// Subscribe/unsubscribe for file updates
    Subscription(AssetSubscriptionMessage),
}
```

#### Server → Client

```rust
#[derive(Serialize, Deserialize)]
pub enum ServerToClientMessage {
    /// Reply to a request (file content or UpToDate)
    Response(AssetResponseMessage),
    /// Push notification when a subscribed file changes
    Update(AssetUpdateNotification),
}
```

### Message Structs

| Message | Direction | Purpose |
|---------|-----------|---------|
| `AssetRequestMessage` | Client→Server | Request file with local hash for comparison |
| `AssetSubscriptionMessage` | Client→Server | Subscribe/unsubscribe paths |
| `AssetResponseMessage` | Server→Client | File content (chunked) or UpToDate flag |
| `AssetUpdateNotification` | Server→Client | Push updated file to subscriber |

---

## Module Reference

### 1. `network_asset_client.rs` - Message Types & Request Tracking

**Location:** `Hello/src/network_asset_client.rs`

Defines all network message types and manages pending requests.

#### Key Types

| Type | Description |
|------|-------------|
| `AssetRequestStatus` | Enum: `Pending`, `Requested`, `Downloading`, `Complete`, `UpToDate`, `Error` |
| `AssetType` | Enum: `Script`, `Image`, `Binary` |
| `ClientToServerMessage` | Wrapper for client-to-server messages |
| `ServerToClientMessage` | Wrapper for server-to-client messages |
| `PendingAssetRequests` | Resource tracking all pending requests |
| `PendingAssetUpdates` | Resource queuing received update notifications |

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

#### `PendingAssetUpdates` Resource

Queues file update notifications between systems:

```rust
// Queue an update for processing
fn queue(&self, notification: AssetUpdateNotification)

// Take all pending updates
fn take_all(&self) -> Vec<AssetUpdateNotification>
```

#### Encryption

Files are encrypted with XOR before transmission:
```rust
pub fn chunk_and_encrypt(data: &[u8]) -> Vec<Vec<u8>>
pub fn decrypt_data(encrypted_data: &[u8]) -> Result<Vec<u8>, String>
```

---

### 2. `subscription_registry.rs` - Server-Side Subscription Tracking

**Location:** `Hello/src/subscription_registry.rs`

Tracks which clients are subscribed to which files and manages file watching.

#### FileWatcherResource

Wraps the `notify` crate for file system monitoring:

```rust
pub struct FileWatcherResource {
    watcher: Option<RecommendedWatcher>,
    rx: Arc<Mutex<Option<Receiver<notify::Result<Event>>>>>,
    watched_paths: HashSet<PathBuf>,
}
```

**Methods:**

| Method | Description |
|--------|-------------|
| `watch(path: &str)` | Start watching a file (canonicalizes path) |
| `unwatch(path: &str)` | Stop watching a file |
| `poll_changes() -> Vec<String>` | Non-blocking, returns changed paths relative to assets/ |

**Important:** Uses `PathBuf::canonicalize()` to normalize paths. `poll_changes()` calculates the canonical assets prefix to properly strip absolute paths.

#### AssetSubscriptionRegistry

Tracks client subscriptions:

```rust
// Structure: path → (client_id → set of instance_ids)
subscriptions: HashMap<String, HashMap<u64, HashSet<u64>>>
```

**Methods:**

```rust
// Subscribe a client to a path, returns true if first subscriber
fn subscribe(&self, client_id: u64, path: &str, instance_id: u64) -> bool

// Unsubscribe a specific instance
fn unsubscribe(&self, client_id: u64, path: &str, instance_id: u64) -> bool

// Unsubscribe all paths for a client (on disconnect)
fn unsubscribe_client(&self, client_id: u64) -> Vec<String>

// Get all client IDs subscribed to a path
fn get_subscribers(&self, path: &str) -> Vec<u64>
```

---

### 3. `asset_server_delivery.rs` - Network I/O

**Location:** `Hello/src/asset_server_delivery.rs`

Bevy systems for sending/receiving asset messages over Renet.

#### Server Systems

| System | Description |
|--------|-------------|
| `handle_asset_requests_global` | Receive `ClientToServerMessage`, process requests & subscriptions |
| `broadcast_file_updates` | Poll file watcher, send updates to subscribers |
| `cleanup_disconnected_clients` | Remove subscriptions for disconnected clients |

#### Client Systems

| System | Description |
|--------|-------------|
| `send_asset_requests_global` | Send pending requests to server |
| `receive_asset_responses_global` | Receive `ServerToClientMessage`, route to handler or queue |
| `check_request_timeouts` | Mark timed-out requests as failed |

#### ConnectedClients Resource

Tracks connected clients for disconnect detection:

```rust
#[derive(Resource, Default)]
pub struct ConnectedClients {
    clients: HashSet<u64>,
}
```

#### Path Resolution

The server resolves asset paths using context:
```rust
fn resolve_asset_path(requested_path: &str, context_path: Option<&str>) -> PathBuf
```

If `context_path` is provided (e.g., the script calling `require()`), relative paths are resolved from that script's directory first.

---

### 4. `network_asset_integration.rs` - Bevy Plugin

**Location:** `Hello/src/network_asset_integration.rs`

The `NetworkAssetPlugin` integrates network downloading with Lua coroutines.

#### Plugin Registration

```rust
fn build(&self, app: &mut App) {
    // Client resources
    app.init_resource::<PendingAssetRequests>()
       .init_resource::<PendingAssetUpdates>()
       .init_resource::<PendingCoroutines>();
    
    // Server resources
    app.init_resource::<AssetSubscriptionRegistry>()
       .insert_resource(FileWatcherResource::new());
    
    // Systems
    app.add_systems(Update, (
        process_download_requests,
        send_asset_requests_global,
        receive_asset_responses_global,
        resume_pending_coroutines,
        send_subscription_messages,
        receive_asset_updates,
    ));
}
```

#### Key Systems

**`process_download_requests`**
- Reads pending download paths from `script_cache.take_pending_download_coroutines()`
- Queues them in `PendingAssetRequests` for network transmission

**`resume_pending_coroutines`**
- Checks for completed/up-to-date downloads
- Resumes Lua coroutines waiting for those paths
- Writes downloaded scripts to disk
- Emits `LuaFileChangeEvent` to trigger hot reload

**`send_subscription_messages`**
- Reads pending subscriptions from `script_cache.take_pending_subscriptions()`
- Sends `ClientToServerMessage::Subscription` to server

**`receive_asset_updates`**
- Reads from `PendingAssetUpdates` queue
- Decrypts and writes updated files to disk
- Updates source cache and triggers `LuaFileChangeEvent`

---

### 5. `script_cache.rs` - Hot Reload & Dependencies

**Location:** `bevy-lua-ecs/src/script_cache.rs`

Manages module caching, dependency tracking, and subscription marking.

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
    context_path: Option<String>,
    should_subscribe: bool,  // NEW: marks for subscription
)

fn take_pending_download_coroutines(&self, path: &str) -> Vec<(Arc<LuaRegistryKey>, u64)>
```

#### Subscription Marking

```rust
// Mark a path for subscription (called when should_subscribe=true)
fn mark_for_subscription(&self, path: String, instance_id: u64)

// Get paths needing subscription messages
fn take_pending_subscriptions(&self) -> HashMap<u64, Vec<String>>
```

---

## Request Flows

### Flow 1: Initial Download

```
1. Lua: require("foo.lua", {network=true, reload=true})
2. lua_integration.rs: Check local file
3. If local missing:
   - Return __PENDING_DOWNLOAD__ table
   - Call register_pending_download_coroutine(..., should_subscribe=true)
   - Lua wrapper yields coroutine
4. process_download_requests: Sees pending path, queues request
5. send_asset_requests_global: Sends ClientToServerMessage::Request
6. Server: Compares hash, wraps response in ServerToClientMessage::Response
7. receive_asset_responses_global: Deserializes wrapper, calls process_asset_response
8. resume_pending_coroutines:
   - Writes file to disk
   - Resumes coroutine
   - Emits LuaFileChangeEvent
9. Module executes and returns
```

### Flow 2: File Sync Subscription

```
1. After download completes, mark_for_subscription queued the path
2. send_subscription_messages: Sends ClientToServerMessage::Subscription
3. Server handle_asset_requests_global:
   - Receives Subscription message
   - Calls registry.subscribe(client_id, path, instance_id)
   - If first subscriber, calls file_watcher.watch(path)
4. Client is now subscribed to updates
```

### Flow 3: Server Push Update

```
1. User edits file on server (e.g., network_test_module.lua)
2. notify crate detects change, sends event to FileWatcherResource
3. broadcast_file_updates:
   - poll_changes() returns changed paths (canonicalized prefix stripped)
   - registry.get_subscribers(path) returns subscribed client IDs
   - Reads file, chunks, encrypts
   - Sends ServerToClientMessage::Update to each subscriber
4. receive_asset_responses_global:
   - Deserializes wrapper as Update variant
   - Queues in PendingAssetUpdates
5. receive_asset_updates:
   - Takes from queue
   - Decrypts and writes to disk
   - Updates source cache
   - Emits LuaFileChangeEvent
6. auto_reload_changed_scripts:
   - Receives LuaFileChangeEvent
   - Invalidates module cache
   - Re-executes affected scripts
```

### Flow 4: Client Disconnect Cleanup

```
1. Client disconnects (network drop, close window)
2. cleanup_disconnected_clients system:
   - Compares current server.clients_id() to ConnectedClients resource
   - Finds missing client IDs
   - Calls registry.unsubscribe_client(client_id)
   - For paths with no remaining subscribers, calls file_watcher.unwatch(path)
3. Prevents "Tried to send to invalid client" errors
```

---

## Examples

### Asset Server (`examples/asset_server.rs`)

```bash
cargo run --example asset_server --features networking
```

Setup:
```rust
app.init_resource::<AssetSubscriptionRegistry>()
   .insert_resource(FileWatcherResource::new())
   .init_resource::<ConnectedClients>()
   .add_systems(Update, (
       handle_asset_requests_global,
       broadcast_file_updates,
       cleanup_disconnected_clients,
   ));
```

- Listens on `127.0.0.1:5000`
- Serves files from `assets/` directory
- Watches subscribed files for changes
- Broadcasts updates to subscribers

### Asset Client (`examples/asset_client.rs`)

```bash
cargo run --example asset_client --features networking
```

- Connects to asset server
- Downloads and executes `scripts/examples/network_test_module.lua`
- Subscribes to files when `reload=true`
- Receives push updates and hot reloads

---

## Lua API

### `require(path, options)`

Synchronous module loading with optional network download.

```lua
-- Local only
local m = require("utils/helper.lua")

-- With network download (blocking)
local m = require("module.lua", {network = true})

-- With network + subscribe for updates
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
local img = load_asset("images/sprite.png", {network = true, reload = true})
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

The asset delivery uses Renet Channel 5 with reliable ordered delivery:

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

---

## Important: Lua Wrapper Bug Fix

The Lua require wrapper stores original Rust functions in globals to survive hot reload:

```lua
if not __RUST_REQUIRE__ then
    __RUST_REQUIRE__ = require  -- Only first time
end
local _orig_require = __RUST_REQUIRE__
```

Without this, hot reload would capture the wrapper function, causing infinite recursion.
