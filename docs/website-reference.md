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

### Default agent

Set the default agent for all subsequent `do` calls with `use`:

```ash
use opencode
do "Review src/"                # uses opencode

use claude-code
do "Refactor the implementation" # uses claude-code
```

The agent can still be overridden per-call with `do "..." with <agent>`.

### Language overview

```ash
use opencode                             # set default agent

do "Review src/"                         # call an agent

fn rollback(FILE) {                      # functions
  exec git restore "${FILE}"
  do "Summarize what has been done"
}

for FILE in FILES {                      # loops, conditionals, retry
  try {
    do "Fix bugs in ${FILE}"
  } fail {
    print "failed on ${FILE}"
  } upto 3
}
```

## Directory Mode

Pass a directory to walk its task tree in numeric-prefix order:

```bash
ash ./tasks
ash ./tasks --dry-run
ash ./tasks --continue-on-error
```

### Task tree layout

```
tasks/
├── 01-intro.md
├── 02-setup/
│   ├── 01-db.md
│   ├── 02-config.ash
│   └── 03-seed-data.md
├── 03-build.ash
└── 04-review.md
```

Files get a numeric prefix (`01-`, `02-`, etc.). Subdirectories group related tasks — the walker recurses into them in order.

Execution order above: `01-intro.md` → `02-setup/01-db.md` → `02-setup/02-config.ash` → `02-setup/03-seed-data.md` → `03-build.ash` → `04-review.md`

### File types

| Type | Extension | How it's handled |
|------|-----------|------------------|
| Markdown | `.md` | Content is sent as a prompt to the configured agent. `${VAR}` interpolation is resolved from the evaluator scope. |
| Ash script | `.ash` | Parsed and executed as an ash script. Has full access to variables, functions, control flow — including `do` statements. |

### Numeric prefix

Files and directories must start with a numeric prefix (`01-`, `02-step-`, etc.) to be included. Files without a prefix are silently skipped.

Duplicate prefixes at the same level (e.g., `01-foo.md` and `01-bar.md` in the same directory) are reported as an error. Each prefix must be unique within its directory to maintain a deterministic ordering contract.

### Frontmatter (`.md` files)

Markdown tasks can set per-task configuration with YAML frontmatter:

```markdown
---
agent: claude-code
model: sonnet
compact: truncate 32000
on_fail: continue
---

# Task Title

The prompt content for the agent goes here...
```

| Key | Values | Default | Description |
|-----|--------|---------|-------------|
| `agent` | agent name | — | Override agent for this task |
| `model` | model name | — | Override model for this task |
| `compact` | directive | — | Context window strategy for this task |
| `on_fail` | `stop`, `continue` | `stop` | Behavior when the task fails |

### Shebang (`.ash` files)

Ash scripts set their agent via shebang, same as standalone scripts:

```ash
#!opencode:1.0:sonnet

do "Fix the migration script"
if $? != 0 {
  do "Rollback changes" with rollback-agent
}
```

The shebang's engine and model become the defaults for `do` statements inside the script. Individual `do` calls can override with `with`/`using` clauses.

### CLI flags

| Flag | Description |
|------|-------------|
| `--dry-run` | Print the task list without executing |
| `--continue-on-error` / `-k` | Keep running after a task fails |
| `--check` / `-c` | Validate syntax without executing |
| `--agent <name>:<model>` | Default agent and model for all tasks |

### Skip behavior

The following files and directories are silently skipped during the walk:

- Hidden files and directories (starting with `.`)
- Files without a numeric prefix
- Non-task file extensions (not `.md` or `.ash`)
- Empty markdown files (no content after frontmatter)
- Empty ash scripts (no statements)

### Directory orchestration inside scripts

The tree walker can also be invoked from within an ash script using `do @"path/"`:

```ash
do @"tasks/" with opencode
```

When the `@` path points to a directory, the same tree walker runs — walking the directory, discovering tasks, and executing them in order. See [File-based Prompts](file-prompts.md).

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
| `$_attempt` | `evaluate` body | Current attempt number, 1-indexed |
| `$_max_attempts` | `evaluate` body | Total allowed attempts |
| `$_feedback` | `evaluate` body | Findings from the previous evaluation iteration |
| `$_evaluator_output` | `evaluate` evaluator | Full stdout from the evaluator |
| `$score` | `evaluate` post-loop | The final score (accepted or last attempted) |
| `$accepted` | `evaluate` post-loop | Whether the threshold was met |

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

### Evaluate blocks

`evaluate` is a top-level statement that runs a body, evaluates the result using an external evaluator (agent, function, or command), and retries until a numeric score threshold is met or attempts are exhausted.

```ash
evaluate {
  do "Write a blog post about Rust"
} by @"reviewer.md" with opencode
   accept by 85
   upto 5
```

The `by` clause selects the evaluator — one of three forms:

| Evaluator | Syntax | Description |
|-----------|--------|-------------|
| Agent | `@"prompt.md" [with <agent>] [using <model>]` | An agent reviews the output and produces a score |
| Function | `fn_name(args...)` | A user-defined function returns the score |
| Command | `exec "<command>"` | A shell command outputs the score |

The agent evaluator receives language-injected scoring instructions so prompts don't need to specify the output format. The agent must output:

```
SCORE: <0-100 integer>
FINDINGS:
<actionable improvement feedback>
```

#### Per-iteration variables

Within each iteration's body, these variables are automatically set:

| Variable | Type | Description |
|----------|------|-------------|
| `$_attempt` | Int | Current attempt number, 1-indexed |
| `$_max_attempts` | Int | Total allowed attempts |
| `$_feedback` | String | Findings from the previous iteration (empty on attempt 1) |

#### Post-loop variables

After the evaluate block completes, these variables are set in the parent scope:

| Variable | Type | On acceptance | On exhaustion |
|----------|------|---------------|---------------|
| `$score` | Int | The accepted score (>= threshold) | The last attempted score |
| `$accepted` | Bool | `true` | `false` |
| `$_evaluator_output` | String | Full evaluator stdout from the accepting run | Full evaluator stdout from the last run |

#### Outcomes

- **Acceptance** — score >= threshold on any attempt. The loop terminates immediately. Side effects from the accepting iteration are preserved.
- **Exhaustion** — all attempts complete without reaching the threshold. The last iteration's side effects are preserved for inspection.
- **Error** — body crash, evaluator failure, or score extraction failure propagates immediately. No silent retry.

#### Example: branching on acceptance

```ash
evaluate {
  do "Write documentation"
} by score_fn()
   accept by 80
   upto 5

if $accepted {
  print "Approved with score $score"
  exec deploy docs/
} else {
  print "Rejected (score $score), check output"
  exit 1
}
```

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

### Directory orchestration

If the `@` path points to a **directory** rather than a file, the task tree walker runs instead — the same mechanism used by `ash tasks/`:

```ash
do @"tasks/" with opencode
```

This walks the directory, discovers numbered `.md` and `.ash` files, and executes them in sorted order using the specified agent. Each `.md` file becomes a standalone task sent to the agent. Each `.ash` file is executed as a script with access to the same evaluator scope.

This lets scripts recursively compose task directories:

```ash
do @"review/" with opencode
do @"fix/" with opencode using sonnet
if $? == 0 {
  do @"deploy/" with opencode
}
```

Individual task files can override the agent and model via YAML frontmatter, just like in directory mode.

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