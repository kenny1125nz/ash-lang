# Ash — Advanced Features

This document covers advanced features for users already familiar with the basics in [ash.md](ash.md).

---

## Engine Declaration

The first line declares the target agent and version. An optional `#!compact` line configures context management globally:

```ash
#!opencode:1.2.0
#!opencode:1.2.0:sonnet

#!opencode:1.2.0
#!compact mode=on window=64000 strategy=truncate
```

| Line | Pattern | Description |
|------|---------|-------------|
| 1 | `#!<engine>:<version>[:<model>]` | Target agent engine with version |
| 2 | `#!compact <key>=<val> ...` | Global compact settings (optional) |

---

## Advanced Variables

### Built-in Special Variables

| Variable | Set by | Description |
|----------|--------|-------------|
| `$?` | any agent call or exec | Exit code of the last agent call or command |
| `${stdout}` | any agent call or exec | Stdout output from the last call |
| `${stderr}` | any agent call or exec | Stderr output from the last call |
| `${report}` | `try { } evaluate with { }` | Stdout from the evaluator block |

The `$` prefix is supported for backward compatibility (`$FILES`, `$DIFF`). Bare names without `$` are preferred.

### Arrays

Arrays are ordered, comma-separated lists:

```ash
COUNTRIES = ["United States", "United Kingdom", "Canada"]
SCORES    = [95, 87, 92, 78]
```

Index access (zero-based):

```ash
print COUNTRIES[0]   # "United States"
```

Concatenation with `+`:

```ash
A = [1, 2, 3]
B = [4, 5]
C = A + B    # [1, 2, 3, 4, 5]
```

Iterate directly:

```ash
for ITEM in ["apple", "banana", "cherry"] {
  print ITEM
}
```

Built-in function:

| Function | Description |
|----------|-------------|
| `len(a)` | Returns the number of elements in `a` |

---

## Strings & Text Blocks

### Text blocks

Multi-line literals delimited by triple backticks. Only ` ``` ` closes the block:

````
PROMPT = ```
Fix the login bug in src/auth/login.ts.
The error is "Invalid token" when the JWT expires.
Use the existing retry utility in lib/retry.ts.
```
````

Variable interpolation `${NAME}` and inline commands `$(cmd)` work inside both string types.

### Escape sequences

| Escape | Result |
|--------|--------|
| `\"` | Literal `"` |
| `\$` | Literal `$` (stops interpolation) |
| `\\` | Literal `\` |

---

## Full Agent Call Syntax

```
do <prompt>
  [with <agent> [subagent <name>]]
  [using <model>]
  [in <path>]
  [compact <directive>]
```

All subclauses are optional. Their order is fixed: `with` → `subagent` → `using` → `in` → `compact`.

| Part | Default | Description |
|------|---------|-------------|
| `do <prompt>` | required | Prompt as a string, text block, variable, or `@file.md` |
| `with <agent>` | shebang engine | Agent instance name from the project config |
| `subagent <name>` | default | Sub-agent capability within the agent |
| `using <model>` | agent default | Override the AI model |
| `in <path>` | current dir | Working directory for the agent |
| `compact <directive>` | none | Post-agent context management |

### Examples

```ash
do "Fix the login bug" with opencode
do "Fix this" with opencode using sonnet
do "Fix this" with opencode in "/workspace" compact "truncate 32000"
```

### Loading prompts from files

`@<path>` loads the prompt from a file relative to the script:

```ash
do @skills/refactor.md with opencode
do @skills/review.md with opencode using sonnet
do @'play_step${n}.md' with opencode   # variable in path
do @${DIR}/prompts/task.md with opencode
```

`${VAR}` and `$(cmd)` placeholders inside the file are resolved before sending to the agent.

### Agent fallback priority

1. Task frontmatter / shebang line
2. CLI `--agent` flag
3. Built-in default (`echo`)

---

## Session Toggle

`session begin` / `session end` replaces block syntax for scripts and REPL use:

```ash
session begin
do "Implement the token types" with opencode
do "Implement the value system" with opencode
session end
```

Toggles and blocks mix freely. Nested sessions are an error.

### Per-agent session behavior

| Agent | Flag | Mechanism |
|-------|------|-----------|
| opencode | `--continue` | Resumes the last session |
| claude-code | `--continue` | Resumes the most recent conversation |
| aider | `--restore-chat-history` | Restores chat history |

---

## Retry

### Binary retry (exit code based)

Retries an agent call when it exits with non-zero. The `fail` block lets each retry learn from the previous failure using `${stderr}` and `${stdout}`:

````
try {
  do "Fix all TypeScript errors in src/" with opencode
} fail {
  do ```
Fix the remaining errors:
${stderr}
Files: $(find src -name '*.ts')
  ``` with opencode compact "truncate 32000"
} upto 3
````

### Evaluated retry (three outcomes)

When exit code alone is not enough, add an evaluator that inspects the output:

```
1. Agent runs
2. exit != 0  →  skip evaluator, go to fail { }
3. exit == 0  →  run evaluate with { }
   exit 0  = accept
   exit 1  = partial
   exit 2+ = fail
