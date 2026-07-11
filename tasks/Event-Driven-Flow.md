# Event-Driven Workflow Execution — Task Definition

## Background

Ash workflows today are triggered exclusively by a user running the `ash` CLI with a directory path, script file, or REPL command. The idea document `event-driven.md` identifies this gap and calls event-triggered execution "a must for enterprise level orchestration."

The project already has:
- A telemetry pipeline that writes events to a local JSONL file with a stubbed remote delivery function
- A config system built around `ash.yml` with YAML-deserialized structs
- Two existing crates in the workspace (`ash`, `ash-wasm`)

This task definition designs the solution.

---

## Intended Solution

### Core decision: a separate daemon binary (`ashd`)

A new binary at `ashd/` alongside `ash/` and `ash-wasm/`. Rationale: the CLI is short-lived and user-invoked; the daemon is long-lived and signal-driven. Conflating them would complicate lifecycle and error handling. A separate binary enables independent lifecycle management and has a distinct dependency set (filesystem watchers, WebSocket server).

### How it works at a high level

```
                +------------------+
                |      ashd        |
                |   (daemon)       |
                |                  |
 file-system  -->  folder watcher --+   |
                |                  |   |
 webhook     -->  message→folder --+--> event channel --> spawn "ash <workflow>"
                |                  |   |        [env: ASH_EVENT_* vars]
 kafka       -->  message→folder --+   |
                |                  |
 telemetry  <--  relay thread    <-- WebSocket server <-- ash CLI telemetry
                |                  (sink for ash processes)   |
                |                  |                        [control state]
                +--------+---------+
                         |
                         v
                    remote sink
                  (kafka / splunk)
```

The daemon has two responsibilities:

1. **Listen for external events** and spawn workflow runs. All event sources converge to a folder path — the folder watcher natively produces subfolder paths; message sources (webhook, Kafka) write their payload to a staging directory and produce a folder path. The daemon spawns `ash` with `ASH_EVENT_PATH` pointing to the content folder.
2. **Act as a telemetry sink** for ash processes — each spawned `ash` process opens a WebSocket connection to `ashd`, sends telemetry events in real time, and `ashd` relays them to configured remote sinks (Kafka, Splunk). The response channel of the same WebSocket connection enables future bidirectional control (cancel, pause, resume).

### Event source types

All event sources converge to a uniform model: the daemon emits an event with a folder path (`ASH_EVENT_PATH`). For the folder watcher, the path is a subdirectory that appeared inside the watched directory. For message sources (webhook, Kafka), ashd writes the payload to a staging directory and the resulting folder path becomes the event.

The MVP covers one source type; the rest are deferred with explicit rationale.

**In scope (MVP):**
- **Folder watcher** — watches a directory, emits an event when a new subdirectory appears inside it (ignoring anything starting with `.`). The subdirectory is the unit of work. Upstream processes should use a temp-write-then-move strategy: write content to a temp location, then atomically `mv` the completed folder into the watched directory. This ensures the event only fires when content is complete. Suitable for: drop folders, data ingestion pipelines.

**Deferred:**
- **Webhook listener** — HTTP server on a configurable port. Receives a POST body, writes it to a staging folder, and emits a folder event. Deferred because the folder watcher covers the highest-value use case.
- **Cron scheduler** — POSIX cron expression evaluation. Generates a content event on a schedule.
- **Message streams** (MQ, Kafka, cloud messaging) — Each broker has unique protocol/client library; webhook endpoints can serve as a bridge in the interim.
- **Chat-based applications** (Slack, Discord, Teams) — Bot auth and platform-specific APIs better scoped as a follow-up.

Message sources (webhook, Kafka, etc.) are message→folder adapters that write content to a staging directory (configured in `ashd.yml`) and inject the folder path into the event channel. The workflow engine never handles raw messages — it always processes file system content.

### Event-to-workflow mapping

Each event source config block specifies one or more `workflows` entries. When an event fires, the daemon spawns `ash <path> <flags>` as a child process with env vars for event context:

| Variable              | Contents                                                       |
| --------------------- | -------------------------------------------------------------- |
| `ASH_EVENT_SOURCE`    | source name from config                                        |
| `ASH_EVENT_PATH`      | path to the event content folder (always a directory)          |
| `ASH_EVENT_ID`        | unique event id (for dedup and idempotency)                    |
| `ASH_EVENT_TIMESTAMP` | ISO 8601 timestamp of event receipt                            |
| `ASHD_WS_URL`         | ashd's WebSocket address for telemetry (fast path, skips mDNS) |

