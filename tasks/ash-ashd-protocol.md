# ash ↔ ashd Protocol

This document defines the communication protocol between `ash` (CLI) and `ashd` (daemon). All communication occurs over a single persistent WebSocket connection over loopback.

---

## Identity model

Every ash process identifies itself with three levels of identity:

| Level | Identifies | Lifetime | Carrier |
|---|---|---|---|
| **Workflow ID** | The workflow definition (`workflow_path`) | Stable across runs | `STA:` frame, telemetry payload |
| **Instance ID** | One specific execution | Per ash invocation | `STA:`, `ABT:`, `PAU:`, `RES:` frames |
| **Event ID** | The ashd event that triggered this instance | Per event | `ASH_EVENT_ID` env var |

For ashd-spawned processes, the event id doubles as the instance id — `ASH_EVENT_ID` is used as the `instance_id` in every frame. For independently run processes, ash generates a UUID on startup as the instance id, and no event id exists.

---

## Protocol verbs

Every message is a WebSocket text frame starting with a 3-letter uppercase verb. Verbs are grouped by direction.

### ash → ashd

#### TEL — telemetry

```
TEL:{opaque bytes}
```

`TEL:` is a fixed 4-byte prefix. Everything after the colon is the opaque body — ashd forwards it to remote sinks without parsing. The payload format (JSON, protobuf, etc.) is decoupled from the protocol.

No `instance_id` at the protocol level — correlation is embedded in the payload, and the WebSocket connection provides the per-instance context. Each frame is forwarded immediately (not batched).

#### STA — lifecycle status

```
STA:<instance_id>:<status>:<workflow_path>
```

- Status is one of: `running`, `paused`, `completed`, `failed`, `cancelled`.
- The first `STA:running` after WebSocket handshake registers the workflow identity. Subsequent `STA:` frames update the lifecycle state.
- Sent on reconnect and as ACK after `ABT:`, `PAU:`, or `RES:`.

No ack from ashd — fire-and-forget.

### ashd → ash

#### ABT — abort

```
ABT:
```

No body. Ash sets control state to aborted. The connection is 1:1 — no `instance_id` needed.

#### PAU — pause

```
PAU:
```

No body. Ash blocks the execution loop until `RES:` or `ABT:` arrives. The WebSocket reader thread stays alive independently, so `RES:` is received even while execution is paused.

#### RES — resume

```
RES:
```

No body. Resumes a paused workflow. Must be paired with a prior `PAU:`.

---

## Routing

All incoming frames pass through a frame router that decouples wire parsing from business logic:

```
WebSocket connections
        │
        ▼
   Frame Router
        │
        ├── TEL: forward body → RelayPipeline (connection provides instance context)
        ├── STA: parse instance_id:status → ProcessRegistry
        ├── ABT: lookup connection → send verb
        ├── PAU: lookup connection → send verb
        ├── RES: lookup connection → send verb
        └── unknown: ignore
```

### Internal components

- **ProcessRegistry** — maintains lifecycle state per instance_id using `STA:` frames. Tracks running, paused, completed, failed, cancelled. Drives sequential concurrency gating.
- **RelayPipeline** — forwards `TEL:` body bytes to configured remote sinks (Kafka, Splunk). The body is opaque — no parsing, no decoding.
- **ConnectionMap** — maps instance_id → active WebSocket connection. Used by control frame routing (ABT, PAU, RES) and TEL frame context lookup.
- **AuditStore** — bounded ring buffer of recent events keyed by instance_id for GUI queries.

### Dispatch logic

```rust
match type {
    "TEL" => relay_pipeline.forward(body),
    "STA" => process_registry.update(instance_id, body),
    "ABT" | "PAU" | "RES" => connection_map.send(instance_id, frame_type),
    _ => /* ignore-unknown */,
}
```

New frame types register a new arm in the match. No structural changes to the router.

---

## Scenarios

### Telemetry flow

```
ash                              ashd
 │                                │
 │  TEL:{opaque bytes}            │
 │ ──────────────────────────────> │
 │                                │  router → RelayPipeline → Kafka/Splunk
 │  TEL:{opaque bytes}            │
 │ ──────────────────────────────> │
 │         ...                     │
```

ash emits one `TEL:` frame per telemetry event, in real time. ashd's frame router forwards the body to the relay pipeline. The body is opaque — no parsing happens inside ashd.

If the WebSocket is disconnected, ash falls back to local JSONL. On reconnect, ash resumes sending `TEL:` frames.

### Status reporting

```
ash                              ashd
 │                                │
 │  STA:inst_abc:running:invoice  │  register identity + lifecycle
 │ ──────────────────────────────>
 │                                │  router → ProcessRegistry
 │                                │  source busy flag set
 │  STA:inst_abc:paused:invoice   │  (after PAU received)
 │ ──────────────────────────────>
 │                                │  router → ProcessRegistry
 │  STA:inst_abc:running:invoice  │  (after RES received)
 │ ──────────────────────────────>
 │                                │
 │  STA:inst_abc:completed:invoice│  workflow done
 │ ──────────────────────────────> │
 │                                │  router → ProcessRegistry
 │                                │  source busy flag cleared → next event dequeued
```

