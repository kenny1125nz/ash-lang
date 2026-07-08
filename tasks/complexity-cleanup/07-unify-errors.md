# 07 — Unify Error Types

## Problem

`Result<_, String>` appears 50+ times across the codebase. Only the eval module has a proper error enum (`EvalError`). Everywhere else uses stringly-typed errors with `format!("...")` construction. This makes error handling inconsistent, untyped, and impossible for callers to pattern-match.

## Fix

1. Add a crate-level error enum in a new file `ash/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AshError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("eval error: {0}")]
    Eval(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Msg(String),
}
```

2. Propagate `AshError` (or `Result<T, AshError>`) through the codebase. Replace `Result<_, String>` returns.

3. Keep `EvalError` as a separate type for the evaluator's internal flow control (exit codes, etc.) — it remains `Exit` + `Msg`. But make `EvalError → AshError` conversion trivial.

4. Add `From<EvalError> for AshError` and `From<std::io::Error> for AshError`.

**Evaluation order**: Only change module boundaries and error return types. Don't touch internal logic. Start with `formats!` that create error messages — those become `AshError::Msg(...)`.

## Files

| Change | File |
|--------|------|
| Create error enum | `ash/src/error.rs` (new) |
| Add `pub mod error;` | `ash/src/lib.rs` |
| Add `pub use error::AshError;` | `ash/src/lib.rs` |
| Update returns | `ash/src/config.rs`, `ash/src/lang/lexer.rs`, `ash/src/lang/parser.rs`, `ash/src/runtime/executor.rs`, `ash/src/engine/*.rs` |
| Use `thiserror` | `ash/Cargo.toml` (add dependency) |

## Acceptance

- `cargo test` — all 211 tests pass
- No `Result<_, String>` returns on public functions (internal helpers may remain)
- Error messages are identical to before
- `from` implementations for `AshError` cover all existing error sources

## Dependencies

- **06-split-eval-mod** — moves code; errors in moved functions are in the new locations. Do 06 first so error changes are applied to final module layout.

## Effort

2-3 days (touches many files)
