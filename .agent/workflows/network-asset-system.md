---
description: Network asset server system for on-demand script/asset downloads
---

# Network Asset System

On-demand asset downloading from server via Renet UDP. Scripts can `require()` files that don't exist locally - they'll download automatically.

## Key Components

| File | Purpose |
|------|---------|
| `Hello/src/network_asset_client.rs` | Request queue, status tracking, chunk reassembly, encryption |
| `Hello/src/asset_server_delivery.rs` | Bevy systems for sending/receiving over Renet |
| `Hello/src/network_asset_integration.rs` | `NetworkAssetPlugin` - ties Lua coroutines to downloads |
| `bevy-lua-ecs/src/script_cache.rs` | Dependency tracking, hot reload, pending coroutines |
| `bevy-lua-ecs/src/network_asset_trait.rs` | Interface trait (optional) |

## How It Works

1. Lua calls `require("foo.lua", {network=true})`
2. If file missing locally, Rust returns `__PENDING_DOWNLOAD__` marker
3. Lua wrapper yields coroutine
4. `process_download_requests` queues request in `PendingAssetRequests`
5. `send_asset_requests_global` sends request to server with local file hash
6. Server compares hash, sends `UpToDate` or file content (chunked, encrypted)
7. `receive_asset_responses_global` stores response
8. `resume_pending_coroutines` writes file, resumes coroutine, emits `LuaFileChangeEvent`
9. If update detected, `auto_reload_changed_scripts` hot-reloads affected scripts

## Important: Wrapper Bug Fix

The Lua require wrapper stores original Rust functions in special globals to survive hot reload:

```lua
if not __RUST_REQUIRE__ then
    __RUST_REQUIRE__ = require  -- Only first time
end
local _orig_require = __RUST_REQUIRE__
```

Without this, hot reload would capture the wrapper function, causing recursion.

## Dependency Tracking (script_cache.rs)

```rust
// Track: importer requires imported_path with reload flag
add_dependency(imported_path, importer_path, should_reload)

// Get all scripts that import a module
get_importers(module_path) -> Vec<(String, bool)>

// Clear cache and return affected scripts
invalidate_module(path) -> Vec<String>
```

## Lua API

```lua
-- Blocking download
local m = require("script.lua", {network = true, reload = true})

-- Async with callback
require_async("script.lua", function(mod) end, {network = true, reload = true})

-- Assets
local id = load_asset("img.png", {network = true})
load_asset_async("img.png", function(id) end, {network = true, reload = true})
```

## Running Examples

```bash
# Terminal 1 - Start server
cargo run --example asset_server --features networking

# Terminal 2 - Start client
cargo run --example asset_client --features networking
```

Server serves `assets/` directory. Client downloads and executes `scripts/examples/network_test_module.lua`.

## Configuration

- Channel: Renet Channel 5
- Chunk size: 64KB
- Timeout: 30s
- Encryption: XOR with magic header `ASET`
