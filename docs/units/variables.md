---
{"id": "variables", "title": "Variables"}
---

## Variables

Assign with `=` and reference by name:

```ash
NAME = "world"
COUNT = 42
MSG = "hello ${NAME}"
```

### Built-in variables

| Variable | Set by | Description |
|----------|--------|-------------|
| `$?` | `do`, `exec`, fn calls | Exit code (0 = success) |
| `stdout` | `do`, `exec` | Stdout from the last call |
| `stderr` | `do`, `exec` | Stderr from the last call |
| `error` | eval try body failure | Error message when a body statement fails |
| `report` | eval try evaluator block | Captured `print` output from the `evaluate with` block |

The `$` prefix is supported for backward compatibility (`$FILES`, `$DIFF`). Bare names without `$` are preferred.
