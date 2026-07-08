# Complexity Artifacts

Analysis of the ash Rust codebase identifying patterns and structures that introduce high complexity, duplicated code, and maintenance risk. See `challenge_me.md` section 8: the expression language is the weakest layer — these artifacts make that weakness worse.

## Top Priority Fixes

### P1: Evaluator fork pattern (5 duplication sites)

The `Evaluator { ... }` 18-field struct literal is hand-constructed in **5 locations**:

| # | Location | Context |
|---|---|---|
| 1 | `eval/conc.rs:21-36` | `eval_wait()` thread::spawn |
| 2 | `eval/conc.rs:62-77` | `eval_background()` thread::spawn |
| 3 | `runtime/tree.rs:462-479` | `execute_task_isolated()` |
| 4 | `runtime/tree.rs:564-579` | `execute_groups_isolated()` |
| 5 | `eval/mod.rs:94-112` | `Evaluator::new()` canonical constructor |

Every field added to `Evaluator` must be added to all 5 sites. This will cause bugs.

**Fix**: Add `Evaluator::fork(&self) -> Evaluator` that clones all fields. This eliminates sites 1-4. The canonical constructor stays.

### P2: Duplicated agent/model clause parsing

`parse_do()` (`lang/parser.rs:383-478`) and `parse_evaluate()` (`lang/parser.rs:592-708`) have near-identical `with`/`agent`/`subagent`/`using` parsing logic (~80 lines each).

**Fix**: Extract `parse_agent_with_clause()` helper.

### P3: Duplicated chrono_now() and is_leap()

Two independent copies of epoch-to-date formatting:

- `log.rs:39-92` / `log.rs:94-96`
- `engine/discovery.rs:245-291` / `engine/discovery.rs:293-295`

**Fix**: Extract shared utility. Or use the `chrono` crate (already a transitive dep via serde).

---

## Architecture Debt

### A. Circular module coupling: runtime → eval

`runtime/tree.rs` imports `eval::Evaluator`, creating a layering violation:

```
eval  → runtime (tree, scope, value, interpolation)
      → engine
      → lang

runtime/tree → eval  ← circular
```

**Fix**: Extract `run_tree()` to accept a `TaskExecutor` trait instead of `&mut Evaluator`. This breaks the direct dependency and lets the engine own execution orchestration.

### B. Global mutable singletons

- **Engine registry**: `engine/mod.rs:24-27` — `OnceLock<Mutex<HashMap<String, Arc<dyn Adapter>>>>` — makes unit tests fragile (shared state)
- **Telemetry pipeline**: `telemetry/mod.rs:17` — `OnceLock<Mutex<Option<Pipeline>>>` — same issue

Both require `lock().unwrap()` on every access. A single poisoned mutex crashes the process.

### C. 35-variant AST Node enum

`lang/ast.rs:336-378`: The `Node` enum has 35 variants. The `pos()` method (`ast.rs:380-425`) is a 35-arm match that must be updated for every new node. No compile-time check ensures it stays in sync.

**Fix**: Add a `pos: Pos` field to every variant (or `NodeBase { pos: Pos, node: NodeKind }` decomposition).

---

## Medium-Term

### D. Split eval/mod.rs (1,956 lines)

Monolithic evaluator file with 23 statement handlers and 1,300 lines of tests.

**Fix**: Extract into submodules:
- `eval/control.rs` — if/for/while/return/break/continue
- `eval/scope.rs` — scope push/pop/management
- Keep `eval/mod.rs` for `Evaluator` struct + top-level dispatch only

### E. Unify error types

`Result<_, String>` appears 50+ times across the codebase. No unified error type outside the eval module.

**Fix**: Use `thiserror` or `anyhow` crate. A single `AshError` enum would replace `format!("...")` error construction everywhere.

### F. Panic-prone mutex usage

~30 `.lock().unwrap()` calls across the codebase. Every one can panic on a poisoned mutex.

**Fix**: Replace with `.lock().map_err(...)` or use `PoisonError::into_inner()` to recover the guard.

### G. Unbounded OS threads

- `eval/conc.rs` — `thread::spawn()` for every `do <prompt> &` and `wait { }` statement
- `engine/adapter.rs` — **2 threads per agent call** (stdout + stderr readers)
- No thread pool, no backpressure

**Fix**: Use `rayon` or `threadpool` crate for background tasks.

### H. String-built YAML in config.rs

`config.rs:110-162`: `generate_yaml()` builds YAML via string concatenation with 6 helper functions (`yaml_line`, `yaml_line_opt`, `yaml_line_bool`, `yaml_line_args`, `agent_type_str`, `header_comment`).

**Fix**: Use `serde_yaml::to_string()` with a serializable struct.

---

## Quick Wins

| Item | Files | Effort | Impact |
|------|-------|--------|--------|
| P1: `Evaluator::fork()` | 5 sites in 2 files | 1 day | Eliminates brittle duplication |
| P2: parser dedup | 2 sites in `parser.rs` | 2 hours | Removes ~120 duplicated lines |
| P3: `chrono_now()` dedup | `log.rs`, `discovery.rs` | 1 hour | Removes ~90 duplicated lines |
| G: parallel group execution dedup | `tree.rs:557-615, 708-732` | 2 hours | Unifies 2 nearly-identical patterns |
| H: `serde_yaml` for config | `config.rs:110-162` | 4 hours | Eliminates 6 helper functions |
