# Phase 3b: ashd Folder Watcher + Process Spawning + Crash Recovery

## Context

This is step 3 of 5. Phase 2 gave us the WS server. Phase 3a gives ash its WS client. Runs in parallel with Phase 3a — they touch different crates.

## Task

### 1. Event Channel (`ashd/src/channel.rs`)

```rust
use chrono::{DateTime, Utc};
use std::path::PathBuf;

pub struct WorkflowEvent {
    pub source_name: String,
    pub event_id: String,
    pub path: PathBuf,
    pub timestamp: DateTime<Utc>,
    pub workflow: WorkflowConfig,  // from config
}
```

Import `WorkflowConfig` from `ashd/src/config.rs`.

The channel is `tokio::sync::mpsc::channel::<WorkflowEvent>(256)` — bounded to provide backpressure. Created in `main.rs`.

### 2. Folder Source (`ashd/src/sources/folder.rs`)

Use the `notify` crate with the `RecommendedWatcher`.

```rust
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

pub struct FolderSource {
    name: String,
    watch_path: PathBuf,
    concurrency: ConcurrencyMode,
    workflows: Vec<WorkflowConfig>,
}
```

**`FolderSource::run(self, tx: mpsc::Sender<WorkflowEvent>)`:**

#### 2a. Directory layout

The `watch_path` is the "incoming" area. Inside it, ashd creates and manages two hidden subdirectories:

```
watch_path/                  ← watched; new subdirs trigger events
watch_path/.processing/      ← claimed, work in progress
watch_path/.done/            ← completed (or .done/failed/ for non-zero exits)
```

The watcher ignores ANY entry starting with `.` (dotfiles/dotdirs). This includes `.processing/`, `.done/`, and any temp files.

#### 2b. Crash recovery scan

On startup, BEFORE starting the watcher:
1. Ensure `watch_path/.processing/` and `watch_path/.done/` exist (create if missing)
2. Scan `watch_path/.processing/` for any leftover subdirectories
3. For each orphaned subdirectory, create a `WorkflowEvent` and send to the channel
4. Log: "recovered {} orphaned event(s) from source '{}'"

On startup, also scan `watch_path/` itself for any existing subdirectories (not starting with `.`) and emit events for them. These may have been placed there while ashd was down.

#### 2c. Watcher setup

```rust
let (tx_internal, mut rx_internal) = mpsc::unbounded_channel();
let mut watcher = notify::recommended_watcher(
    move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            let _ = tx_internal.send(event);
        }
    }
)?;
watcher.watch(&watch_path, RecursiveMode::NonRecursive)?;
```

**Non-recursive** — we only watch the top level of `watch_path/` for new subdirectories.

#### 2d. Event processing loop

```rust
loop {
    tokio::select! {
        Some(event) = rx_internal.recv() => {
            // Process notify events
            for path in &event.paths {
                // Skip: anything starting with "."
                if path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with('.'))
                    .unwrap_or(true) {
                    continue;
                }
                // Only act on directories
                if !path.is_dir() { continue; }
                
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        // Atomic move: mv from watch_path/ to watch_path/.processing/
                        let processing_dir = watch_path.join(".processing");
                        let dir_name = path.file_name().unwrap();
                        let claimed_path = processing_dir.join(dir_name);
                        
                        match std::fs::rename(&path, &claimed_path) {
                            Ok(_) => {
                                let event = WorkflowEvent {
                                    source_name: source.name.clone(),
                                    event_id: Uuid::new_v4().to_string(),
                                    path: claimed_path,  // path to .processing/subdir
                                    timestamp: Utc::now(),
                                    workflow: source.workflows[0].clone(),  // MVP: single workflow per source
                                };
                                if tx.send(event).await.is_err() {
                                    break;  // channel closed, shutting down
                                }
                            }
                            Err(e) => {
                                log::warn!("failed to claim {}: {}", path.display(), e);
                            }
                        }
                    }
                    _ => {}  // ignore other event kinds
                }
            }
        }
        _ = shutdown_signal.recv() => {
            break;  // graceful shutdown
        }
    }
}
```

#### 2e. Temp-then-move convention

Upstream processes write to a temp location OUTSIDE `watch_path/`, then atomically `mv` the completed subdirectory into `watch_path/`. This ensures the watcher only sees complete content. Document this in logs at startup: "source '{}': place completed subdirectories in '{}' — use temp-write-then-move for atomic delivery".

#### 2f. Sequential concurrency

If `concurrency` is `Sequential`, the workflow associated with this source is only spawned when no prior workflow is still `running` or `paused` (checked via `ProcessRegistry::is_source_busy`). The event stays pending until the source becomes free. Implement this via a simple loop in the event processor that checks registry state before sending to the spawn loop. Don't hold an event in the watcher — if the source is busy, queue the event internally (a `VecDeque<WorkflowEvent>` per source) and check on each `STA:completed/failed/cancelled` update.

