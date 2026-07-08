---
{"id": "builtins", "title": "Built-in Tools"}
---

## Built-in Tools

| Statement | Description |
|-----------|-------------|
| `exec cmd` | Run a shell command |
| `evaluate { } by ... accept by ... upto N` | Retry body until a score threshold is met, evaluated by an agent/function/command |
| `use <agent>` | Set the default agent for subsequent `do` calls |
| `print msg` | Print output |
| `include "file.ash"` | Load another ash script |
| `env KEY` | Read an environment variable |
| `exit code` | Exit the script |
| `return [val]` | Return from a function |
| `break` | Exit a for/while loop |
| `continue` | Skip to next loop iteration |
| `compact "directive"` | Set context window strategy |
| `within <dir> { }` | Run block in a different directory |
