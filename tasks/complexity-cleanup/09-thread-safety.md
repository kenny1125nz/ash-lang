# 09 — Thread Safety Hardening

## Problem

### Unbounded OS threads
- `eval/conc.rs` — `thread::spawn()` for every `do <prompt> &` and `wait { }` statement
- `engine/adapter.rs` — **2 threads per agent call** (stdout + stderr readers)
- No thread pool, no backpressure, no resource limits

### Poison-prone mutexes
~30 `.lock().unwrap()` calls across the codebase. A single panic in a locked scope poisons the mutex and crashes every subsequent access.

## Fix

### Part A — Thread pool for adapter I/O

Replace the 2-threads-per-call pattern in `engine/adapter.rs` with a bounded thread pool (use `rayon` or the `threadpool` crate, or `std::thread` with a manually-managed pool). Limit to 4 worker threads.

### Part B — Thread pool for background/parallel tasks

Replace `thread::spawn()` in `eval/conc.rs` with a shared thread pool. `wait { }` and `do <prompt> &` submit tasks to the pool instead of creating new threads.

### Part C — Mutex poison recovery

Replace `.lock().unwrap()` with a helper that recovers from poisoning:
```rust
fn lock_guard<T>(mu: &Mutex<T>) -> MutexGuard<'_, T> {
    mu.lock().unwrap_or_else(|e| e.into_inner())
}
```
Apply to all 30+ call sites.

Or, if using `AshError` from task 07: return `Result` instead of panicking, converting poison into `AshError::Msg("mutex poisoned")`.

## Files

| Part | Files |
|------|-------|
| A | `ash/src/engine/adapter.rs:L73-115` |
| B | `ash/src/eval/conc.rs:L20-80` |
| C | `ash/src/eval/mod.rs`, `ash/src/runtime/scope.rs`, `ash/src/engine/mod.rs`, `ash/src/telemetry/*.rs` — all `.lock().unwrap()` sites |
| Config | `ash/Cargo.toml` (add `rayon` or `threadpool`) |

## Acceptance

- `cargo test` — all 211 tests pass
- Adapter tests (`engine::driver::tests::*`) pass with thread pool
- Parallel tests (`eval::conc::tests::*`) pass with thread pool
- No `.lock().unwrap()` calls remain (all use poison-safe helper)
- Concurrent agent calls don't exceed thread pool limit
- If a task panics inside a lock, subsequent tasks still run (poison is recovered)

## Dependencies

- **07-unify-errors** — poison recovery returns `AshError` instead of panicking
- **08-break-coupling** — thread pool works through the `TaskExecutor` trait, not raw evaluator

## Effort

2 days
