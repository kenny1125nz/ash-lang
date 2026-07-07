# Ash Agent Shell — VS Code Extension

Syntax highlighting, check, and run commands for Ash (`.ash`) agent shell scripts.

This extension provides language tooling only — the Ash runtime is a separate CLI. Install it via npm or download from [GitHub Releases](https://github.com/kenny1125nz/ash-lang/releases/latest):

```sh
npm i -g @ash-lang/cli
```

## Getting Started

Ash needs an AI agent to execute `do` calls. Agents are auto-discovered on your PATH — install one and ash finds it automatically. If you add a new agent after ash was installed, run:

```sh
ash discover
```

**Built-in support:** opencode, claude-code, codex, gemini-cli, kimi. Any CLI-based agent can be configured manually via `ash.yml`:

```yaml
agents:
  my-tool:
    type: local-cli
    cmd: my-tool
    message_flag: "--prompt"
    yes_flag: "--yes"
```

**Supported agents:** opencode, claude-code, aider, codex, gemini-cli, kimi (echo is the built-in default).

Use `--agent` to specify the agent:

```sh
ash --agent opencode path/to/script.ash
```

For `.ash` scripts, declare the agent with a shebang:

```ash
#!opencode:1.0

do "Review src/" with opencode
```

For `.md` tasks, optionally set the agent in YAML frontmatter:

```markdown
---
agent: opencode
---

# Task Title

The prompt content goes here...
```

## Features

- **Syntax highlighting** for `.ash` files — variables, strings, control flow, agent calls
- **Run script** — executes the current `.ash` file with the ash runtime
- **Check script** — validates syntax without executing

## Commands

| Command | Title | Description |
|---------|-------|-------------|
| `ash.runScript` | Ash: Run Script | Run the active `.ash` file |
| `ash.checkScript` | Ash: Check Script | Validate syntax of the active `.ash` file |
| `ash.stopScript` | Ash: Stop Script | Stop a running script |

Run from the Command Palette (`Ctrl+Shift+P`) or right-click an `.ash` file in the editor.
