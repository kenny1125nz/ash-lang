# 03 — Parser Agent+Model Clause Dedup

## Problem

`parse_do()` (`lang/parser.rs:383-478`) and `parse_evaluate()` (`lang/parser.rs:592-708`) contain near-identical parsing for `with <agent> [subagent <name>] [using <model>] [compact <directive>]` — ~80 duplicated lines.

Every bugfix or addition to the agent-clause grammar must be applied to two places.

## Fix

Extract `parse_agent_clause(&self, tokens: &mut ...) -> (Option<Expr>, Option<String>, Option<Expr>, Option<CompactDirective>)` that handles the optional trailing keywords after the body.

Returning:
- `agent` — the `with <agent>` expression
- `subagent` — the `subagent <name>` string  
- `model` — the `using <model>` expression
- `compact` — the `compact <directive>` parsed directive

## Files

| Change | File |
|--------|------|
| Add `fn parse_agent_clause()` | `ash/src/lang/parser.rs` (between parse_do and parse_evaluate) |
| Replace inline parsing in `parse_do()` | `ash/src/lang/parser.rs:L401-459` |
| Replace inline parsing in `parse_evaluate()` | `ash/src/lang/parser.rs:L620-675` |

## Acceptance

- `cargo test` — all 211 tests pass (especially parser tests)
- Both `parse_do` and `parse_evaluate` call the shared helper
- No logic changes — identical AST output for all valid programs

## Dependencies

None — parser is self-contained. However, task **10-ast-pos-field** will also touch the parser; complete 03 first to avoid conflicts in that area.

## Effort

< 3 hours