```

````
try {
  do "Refactor src/auth/" with opencode
} evaluate with {
  do @skills/check-quality.md with opencode
} accept {
  print "refactor accepted"
} partial {
  do "Refine: ${report}" with opencode compact "truncate 16000"
} fail {
  do "Reset and re-review: ${report}" with opencode
} upto 3
````

| Clause | Triggered when | Effect |
|--------|---------------|--------|
| `evaluate with { }` | Agent succeeded (exit 0) | Runs the evaluator. Its exit code routes to accept/partial/fail |
| `accept { }` | Evaluator exit 0 | Exits the retry loop |
| `partial { }` | Evaluator exit 1 | Retries. `${report}` has evaluation findings |
| `fail { }` | Agent exit != 0 or evaluator exit >= 2 | Retries from clean state |
| `${report}` | In partial / fail | Stdout from the evaluator block |

---

## Context Compacting

Coding agents accumulate large contexts. Compact shrinks context between steps.

### Per-agent subclause

```ash
do "Fix bugs in src/" with opencode compact "truncate 32000"
do PROMPT with opencode compact "summarize"
```

### Standalone directive

```ash
compact "truncate 32000"
compact "summarize"
compact "drop node_modules/,dist/"
```

### Global declaration

```ash
#!compact mode=auto window=32000 strategy=truncate
```

| Key | Values | Description |
|-----|--------|-------------|
| `mode` | `auto` (default), `on`, `off` | Whether compact is active |
| `window` | token count | Context window limit |
| `strategy` | `truncate`, `summarize`, `window`, `drop` | How to reduce context |

Per-agent `compact` overrides global settings.

---

## Advanced Control Flow

### `within` — working directory scoping

`within <path> { ... }` sets the working directory for the block:

```ash
within "/project/src" {
  do "fix the login bug"
  do "add tests"
  do "crop image" in "/tmp"   # per-agent override
}
```

### `within` toggle

```ash
within begin "/project/src"
do "fix the login bug"
exec cargo build
within end
```

Each `begin` pushes onto a stack, each `end` pops and restores. Nested toggles are supported.

### Break / Continue

```ash
for FILE in FILES {
  if FILE == "skip_this.ts" { continue }
  do "Review ${FILE}" with opencode
  if $? != 0 { break }
}
```

### Parallel execution

`wait { }` runs statements concurrently and waits for all:

```ash
wait {
  do "Review src/a.ts" with opencode
  do "Review src/b.ts" with opencode
  do "Review src/c.ts" with opencode
}
```

The `&` suffix fires a statement in the background and continues immediately:

```ash
exec npm install &
do "Deploy to staging" with opencode &
print "continues immediately"
```

---

## Advanced Functions

### Scoping

- Variables created inside `{ }` are local — not visible outside.
- Outer variables are readable but not writable inside a function.

### Error handling

Functions compose naturally with `try`:

```ash
fn review_files(PATTERN, MODEL) {
  for FILE in exec find . -name PATTERN {
    try {
      do "Review ${FILE}" with opencode using MODEL
    } fail {
      do "Fix errors: ${stderr}" with opencode compact "truncate 16000"
    } upto 2
  }
}
```

---

## Logging

Configure via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `ASH_LOG` | `warn` | Level: `error`, `warn`, `info`, `debug`, `trace` |
| `ASH_LOG_FILE` | `ash.log` | Log file path (appended on each run) |

```bash
ASH_LOG=debug ash run tasks/
ASH_LOG=info ash tasks/ready/
```

Log format: `{timestamp} [{level}] {module} — {message}`

---

## VSCode Extension

Commands available from the Command Palette and editor context menu for `.ash` files:

| Command | Description |
|---------|-------------|
| `Ash: Run Script` | Runs the active file, shows output in the Ash output channel |
| `Ash: Check Script` | Validates syntax (dry-run) |
| `Ash: Stop Script` | Terminates a running script |

The extension resolves the `ash` binary from PATH or bundled binaries.

---

## Full Example

````ash
#!opencode:1.2.0
#!compact mode=on strategy=truncate window=64000

include "lib/prompts.ash"
include "lib/review.ash"

fn review_and_fix(FILE, MODEL) {
  try {
    do "Review ${FILE} for bugs and performance issues" with opencode using MODEL
  } evaluate with {
    do "Check if review addressed all issues in ${FILE}" with opencode
  } accept {
    print "${FILE} looks good"
  } partial {
    do "Refine: ${report}" with opencode compact "truncate 16000"
  } fail {
    do "Reset and re-review: ${report}" with opencode
  } upto 2
}

for FILE in exec find src -name '*.ts' {
  review_and_fix(FILE, "sonnet")
}

exec npm test
if $? != 0 {
  try {
    do "Fix test failures in $(git diff --name-only -- '*.test.ts')" with opencode
  } fail {
    do ```
Previous fix failed. Errors:
${stderr}
    ``` with opencode
  } upto 3

  exec npm test
  if $? != 0 {
    print "tests still failing, aborting"
    exit 1
  }
}

compact "truncate 16000"
do "Deploy to staging" with opencode
````

---

## References

- [ash.md](ash.md) — core features and quick start
- [architecture.md](architecture.md) — Rust project architecture
- [enhanced-agent-abstraction.md](enhanced-agent-abstraction.md) — agent abstraction design
- [human-in-the-loop.md](human-in-the-loop.md) — human-in-the-loop design proposal
