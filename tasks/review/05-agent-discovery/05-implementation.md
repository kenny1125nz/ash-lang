# Agent Discovery — Implementation

Build the agent discovery and self-configuration system so ash is ready to use right after install.

## Background

Currently, users must manually configure agents in `ash-project.yaml` before ash can do anything useful. Ash ships with default drivers (opencode, claude-code, aider, echo) but has no way to detect which are actually installed or configure itself automatically. This adds friction to the first-run experience.

The goal is to make ash self-configuring: scan the system for installed agents, probe their capabilities, and generate configuration so `ash` works out of the box.

## Intended Solution

Add an `ash discover` subcommand that:

1. Scans the system for known agent binaries (PATH lookup, common install paths)
2. Probes each found agent for capabilities (version, model support, session support — using `--version`, `--help`, or other discovery flags)
3. Generates or updates `ash-project.yaml` with discovered agents and their capabilities
4. Falls back gracefully when no agents are found (suggests install commands)

The discovery engine should be reusable — called by `ash discover` explicitly and also triggered implicitly on first run when no config exists.

### Per-agent integration guidelines

Research subtasks fill in each subsection with concrete discovery details.

#### opencode

- **Binary name**: `opencode`
- **Common install paths**:
  - npm: `npm install -g opencode-ai`
  - Bun: `bun install -g opencode-ai`
  - pnpm/yarn: `pnpm install -g opencode-ai` / `yarn global add opencode-ai`
  - Homebrew: `brew install anomalyco/tap/opencode`
  - Arch: `pacman -S opencode` or `paru -S opencode-bin`
  - curl: `curl -fsSL https://opencode.ai/install | bash`
  - chocolatey/scoop (Windows): `choco install opencode` / `scoop install opencode`
- **Detection**: PATH lookup (`which opencode` or equivalent)
- **Probe flags**:
  - `opencode --version` / `-v` — prints version number (e.g. `v0.1.48`)
  - `opencode --help` / `-h` — prints full CLI usage and subcommands
  - `opencode models` — lists available models in `provider/model` format
  - `opencode session list` — lists existing sessions (verifies session support)
  - `opencode agent list` — lists configured agents
- **Capabilities to report**:
  - **Model support**: Uses `provider/model` format (e.g. `anthropic/claude-sonnet-4-20250514`). 75+ providers via Models.dev. Configurable per-run with `--model` / `-m`.
  - **Session support**: Full session management (`--continue`, `--session <id>`, `--fork`). `opencode session list` / `delete`. Sessions persisted locally.
  - **Multi-session**: Can run multiple agents in parallel on the same project.
  - **Non-interactive mode**: `opencode run` accepts prompts via CLI for scripting.
  - **Interfaces**: TUI (default), CLI (`run`), HTTP API (`serve`), web UI (`web`), ACP (`acp`).
  - **No atomic retry subcommand**: Model, agent, and session controlled via flags on `run`.
- **Install suggestion text**: `npm install -g opencode-ai`

#### claude-code

- **Binary name**: `claude`
- **Common install paths**:
  - Native: `curl -fsSL https://claude.ai/install.sh | bash` (macOS, Linux, WSL)
  - npm: `npm install -g @anthropic-ai/claude-code`
  - Homebrew: `brew install --cask claude-code` (stable) or `brew install --cask claude-code@latest` (latest)
  - WinGet: `winget install Anthropic.ClaudeCode`
  - apt: `sudo apt install claude-code` (Debian/Ubuntu)
  - dnf: `sudo dnf install claude-code` (Fedora/RHEL)
  - apk: `apk add claude-code` (Alpine)
- **Detection**: PATH lookup (`which claude` or equivalent). Native install places binary at `~/.local/bin/claude`.
- **Probe flags**:
  - `claude --version` — prints version number
  - `claude --help` — prints CLI usage (does not list every flag, per docs)
  - `claude doctor` — diagnostic check of installation and configuration
