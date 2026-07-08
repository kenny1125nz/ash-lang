# 02 — Utility Dedup

## Problem

Two files contain identical `chrono_now()` and `is_leap()` implementations:

- `ash/src/log.rs:39-92` / `ash/src/log.rs:94-96`
- `ash/src/engine/discovery.rs:245-291` / `ash/src/engine/discovery.rs:293-295`

Both implement epoch-to-date formatting with leap-year calculation.

## Fix

1. **Extract into a shared module** `ash/src/runtime/date.rs` (or use the `chrono` crate if it's already a dependency):

```rust
// ash/src/runtime/date.rs
pub fn timestamp_now() -> String {
    // ... current chrono_now logic, use nanosecond variant ...
}

pub fn is_leap(year: u64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}
```

2. Update both call sites to use the shared function

## Files

| Change | File |
|--------|------|
| Create new file | `ash/src/runtime/date.rs` |
| Add `pub mod date;` | `ash/src/runtime/mod.rs` |
| Replace `chrono_now()` + `is_leap()` | `ash/src/log.rs:L39-96` |
| Replace `chrono_now()` + `is_leap()` | `ash/src/engine/discovery.rs:L245-295` |

## Acceptance

- `cargo test` — all 211 tests pass
- No warnings
- No duplicated `is_leap` or `chrono_now` implementations remain
- Timestamps produced are identical to before the change

## Dependencies

None — completely independent.

## Effort

< 2 hours
