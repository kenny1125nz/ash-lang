# Phase 3a: ash WebSocket Client + Telemetry Forwarding + Control State

## Context

This is step 3 of 5. Phase 2 gave us an ashd WebSocket server. Runs in parallel with Phase 3b (folder watcher). Now ash (the CLI) gains a WebSocket client that forwards telemetry in real time to ashd and reads control frames. The existing `deliver_remote()` stub and the remote consumer thread are removed — ashd owns remote delivery.

Key identity model: ash uses `ASH_EVENT_ID` as its `instance_id` when spawned by ashd, otherwise generates a UUID. The `workflow_path` is the path to the workflow script/directory being executed.

## Task

### 1. Extend TelemetryConfig (`ash/src/telemetry/config.rs`)

Add `ws_url: Option<String>` field:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub file: Option<FileConfig>,
    pub filter: Option<String>,
    #[serde(default)]
    pub remote: HashMap<String, RemoteAdapterConfig>,
    #[serde(default)]
    pub ws_url: Option<String>,
    // ... existing fields
}
```

The `remote` field and `RemoteAdapterConfig` struct are **removed entirely** (Phase 1 of ashd added equivalent config there). But do this carefully — check all references first. If removing `remote`/`RemoteAdapterConfig` causes too much churn, just mark them `#[serde(default)]` and stop using them (keep struct defs but deprecate — the clean removal happens when we delete `spawn_remote_consumer`).

### 2. WebSocket Client (`ash/src/telemetry/client.rs`)

Create a new module:

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread;
use std::time::Duration;

pub struct WsClient {
    instance_id: String,
    workflow_path: String,
    control_state: Arc<AtomicU8>,
}

/// Control states: 0 = normal, 1 = abort, 2 = paused
pub const STATE_NORMAL: u8 = 0;
pub const STATE_ABORT: u8 = 1;
pub const STATE_PAUSED: u8 = 2;
```

**`WsClient::connect()` logic (runs in a dedicated thread):**

1. **Resolve ashd address** in order:
   - `std::env::var("ASHD_WS_URL")` — set by ashd when spawning, highest priority
   - `config.ws_url` from telemetry config
   - Default: `ws://127.0.0.1:9877`

2. **Connect** using `tungstenite::connect()` (sync API — ash is not tokio-based). Use the `tungstenite` crate (add to `ash/Cargo.toml`).