- **Capabilities to report**:
  - **Model support**: Multiple models via aliases (`sonnet`, `opus`, `haiku`, `fable`, `best`, `default`) and full model names (e.g. `claude-opus-4-8`). Configurable per-run with `--model` / `-m`. Supports effort levels (`low`, `medium`, `high`, `xhigh`, `max`). Third-party providers: Bedrock, Vertex AI, Foundry.
  - **Session support**: Full session management (`--continue` / `-c`, `--resume <name>`, `--fork-session`). `/resume` opens interactive session picker. `/rename`, `/branch`, `/clear`, `/compact`. Sessions persisted as JSONL at `~/.claude/projects/`. `/export` for transcript export.
  - **Multi-session**: Background agents (`--bg` / `--background`), agent view (`claude agents`), git worktrees for isolated parallel sessions.
  - **Non-interactive mode**: `claude -p "query"` for headless/print mode. `--output-format json` / `stream-json` for structured output. Pipe support: `cat file | claude -p "query"`.
  - **Sub-agents**: Full subagent support via `Task` tool, custom subagents with `--agent` flag, dynamic workflows for orchestration at scale.
  - **MCP support**: Full Model Context Protocol integration for external tool connections.
  - **Extensibility**: Hooks, skills, plugins, CLAUDE.md project instructions.
  - **Interfaces**: TUI (default), VS Code extension, JetBrains plugin, Desktop app, Web (claude.ai/code).
  - **No atomic retry subcommand**: Model selection via `--model` flag or `/model` slash command during a session.
- **Install suggestion text**: `npm install -g @anthropic-ai/claude-code`

#### aider

- **Binary name**: `aider`
- **Common install paths**:
  - aider-install (recommended): `pip install aider-install && aider-install` (creates isolated venv)
  - pip: `pip install aider-chat`
  - pipx: `pipx install aider-chat`
  - uv: `uv tool install --force --python python3.12 --with pip aider-chat@latest`
  - Shell one-liner: `curl -LsSf https://aider.chat/install.sh | sh`
  - Docker: `docker pull paulgauthier/aider`
- **Detection**: PATH lookup (`which aider` or equivalent). Fallback: `python -m aider`. Binary may not be on PATH in some environments (Windows, restrictive permissions).
- **Probe flags**:
  - `aider --version` — prints version number (e.g. `0.62.0`)
  - `aider --help` — prints full CLI usage with all options, model settings, and environment variables
  - `aider --list-models MODEL` / `--models` — lists known available models matching a partial name
  - `aider --just-check-update` — check for updates and return status in exit code
- **Capabilities to report**:
  - **Model support**: 17+ providers (OpenAI, Anthropic, Gemini, DeepSeek, Ollama, OpenRouter, Groq, xAI, Cohere, Azure, Vertex AI, Bedrock, LM Studio, and more). `--model` to select per-run. `/model` and `/models` in-chat commands. Model aliases via `--alias`. Architect/editor mode with dual-model setup (`--architect`, `--editor-model`). Weak model for commits and summarization (`--weak-model`). Reasoning effort (`--reasoning-effort`) and thinking tokens (`--thinking-tokens`).
  - **Chat history (session-like)**: Chat history persisted to `.aider.chat.history.md` in the git root. `--restore-chat-history` restores previous messages. `--input-history-file` for command-line input history. `--llm-history-file` for full LLM conversation log. In-chat: `/clear` to clear chat, `/reset` to drop files and clear, `/undo` to undo last commit, `/save`/`/load` to save/restore chat context. No named sessions — history is per-project via git root files.
  - **Interactive chat modes**: Four modes — `code` (edit files, default), `architect` (two-model editing via `--architect`), `ask` (questions only, no edits), `context` (view code context). Switchable via `/chat-mode` or `--edit-format`.
  - **Non-interactive mode**: `aider --message "..."` / `-m` for single-shot execution. `--message-file` / `-f` for file-based prompts. `--yes-always` to skip confirmations. `--exit` for debug startup. `--dry-run` to preview without modifying files. Pipe-friendly and shell-scriptable.
  - **Git integration**: Tightly integrated (enabled by default). Auto-commits with descriptive messages. `/undo` to undo commits, `/commit` to commit dirty changes, `/git` for raw git commands, `/diff` to show changes. `--no-git`, `--no-auto-commits`, `--no-dirty-commits` to disable.
  - **Lint/test integration**: `--lint-cmd` for per-language lint commands, `--auto-lint` (default on). `--test-cmd` for test runner, `--auto-test` for automatic testing on changes.
  - **Interfaces**: Terminal TUI (default), browser mode (`--gui` / `--browser`), VS Code extension (via `/web`), IDE watch-files mode (`--watch-files`). Python scripting API via `aider.coders.Coder` (unofficial, may change).
  - **Other**: Voice input (`/voice`, `--voice-format`, `--voice-language`). Web scraping (`/web`). Clipboard paste (`/paste`). Custom editor (`/editor`, `--editor`). VI keybindings (`--vim`). Shell command execution (`/run`, `/test`).