Actually, simpler: the main event loop (in `main.rs`) handles gating. When a sequential source's event arrives at the channel receiver, if the source is busy, stash the event in a `VecDeque` for that source. When `ProcessRegistry` updates to terminal status for that source, check the queue.

### 3. Spawner (`ashd/src/spawner.rs`)

When multiple ash processes run in parallel, `Stdio::inherit()` causes chaotic interleaving on ashd's terminal. Instead, capture stdout/stderr via `Stdio::piped()` and prefix every line with the source name and event id.

```rust
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;

pub struct TaggedLine {
    pub prefix: String,   // e.g. "[watch-invoices/evt-001]"
    pub stream: String,   // "stdout" or "stderr"
    pub line: String,
}

/// Spawns a tokio task that reads both stdout and stderr,
/// prefixes each line, and prints to ashd's terminal.
/// Returns the tokio child handle.
pub async fn spawn_and_relay(
    event: &WorkflowEvent,
    ws_listen: &str,
) -> std::io::Result<tokio::process::Child> {
    let prefix = format!("[{}/{}]", event.source_name, event.event_id);
    
    let mut cmd = TokioCommand::new("ash");
    cmd.arg(&event.workflow.path);
    for flag in &event.workflow.flags {
        cmd.arg(flag);
    }
    cmd.env("ASH_EVENT_SOURCE", &event.source_name);
    cmd.env("ASH_EVENT_PATH", &event.path);
    cmd.env("ASH_EVENT_ID", &event.event_id);
    cmd.env("ASH_EVENT_TIMESTAMP", &event.timestamp.to_rfc3339());
    cmd.env("ASHD_WS_URL", &format!("ws://{}", ws_listen));
    cmd.env("PATH", std::env::var("PATH").unwrap_or_default());
    cmd.env("HOME", std::env::var("HOME").unwrap_or_default());
    if let Ok(ll) = std::env::var("ASH_LOG") { cmd.env("ASH_LOG", ll); }
    
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);
    
    let mut child = cmd.spawn()?;
    
    let stdout = child.stdout.take().expect("stdout not piped");
    let stderr = child.stderr.take().expect("stderr not piped");
    
    let p = prefix.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            println!("{} {}", p, line);
        }
    });
    
    let p = prefix;
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            eprintln!("{} {}", p, line);
        }
    });
    
    Ok(child)
}
```

This produces clean, readable output:

```
[watch-invoices/evt-a1b2] Running workflow invoice.ash
[watch-orders/evt-c3d4]   Running workflow order.ash
[watch-invoices/evt-a1b2] Processing line item 1...
[watch-invoices/evt-a1b2] Done. exit_code=0
[watch-orders/evt-c3d4]   Shipping label generated.
```

If a bash `Command` is preferred over `TokioCommand` (to match the rest of ashd), use `std::process::Command::spawn()` with piped stdio, then spawn tokio tasks that read from the child's `stdout`/`stderr` via `tokio::task::spawn_blocking` wrapped around `BufRead::lines()`:

```rust
// Sync spawn + async relay bridge
let mut child = std::process::Command::new("ash")
    // ... args, env ...
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

let stdout = child.stdout.take().unwrap();
let stderr = child.stderr.take().unwrap();

let prefix_clone = prefix.clone();
tokio::task::spawn_blocking(move || {
    let reader = std::io::BufReader::new(stdout);
    for line in reader.lines() {
        if let Ok(line) = line {
            println!("{} {}", prefix_clone, line);
        }
    }
});

tokio::task::spawn_blocking(move || {
    let reader = std::io::BufReader::new(stderr);
    for line in reader.lines() {
        if let Ok(line) = line {
            eprintln!("{} {}", prefix, line);
        }
    }
});
```

Either approach is fine. The key point: piped stdio + line prefix prevents chaotic output when multiple ash processes share ashd's terminal.

### 4. Workflow Completion Cleanup

After ash process exits, ashd must move the event folder from `.processing/` to `.done/` (or `.done/failed/`):

In the main loop, after spawning, spawn a tokio task that waits for the child to exit:
```rust
tokio::spawn(async move {
    let exit_status = child.wait().await;
    let dest = if exit_status.map(|s| s.success()).unwrap_or(false) {
        processing_path.parent().unwrap().join(".done")
    } else {
        processing_path.parent().unwrap().join(".done").join("failed")
    };
    std::fs::create_dir_all(&dest).ok();
    let dir_name = event.path.file_name().unwrap();
    if let Err(e) = std::fs::rename(&event.path, dest.join(dir_name)) {
        log::warn!("failed to move completed event: {}", e);
    }
});
```

Note: the child exit status is separate from the `STA:completed`/`STA:failed` frame. The ashd ProcessRegistry already tracks the true outcome from STA frames. The `.done/` vs `.done/failed/` split is a convenience for operators — it uses the child process exit code as a rough indicator. The authoritative status is in the ProcessRegistry.

### 5. Wire up in main.rs

