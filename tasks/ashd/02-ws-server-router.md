# Phase 2: ashd WebSocket Server + Frame Router

## Context

This is step 2 of 5. Phase 1 created the workspace and config. Now we build the WebSocket server that ash processes connect to for telemetry and control. See `tasks/ash-ashd-protocol.md` for the full protocol spec — all frames use 3-letter uppercase verb prefixes separated by `:`.

## Task

Implement the WebSocket server, frame router, process registry, connection map, and relay pipeline stub.

### Protocol Frame Format

All messages are WebSocket text frames. Every frame starts with a 3-letter uppercase verb followed by `:` and an optional body.

```
VERB:body content here
```

**ash → ashd frames:**
| Frame | Format | Notes |
|-------|--------|-------|
| `TEL:` | `TEL:{opaque bytes}` | Body is opaque JSON — ashd never parses it |
| `STA:` | `STA:<instance_id>:<status>:<workflow_path>` | Status: `running`, `paused`, `completed`, `failed`, `cancelled` |

**ashd → ash frames:**
| Frame | Format | Notes |
|-------|--------|-------|
| `ABT:` | `ABT:` | No body — 1:1 connection |
| `PAU:` | `PAU:` | No body — 1:1 connection |
| `RES:` | `RES:` | No body — 1:1 connection |

Unknown verbs are silently ignored.

### 1. WebSocket Server (`ashd/src/ws.rs`)

Create a `WsServer` struct:

```rust
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

pub struct WsServer {
    listen_addr: String,
    frame_tx: mpsc::Sender<IncomingFrame>,
}
```

`WsServer::start(self)`:
- Bind `TcpListener` to `self.listen_addr`
- Accept loop: for each connection, spawn a tokio task
- Each connection task: upgrade to WebSocket, read loop
- Each incoming text frame → parse verb + body → send `IncomingFrame` on `frame_tx`
- On connection close: send `IncomingFrame { verb: "DISCONNECT", instance_id: None, body: String::new() }`
- On read error: log warning, close connection

`IncomingFrame` struct (in `ashd/src/router.rs`):

```rust
pub struct IncomingFrame {
    pub verb: String,
    pub instance_id: Option<String>,
    pub body: String,
    pub reply_tx: tokio::sync::mpsc::UnboundedSender<String>, // for ashd→ash frames
}
```

The `reply_tx` is held per-connection to send `ABT:`, `PAU:`, `RES:` back to ash. Parse `instance_id` from `STA:` frames (second colon-separated field). For `TEL:`, `instance_id` is `None`.

Path for `workflow_path` in STA frames: if the path contains colons (Windows paths), use the rest of the string after instance_id:status as the path. On Unix this is a non-issue since paths don't contain colons in the common case.

### 2. Frame Router (`ashd/src/router.rs`)

```rust
pub struct FrameRouter {
    relay_pipeline: Arc<RelayPipeline>,
    process_registry: Arc<ProcessRegistry>,
    connection_map: Arc<ConnectionMap>,
}
```

`FrameRouter::run(self, mut frame_rx: mpsc::Receiver<IncomingFrame>)`:
- Loop receiving frames
- Match on `verb.as_str()`:
  - `"TEL"` → `relay_pipeline.forward(&frame.body)`
  - `"STA"` → parse instance_id, status, workflow_path → `process_registry.update(instance_id, status, workflow_path, frame.reply_tx)`; also register in `connection_map`
  - `"ABT" | "PAU" | "RES"` → lookup instance_id in connection_map, send verb frame back (these are ashd→ash; in MVP no external controller sends them — the routing is in place for future use)
  - `"DISCONNECT"` → if instance_id present, `connection_map.remove(instance_id)`, `process_registry.disconnect(instance_id)`
  - `_` → `log::debug!("ignoring unknown frame: {}", verb)`

All arms are non-blocking. Unknown verbs are silently ignored.

### 3. Process Registry (`ashd/src/registry.rs`)

```rust
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc;

pub struct ProcessRegistry {
    inner: Mutex<HashMap<String, ProcessState>>,
}

pub struct ProcessState {
    pub status: String,
    pub workflow_path: String,
    pub last_seen: std::time::Instant,
    pub reply_tx: Option<mpsc::UnboundedSender<String>>,
}
```

