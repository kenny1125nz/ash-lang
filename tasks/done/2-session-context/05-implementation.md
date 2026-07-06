# Session/Context — Implementation

Build the `session { }` language feature and wire it into the agent engine.

## Background

Currently, every `do` statement in ash spawns an independent, one-shot agent job. For multi-step workflows, each step starts from a cold context — the agent must re-explore the codebase on every call, wasting tokens. Some agents support continuing from a prior session (e.g., opencode's `--continue` flag), but ash has no way to control this. In fact, `OpenCodeDriver` currently **always** passes `--continue`, which is wrong — it means every opencode call shares one persistent session, causing issues with unrelated tasks and concurrent requests.

Additionally, compact (context window management) is tied to sessions: for opencode, compacting outside of a session has no value. Right now compact operates independently with no awareness of session state.

## Intended Solution

### Language syntax

`session { }` wraps multiple `do` calls in a session block:

```ash
session {
  do "implement token types" with opencode
  do "implement the value system" with opencode
  do "refactor the AST" with opencode using sonnet
}
```

All `do` calls within the block share the same session. Outside the block, `do` calls run one-shot with no session. This is consistent with ash's existing `{}` block constructs (`if`, `for`, `while`, `try`, `wait`).

### Per-agent integration guidelines

#### opencode

Supports sessions natively via CLI flags on `opencode run`:
- `--continue` / `-c` — continue the most recent session in the working directory
- `--session` / `-s <id>` — continue a specific session by ID
- `--fork` — fork the session when continuing (create a branch, keep original)

Sessions are identified by UUID, persisted on disk (`~/.local/share/opencode/`), and can coexist concurrently. Session state is scoped to the working directory — `--continue` picks up the last session in that directory.

**Driver integration:**
- `session { }` block: pass `--continue` on every `do` call inside the block. The first call auto-starts a session, subsequent calls reuse it.
- Outside `session { }`: omit `--continue` — each `do` runs as a fresh one-shot call.
- The existing hardcoded `--continue` in `OpenCodeDriver::build_command()` (driver.rs:23) must be removed and made conditional on `ExecuteRequest.session`.

**Compact interaction:**
- OpenCode auto-compacts by default. Set `OPENCODE_DISABLE_AUTOCOMPACT=true` to disable.
- When `session { }` is active, compact operates normally (OpenCode handles it internally).
- Outside a session, compact directives on `do` calls are not useful for opencode — logging a warning is appropriate.

#### claude-code

Supports sessions natively via CLI flags:
- `--continue` / `-c` — load the most recent conversation in the current directory
- `-r "<id>" "<query>"` — resume a session by ID or name
- Background sessions: `claude attach <id>`, `claude respawn <id>`, `claude rm <id>`
- Sessions are persisted on disk (transcripts and state under `~/.claude/`)

**Driver integration:**
- `session { }` block: pass `--continue` on every `do` call inside the block. The first call starts a session, subsequent calls resume it.
- Outside `session { }`: omit `--continue` — each `do` runs as a fresh one-shot call.
- The existing `ClaudeDriver::build_command()` (driver.rs:37-53) currently passes `--msg`. For session mode, `--continue` should be added alongside `--msg`.

`ClaudeDriver` was previously assumed to have no session support. This research confirms it does — the driver must be updated to handle `ExecuteRequest.session`.

#### aider

Supports session-like behavior via chat history:
- `--chat-history-file <file>` — specifies the chat history file (default: `.aider.chat.history.md`)
- `--restore-chat-history` — restores previous chat history messages on startup (default: false)
- `--message` / `--msg` / `-m <prompt>` — sends a single message and exits (one-shot mode)
- `--message-file` / `-f <file>` — sends a message from a file and exits

Chat history is scoped per directory (the `.aider.chat.history.md` file lives in the working directory).

**Driver integration:**
- `session { }` block: pass `--restore-chat-history` on every `do` call inside the block. The first call writes to `.aider.chat.history.md`, subsequent calls resume from it.
- Outside `session { }`: omit `--restore-chat-history` — each `do` runs one-shot via `--message`.

The existing `AiderDriver::build_command()` (driver.rs:56-77) currently passes `--model` and `--msg`. For session mode, add `--restore-chat-history` alongside `--msg`.

#### echo

Built-in test agent. No session support — confirmed no-op. `EchoDriver::build_command()` ignores `ExecuteRequest.session`. `session { }` blocks should parse and execute normally, just without any session flags.

Agents without session support should silently accept `session { }` — the block is parsed and `do` calls execute normally, just without session flags.

### Interaction with compact

When session is active, compact operates normally. When no session is active, `compact` directives on a `do` call produce a warning to stderr (compacting without session context is meaningless for agents like opencode).

### Engine integration

`ExecuteRequest` gains a `session: bool` field. Each driver reads this and applies the appropriate flags:
- `OpenCodeDriver`: passes `--continue` when `session == true`, omitted when `false`. 
- `ClaudeDriver`: passes `--continue` when `session == true`, omitted when `false` (same pattern as opencode).
- `AiderDriver`: passes `--restore-chat-history` when `session == true`, omitted when `false`.
- `EchoDriver`: ignores the field (no-op)

## Acceptance Criteria

1. **`session { }` block scopes `do` calls**
   - `do` calls inside a `session { }` block pass session flags to the agent
   - `do` calls outside a session block do NOT pass session flags

2. **`OpenCodeDriver` drops the hardcoded `--continue`**
   - The driver no longer always appends `--continue`. It reads `ExecuteRequest.session` to decide.

3. **Agents without session support are unaffected**
   - Claude code, echo, etc. accept `session { }` — no parse errors, no unexpected flags

4. **Nested session blocks are an error**
   - A `session { }` inside another `session { }` produces a parse or runtime error

5. **Session state is lexically scoped**
   - A `session {}` block only affects `do` calls within its own `{}` body
   - `session {}` inside a function or nested `{}` block is scoped to that block

6. **Compact warns outside sessions**
   - Using `compact` on a `do` call when no session is active prints a warning to stderr

## Implementation Hints

### Relevant project context

The project is a single Rust crate at `ash/`. The session feature spans the language layer (parser/AST/evaluator) and the agent abstraction layer (engine).

**Language layer** — how `do` works today:
- `ast.rs:153-162` — `AgentCall` struct. No changes needed — session control is at the block level, not per-call. Each `do` inside a `session {}` checks the evaluator's session flag.
- `parser.rs:380-453` — `parse_do()` builds an `AgentCall` from `do <prompt> [with <agent>] [using <model>] [in <dir>] [compact <directive>]`. No changes needed. The `session {}` block is a new statement in `parse_statement()`, following the same pattern as `if`, `for`, `while` etc. — parse `session` keyword followed by a `{}` body.
- `eval/agent.rs:9-103` — `eval_agent_call()` builds an `ExecuteRequest` and dispatches via `engine::get()`. This is where the evaluator's session flag is read and passed as `ExecuteRequest.session`.
- `eval/mod.rs:72-83` — `Evaluator` struct holds mutable runtime state. Session state is a boolean flag pushed/popped on entering/exiting a `session {}` block, similar to how scopes are pushed/popped.
- `token.rs:101-112` — keyword table. Only `session` needs to be added.

**Engine layer** — how agents are dispatched:
- `engine/types.rs` — `ExecuteRequest { prompt, model, dir }`. Gains a `session: bool` field.
- `engine/driver.rs:15-35` — `OpenCodeDriver::build_command()` currently hardcodes `--continue` on line 23. Make this conditional on `req.session`.
- `engine/driver.rs:37-53` — `ClaudeDriver` ignores the session field (no-op).
- `engine/adapter.rs:14-26` — `LocalCliAdapter` passes `ExecuteRequest` through; no changes needed.

**Existing patterns for adding block statements:**
- Parser: see `parse_if()`, `parse_for()`, `parse_while()` in `parser.rs` — they parse a keyword, then a `{}` body. Same pattern for `parse_session()`.
- Evaluator: see `eval_if_stmt()`, `eval_for_stmt()` in `eval/mod.rs` — they push scope, eval body, pop scope. Same pattern for `eval_session_block()`.

### What not to touch

The scope system (`scope.rs`), value system (`value.rs`), interpreter core (`eval/expr.rs`, `eval/conc.rs`), and execution subsystem (`executor.rs`) are unrelated to session control.
