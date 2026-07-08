---
{"id": "ash-scripts", "title": "Ash Script Language"}
---

## Ash Script Language

Ash scripts (`.ash`) provide a full scripting language for composing AI agents into automated workflows.

### Agent calls

The `do` statement invokes an agent:

```ash
do "Fix the login bug in src/auth/login.ts" with opencode
```

| Clause | Example | Description |
|--------|---------|-------------|
| `with <agent>` | `with opencode` | Which agent to invoke |
| `using <model>` | `using sonnet` | Override the AI model |
| `in <path>` | `in "/workspace"` | Working directory for the agent |
| `compact <directive>` | `compact "truncate 32000"` | Context management |

### CLI flags

```bash
ash script.ash                     # run a script
ash tasks/                         # run directory mode
ash --check script.ash             # validate syntax only
ash -c script.ash                  # shorthand for --check
ash --agent opencode:sonnet        # default agent and model
ash tasks/ --dry-run               # preview without executing
ash --config path/to/ash.yml      # custom config file path
ash --continue-on-error            # keep going after task failure
ash -k                             # shorthand for --continue-on-error
```
