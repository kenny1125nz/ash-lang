# 10 — AST Node Position Refactor

## Problem

`lang/ast.rs:336-378`: The `Node` enum has 35 variants. The `pos()` method (`ast.rs:380-425`) is a 35-arm match that must be manually updated whenever a variant is added or changed. No compile-time check ensures consistency.

## Fix

Add a `pos: Pos` field to every variant. Then `pos()` becomes a delegate to a shared accessor.

**Option A — Flatten pos into each variant** (preferred):
```rust
pub enum Node {
    AgentCall(Box<AgentCall>, Pos),
    BinaryTry(Box<BinaryTry>, Pos),
    // ... each variant gets a Pos field
}
```

**Option B — Wrap in a struct**:
```rust
pub struct AstNode {
    pub pos: Pos,
    pub node: NodeKind,
}

pub enum NodeKind { /* 35 variants */ }
```

Use **Option A** — it's less disruptive to existing code (all other fields stay the same, just add `pos` parameter to constructors).

**Implementation**:
1. Add `Pos` type (or reuse `LexerPos`) if not already defined
2. Add `Pos` field to each Node variant
3. Update all constructor call sites (parser.rs creates nodes) — pass position from the lexer token
4. `pub fn pos(&self) -> Pos` becomes a simple match that extracts the pos from each variant
5. Eventually: replace the 35-arm match with `#[automatically-derived]` or keep the match as it can't be derived

## Files

| Change | File |
|--------|------|
| Add `Pos` field to 35 Node variants | `ash/src/lang/ast.rs:L336-378` |
| Update constructor sites | `ash/src/lang/parser.rs` (all parse_* functions) |
| Update constructor sites | `ash/src/eval/agent.rs` (any AST construction) |
| Update `pos()` method | `ash/src/lang/ast.rs:L380-425` |

## Acceptance

- `cargo test` — all 211 tests pass
- `pos()` returns correct position for every node
- Adding a new Node variant requires adding a `Pos` parameter — compiler enforces it
- Error messages with position info (`at X:Y`) are unchanged

## Dependencies

- **03-parser-agent-dedup** — complete 03 first to avoid merge conflicts in parser.rs where most node constructors live

## Effort

3-4 hours (mechanically repetitive but straightforward)
