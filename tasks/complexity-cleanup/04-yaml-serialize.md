# 04 — YAML Serialization Cleanup

## Problem

`config.rs:110-162`: `generate_yaml()` builds YAML via string concatenation with 6 helper functions:
- `yaml_line()`
- `yaml_line_opt()`
- `yaml_line_bool()`
- `yaml_line_args()`
- `agent_type_str()`
- `header_comment()`

The config is already deserialized with `serde_yaml`. Serialization should use the same mechanism.

## Fix

1. Derive `Serialize` on the YAML-facing structs (`ConfigFile`, `AgentDef`, `AgentType`)
2. Replace `generate_yaml()` with `serde_yaml::to_string()`
3. Remove the 6 helper functions
4. Update tests if YAML output format changes slightly (serde will produce standard formatting)

## Files

| Change | File |
|--------|------|
| Add `#[derive(Serialize)]` to structs | `ash/src/config.rs` |
| Replace `generate_yaml()` body | `ash/src/config.rs:L110-162` |
| Remove helper functions | `ash/src/config.rs:L164-197` |
| Update tests (maybe) | `ash/src/config.rs:L199-401` |

## Acceptance

- `cargo test` — all 211 tests pass
- Generated YAML is valid and contains the same data
- `ash discover --write` produces parseable ash.yml
- No manual string building for YAML output remains

## Dependencies

None.

## Effort

< 4 hours
