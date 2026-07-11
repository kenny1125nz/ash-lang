# Phase 1: Workspace & ashd Skeleton + Config

## Context

This is step 1 of 5 for building `ashd`, a daemon for event-driven workflow execution alongside `ash`. See `tasks/Event-Driven-Flow.md` for the full design and `tasks/ash-ashd-protocol.md` for the protocol spec.

## Task

Create the build infrastructure and configuration system for `ashd`.

### 1. Root Cargo Workspace

Create `/opt/apps/agents/ash/Cargo.toml` (root-level workspace):

```toml
[workspace]
members = ["ash", "ash-wasm", "ashd"]
resolver = "2"
```

Adjust `ash/Cargo.toml` and `ash-wasm/Cargo.toml` to remove any `[workspace]` or version fields that conflict with being a workspace member. Existing `[package]` sections stay as-is. The workspace defines only `members` and `resolver`.

Verify: `cargo build -p ash` and `cargo build -p ash-wasm` still work after the workspace is added.

### 2. ashd Crate Scaffold

Create `ashd/Cargo.toml`:

```toml
[package]
name = "ashd"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.24"
futures-util = "0.3"
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
notify = { version = "7", features = ["macos_kqueue"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
log = "0.4"
env_logger = "0.11"
thiserror = "2"
```

Create `ashd/src/main.rs` — minimal binary that initializes logging and prints "ashd starting":

```rust
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("ashd starting");
}
```

Verify: `cargo build -p ashd` succeeds.

### 3. Configuration Structs

Create `ashd/src/config.rs` with these structs (all derive `Debug, Clone, Serialize, Deserialize`):

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AshdConfig {
    pub daemon: DaemonConfig,
    #[serde(default)]
    pub sources: Vec<SourceConfig>,
    #[serde(default)]
    pub telemetry_relay: Option<TelemetryRelayConfig>,
    #[serde(default)]
    pub staging_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_ws_listen")]
    pub ws_listen: String,
    #[serde(default = "default_grace_period_secs")]
    pub grace_period_secs: u64,
}

fn default_log_level() -> String { "info".into() }
fn default_ws_listen() -> String { "127.0.0.1:9877".into() }
fn default_grace_period_secs() -> u64 { 30 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    #[serde(rename = "type")]
    pub source_type: String,
    pub name: String,
    pub path: Option<PathBuf>,
    #[serde(default = "default_concurrency")]
    pub concurrency: ConcurrencyMode,
    pub workflows: Vec<WorkflowConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConcurrencyMode {
    Parallel,
    Sequential,
}

impl Default for ConcurrencyMode {
    fn default() -> Self { ConcurrencyMode::Parallel }
}

fn default_concurrency() -> ConcurrencyMode { ConcurrencyMode::Parallel }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub path: PathBuf,
    #[serde(default)]
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryRelayConfig {
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub splunk: Option<SplunkConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    pub enabled: bool,
    pub brokers: Vec<String>,
    pub topic: String,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_flush_interval_ms")]
    pub flush_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplunkConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub token: String,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_flush_interval_ms")]
    pub flush_interval_ms: u64,
}

fn default_batch_size() -> usize { 100 }
fn default_flush_interval_ms() -> u64 { 5000 }
```

Implement `AshdConfig::load()`:

1. Resolution order: `ASHD_CONFIG` env var → `./ashd.yml` → `~/.ash/ashd.yml` (POSIX: `$HOME` or `$XDG_CONFIG_HOME/ash/ashd.yml`)
2. Read file, deserialize with `serde_yaml`
3. Validate:
   - Each folder-type source must have a `path` field set
   - `ws_listen` must be a valid `host:port` (parse with Rust's `SocketAddr`)
   - `grace_period_secs` must be > 0
   - At least one source must be defined
   - Each workflow `path` must exist (use `std::fs::metadata` or `Path::exists`)
4. Return `Result<AshdConfig, anyhow::Error>` with descriptive errors

### 4. Sample Config

Create `/opt/apps/agents/ash/ashd.yml` at repo root:

```yaml
daemon:
  log_level: debug
  ws_listen: "127.0.0.1:9877"
  grace_period_secs: 30

sources:
  - type: folder
    name: watch-invoices
    path: /tmp/ashd-test/incoming
    concurrency: parallel
    workflows:
      - path: ./workflows/process-invoice
        flags: ["--agent", "opencode:sonnet", "--yes"]

# telemetry_relay:
#   kafka:
#     enabled: true
#     brokers: ["localhost:9092"]
#     topic: ash-events
#   splunk:
#     enabled: false
#     endpoint: "https://splunk.example.com:8088/services/collector"
#     token: "${SPLUNK_HEC_TOKEN}"
```

### 5. Wire up main.rs

Update `ashd/src/main.rs` to:
- Call `AshdConfig::load()`
- On success: print parsed config (debug format for now)
- On error: print error to stderr and exit with code 1
- Print "config OK" and exit (full daemon loop comes in later phases)

### Acceptance

- `cargo build -p ashd` compiles without errors
- Running `cargo run -p ashd` with the sample `ashd.yml` prints the parsed config
- Running with a missing config file prints a descriptive error
- Running with a config where a folder source has no `path` prints a validation error
- `cargo build -p ash` and `cargo build -p ash-wasm` still compile

### Do NOT do

- Do not start a WebSocket server
- Do not start any watchers
- Do not modify ash or ash-wasm code beyond what's needed for workspace compat
- Do not create any protocol handling code
