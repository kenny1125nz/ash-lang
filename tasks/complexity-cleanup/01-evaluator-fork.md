# 01 — Evaluator::fork()

## Problem

The `Evaluator { ... }` 18-field struct literal is hand-constructed at **4 parallel-execution sites** plus the canonical `Evaluator::new()`. Every field addition to `Evaluator` must be manually added to all 5 sites.

## Files

| Change | File |
|--------|------|
| Add `fn fork(&self) -> Evaluator` | `ash/src/eval/mod.rs:L93-112` (after `new()`) |
| Replace struct literal with `eval.fork()` | `ash/src/eval/conc.rs:L21-36` |
| Replace struct literal with `eval.fork()` | `ash/src/eval/conc.rs:L62-77` |
| Replace struct literal with `eval.fork()` | `ash/src/runtime/tree.rs:L457-476` (`execute_task_isolated`) |
| Replace struct literal with `eval.fork()` | `ash/src/runtime/tree.rs:L563-579` (`execute_groups_isolated`) |

## Design

```rust
pub fn fork(&self) -> Evaluator {
    Evaluator {
        current_scope:   self.current_scope.clone(),
        global_scope:    self.global_scope.clone(),
        stdout:          self.stdout.clone(),
        stderr:          self.stderr.clone(),
        executor:        Executor::new(),                    // fresh per-thread
        compact_config:  self.compact_config.clone(),
        signal:          self.signal.clone(),
        bg_handles:      self.bg_handles.clone(),
        default_agent:   self.default_agent.clone(),
        default_model:   self.default_model.clone(),
        session_depth:   0,                                  // reset per-thread
        within_stack:    Vec::new(),                         // reset per-thread
        telemetry_ctx:   None,                               // reset per-thread
        script_args:     self.script_args.clone(),
    }
}
```

Clones all shareable state. Resets per-thread state. Callers that need different values mutate after forking.

## Acceptance

- `cargo test` — all 211 tests pass
- No new `Evaluator { ... }` struct literals appear outside `new()` and `fork()`
- Each of the 4 sites that previously built the struct literal now calls `fork()`

## Dependencies

None — this is the foundation task. Other cleanup tasks depend on the `fork()` interface being stable.

## Effort

< 1 day
