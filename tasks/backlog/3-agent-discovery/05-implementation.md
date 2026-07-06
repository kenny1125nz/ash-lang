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

*TODO*

#### claude-code

*TODO*

#### aider

*TODO*

#### echo

*TODO*

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