3. **On connect success:**
   - Send `STA:<instance_id>:running:<workflow_path>`
   - Create a `crossbeam::channel` or `std::sync::mpsc` bounded channel (capacity 256) for outgoing telemetry
   - Spawn two sub-threads or use non-blocking alternation:
     - **Reader thread**: loop `read_message()`, parse control frames (`ABT:`, `PAU:`, `RES:`), update `control_state`
     - **Writer thread**: loop receiving from the telemetry channel, send `TEL:{json}` frames

   Actually, since ash is sync and `tungstenite`'s read/write are on the same `WebSocket` stream, use the approach of:
   - Main WS thread: loop doing `set_nonblocking(true)` on the TCP stream, then alternate reads and channel receives
   - Or simpler: split the WebSocket into read/write halves (if tungstenite supports it)
   - Or simplest: use a dedicated writer thread that holds a `Mutex<WebSocket<...>>` shared with the reader thread

   Preferred approach (simplest, matches ash's sync style):
   - One thread owns the WebSocket
   - It does `set_nonblocking(true)` and a tight loop with `std::thread::sleep(Duration::from_millis(10))`
   - It polls `ws.read_message()` (non-blocking) for control frames
   - It polls `rx.try_recv()` for outgoing telemetry

4. **Control frame handling:**
   - `ABT:` → set `control_state` to `STATE_ABORT`
   - `PAU:` → set `control_state` to `STATE_PAUSED`
   - `RES:` → set `control_state` to `STATE_NORMAL`
   - On pause→resume transition: send `STA:<instance_id>:running:<workflow_path>`
   - On abort: send `STA:<instance_id>:cancelled:<workflow_path>`, then exit

5. **On disconnect:**
   - Set a flag "disconnected = true"
   - Telemetry falls back to JSONL (Phase 3.5 below)
   - Retry with exponential backoff: 1s, 2s, 4s, 8s, 16s, cap at 30s
   - On reconnect: send `STA:<instance_id>:running:<workflow_path>` followed by `STA:<instance_id>:<current_status>:<workflow_path>` (current status is `paused` if control_state == 2, `running` otherwise)

6. **On clean shutdown:**
   - Send `STA:<instance_id>:completed:<workflow_path>` (or `failed`/`cancelled` depending on outcome)
   - Close WebSocket normally
   - Join the thread

### 3. Instance ID Generation

`ash/src/telemetry/mod.rs` gains:

```rust
pub fn instance_id() -> &str { ... }
pub fn set_instance_id(id: String) { ... }
pub fn workflow_path() -> &str { ... }
pub fn set_workflow_path(path: String) { ... }
```

When spawned by ashd, ash reads `ASH_EVENT_ID` env var — that IS the `instance_id`. When run manually, ash generates a UUID v4. Store in a `OnceLock<String>`.

The workflow path is the argument passed to ash (script file or directory). Set it in `main.rs` before telemetry init.

### 4. Telemetry Forwarding

Modify `ash/src/telemetry/mod.rs`:

The `Pipeline` struct gains a `ws_tx: Option<std::sync::mpsc::Sender<String>>`. When `init()` is called:
- Create the channel
- Spawn the `WsClient::connect()` thread in a `std::thread::spawn()`
- Store the sender in the pipeline

`emit()`:
- If `ws_tx` is `Some` and the channel is not full: `ws_tx.try_send(json_string)` — real-time forwarding
- If `ws_tx` is `None` or channel is full or send fails: write to local JSONL (existing behavior)

The local JSONL file is always written as well (it's the durable log). The WS channel is best-effort.

### 5. Remove deliver_remote() and Remote Consumer Thread

In `ash/src/telemetry/pipeline.rs`:

- Remove `fn deliver_remote()` (lines 204-207)
- Remove `fn spawn_remote_consumer()` (lines 116-202)
- Remove fields from `Pipeline`: `remote_running`, `remote_handle`
- Clean up `start()` to not call `spawn_remote_consumer()`

In `ash/src/telemetry/config.rs`:

- Remove `RemoteAdapterConfig` struct entirely
- Remove `remote: HashMap<String, RemoteAdapterConfig>` from `TelemetryConfig`

In `ash/src/telemetry/mod.rs`:

- `shutdown()` still flushes the JSONL file and signals the WS thread to close (sentinel message or atomic flag)

### 6. Control State in Runtime

`ash/src/telemetry/mod.rs` exports:

```rust
pub fn control_state() -> Arc<AtomicU8> { ... }
```

The `Evaluator` in `ash/src/eval/mod.rs` holds `control_state: Arc<AtomicU8>` (populated from telemetry during init).

In the execution loop (`eval_statement`, `eval_script`, and the task tree `run_tree`), before each step:

```rust
match self.control_state.load(Ordering::Relaxed) {
    STATE_ABORT => return Err(EvalError::Msg("aborted by ashd".into())),
    STATE_PAUSED => {
        while self.control_state.load(Ordering::Relaxed) == STATE_PAUSED {
            std::thread::sleep(Duration::from_millis(100));
        }
        // After resume, check again (could have been aborted)
        if self.control_state.load(Ordering::Relaxed) == STATE_ABORT {
            return Err(EvalError::Msg("aborted by ashd".into()));
        }
    }
    _ => {} // normal
}
```

For `eval_agent_call` and `eval_exec` (subprocess spawns): check before spawning and after completion. During a running subprocess, pause/abort are checked at the next iteration.

For task tree: check before processing each task group.

### 7. Wire up in main.rs (`ash/src/main.rs`)

In `init_telemetry()` or equivalent:
- Set `instance_id` and `workflow_path` before `telemetry::init()`
- Read `ASH_EVENT_ID` from env; if present use as instance_id, else generate UUID

In `ensure_agents_registered()`:
- Pass instance_id to telemetry init

### 8. Dependencies to add to `ash/Cargo.toml`

```toml
tungstenite = "0.24"
url = "2"
uuid = { version = "1", features = ["v4"] }
```

### Acceptance

1. Start ashd (Phase 2), then run `ash -c '!echo hello'` — ashd logs `TEL:` frames arriving
2. ashd logs `STA:<id>:running:<path>` when ash starts
3. ashd logs `STA:<id>:completed:<path>` when ash exits cleanly
4. Kill ashd while ash is running — ash logs warning, continues executing, writes to JSONL
5. Restart ashd — ash reconnects (observe backoff in logs), resumes TEL forwarding
6. Run ash without ashd running — ash logs warning, falls back to JSONL, no crash
7. `ASH_EVENT_ID=evt-001 ash -c '!echo hello'` — instance_id in STA frames is `evt-001`
8. Run ash without ASHD_WS_URL — uses default `ws://127.0.0.1:9877`
9. Control state check in execution loop: hard to test manually (no controller sends ABT/PAU yet), but verify the `Arc<AtomicU8>` is wired, and the check compiles.

### Do NOT do

- Do not add WebSocket server code to ash (that's ashd)
- Do not implement Kafka/Splunk delivery (that's ashd Phase 4)
- Do not modify the wasm crate
- Do not change the public API of telemetry beyond adding the new functions
- Do not remove the local JSONL write — it stays as fallback