- **Install suggestion text**: `pip install aider-chat` or `curl -LsSf https://aider.chat/install.sh | sh`

#### echo

Echo is a built-in agent that ships with ash. No discovery needed — it is always available after install with no additional setup.

- **Binary name**: N/A (built-in to ash, no external binary)
- **Detection**: Always present. Echo is registered as a default driver in `register_defaults()`.
- **Probe flags**: None (no external process to probe)
- **Capabilities to report**:
  - **Model support**: None. Echo does not call any AI model — it simply echoes prompts back.
  - **Session support**: None.
  - **Purpose**: Test/noop agent for validating ash’s agent orchestration without requiring an external AI tool. Useful for debugging pipeline structure and verifying prompt flow.
  - **Non-interactive mode**: Always operates in passthrough/echo mode.
- **Install suggestion text**: N/A (built-in)

## Acceptance Criteria

1. **`ash discover` prints a list of found agents**
   - Agent name, binary path, version, detected capabilities
   - Agents not found are listed with install suggestions

2. **`ash discover --write` generates `ash-project.yaml`**
   - Outputs a valid config file with discovered agents and their capabilities
   - Does not overwrite existing config without `--force`

3. **First-run auto-discovery**
   - If `ash` is invoked (script or tree mode) and no `ash-project.yaml` exists, run a lightweight discovery and use found agents
   - Print a one-line summary: "Found: opencode, aider. Run `ash discover` for details."

4. **Graceful when nothing is found**
   - If no agents are installed, print: "No agents found. Install one: npm i -g @anthropic-ai/claude-code" (or equivalent)
   - `ash discover` exits 0 (informational, not an error)

5. **Capability probing is non-destructive**
   - Uses `--version`, `--help`, `--dry-run` or equivalent flags only
   - Never actually runs an agent task during discovery

## Implementation Hints

### Relevant project context

The project is a single Rust crate at `ash/`. Discovery should be independent of the ash language layer — it only touches the engine and CLI entry.

**Engine layer** — what agent configuration looks like:
- `engine/config.rs` — `AgentConfig { name, agent_type, driver, cmd, args, ... }` struct (currently defined but not populated from YAML at runtime)
- `engine/mod.rs:34-54` — `from_config(cfg: &AgentConfig) -> Arc<dyn Adapter>` factory that builds adapters from config
- `engine/mod.rs:57-78` — `register_defaults()` registers echo, opencode, claude-code, aider with hardcoded settings
- `engine/driver.rs` — each `LocalCliDriver` knows its binary name via `name()`

**CLI entry** — how ash starts up:
- `main.rs:118-152` — `validate_agents()` hand-parses a minimal `ash-project.yaml` looking for agent names
- `main.rs:175-247` — `run()` is the main CLI entry point, currently handles `--check` / `-c` / `--dry-run` and a positional file argument

**What to reuse**:
- Each driver's `name()` method already knows the binary name — use this as the discovery key
- `from_config()` is ready to accept dynamically-built `AgentConfig` structs
- The hand-rolled YAML parser pattern in `validate_agents()` can be extended or replaced for config generation
