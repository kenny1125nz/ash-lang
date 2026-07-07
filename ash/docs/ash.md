# Ash — Quick Start

**Version:** 0.2.1   **File Extension:** `.ash`

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
| `with <agent>` | `with opencode` | Which agent to invoke (opencode, claude-code, kimi, codex, ...) |
| `using <model>` | `using sonnet` | Override the AI model |

---

## CLI Flags

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

| Frontmatter key | Values         | Default | Description                              |
|-----------------|----------------|---------|------------------------------------------|
| `agent`         | agent name     | —       | Override agent for this task             |
| `model`         | model name     | —       | Override model for this task             |
| `on_fail`       | `stop`, `continue` | `stop` | Behavior when the task fails           |
| `compact`       | directive      | —       | Context window strategy for this task    |

Use `ash --continue-on-error` (or `-k`) to keep running after task failures regardless of `on_fail`.

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

A toggle form (`begin` / `end`) spans non-contiguous code without nesting:

```ash
session begin
do "Research approach"
do "Draft prototype"
session end

# ... intervening code ...

session begin
do "Finalize implementation"
session end
```

Mix toggle and block forms as long as they are not nested. `session begin` errs if a session is already active. `session end` errs without a matching `begin`.

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

### Try blocks

Binary try — retries on failure, runs an optional fail block:

```ash
try {
  do "Deploy to staging"
} fail {
  print "deployment failed, rolling back"
} upto 3
```

Eval try — evaluates a condition after each attempt:

```ash
try {
  do "Generate report"
} evaluate with {
  SCORE >= 85
} accept {
  print "quality threshold met"
} partial {
  print "attempt ${ATTEMPT} below threshold, retrying"
  ATTEMPT = ATTEMPT + 1
} fail {
  print "unexpected error"
} upto 5
```

| Clause     | Runs when                         |
|------------|-----------------------------------|
| `accept`   | Evaluator returns truthy / $? == 0 |
| `partial`  | Evaluator returns falsy / $? == 1 |
| `fail`     | Body errors or evaluator $? >= 2  |
| `upto N`   | Maximum retry count                |

The `error` variable is set to the error message when a body statement fails. The `report` variable captures `print` output from the evaluator block.

---

## Directory Scoping (`within`)

Run code inside a specific directory — block form:

```ash
within "/tmp" {
  CWD = $(pwd)
  print "inside ${CWD}"
}
```

Toggle form (`begin` / `end`) with stack-based nesting:

```ash
within begin "/tmp"
  do "work in tmp"
within begin "/var"
  do "also work in var"
within end
within end
```

`within begin` with a non-existent path errors. `within end` without matching `begin` errors.

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
| `return [val]` | Return from a function |
| `break` | Exit a for/while loop |
| `continue` | Skip to next loop iteration |
| `compact "directive"` | Set context window strategy |
| `within <dir> { }` | Run block in a different directory |

---

## Background & Parallel Execution

Run a statement in the background with `&`:

```ash
do "Long running analysis" &
print "main continues immediately"
wait
```

Run multiple statements in parallel with `wait { }`:

```ash
wait {
  do "Train model A"
  do "Train model B"
}
print "both models done"
```

---

## Arrays

Array literals and index access:

```ash
FRUITS = ["apple", "banana", "cherry"]
print len(FRUITS)           # → 3
print FRUITS[0]              # → apple
MIXED = ["hello", 42, true]  # mixed types
EMPTY = []
```

Concatenate with `+`:

```ash
A = [1, 2]
B = [3, 4]
C = A + B                    # [1, 2, 3, 4]
```

Loop over arrays with `for`:

```ash
for ITEM in ["x", "y", "z"] {
  print ITEM
}
```

Built-in functions: `len()` (string or array length), `range(N)` / `range(start, end)`.

---

## File-based Prompts (`@file`)

Load a prompt from a file with variable interpolation:

```ash
VAR1 = "hello"
VAR2 = "world"
do @"path/to/prompt.md"
```

The file is read, `${VAR}` placeholders are resolved, and the result is sent to the agent.

---

## Compact Mode

Control context window strategy:

```ash
compact "truncate 32000"    # truncate to 32K tokens
compact "summarize"          # summarize older context
```

Per-call compact overrides on `do`:

```ash
do "Review this file" compact "summarize"
```

---

## Agent Discovery

```bash
ash discover            # list available agents (parallel probe)
ash discover --write    # generate ash.yml
```

On startup, ash auto-discovers installed agents from a built-in template (opencode, claude-code, kimi, codex, gemini-cli, pi, goose, qwen-code, amazon-q, aider, echo). Only agents found on PATH are registered. Custom agents can be added to `ash.yml`:

```yaml
agents:
  custom-tool:
    type: local-cli
    cmd: my-tool
    message_flag: "--prompt"
    yes_flag: "--yes"
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

| Variable  | Set by                   | Description                                              |
|-----------|--------------------------|----------------------------------------------------------|
| `$?`      | `do`, `exec`, fn calls   | Exit code (0 = success)                                  |
| `stdout`  | `do`, `exec`             | Stdout from the last call                                |
| `stderr`  | `do`, `exec`             | Stderr from the last call                                |
| `error`   | eval try body failure     | Error message when a body statement fails                |
| `report`  | eval try evaluator block  | Captured `print` output from the `evaluate with` block   |

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
