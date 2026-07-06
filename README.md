# Ash

A CLI agent orchestration tool and scripting language for composing multi-agent workflows.

Ash lets you write scripts that coordinate AI agents — send prompts, chain their outputs, run them in parallel, and handle errors — all from a single `.ash` file.

## Quick Start

### Install

```bash
cargo install ash
```

Or download a prebuilt binary from [GitHub Releases](https://github.com/kenny1125nz/ash-lang/releases).

### Your first script

```bash
#!/usr/bin/env ash
#! opencode:deepseek-v3

"Write a hello world program in Rust"
```

```bash
ash run hello.ash
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

## Usage

```
ash [options] [script.ash]
ash [options] [directory/]

Options:
  --check, -c            Validate script without executing
  --dry-run              Walk the directory tree without executing
  --agent <name[:model]> Set default agent and optional model
  --continue-on-error, -k  Continue past failing tasks
```

## Directory-Based Orchestration

Organize tasks as a directory tree. Ash walks the tree in order and sends each file to the configured agent:

```
tasks/
├── 1-setup/
│   ├── 01-init.md
│   └── 02-deps.md
└── 2-implement/
    ├── 01-core.md
    └── 02-tests.md
```

```bash
ash tasks/
```

## Building from Source

```bash
git clone https://github.com/kenny1125nz/ash-lang.git
cd ash
cargo build --release
```

Requirements: Rust 1.70+

## License

MIT