The daemon does not parse workflow syntax or inject DSL variables. Env vars avoid coupling the daemon to the DSL parser, avoid injection risks, and are readable by child processes spawned by the workflow.

### Event channel and spawning

- `std::sync::mpsc` for the event channel.
- The daemon main loop blocks on `rx.recv()`.
- Each event spawns `ash` via `std::process::Command::spawn()`.
- Concurrency is controlled per source via config:
  - `parallel` (default) — each event spawns immediately, all run concurrently.
  - `sequential` — events for that source are queued; the next spawn fires after the previous workflow completes (WebSocket disconnect with terminal `STA:`, or connection timeout if ash never connects).

### Workflow recovery (folder watcher)

See "Design decisions > Workflow recovery".

### Telemetry sink (core capability)

Each ash process opens a persistent WebSocket connection to `ashd` over loopback. ash discovers ashd via mDNS/DNS-SD (or `ASHD_WS_URL` env var for spawned processes). The daemon acts as a telemetry aggregator: it receives events from all running `ash` processes and relays them to configured remote sinks (Kafka, Splunk). The same connection is also used for future bidirectional control (cancel, pause, resume) without a secondary channel.

**Fallback:** if the WebSocket connection fails (ashd not running), ash logs a warning and continues with local JSONL file writing (existing behavior preserved).

**Changes to the ash telemetry pipeline:**
- A WebSocket client thread connects to ashd on startup
- Telemetry events are sent as `TEL` frames
- The existing local JSONL write is kept as a fallback
- The stubbed `deliver_remote()` in-process consumer is removed — remote delivery is entirely ashd's responsibility

### Protocol, identity, discovery, backward compatibility, connectivity recovery

See `tasks/ash-ashd-protocol.md`.

### Impact on ash

#### Telemetry client

ash gains a WebSocket client thread that runs for the lifetime of the process:

1. On startup, resolves ashd's address via the discovery chain (env var → default).
2. Opens a persistent WebSocket connection to `ws://<ashd>:9877`.
3. Sends each telemetry event as a `TEL:` frame as soon as it is emitted (real-time).
4. Reads `ABT:`, `PAU:`, `RES:` frames from the connection and updates an in-memory `Arc<AtomicU8>` control state (`0` = normal, `1` = abort, `2` = pause).
5. On connection failure, logs a warning and falls back to local JSONL writing (existing behavior preserved). The client retries periodically.

The existing `deliver_remote()` consumer thread is removed — remote delivery is entirely ashd's responsibility.

#### Connectivity recovery

See `tasks/ash-ashd-protocol.md` > Connectivity recovery.

#### Control state in the runtime

The `Arc<AtomicU8>` control state is accessible to the workflow execution loop. Before executing each step, the runtime checks the control state:

- **abort** — abort the workflow with an exit code.
- **pause** — block the execution loop until `RES:` or `ABT:` arrives. The WebSocket reader thread stays alive independently, so `RES:` is received and processed even while execution is paused.
- **normal** — proceed.

In MVP, no control frames are sent by ashd (no GUI yet), so the state always reads normal. The check is a single atomic load and is effectively free. The machinery is in place for when the GUI and control frames are added.

#### Configuration

ash's `ash.yml` gains an optional `ws_url` field under telemetry config:

```yaml
telemetry:
  ws_url: ws://127.0.0.1:9877   # optional, overrides discovery default
```

If absent, ash uses the discovery default. This field is purely a user-facing override for non-standard deployments and is not required for normal operation.

### Config format

A new YAML file at `./ashd.yml` (then `~/.ash/ashd.yml`), separate from `ash.yml`:

```yaml
daemon:
  log_level: info
  ws_listen: "127.0.0.1:9877"   # default, override for custom port

sources:
  - type: folder
    name: watch-invoices
    path: /data
    concurrency: parallel       # parallel (default) or sequential
    workflows:
      - path: ./workflows/process-invoice
        flags: ["--agent", "opencode:sonnet", "--yes"]

# staging_dir: /var/lib/ashd/staging
# used by message sources to convert payloads to folder events

telemetry_relay:
  kafka:
    enabled: true
    brokers: ["localhost:9092"]
    topic: ash-events
    batch_size: 100
    flush_interval_ms: 5000
```