Methods:
- `update(instance_id, status, workflow_path, reply_tx)` — insert or update entry
- `disconnect(instance_id)` — mark the reply channel as closed
- `get(instance_id) -> Option<ProcessState>` — read-only lookup
- `is_source_busy(source_name) -> bool` — check if any process from a given source is `running` or `paused` (for sequential concurrency gating — used in Phase 3b)
- `source_from_instance(instance_id) -> Option<String>` — map instance_id back to source name (store source_name in a side map populated by the spawner in Phase 3b; for now just store and return)

The registry is the sole authority on process lifecycle within ashd.

### 4. Connection Map (`ashd/src/connection.rs`)

```rust
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc;

pub struct ConnectionMap {
    inner: Mutex<HashMap<String, mpsc::UnboundedSender<String>>>,
}
```

Methods:
- `register(instance_id, reply_tx)` — store the sender
- `remove(instance_id)` — remove and drop sender
- `send(instance_id, frame: &str) -> Result<()>` — send a frame to the ash process (used for ABT/PAU/RES)

### 5. Relay Pipeline Stub (`ashd/src/relay.rs`)

```rust
pub struct RelayPipeline {
    // Phase 4 will add KafkaSink, SplunkSink here
}
```

`RelayPipeline::new(config: &TelemetryRelayConfig) -> Self` — accepts config but does nothing with it yet.

`RelayPipeline::forward(&self, body: &str)` — log at debug level: `log::debug!("TEL relay: {}", body)`. This stub is replaced in Phase 4 with real Kafka/Splunk delivery.

### 6. Wire up in `ashd/src/main.rs`

Replace the Phase 1 main with:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    let config = AshdConfig::load()?;
    log::info!("config loaded, {} source(s)", config.sources.len());
    
    // Create shared components
    let (frame_tx, frame_rx) = mpsc::channel(1024);
    let connection_map = Arc::new(ConnectionMap::new());
    let process_registry = Arc::new(ProcessRegistry::new());
    let relay_pipeline = Arc::new(RelayPipeline::new(config.telemetry_relay.as_ref()));
    
    // Start WebSocket server
    let ws_server = WsServer::new(config.daemon.ws_listen.clone(), frame_tx.clone());
    let ws_handle = tokio::spawn(async move { ws_server.start().await });
    
    // Start frame router
    let router = FrameRouter::new(relay_pipeline, process_registry.clone(), connection_map.clone());
    let router_handle = tokio::spawn(async move { router.run(frame_rx).await });
    
    // Keep alive (Phase 3b will add the event channel + watchers)
    log::info!("ashd listening on {}", config.daemon.ws_listen);
    tokio::signal::ctrl_c().await?;
    log::info!("shutting down");
    
    // Graceful shutdown
    ws_handle.abort();
    router_handle.abort();
    
    Ok(())
}
```

Handle `Result<()>` on aborts — ignore JoinError from abort.

### 7. Module structure

```
ashd/src/
  main.rs
  config.rs          (from Phase 1, may need minor updates)
  ws.rs              (WebSocket server — new)
  router.rs          (FrameRouter + IncomingFrame — new)
  registry.rs        (ProcessRegistry — new)
  connection.rs      (ConnectionMap — new)
  relay.rs           (RelayPipeline stub — new)
```

### Acceptance

1. Start ashd: `cargo run -p ashd` — prints "listening on 127.0.0.1:9877"
2. Connect with `websocat ws://127.0.0.1:9877`
3. Send: `STA:abc123:running:/tmp/test-workflow` → ashd log shows registry update
4. Send: `TEL:{"trace_id":1,"kind":"AgentCall","payload":{}}` → ashd log shows "TEL relay: ..."
5. Send: `STA:abc123:completed:/tmp/test-workflow` → ashd logs status change
6. Close connection → ashd logs disconnect, removes from registry
7. Send unknown frame `FOO:bar` → ashd does NOT crash, logs debug
8. Send `TEL:` with empty body → handled gracefully (empty forward)
9. Send malformed `STA:onlyonefield` → handled gracefully (log warning, don't panic)

### Do NOT do

- Do not start any folder watchers or event sources
- Do not spawn child processes
- Do not implement real Kafka/Splunk delivery (stub only)
- Do not modify ash or ash-wasm code
- Do not implement the control frame sending logic (ABT/PAU/RES from ashd→ash is wired but never triggered — it waits for Phase 3b+ or a future GUI)
