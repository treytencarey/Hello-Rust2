---
description: Network asset server system for on-demand script/asset downloads
---

# Network Asset System

On-demand asset downloading from server via Renet UDP. Scripts can `require()` files that don't exist locally - they'll download automatically. Server pushes updates when subscribed files change.

## Key Components

| File | Purpose |
|------|---------|
| `Hello/src/network_asset_client.rs` | Message types, request queue, chunk reassembly, encryption |
| `Hello/src/asset_server_delivery.rs` | Server request handling, broadcast updates, client cleanup |
| `Hello/src/network_asset_integration.rs` | `NetworkAssetPlugin` - ties Lua coroutines to downloads |
| `Hello/src/subscription_registry.rs` | File watcher, client subscription tracking (server-side) |
| `bevy-lua-ecs/src/script_cache.rs` | Dependency tracking, subscription marking, pending coroutines |

## Message Types (network_asset_client.rs)

Two wrapper enums ensure proper bincode deserialization:

```rust
// Client → Server
enum ClientToServerMessage {
    Request(AssetRequestMessage),    // Initial file download
    Subscription(AssetSubscriptionMessage), // Subscribe/unsubscribe
}

// Server → Client
enum ServerToClientMessage {
    Response(AssetResponseMessage),  // Reply to request
    Update(AssetUpdateNotification), // Push when file changes
}
```

**Why wrappers?** Bincode greedily deserializes similar structs. Without explicit tagging, `AssetRequestMessage` could parse as `AssetSubscriptionMessage`.

## Flow: Initial Download

1. Lua calls `require("foo.lua", {network=true, reload=true})`
2. File missing → Rust returns `__PENDING_DOWNLOAD__`
3. Lua wrapper yields coroutine
4. `register_pending_download_coroutine` queues request and marks for subscription if `reload=true`
5. `send_asset_requests_global` sends `ClientToServerMessage::Request`
6. Server compares hash → sends `ServerToClientMessage::Response` (UpToDate or file chunks)
7. `receive_asset_responses_global` deserializes wrapper, routes to `process_asset_response`
8. `resume_pending_coroutines` writes file, resumes coroutine

## Flow: File Sync (Hot Reload)

1. Client sends `ClientToServerMessage::Subscription` with paths and `subscribe: true`
2. Server's `handle_asset_requests_global` registers in `AssetSubscriptionRegistry`
3. Server's `FileWatcherResource` watches file via `notify` crate
4. File modified → `poll_changes()` returns changed paths (canonicalized prefix stripped)
5. `broadcast_file_updates` sends `ServerToClientMessage::Update` to all subscribers
6. Client's `receive_asset_responses_global` queues update in `PendingAssetUpdates`
7. `receive_asset_updates` processes queue: decrypt, write to disk, trigger `LuaFileChangeEvent`
8. `auto_reload_changed_scripts` hot-reloads affected scripts

## Subscription Registry (subscription_registry.rs)

Server-side tracking of which clients want updates for which files:

```rust
// Structure: path → (client_id → set of instance_ids)
subscriptions: HashMap<String, HashMap<u64, HashSet<u64>>>

// Key methods:
subscribe(client_id, path, instance_id) → bool  // Returns true if first subscriber
unsubscribe_client(client_id) → Vec<String>     // Returns paths with no subscribers
get_subscribers(path) → Vec<u64>                // Client IDs for broadcast
```

## FileWatcherResource (subscription_registry.rs)

Wraps `notify` crate for file system monitoring:

```rust
watch(path: &str)     // Start watching (canonicalizes path)
unwatch(path: &str)   // Stop watching
poll_changes() → Vec<String>  // Non-blocking, returns relative paths
```

**Important:** Uses `PathBuf::canonicalize()` to normalize paths. `poll_changes()` strips the canonical assets prefix.

## Client Disconnect Cleanup

`cleanup_disconnected_clients` system detects when clients leave:

1. Compares current `server.clients_id()` to `ConnectedClients` resource
2. For each disconnected client, calls `registry.unsubscribe_client(client_id)`
3. Unwatches paths that have no remaining subscribers

## Script Cache Subscription (script_cache.rs)

Client-side subscription tracking:

```rust
// Mark path for subscription when download completes
mark_for_subscription(path, instance_id)

// Get paths needing subscription messages sent
take_pending_subscriptions() → HashMap<instance_id, Vec<paths>>

// Called when download completes with reload=true
register_pending_download_coroutine(..., should_subscribe: bool)
```

## Important: Lua Wrapper Bug Fix

The Lua require wrapper stores original Rust functions in globals to survive hot reload:

```lua
if not __RUST_REQUIRE__ then
    __RUST_REQUIRE__ = require  -- Only first time
end
local _orig_require = __RUST_REQUIRE__
```

Without this, hot reload captures the wrapper, causing infinite recursion.

## Lua API

```lua
-- Blocking download + subscribe for updates
local m = require("script.lua", {network = true, reload = true})

-- Async with callback + subscribe
require_async("script.lua", function(mod) end, {network = true, reload = true})

-- Assets
local id = load_asset("img.png", {network = true, reload = true})
load_asset_async("img.png", function(id) end, {network = true, reload = true})
```

## Running Examples

```bash
# Terminal 1 - Server (serves assets/ directory)
cargo run --example asset_server --features networking

# Terminal 2 - Client
cargo run --example asset_client --features networking
```

## Configuration

- Channel: Renet Channel 5 (`ASSET_CHANNEL`)
- Chunk size: 64KB
- Timeout: 30s
- Encryption: XOR with magic header `ASET`

## System Registration (asset_server)

```rust
app.init_resource::<AssetSubscriptionRegistry>()
   .insert_resource(FileWatcherResource::new())
   .init_resource::<ConnectedClients>()
   .add_systems(Update, (
       handle_asset_requests_global,     // Process requests & subscriptions
       broadcast_file_updates,           // Push file changes
       cleanup_disconnected_clients,     // Remove stale subscriptions
   ));
```

## System Registration (asset_client via NetworkAssetPlugin)

```rust
app.init_resource::<PendingAssetRequests>()
   .init_resource::<PendingAssetUpdates>()
   .init_resource::<PendingCoroutines>()
   .add_systems(Update, (
       process_download_requests,        // Queue awaiting downloads
       send_asset_requests_global,       // Send to server
       receive_asset_responses_global,   // Handle responses + updates
       resume_pending_coroutines,        // Complete downloads
       send_subscription_messages,       // Send subscribe/unsubscribe
       receive_asset_updates,            // Process queued file changes
   ));
```
