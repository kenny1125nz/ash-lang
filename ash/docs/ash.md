# Ash — Quick Start

**Version:** 0.1.0   **File Extension:** `.ash`

Ash is a scripting language for composing AI coding agents into automated workflows. You tell agents what to do, and ash orchestrates their execution.

See [ash-advanced.md](ash-advanced.md) for retry logic, context compacting, parallel execution, and other advanced features.

---

## Agent Calls

The `do` statement invokes an agent:

```ash
do "Fix the login bug in src/auth/login.ts" with opencode
```

| Clause | Example | Description |
|--------|---------|-------------|
| `with <agent>` | `with opencode` | Which agent to invoke (opencode, claude-code, aider) |
| `using <model>` | `using sonnet` | Override the AI model |

---

## CLI Flags

```bash
ash script.ash                     # run a script
ash tasks/                         # run directory mode
ash --check script.ash             # validate syntax only
ash --agent opencode:sonnet        # default agent and model
ash tasks/ --dry-run               # preview without executing
```

---

## Directory-Based Orchestration

Pass a directory to run task files in numeric-prefix order:

```bash
ash ./tasks
ash ./tasks --dry-run
```

```
tasks/
├── 01-intro.md
├── 02-types/
│   ├── 01-token.md
│   ├── 02-value.md
│   └── 03-ast.md
└── 03-conclusion.md
```

Execution order: `01-intro.md` → `02-types/01-token.md` → `02-types/02-value.md` → `02-types/03-ast.md` → `03-conclusion.md`

Files are executed when they have a numeric prefix (`01-`, `02-`, etc.). Files with duplicate prefixes at the same level are reported as errors.

Markdown files (`.md`) can set per-task settings with YAML frontmatter:

```markdown
---
agent: opencode
model: sonnet
on_fail: continue
---

# Task Title

The prompt content for the agent goes here...
```

Ash scripts (`.ash`) use a shebang line: `#!opencode:1.0`. Run `ash tasks/` to execute all tasks in sequence.

---

## Session Blocks

Keep multiple `do` calls in a shared agent session:

```ash
session {
  do "Implement the token types"
  do "Implement the value system"
  do "Refactor the AST"
}
```

---

## Control Flow

All bodies use `{ }`:

```ash
if $? == 0 {
  print "build passed"
} else if SCORE > 0.8 {
  print "good enough"
} else {
  print "fail"
  exit 1
}
```

```ash
for FILE in FILES {
  exec eslint FILE
}
```

```ash
while RETRIES < 3 {
  do "Fix remaining issues"
  RETRIES = RETRIES + 1
}
```

---

## Functions

```ash
fn review(FILE, MODEL) {
  do "Review ${FILE}" with opencode using MODEL
}

fn build() {
  exec npm run build
}
```

Call with parentheses — required for both built-in and user functions:

```ash
FILES = get_src_files()
review("src/auth.ts", "sonnet")
build()
```

---

## Built-in Tools

| Statement | Description |
|-----------|-------------|
| `exec cmd` | Run a shell command |
| `print msg` | Print output |
| `include "file.ash"` | Load another ash script |
| `env KEY` | Read an environment variable |
| `exit code` | Exit the script |

Inline command substitution inside strings:

```ash
FILES = "$(find src -name '*.ts')"
```

---

## Agent Discovery

```bash
ash discover            # list available agents
ash discover --write    # generate ash-project.yaml
```

On startup, ash auto-discovers installed agents. Add custom agents to `ash-project.yaml`:

```yaml
agents:
  custom-tool:
    type: local-cli
    cmd: my-tool
    message_flag: "--prompt"
```

---

## REPL

Run `ash` with no arguments to enter interactive mode:

```bash
$ ash
ash> NAME = "world"
ash> print "hello ${NAME}"
hello world
```

Commands: `.help`, `.clear`, `.vars`, `.exit`. Up/down arrows navigate history. Multi-line blocks (`if`, `for`, `session`) auto-detect continuation.

---

## Variables

Assign with `=` and reference by name:

```ash
NAME = "world"
COUNT = 42
MSG = "hello ${NAME}"
```

Built-in variables:

| Variable | Set by | Description |
|----------|--------|-------------|
| `$?` | `do`, `exec`, function calls | Exit code (0 = success) |
| `stdout` | `do`, `exec` | Stdout from the last call |
| `stderr` | `do`, `exec` | Stderr from the last call |

---

## Strings & Expressions

Double-quoted strings with `${var}` and `$(cmd)` interpolation:

```ash
MSG = "Branch: $(git branch --show-current), user: ${USER}"
```

Expressions:

```ash
$? == 0              # equality
COUNT > 5            # comparison
OK and READY         # boolean logic
not FAILED
TOTAL + 1            # arithmetic
N * 2                # multiplication
(X > 0 and Y < 10)   # grouping
```

---

## Complete Example

```ash
fn review(FILE) {
  do "Review ${FILE} for bugs" with opencode
}

FILES = $(find src -name '*.ts')
for FILE in FILES {
  review(FILE)
}

exec npm test
if $? == 0 {
  print "all good"
} else {
  print "tests failed"
  exit 1
}
```
