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
| `$_attempt` | `evaluate` body | Current attempt number, 1-indexed |
| `$_max_attempts` | `evaluate` body | Total allowed attempts |
| `$_feedback` | `evaluate` body | Findings from the previous evaluation iteration |
| `$_evaluator_output` | `evaluate` evaluator | Full stdout from the evaluator |
| `$score` | `evaluate` post-loop | The final score (accepted or last attempted) |
| `$accepted` | `evaluate` post-loop | Whether the threshold was met |

The `$` prefix is supported for backward compatibility (`$FILES`, `$DIFF`). Bare names without `$` are preferred.
