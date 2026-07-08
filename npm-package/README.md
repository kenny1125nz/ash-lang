# @ash-lang/cli

Ash is a task runner for AI agents — a scripting language that composes AI agents into automated workflows. Drop markdown files in a folder, number them, and run. When you need loops, retries, or conditional logic — add an `.ash` script. Start simple. Grow as needed.

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

## REPL

Run `ash` with no arguments to enter interactive mode:

```bash
$ ash
ash> NAME = "world"
ash> print "hello ${NAME}"
hello world
```

Commands: `.help`, `.clear`, `.vars`, `.exit`. Up/down arrows navigate history. Multi-line blocks (`if`, `for`, `session`) auto-detect continuation.