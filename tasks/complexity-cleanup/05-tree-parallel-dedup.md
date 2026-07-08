# 05 — Tree Parallel Execution Dedup

## Problem

`runtime/tree.rs` has **two** nearly-identical parallel execution patterns that both spawn scoped threads:

1. `execute_groups_isolated()` (L557-615) — used by subdirectory walks
2. Inline scope in `run_tree()` (L706-732) — used by top-level groups

Both capture `config_ref`/`eval_ref`, spawn threads per file task, spawn a subdir thread, collect results. They differ slightly in result collection but do the same thing.

## Fix

Extract `execute_group_parallel()` that takes a `&TaskGroup`, a `&WalkConfig`, and a `&Evaluator`, spawns threads for all files + optional subdir, and returns `Vec<(bool, Option<&Task>)>`.

Both `run_tree()` and `execute_groups_isolated()` call this shared function. `execute_groups_isolated` should become a thin wrapper: iterate groups, for sequential ones call `execute_task()`, for parallel ones call `execute_group_parallel()`.

Note: this task requires `Evaluator::fork()` to exist (01) — the shared function uses it internally.

## Files

| Change | File |
|--------|------|
| Add `fn execute_group_parallel()` | `ash/src/runtime/tree.rs` |
| Replace inline scope in `run_tree()` | `ash/src/runtime/tree.rs:L706-732` |
| Replace parallel block in `execute_groups_isolated()` | `ash/src/runtime/tree.rs:L589-613` |

## Acceptance

- `cargo test` — all 211 tests pass
- Only one implementation of the scoped-thread-spawn pattern exists
- Dry run, `--yes` parallel execution, and file+subdir parallel all work identically to before

## Dependencies

- **01-evaluator-fork** — uses `Evaluator::fork()` inside the shared function

## Effort

< 3 hours
