# Improved Logging

## Background

Ash currently has no structured logging. Debugging requires ad-hoc `eprintln!` / `println!` statements or relying on agent output. There is no way to control verbosity or persist logs for post-mortem analysis. Key integration points — agent communication, task dispatch, file operations — have no instrumentation.

The only existing mechanism is the `ASH_LOG` environment variable mentioned in design notes, but it is not implemented.

## Intended Solution

Add a logging module that writes to a log file with configurable levels controlled by the `ASH_LOG` environment variable.

### Log levels (increasing verbosity)

| Level   | Purpose                         |
| ------- | ------------------------------- |
| `error` | Recoverable failures            |
| `warn`  | Unexpected but non-fatal states |
| `info`  | Major lifecycle events          |
| `debug` | High-level flow tracing         |
| `trace` | Detailed function-level logging |

### CLI invocation

```
ASH_LOG=debug ash run tasks/...
ASH_LOG=info ash tasks/ready/
ASH_LOG=error ash tasks/backlog/01-improved-logging.md
```

Default level (when `ASH_LOG` is unset): `warn`.

### Log output format

Each line: `{timestamp} [{level}] {module} — {message}`

Example:
```
2025-07-06T14:30:01Z [info] engine — dispatching task tasks/ready/04-github.md
2025-07-06T14:30:01Z [debug] agent — connecting to opencode at localhost:8080
2025-07-06T14:30:02Z [error] agent — connection refused, retry 1/3
```

### Integration points to instrument

- **Engine startup / shutdown** (`engine/`)
- **Task file load and dispatch** (`eval/`, `executor.rs`)
- **Agent connect / send / receive** (`engine/`)
- **Agent stdout and stderr** (`engine/`)
- **File read / write** (`interpolation.rs`, `tree.rs`)
- **Scope entry / exit** (`scope.rs`)

## Acceptance Criteria

1. **Environment control** — `ASH_LOG` controls the minimum log level; unset defaults to `warn`
2. **Log file output** — logs are written to `ash.log` (or `ASH_LOG_FILE` if set) alongside the program output
3. **Level filtering** — `ASH_LOG=error` shows only errors; `ASH_LOG=debug` shows debug and above
4. **Instrumentation** — each integration point above produces at least one log entry at the appropriate level
5. **No regression** — existing tests pass with no behavioral change when `ASH_LOG` is unset

## Implementation Hints

### Relevant project context

The project is a single Rust crate at `ash/`. Current dependencies (`Cargo.toml`) only include `regex = "1"`.

**Existing print statements to replace / augment:**
- `eprintln!` / `println!` used in `main.rs`, `repl.rs`, `executor.rs` for errors and diagnostics
- `tree.rs` prints parsed task structure for debugging

**Logging approach:**
- Use the `log` crate for the logging facade — it's the Rust standard and has zero dependencies
- Use a lightweight file logger backend (e.g., `simplelog` or a minimal custom `Write`-based logger)
- Alternatively, implement a minimal logger directly using the `log` crate's `Log` trait — this avoids adding any transitive dependencies beyond `log`
- The logger should be initialized once in `main.rs` during startup, before the engine begins

**Pattern from existing code:**
- `main.rs` parses CLI args and dispatches to `executor.rs` or `repl.rs`
- `library.rs` (if it exists) may be the right place for shared utilities like logging — or create a new `log.rs` module

### What not to touch

- The ash scripting language parser (`lexer.rs`, `parser.rs`, `ast.rs`, `token.rs`, `value.rs`)
- Test data and test scripts (`testdata/`)
- Task definition files (`tasks/`)

### Implementation order

1. Add `log` crate (and choice of backend) to `Cargo.toml`
2. Create a logging module that reads `ASH_LOG`, initializes the logger to append to a file
3. Replace existing `eprintln!` / `println!` calls at integration points with `log::info!`, `log::debug!`, etc.
4. Run tests to verify no regression
