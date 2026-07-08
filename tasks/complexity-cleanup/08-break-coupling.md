# 08 — Break runtime↔eval Circular Coupling

## Problem

`runtime/tree.rs` imports `eval::Evaluator` directly, creating a circular layering violation:

```
eval → runtime (tree, scope, value)  
runtime/tree → eval (Evaluator, EvalError)  ← circular
```

This means the "lower" runtime layer depends on the "higher" evaluation layer. Any change to the Evaluator struct potentially breaks the tree walker.

## Fix

Inject a `TaskExecutor` trait instead of `&mut Evaluator` into `run_tree()` and all helper functions.

```rust
pub trait TaskExecutor {
    fn execute_task(&self, task: &Task, config: &WalkConfig) -> Result<(), Error>;
    fn fork(&self) -> Box<dyn TaskExecutor + Send>;
    fn default_agent(&self) -> &str;
    fn default_model(&self) -> &str;
    // etc.
}
```

The `Evaluator` implements `TaskExecutor`. The tree walker only knows about the trait, not the concrete type.

**Impact on known callers:**

| Caller | Change |
|--------|--------|
| `main.rs` L340-346 | `run_tree(config, &mut eval)` → `run_tree(config, &eval as &dyn TaskExecutor)` |
| `eval/agent.rs` L208 | Already passes `self` (an Evaluator) — now passed as trait object |

**Implementation:**

1. Define `TaskExecutor` trait in `runtime/tree.rs` (not in eval — keeps trait in runtime domain)
2. Parameterize `run_tree()`, `execute_task()`, `execute_md_task()`, `execute_ash_task()`, `execute_task_isolated()`, `execute_groups_isolated()` on `&dyn TaskExecutor` instead of `&mut Evaluator` / `&Evaluator`
3. Move the Evaluator → TaskExecutor impl into `eval/mod.rs` (a new `impl` block)
4. Remove `use crate::eval::{EvalError, Evaluator}` from `tree.rs`

## Files

| Change | File |
|--------|------|
| Define `TaskExecutor` trait | `ash/src/runtime/tree.rs` |
| Replace `Evaluator` with `dyn TaskExecutor` | `ash/src/runtime/tree.rs` (all execution functions) |
| Remove eval imports | `ash/src/runtime/tree.rs:L7` |
| Add `impl TaskExecutor for Evaluator` | `ash/src/eval/mod.rs` |
| Update call sites | `ash/src/main.rs:L340-346`, `ash/src/eval/agent.rs:L208` |

## Acceptance

- `cargo test` — all 211 tests pass
- `runtime/tree.rs` has zero imports from `crate::eval`
- Both CLI mode and `do @"dir/"` in scripts work identically
- Parallel execution works (trait object must be `Send + Sync`)

## Dependencies

- **01-evaluator-fork** — the trait's `fork()` method delegates to `Evaluator::fork()`
- **05-tree-parallel-dedup** — 05 simplifies the tree's parallelism before we add abstraction
- **06-split-eval-mod** — the `impl TaskExecutor for Evaluator` goes in the refactored eval structure

## Effort

2 days
