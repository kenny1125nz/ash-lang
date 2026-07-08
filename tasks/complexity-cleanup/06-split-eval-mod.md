# 06 — Split eval/mod.rs

## Problem

`eval/mod.rs` is 1,956 lines — 23 statement handlers, scope management, control flow, and 1,300 lines of tests. It's the largest file in the codebase and has the most responsibilities.

## Fix

Move categories of code into new submodules. The `Evaluator` struct, `new()`, `fork()`, top-level `eval_script()`, and `eval_statement()` dispatch stay in `mod.rs`. Everything else moves out.

**New files:**

| New file | Content moved from `mod.rs` |
|----------|----------------------------|
| `eval/control.rs` | `eval_if`, `eval_for`, `eval_while`, `eval_return`, `eval_break`, `eval_continue`, `eval_exit`, `eval_include` |
| `eval/scope.rs` | `push_scope`, `pop_scope`, scope accessor methods |

`eval/agent.rs` already handles agent calls, `eval/conc.rs` handles concurrency, `eval/expr.rs` handles expressions — these stay.

**Kept in `mod.rs`:**

- `Evaluator` struct + `new()` + `fork()`
- `eval_script()`
- `eval_statement()` — the main dispatch (match on `Node::*` → delegate)
- `eval_statements()` — loop over statements
- Top-level state: `ScopeRef`, `SharedWriter`, `SignalKind`, `FlowSignal`, `EvalError`, `ExitError`
- Public accessors: `set_default_agent`, `set_default_model`, `set_args`

**Tests** — each test moves to the new module that owns the function it tests. Tests for `eval_script()` order/variables stay in `mod.rs`.

## Files

| Change | File |
|--------|------|
| Create new file | `ash/src/eval/control.rs` |
| Create new file | `ash/src/eval/scope.rs` |
| Remove moved code + `mod` declarations | `ash/src/eval/mod.rs` |
| Move scope/control tests | `ash/src/eval/{mod → control,scope}.rs` |

## Acceptance

- `cargo test` — all 211 tests pass
- `mod.rs` is < 800 lines (down from 1,956)
- Each new file is < 500 lines
- No logic changes — pure code movement
- `pub(super)` visibility on moved functions (only accessible within `eval` module)

## Dependencies

- **01-evaluator-fork** — the moved functions may reference `fork()`; this task works with the new method
- **05-tree-parallel-dedup** — do 05 first so tree.rs is stable before touching eval structure

## Effort

1-2 days
