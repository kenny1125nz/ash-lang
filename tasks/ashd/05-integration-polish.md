# Phase 5: Integration & Polish

## Context

This is step 5 of 5. All major components are built. This phase ties loose ends, ensures robustness, and validates the end-to-end flow.

## Task

### 1. Graceful Shutdown

Shutdown is triggered by SIGTERM, SIGINT, or (on Windows) Ctrl+C.

**Current state from Phase 3b:** The main loop catches `ctrl_c()`, aborts source handles, sleeps for grace period, then aborts WS/router handles.

**Improvements:**
- **Stop accepting new events:** When shutdown is signaled, close the `event_tx` sender (`drop(event_tx)`) so watchers can't send new events
- **Drain pending events:** Before aborting the spawn handle, let it process any remaining events in the channel. Use `event_rx.recv()` with a timeout.
- **Wait for child processes:** Track spawned children in a `Vec<std::process::Child>` (or a `HashMap<u32, Child>`). On shutdown, send SIGTERM to each child, wait up to `grace_period_secs`, then SIGKILL.
- **Close WebSocket connections gracefully:** Send close frames to connected ash processes before dropping the WS server.

```rust
// Shutdown sequence:
// 1. Stop watchers (abort source handles)
// 2. Drop event_tx — no new events accepted
// 3. Drain event channel (process remaining events with timeout)
// 4. Send SIGTERM to child processes
// 5. Wait grace_period_secs
// 6. SIGKILL any remaining children
// 7. Send WS close frames
// 8. Abort WS + router handles
// 9. Flush relay pipeline
```

### 2. Structured Logging

Throughout ashd, use structured key-value logging:

```rust
log::info!("spawned workflow"; 
    "event_id" => &event.event_id,
    "source" => &event.source_name,
    "workflow" => &event.workflow.path.to_string_lossy(),
    "pid" => child.id(),
);

log::warn!("relay delivery failed";
    "sink" => sink.name(),
    "events_dropped" => batch.len(),
    "error" => %err,
);
```

The `log` crate with `env_logger` supports this via the `kv` feature (enable in Cargo.toml). Format: `key=value` pairs.

### 3. Unix Socket Support

Currently the WS server only supports TCP (`127.0.0.1:9877`). Add optional Unix domain socket support for same-machine deployments where loopback TCP port conflicts are a concern.

**Config change:** Allow `ws_listen` to be a Unix socket path (detect by checking if the string starts with `/` or `.`):

```rust
// In ws.rs:
if ws_listen.starts_with('/') || ws_listen.starts_with('.') {
    // Bind to Unix socket
    let listener = tokio::net::UnixListener::bind(ws_listen)?;
    // Accept loop with tokio_tungstenite::accept_async_with_config on UnixStream
} else {
    // Bind to TCP as before
}
```

Add `tokio-stream` dependency if needed for Unix listener streams.

### 4. Discovery: Local File Fallback

In `ash/src/telemetry/client.rs`, add a third discovery option after the default address fails:

1. `ASHD_WS_URL` env var
2. `config.ws_url` config field
3. `ws://127.0.0.1:9877` (default)
4. **NEW:** Read `~/.ash/ashd.sock` file — if exists and contains a valid URL, use it

```rust
fn discover_ashd_url(config: &TelemetryConfig) -> Option<String> {
    // 1. env
    if let Ok(url) = std::env::var("ASHD_WS_URL") { return Some(url); }
    // 2. config
    if let Some(ref url) = config.ws_url { return Some(url.clone()); }
    
    // 3. default
    let default = "ws://127.0.0.1:9877";
    
    // 4. local file
    let sock_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".ash/ashd.sock");
    if let Ok(contents) = std::fs::read_to_string(&sock_path) {
        let url = contents.trim();
        if !url.is_empty() { return Some(url.to_string()); }
    }
    
    Some(default.to_string())
}
```

Add `dirs` crate to `ash/Cargo.toml` (lightweight, no system deps).

### 5. ashd --check Flag

Add a `--check`/`-c` flag to ashd that validates the config and exits:

