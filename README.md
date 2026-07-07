# Ash

**Deterministic agent orchestration — structured task files, scriptable when you need more.**

Point Ash at a directory — it walks the tree in sorted order and sends
every `.md` file to your configured AI agent. One task per file.
Deterministic, repeatable, no scripting required.

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

## Quick Start

### Install

```bash
npm install -g @ash-lang/cli
```

Or download a prebuilt binary from [GitHub Releases](https://github.com/kenny1125nz/ash-lang/releases).

### Configure your agent

Create `ash.yaml` in your project root:

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
├── ash.yaml
└── tasks/
    ├── 1-init/
    │   └── 01-setup.md
    └── 2-feature/
        └── 01-add-login.md
```

```bash
ash my-project/tasks/
```

Ash prints each task and its result as the agent completes it. Tasks
that return a non-zero exit code are marked as failures.

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

Each `.md` file is a standalone prompt sent to the agent. Optional
YAML frontmatter sets per-task config (agent, model, etc.). The
filename sets the order — Ash sorts alphanumerically. Subdirectories
group related tasks.

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

## Scripting (Advanced)

When you need more than one-shot prompts — chaining, conditionals,
parallelism — write an `.ash` script:

```ash
#!opencode
do "Write a hello world program in Rust"
print stdout
```

```bash
ash hello.ash
```

### REPL

```bash
ash
```

## Supported Agents

| Agent | Description |
|-------|-------------|
| `echo` | Built-in passthrough for testing |
| `opencode` | OpenCode CLI agent |
| `claude-code` | Anthropic Claude Code |
| `aider` | Aider AI pair programming |

## Building from Source

```bash
git clone https://github.com/kenny1125nz/ash-lang.git
cd ash
cargo build --release
```

Requirements: Rust 1.70+

## License

AGPLv3