The first `STA:running` registers both the instance_id and the workflow_path. Subsequent `STA:` frames update the lifecycle state. On terminal `STA:` (completed/failed/cancelled), the sequential concurrency gate is released.

### Control behavior

```
ashd                             ash
 │                                │
 │  PAU:                         │
 │ ──────────────────────────────>
 │                                │  control state ← paused
 │  STA:inst_abc:paused:invoice   │  ACK
 │ <────────────────────────────── │
 │                                │  execution loop blocked
 │  RES:                          │
 │ ──────────────────────────────>
 │                                │  control state ← normal
 │  STA:inst_abc:running:invoice  │  ACK
 │ <────────────────────────────── │
 │                                │  execution loop unblocked
```

`ABT:` is terminal — ash aborts the workflow and sends `STA:inst_abc:cancelled:invoice`.

### Connectivity recovery

The WebSocket runs over loopback — reconnection is inherently reliable.

#### ashd crash

```
ash                              ashd
 │                                │
 │  X─── connection drops ───X    │  ashd crashes
 │                                │
 │  [writes TEL to local JSONL]   │  fallback
 │  [retries with backoff]        │
 │                                │
 │  ── reconnect ──────────────>  │  ashd restarts
 │  STA:inst_abc:running:invoice  │  re-register identity
 │ ──────────────────────────────> │
 │  TEL:{opaque bytes}            │  normal flow resumes
 │ ──────────────────────────────> │
```

1. ash detects WebSocket disconnect. Telemetry falls back to local JSONL. Workflow continues executing.
2. ash retries with exponential backoff. Since both run on the same machine, reconnection always succeeds eventually.
3. On reconnect, ash sends `STA:<instance_id>:running:<workflow_path>` then `STA:<instance_id>:<current_status>:<workflow_path>`. Ashd is immediately synchronized — no prior state needed.
4. If paused during disconnect: ash stays paused. On reconnect, sends `STA:<instance_id>:running:<workflow_path>` followed by `STA:<instance_id>:paused:<workflow_path>`. Workflow remains paused until `RES:` or `ABT:` arrives. No timeout.

#### ash crash

ashd detects ash crash via WebSocket disconnect — identical regardless of whether ashd spawned the process:

1. **Clean exit** — ash sent a terminal `STA:` (`completed`, `failed`, or `cancelled`) before disconnecting. Ashd logs the outcome.
2. **Crash** — disconnect without a terminal `STA:` (last state was `running` or `paused`). Ashd infers a crash and logs accordingly.
3. In both cases, ashd removes the workflow from the active process registry. No reconnection is attempted — the exit is terminal.

### Extensibility

New verbs can be added without structural changes. For example, `ERR` for error reporting. Both sides implement ignore-unknown for unrecognized frame types and instance_ids.

---

## Discovery

ash connects to ashd via the following resolution order:

1. **`ASHD_WS_URL` env var** — set by ashd at spawn time. Highest precedence.
2. **Default** — `ws://127.0.0.1:9877` (configurable via `ws_listen` in `ashd.yml` or `ws_url` in `ash.yml`).
3. **Local file** — if all above fail, ash falls back to local JSONL (existing behavior preserved).

Port binding provides mutual exclusion — only one ashd instance can bind to the port. Deferred: mDNS/DNS-SD service registration for multi-machine discovery.

---

## Backward compatibility

ash and ashd are independent binaries that may be updated on different schedules.

### What is robust

- **Payload transparency** — ashd never parses `TEL:` body bytes. The telemetry schema can evolve freely.
- **Additive changes** — new frame types and new control actions are silently ignored by whichever side doesn't understand them.
- **Discovery chain** — `ASHD_WS_URL` env var always gives the right address for spawned processes.
- **File fallback** — if ash cannot connect to ashd (any version, any reason), it falls back to the original local JSONL behavior.

### What is a breaking change

| Change | Example | Risk |
|---|---|---|
| Removing a frame type | Dropping `ABT:` support | Older clients never receive cancel signals |
| Redefining a control verb | `RES:` → `RESUME:` | Older ashd sends `RES:`, newer ash doesn't recognize it (no-op) |
| Changing env var semantics | `ASH_EVENT_PATH` format or encoding | Spawned processes misinterpret event context |
| Changing the default port | `9877` → `9878` | Standalone ash cannot discover ashd |
| Upgrading WebSocket library with breaking protocol change | Tungstenite major version bump | Connection handshake fails |

### Versioning convention

Neither side sends an explicit protocol version. Frame type prefixes serve as implicit version markers. Both binaries follow semver independently:

- **Backward-compatible changes** (new frame types, new control actions, new env vars) → minor version bump.
- **Breaking changes** (removing types, redefining semantics) → major version bump. The ignore-unknown principle and payload-transparency cover all expected evolution paths.