```rust
// In main.rs, before starting the daemon:
if args.check {
    match AshdConfig::load() {
        Ok(config) => {
            println!("Configuration OK");
            println!("  WebSocket: {}", config.daemon.ws_listen);
            println!("  Sources: {}", config.sources.len());
            for src in &config.sources {
                println!("    - {} ({})", src.name, src.source_type);
                if src.source_type == "folder" {
                    if let Some(ref path) = src.path {
                        println!("      path: {}", path.display());
                    }
                }
            }
            if let Some(ref relay) = config.telemetry_relay {
                if let Some(ref kafka) = relay.kafka {
                    if kafka.enabled {
                        println!("  Kafka relay: {} -> {}", kafka.brokers.join(","), kafka.topic);
                    }
                }
                if let Some(ref splunk) = relay.splunk {
                    if splunk.enabled {
                        println!("  Splunk relay: {}", splunk.endpoint);
                    }
                }
            }
            return Ok(());
        }
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    }
}
```

Parse `--check` with a simple arg scan (don't pull in clap for one flag):

```rust
let args: Vec<String> = std::env::args().collect();
let check = args.iter().any(|a| a == "--check" || a == "-c");
```

### 6. Smoke Test Script

Create `tests/smoke.sh` (bash script, not a Rust test):

```bash
#!/bin/bash
set -euo pipefail

ASH="./target/debug/ash"
ASHD="./target/debug/ashd"
TEST_DIR="/tmp/ashd-smoke-test"
WATCH_DIR="$TEST_DIR/incoming"

echo "=== ashd smoke test ==="

# Cleanup from previous runs
rm -rf "$TEST_DIR"
mkdir -p "$WATCH_DIR"

# Start ashd in background
"$ASHD" --config "$TEST_DIR/ashd.yml" &
ASHD_PID=$!
sleep 2

# Verify ashd is listening
if ! kill -0 $ASHD_PID 2>/dev/null; then
    echo "FAIL: ashd failed to start"
    exit 1
fi
echo "PASS: ashd started (pid=$ASHD_PID)"

# Drop a test event
EVENT_DIR="$WATCH_DIR/event-001"
mkdir -p "$EVENT_DIR"
echo "echo hello from smoke test" > "$EVENT_DIR/test.ash"

# Wait for ashd to spawn ash
sleep 3

# Check that event was claimed (moved to .processing)
if [ -d "$WATCH_DIR/.processing/event-001" ]; then
    echo "PASS: event claimed by ashd"
else
    echo "FAIL: event not claimed"
    exit 1
fi

# Wait for completion
sleep 10

# Check that event was completed (moved to .done)
if [ -d "$WATCH_DIR/.done/event-001" ]; then
    echo "PASS: event completed"
else
    echo "FAIL: event not completed"
    exit 1
fi

# Clean shutdown
kill $ASHD_PID
wait $ASHD_PID 2>/dev/null || true

echo "=== All smoke tests passed ==="
```

### 7. Telemetry File Fallback Verification

Add a test case (either in the smoke test or as a documented manual test) that verifies the telemetry fallback chain:

1. Start ash WITHOUT ashd running
2. Verify: ash completes normally, `telemetry.jsonl` is written to (existing behavior)
3. Verify: log contains "ashd unreachable" warning at info level, not error

### 8. Edge Cases to Handle

- **Empty watched directory:** Watcher starts, no events. No crash, no error.
- **Rapid folder creation:** 100 subdirectories created in quick succession. All claimed without race conditions. Verify: no duplicate claims.
- **Folder deleted between notification and claim:** `mv` to `.processing/` fails because the folder no longer exists. Log warning, skip.
- **Non-existent workflow path:** Config validation should catch this (Phase 1), but add a runtime check in `spawn_and_relay()` before spawning.
- **ash binary not in PATH:** `spawn_and_relay()` returns `Err`. Log error, skip event, do not crash.
- **WebSocket port in use on startup:** `TcpListener::bind()` fails. Log error and exit with code 1. Do not start without binding successfully.
- **Temp directory not writable:** Creating `.processing/` or `.done/` fails. Log error, exit with code 1 (this is fatal for folder sources).

### Acceptance

1. `cargo build --workspace` compiles all three crates
2. `cargo build -p ashd --release` produces a release binary
3. Smoke test script passes
4. `ashd --check` validates config and exits cleanly
5. `ashd --check` with bad config prints error and exits with code 1
6. SIGTERM during workflow execution: ashd shuts down gracefully, child processes are terminated
7. All Phases 1-5 acceptance criteria still hold
8. ash running without ashd → writes to local JSONL, no crash

### Do NOT do

- Do not add a CLI argument parser (clap, etc.) — simple arg scanning is sufficient
- Do not write integration tests as Rust tests — bash smoke test is sufficient for MVP
- Do not add systemd/launchd service files
- Do not add Docker support
- Do not modify ash-wasm
