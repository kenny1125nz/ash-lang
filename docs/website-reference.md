# Ash — Feature Reference

Complete feature reference for Ash agent shell scripts.

## What is Ash?

Ash is a task runner for AI agents — deterministic, repeatable, no scripting required. Point Ash at a directory of markdown files and it walks the tree in sorted order, sending each file to your configured AI agent. One task per file.

```
tasks/
├── 1-plan/
│   ├── 01-requirements.md
│   └── 02-architecture.md
├── 2-implement/
│   ├── 01-auth.md
│   ├── 02-api.md
│   └── 03-tests.md
└── 3-review/
    └── 01-code-review.md

→ ash tasks/
```

When you need more than one-shot prompts — chaining, conditionals, parallelism — write an `.ash` script. Start simple. Grow as needed.

## Installation

### npm

```bash
npm install -g @ash-lang/cli
```

On `npm install`, the correct platform binary is downloaded from GitHub Releases automatically.

### Prebuilt binaries

Download from [GitHub Releases](https://github.com/kenny1125nz/ash-lang/releases).

Place the binary alongside `ash.js` in the npm package directory if the automatic download fails.

### Requirements

- Rust 1.70+ (for building from source)
- Node.js (for npm install)

## Quick Start

### Configure your agent

Create `ash.yml` in your project root:

```yaml
default_agent: opencode
```

Or set it per-run:

```bash
ash --agent opencode tasks/
```

### Run your first project

```
my-project/
├── ash.yml
└── tasks/
    ├── 1-init/
    │   └── 01-setup.md
    └── 2-feature/
        └── 01-add-login.md
```

```bash
ash my-project/tasks/
```

Ash prints each task and its result as the agent completes it. Tasks that return a non-zero exit code are marked as failures.

### Skip failures, keep going

```bash
ash --continue-on-error tasks/
```

### Validate without running

```bash
ash --check tasks/
```

### See what would run

```bash
ash --dry-run tasks/
```

## Writing Tasks

Each `.md` file is a standalone prompt sent to the agent. Optional YAML frontmatter sets per-task config (agent, model, etc.). The filename sets the order — Ash sorts alphanumerically. Subdirectories group related tasks.

**tasks/1-plan/01-requirements.md:**

```markdown
---
agent: opencode
---

Write a requirements document for a login system that supports:

- Email/password authentication
- OAuth with Google and GitHub
- Session management with JWT
- Rate limiting on failed attempts
```

**tasks/2-implement/01-auth.md:**

```markdown
---
agent: claude-code
---

Implement the login API endpoint. Cover:

- POST /auth/login — validates email/password, returns JWT
- POST /auth/register — creates user, sends verification email
- POST /auth/refresh — refreshes expired tokens

Use the requirements from tasks/1-plan/01-requirements.md.
```

### Frontmatter reference

| Key | Values | Default | Description |
|-----|--------|---------|-------------|
| `agent` | agent name | — | Override agent for this task |
| `model` | model name | — | Override model for this task |
| `on_fail` | `stop`, `continue` | `stop` | Behavior when the task fails |
| `compact` | directive | — | Context window strategy for this task |

## Scripting with .ash Files

When you need more than one-shot prompts — chaining, conditionals, parallelism — write an `.ash` script:

```ash
#!opencode
do "Write a hello world program in Rust"
print stdout
```

```bash
ash hello.ash
```

### Agent shebang

Declare the agent with a shebang:

```ash
#!opencode:1.0

do "Review src/" with opencode
```

### Language overview

```ash
do "Review src/" with opencode      # call an agent

fn rollback(FILE) {                  # functions
  exec git restore "${FILE}"
  do "Summarize what has been done"
}

for FILE in FILES {                  # loops, conditionals, retry
  try {
    do "Fix bugs in ${FILE}"
  } fail {
    print "failed on ${FILE}"
  } upto 3
}
```

## Directory Mode

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

Markdown files can set per-task settings with YAML frontmatter:

```markdown
---
agent: opencode
model: sonnet
on_fail: continue
---

# Task Title

The prompt content for the agent goes here...
```

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

## Control Flow

All bodies use `{ }`:

### If / else

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

### For loops

```ash
for FILE in FILES {
  exec eslint FILE
}
```

### While loops

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
} fail {
  print "unexpected error"
} upto 5
```

| Clause | Runs when |
|--------|-----------|
| `accept` | Evaluator returns truthy / `$?` == 0 |
| `partial` | Evaluator returns falsy / `$?` == 1 |
| `fail` | Body errors or evaluator `$?` >= 2 |
| `upto N` | Maximum retry count |

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

### Scoping

Variables created inside `{ }` are local — not visible outside. Outer variables are readable but not writable inside a function.

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

## Background and Parallel Execution

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

## File-based Prompts (`@file`)

Load a prompt from a file with variable interpolation:

```ash
do @"path/to/prompt.md"
```

The file is read, `${VAR}` placeholders are resolved, and the result is sent to the agent.

`@<path>` loads the prompt from a file relative to the script:

```ash
do @skills/refactor.md with opencode
do @skills/review.md with opencode using sonnet
do @'play_step${n}.md' with opencode
do @${DIR}/prompts/task.md with opencode
```

## REPL

Run `ash` with no arguments to enter interactive mode:

```bash
$ ash
ash> NAME = "world"
ash> print "hello ${NAME}"
hello world
```

Commands: `.help`, `.clear`, `.vars`, `.exit`. Up/down arrows navigate history. Multi-line blocks (`if`, `for`, `session`) auto-detect continuation.

## Supported Agents

| Agent | Description |
|-------|-------------|
| `echo` | Built-in passthrough for testing |
| `opencode` | OpenCode CLI agent |
| `claude-code` | Anthropic Claude Code |
| `aider` | Aider AI pair programming |

### Auto-discovery

Agents are auto-discovered on your PATH. Run to refresh:

```bash
ash discover
```

### Custom agents

Add custom CLI-based agents in `ash.yml`:

```yaml
agents:
  my-tool:
    type: local-cli
    cmd: my-tool
    message_flag: "--prompt"
    yes_flag: "--yes"
```

## Building from Source

```bash
git clone https://github.com/kenny1125nz/ash-lang.git
cd ash
cargo build --release
```

Requirements: Rust 1.70+

## License

AGPLv3

## VS Code Extension

Syntax highlighting, check, and run commands for Ash (`.ash`) agent shell scripts.

The extension provides language tooling only — the Ash runtime is a separate CLI. Install it via npm or download from GitHub Releases:

```sh
npm i -g @ash-lang/cli
```

### Features

- Syntax highlighting for `.ash` files — variables, strings, control flow, agent calls
- Run script — executes the current `.ash` file with the ash runtime
- Check script — validates syntax without executing

### Commands

| Command | Title | Description |
|---------|-------|-------------|
| `ash.runScript` | Ash: Run Script | Run the active `.ash` file |
| `ash.checkScript` | Ash: Check Script | Validate syntax of the active `.ash` file |
| `ash.stopScript` | Ash: Stop Script | Stop a running script |

Run from the Command Palette (`Ctrl+Shift+P`) or right-click an `.ash` file in the editor.