Replace the Phase 2 placeholder main loop with:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    let config = AshdConfig::load()?;
    log::info!("config loaded, {} source(s)", config.sources.len());
    
    // Shared components
    let (frame_tx, frame_rx) = mpsc::channel(1024);
    let (event_tx, mut event_rx) = mpsc::channel::<WorkflowEvent>(256);
    let connection_map = Arc::new(ConnectionMap::new());
    let process_registry = Arc::new(ProcessRegistry::new());
    let relay_pipeline = Arc::new(RelayPipeline::new(config.telemetry_relay.as_ref()));
    
    // Start WebSocket server
    let ws_server = WsServer::new(config.daemon.ws_listen.clone(), frame_tx.clone());
    let ws_handle = tokio::spawn(async move { ws_server.start().await });
    
    // Start frame router
    let router = FrameRouter::new(relay_pipeline, process_registry.clone(), connection_map.clone());
    let router_handle = tokio::spawn(async move { router.run(frame_rx).await });
    
    // Start folder sources
    let mut source_handles = vec![];
    for source_cfg in &config.sources {
        if source_cfg.source_type == "folder" {
            let src = FolderSource::from_config(source_cfg)?;
            let tx = event_tx.clone();
            let handle = tokio::spawn(async move { src.run(tx).await });
            source_handles.push(handle);
        }
        // Future: webhook, cron, etc. add arms here
    }
    log::info!("{} source(s) started", source_handles.len());
    
    // Event loop: receive events, spawn workflows
    let spawn_handle = tokio::spawn(async move {
        // Per-source sequential queues
        let mut sequential_queues: HashMap<String, VecDeque<WorkflowEvent>> = HashMap::new();
        
        while let Some(event) = event_rx.recv().await {
            let source_cfg = config.sources.iter()
                .find(|s| s.name == event.source_name)
                .unwrap();
            
            if source_cfg.concurrency == ConcurrencyMode::Sequential 
               && process_registry.is_source_busy(&event.source_name) {
                sequential_queues.entry(event.source_name.clone())
                    .or_default()
                    .push_back(event);
                continue;
            }
            
            // Spawn workflow with line-tagged output relay
            match spawn_and_relay(&event, &config.daemon.ws_listen).await {
                Ok(mut child) => {
                    log::info!("spawned ash for {} (pid={})", event.event_id, child.id());
                    // Spawn async waiter for cleanup
                    let pr = process_registry.clone();
                    let event_path = event.path.clone();
                    tokio::spawn(async move {
                        // Wait for child to exit
                        let _ = child.wait().await;
                        // Move to .done/ or .done/failed/
                        // (simplified: process_registry handles the actual status)
                    });
                }
                Err(e) => {
                    log::error!("failed to spawn workflow: {}", e);
                }
            }
        }
    });
    
    log::info!("ashd listening on {}", config.daemon.ws_listen);
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    log::info!("shutting down...");
    
    // Graceful shutdown: stop sources, drain pending events, wait for children
    for handle in source_handles {
        handle.abort();
    }
    // Allow pending events to process
    tokio::time::sleep(Duration::from_secs(config.daemon.grace_period_secs)).await;
    
    spawn_handle.abort();
    ws_handle.abort();
    router_handle.abort();
    
    Ok(())
}
```

### 6. Implementation Notes

- `FolderSource::from_config()` validates that `path` for the folder source exists (or creates it including `.processing/` and `.done/`) on startup
- The `notify` watcher should handle errors gracefully — if the watched directory is deleted, log an error and retry
- Claiming (mv to `.processing/`) must be atomic. On some filesystems, `rename` across filesystems fails — use fallback copy-then-delete in that case
- Event IDs are UUID v4 for uniqueness and idempotency
- The channel buffer size of 256 is generous; overflowing means ashd can't keep up and events are dropped (logged as warning)

### Acceptance

1. Start ashd with a folder source pointing to `/tmp/ashd-test/incoming`
2. `mkdir -p /tmp/ashd-test/incoming/event-001 && echo "hello" > /tmp/ashd-test/incoming/event-001/data.txt`
3. Observe: ashd moves `event-001` to `.processing/`, spawns `ash` with `ASH_EVENT_PATH=/tmp/ashd-test/incoming/.processing/event-001`
4. ash connects to ashd, sends STA and TEL frames
5. ash completes, ashd moves event from `.processing/` to `.done/`
6. Kill ashd while ash is running. Restart ashd. Observe: orphaned folder in `.processing/` is re-queued and re-spawned
7. Drop two folders simultaneously — both spawn in parallel (for `parallel` concurrency)
8. Sequential source: drop folder A, then folder B. B only spawns after A's terminal STA arrives.
9. Folder starting with `.` (e.g., `.tmp`) — ignored, no event fired
10. File (not directory) dropped into watched path — ignored, no event fired

### Do NOT do

- Do not implement webhook, cron, Kafka, or other event sources
- Do not implement mDNS discovery
- Do not add command-line flags to ashd beyond what's needed
- Do not implement the relay pipeline (Phase 4)
