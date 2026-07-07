# Feature Refactoring: Telemetry + Online Catalog

Keep optional features manageable without complicating the core application.

## Motivation

Telemetry and online-catalog introduce extra deps and complexity into the core binary. Current state:

| Feature        | Feature-gated?                                     | Core code impact                                           |
| -------------- | -------------------------------------------------- | ---------------------------------------------------------- |
| REPL           | Yes (`repl` → `rustyline`)                         | Minimal                                                    |
| Online Catalog | Partial (`online-catalog` → `ureq` for fetch only) | Low                                                        |
| Telemetry      | **Not at all**                                     | **High** — 20+ refs across `eval/`, `main.rs`, `config.rs` |

## Changes

### Telemetry: Feature-gate + thin handle

#### 1. `ash/Cargo.toml`

```toml
[features]
default = ["repl", "telemetry"]
repl = ["rustyline"]
telemetry = []
```

Remove `online-catalog` feature and `ureq` dependency.

#### 2. `ash/src/telemetry_handle.rs` (NEW — always compiled)

Lightweight struct that either wraps a real `SpanContext` or is a no-op:

- `#[cfg(feature = "telemetry")]` impl delegates to real telemetry pipeline
- `#[cfg(not(feature = "telemetry"))]` impl is all no-ops (compiler eliminates them)

Methods: `try_init(config)`, `emit(kind, payload)`, `shutdown()`.

#### 3. `ash/src/lib.rs`

```rust
#[cfg(feature = "telemetry")]
pub mod telemetry;          // gates all 8 pipeline files
pub mod telemetry_handle;   // always compiled
```

#### 4. `ash/src/eval/mod.rs`

Replace `telemetry_ctx: Option<telemetry::context::SpanContext>` with `telemetry: Option<TelemetryHandle>`. All emit calls go through `self.telemetry.as_ref()?.emit(...)`.

#### 5. `ash/src/eval/agent.rs`

Remove direct `crate::telemetry::*` imports. Use `self.telemetry` handle methods.

#### 6. `ash/src/main.rs`

Replace 9 scattered `ash::telemetry::shutdown()` calls with a single centralized shutdown via the handle. No `#[cfg]` needed.

#### 7. `ash/src/config.rs`

Keep `TelemetryConfig` import — it is a lightweight data type with only serde deps, always compiled.

### Online Catalog: Remove online fetch, add hint

Online fetch brings an external dep (`ureq`) with little upside — the catalog has 11 stable agent entries. Once released, the endpoint becomes a backward-compat contract across versions. The embedded JSON + hardcoded fallback already cover all scenarios.

#### 8. `ash/src/engine/catalog.rs`

- Remove `CATALOG_URL`, `fetch_catalog()`, `load_catalog()` (or simplify to one-liner wrapping `embedded_catalog()`)
- Keep `embedded_catalog()`, `hardcoded_catalog()`, `parse_catalog_json()` — all zero-dep

#### 9. `ash/src/engine/discovery.rs`

- Replace `catalog::load_catalog()` → `catalog::embedded_catalog()` (3 callers)
- Print `"discovering agents available..."` before probing begins

#### 10. `ash/src/engine/mod.rs`

Add a function to print startup banner showing available agents:
```rust
pub fn print_agents_banner() {
    let agents = registered_agents();
    // print each configured/discovered agent name
    // then: println!("For supported agents and config docs, visit https://ash-lang.com");
}
```

#### 11. `ash/src/main.rs`

- Call `engine::print_agents_banner()` after `ensure_agents_registered()` on every startup
- The banner lists configured/discovered agents + the website hint

#### 12. `ash/src/engine/mod.rs`

- Remove `fetch_catalog` and `CATALOG_URL` from `pub use` exports

#### 13. `ash/Cargo.toml`

- Remove `ureq = { version = "3", optional = true }` from `[dependencies]`
- Remove `online-catalog = ["ureq"]` from `[features]`
- Update `default = ["repl", "telemetry"]`

#### 14. Move `agents.json` from web-site into crate

- The file is still needed for `include_str!` in catalog.rs but is no longer served by the website
- Move `web-site/agents.json` → `ash/src/engine/agents.json`
- Update `include_str!` path in catalog.rs: `"../../../web-site/agents.json"` → `"agents.json"`

#### 15. Website: supported agents page

- Create `web-site/agents.md` (or `.html`) listing all supported agents and their default configs
- Add link from `web-site/index.html` to the new page
- This replaces the discoverability role that `agents.json` served online

## Result

| Aspect                               | Before                         | After                                   |
| ------------------------------------ | ------------------------------ | --------------------------------------- |
| Telemetry compiled when disabled     | Yes (unused code)              | No (zero bytes)                         |
| `#[cfg]` clutter in evaluator        | 0 (no flag existed)            | 0 (handle hides it)                     |
| `#[cfg]` clutter in main.rs          | 0                              | 0 (shutdown centralized)                |
| Core eval code                       | Direct telemetry imports       | Handle method calls                     |
| Config parsing with telemetry off    | Works                          | Works (TelemetryConfig always compiled) |
| `ureq` dependency                    | Yes (optional)                 | Gone                                    |
| `online-catalog` feature flag        | Exists                         | Gone                                    |
| Agent discovery help                 | Network fetch to `agents.json` | Printed hint to website                 |
| Public API contract on `agents.json` | Yes                            | Gone                                    |
