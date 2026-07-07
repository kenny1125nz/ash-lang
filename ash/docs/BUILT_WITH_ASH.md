# Built with Ash

> Ash runs a folder of markdown files through AI agents — in numbered order. When you need loops, variables, or complex logic, `.ash` scripts give you the full pipeline.

---

## What Ash Is

Ash is a **task runner for AI agents**. Drop markdown files in a folder, number them, and run `ash tasks/`. Each file is one task sent to an AI agent. Frontmatter sets the agent and model. Output flows between steps.

That's the core. No scripting required.

As your tasks grow more complex, ash grows with you — `.ash` files can live in the same directory tree, and standalone `.ash` scripts give you a compact language for loops, retries, and conditional pipelines.

---

## Directory Mode — the 80% case

```
tasks/
├── 01-research.md
├── 02-implement.md
├── 03-review.ash
└── 04-deploy.md
```

```bash
$ ash tasks/
[1/4] tasks/01-research.md  [ok]
[2/4] tasks/02-implement.md [ok]
[3/4] tasks/03-review.ash   [ok]
[4/4] tasks/04-deploy.md    [ok]
4 tasks, 4 passed
```

### Markdown tasks

The simplest unit of work. The body is the prompt. Frontmatter sets the
agent and model:

```markdown
---
agent: opencode
model: sonnet
---

# Research the login system

Trace all files related to the login flow. For each file document its
responsibilities, inputs, and test coverage.
```

State passes naturally: `${stdout}` from task 1 is available in task 2.
`$?` holds the exit code of the previous task.

By default a failed task stops the run. Set `on_fail: continue` in
frontmatter to keep going — useful for non-critical steps.

### Ash tasks in the tree

When a single prompt isn't enough for one step, drop an `.ash` file into
the tree instead of a `.md` file. It runs as a mini-script with full
access to the language:

```ash
#!opencode:1.0

RESULT = $(npm test 2>&1)
if $? != 0 {
  do "The tests failed. Fix the errors below and recompile.

Test output:
${RESULT}" with opencode
}
```

`.ash` files use a shebang line (`#!opencode:1.0`) to declare the
default agent. They can define variables, call shell commands, branch
on results, and invoke agents — all within one step of the directory
pipeline. The output still flows into the next task as `${stdout}`.

Markdown and ash files sort together by numeric prefix. You choose which
format fits each step.

### Agent discovery

```bash
ash discover            # list installed agents (parallel probe)
ash discover --write    # generate ash.yml
```

Nested folders create hierarchy. Prefixes enforce order. Duplicates are
caught before anything runs. It's a build system for AI tasks.

---

## Standalone Ash Scripts — when the whole pipeline needs logic

Sometimes the workflow itself needs programming — not just one step,
but the entire orchestration: iterate over every changed file, review
each one, retry on failure, run tests, branch the deployment.

For this you write a standalone `.ash` file:

```ash
fn review(FILE) {
  try {
    do "Review ${FILE} for bugs" with opencode
  } fail {
    do "Fix the remaining issues: ${stderr}" with opencode
  } upto 2
}

FILES = $(find src -name '*.ts')
for FILE in FILES {
  review(FILE)
}

exec npm test
if $? == 0 {
  print "all good"
} else {
  print "tests failed after retries"
  exit 1
}
```

```
$ ash review.ash
all good
```

### Core language features

The language is small — designed to be readable by someone who doesn't
write code, and predictable enough that an LLM can generate it reliably.

| Feature | Syntax | What it does |
|---------|--------|--------------|
| Agent call | `do "prompt" with opencode` | Send a prompt to an AI agent |
| Variables | `NAME = "value"` | Store and reference data |
| Strings | `"hello ${NAME}"`, `$(cmd)` | Interpolation and command substitution |
| Shell commands | `exec cmd` | Run a command, capture output |
| Conditionals | `if ... else if ... else` | Branch on exit codes and expressions |
| Loops | `for X in LIST`, `while COND` | Iterate or loop with a condition |
| Functions | `fn name(params) { ... }` | Reusable blocks with parameters |
| Retry | `try { } fail { } upto N` | Retry a failed agent call with learning context |
| Session | `session { ... }` | Group calls in a shared agent context |
| Parallel | `wait { ... }` | Run multiple agent tasks concurrently |
| Includes | `include "file.ash"` | Load another script |

See [ash.md](ash.md) for the full language reference.

---

## Progressive Complexity

You never face more complexity than you need. Start at level 1, add one
concept at a time:

| Level | What you use | What you get |
|-------|-------------|--------------|
| 1 | Folder of `.md` files | Sequential AI task execution |
| 2 | YAML frontmatter | Per-task agent, model, `on_fail` |
| 3 | `.ash` files in the tree | Logic within a single step |
| 4 | `${stdout}`, `$?` | State passing between tasks |
| 5 | Standalone `.ash` scripts | Full orchestration: loops, retry, parallelism |

---

## Why This Model

AI agents are good at handling ambiguity within a step — understanding
intent, making judgment calls, handling variation. They're bad at
sequencing, branching, retrying, and deciding *what to do next*.

Ash gives you the best of both: **deterministic control flow** where it
matters (what runs, in what order, under what conditions, with what
guardrails) and **AI autonomy** where it shines (the content of each step).

The script runs the same way every time. The agent handles ambiguity
within each step. You decide the skeleton. The AI fills in the flesh.

This is the same model as human delegation. A manager's process is rigid
— assign, review, accept, escalate. The worker's output varies. The
value is in encoding the process, not guaranteeing the outcome.

---

## Agent-Agnostic

Ash doesn't ship with AI models. It doesn't call APIs directly. It
invokes whatever agent CLI tools are on your system — opencode, Claude
Code, Aider, or a custom tool defined in `ash-project.yaml`:

```yaml
agents:
  opencode:
    type: local-cli
    cmd: opencode
    args: ["run"]
    model_flag: "--model"
    session_flag: "--continue"

  custom-tool:
    type: local-cli
    cmd: my-agent
    message_flag: "--prompt"
```

Swap providers without touching your workflows. The config decides
which binaries to invoke; the tasks and scripts describe what to do.

---

## Why Not a Shell Script?

Shell scripts are deterministic. AI agents are autonomous. Neither
alone handles the multi-step intelligent workflow pattern:

- Shell scripts are rigid — every step must be explicitly coded. They
  can't handle ambiguity, judgment, or creative variation.
- AI chat threads are opaque — no structured retry, no evaluation gates,
  no guaranteed sequencing, no version control.
- Ash fuses them: deterministic control where it's optimal, AI autonomy
  where it shines.

---

## Why Not a Visual Tool?

Visual workflow tools (n8n, Zapier) give you drag-and-drop, but their
internal representation is unreadable JSON. Lose the tool, lose the
workflow.

Ash workflows are plain markdown and text. They're version-controllable,
diffable, searchable, reviewable in any editor. An LLM can generate
them. A CI system can run them. They survive any specific tool.

---

## Getting Started

```bash
# Create a task folder
mkdir my-tasks

cat > my-tasks/01-hello.md << 'EOF'
---
agent: opencode
---

# Hello Task

Print "Hello from ash" and explain how the markdown task system works.
EOF

# Discover installed agents
ash discover

# Run the task
ash my-tasks/
```

```
[1/1] my-tasks/01-hello.md  [ok]
1 tasks, 1 passed
```

When you need more — loops, retries, parallelism — graduate to [`.ash` scripts](ash.md).

Ash is **open source**, **language-agnostic**, and **agent-agnostic**. Whatever agents
you run, ash can orchestrate them.