### Process lifecycle

- Starts, reads config, initializes folder watcher and WebSocket server, runs until SIGTERM/SIGINT.
- On shutdown: stops accepting connections, drains pending events, spawns remaining workflows, then exits (configurable grace period, default 30s).
- Port binding prevents duplicate instances — only one process can bind to the configured `ws_listen` address.

### Acceptance criteria

**Core (MVP):**

1. Binary builds — `ashd` compiles as a standalone binary from the workspace.
2. Daemon starts and stops cleanly; PID file prevents duplicates.
3. Config validated at startup — missing fields, bad paths, or port conflicts cause a descriptive error before any listener starts.
4. Folder watcher triggers workflows — a new subdirectory appearing in the watched directory spawns `ash` with `ASH_EVENT_PATH` set to the subdirectory path.
5. Workflow failures are logged, not fatal.
6. WebSocket telemetry sink receives events — ash processes connect and send telemetry frames; ashd acknowledges and relays to configured remote sinks (Kafka, Splunk).
7. Graceful fallback when ashd is unreachable — ash writes to local file.

**Boundaries:**
- Does not modify the `ash` CLI evaluator, engine, or runtime. The daemon invokes `ash` via subprocess.
- Does not build a GUI.
- Folder watcher only for MVP (no cron, webhook, message streams, or chat sources).

### Design decisions

#### Daemon architecture
- `ashd` is a separate binary (not embedded in `ash` CLI). Parallel-safe via configurable `concurrency`.
- No PID file — port binding prevents duplicate instances.
- Three-directory layout in the watched path for crash recovery: `incoming/` (watched), `.processing/` (claimed), `.done/` (completed). The filesystem is the durable queue.

#### Event model
- All sources converge to a folder path — event unit is a subfolder, not a file. `ASH_EVENT_PATH` is always a directory.
- Upstream processes use a temp-then-move convention to ensure atomic delivery.
- Message sources (webhook, Kafka, future) are message→folder adapters: write payload to staging, then inject the folder path into the event channel.
- ashd does not track workflow state — ash is the sole source of truth.

#### Protocol, identity, connectivity recovery

See `tasks/ash-ashd-protocol.md`.

#### Discovery
- Default: `ws://127.0.0.1:9877`. Configurable via `ws_listen` in `ashd.yml` or `ws_url` in `ash.yml`.
- Resolution order: `ASHD_WS_URL` env → config → default. mDNS deferred.

#### Workflow recovery

Each folder event source uses hidden subdirectories inside its `path` to persist event state on disk. The watcher ignores anything starting with `.`:

- **`path/`** — the configured directory, watched by `notify`. Upstream processes `mv` completed subfolders here. This is the event trigger point.
- **`path/.processing/`** — ashd atomically `mv`s the subfolder from `path/` to `path/.processing/` before spawning. This signals "claimed, work in progress."
- **`path/.done/`** — on workflow completion (success or failure), ashd `mv`s the subfolder from `.processing/` to `.done/` (or `.done/failed/` for non-zero exits).

On restart, ashd scans `path/.processing/` for any leftover subfolders (orphaned by a crash) and re-queues them. The filesystem is the durable queue — no database, no in-memory queue to lose.

### Future extensibility

The architecture is designed for these additions without structural changes:

- **GUI/audit trail** — ashd's in-memory process registry and telemetry store (bounded ring buffer keyed by `trace_id`) provide the query surface. A separate frontend can poll or WebSocket to ashd's HTTP API. Control actions (abort, pause, resume) are already defined in the protocol (`ABT:`, `PAU:`, `RES:` frames).
- **mDNS/DNS-SD** — register `_ashd._tcp` on the local network for multi-machine discovery, replacing the hardcoded default address.
- **Additional event sources** (webhook, cron, message streams) — all sources converge to the same folder-path event model via the message→folder adapter pattern. Implement the adapter, register it; no changes to the core loop or spawning logic.
- **Message→folder adapters** — webhook, Kafka, and chat sources write their payload to a staging directory, then inject the folder path into the event channel. The workflow engine never distinguishes between sources — it always receives a folder path.
