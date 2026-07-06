# Directory-Based Orchestration

## Background

Currently, ash operates as a one-shot script executor — you write a `.ash` script, then ash parses and runs it sequentially. This works well for linear workflows but is awkward for larger, multi-step projects. Developers naturally decompose complex work into a tree of folders and files, which maps perfectly to task decomposition.

The goal is to let ash consume a directory tree of markdown files as an alternative to `.ash` scripts, enabling a simpler, more natural way to organize and orchestrate agent tasks.

## Intended Solution

Detect when `ash` is invoked with a directory path instead of a `.ash` file:

1. **Walks** the directory tree in depth-first order. At each level, files and directories are sorted together by their numeric prefix — a directory is descended into when its position in the sorted order is reached, not grouped separately.
2. **Reads** each `.md` file — the content becomes the task prompt sent to an agent
3. **Dispatches** each task to the configured agent (with optional per-task agent/model/compact overrides via frontmatter)
4. **Reports** progress (which task, success/failure, timing) to stdout

### Directory structure example

```
tasks/
├── 01-intro.md                 # Task prompt
├── 02-types/                   # (directory, descended into when its turn comes)
│   ├── 01-token.md             # Task prompt
│   ├── 02-value.md             # Task prompt
│   └── 03-ast.md               # Task prompt
├── 03-transforms/              # (directory)
│   ├── 01-lexer.md             # Task prompt
│   ├── 02-parser.md            # Task prompt
│   └── 03-interp.md            # Task prompt
└── 04-conclusion.md            # Task prompt
```

Execution order: `01-intro.md` → `02-types/01-token.md` → `02-types/02-value.md` → `02-types/03-ast.md` → `03-transforms/01-lexer.md` → … → `04-conclusion.md`

### Per-task configuration via markdown frontmatter

Each `.md` file can optionally include YAML frontmatter to override the agent, model, or compact settings:

```markdown
---
agent: opencode
model: sonnet
compact: on
on_fail: continue
---

# Task title

The content of the task prompt goes here...
```

Supported frontmatter keys:
- `agent` — agent name (e.g. `opencode`, `claude-code`)
- `model` — model name (e.g. `sonnet`, `gpt-4o`)
- `compact` — compact mode (`on`/`off`/`auto`)
- `on_fail` — failure behavior: `stop` (default, halt the entire run) or `continue` (proceed to next task)

If no frontmatter is present, the task falls back to ash defaults (the default agent/model from the global config or built-in constants).

### CLI invocation

```bash
ash /path/to/task-tree
ash .                            # current directory
ash --dry-run ./tasks            # print what would run, don't execute
```

## Acceptance Criteria

1. **`ash ./tasks` walks the test tree and executes every `.md` file in order**
   - At each level, files and subdirectories are sorted together by numeric prefix
   - When a directory is reached in sorted order, it's descended into depth-first before continuing to the next sibling
   - This means `01-intro.md` runs before `02-types/01-token.md`, which runs before `03-conclusion.md`

2. **Per-task frontmatter overrides agent/model/compact/on_fail**
   - Task with `agent: opencode` in frontmatter uses opencode, not the default
   - Task with `on_fail: continue` keeps running despite failure for that task
   - Task with no frontmatter falls back to ash defaults
   - Frontmatter is stripped from the prompt before sending to the agent

3. **`--dry-run` prints tasks without executing them**
   - Shows file path, agent, model, and prompt preview (first 80 chars) for each task

4. **Progress reporting**
   - Prints `[1/5] 1-types/01-token.md` before execution
   - Prints `[ok]` or `[fail]` with exit code after each task
   - At end, prints summary: `N tasks, X passed, Y failed`

5. **Task file eligibility — only valid task files are executed**
   Files that don't qualify are skipped with a line printed to stdout:
   - `[skip] non-md: <path>` — not a `.md` file
   - `[skip] no-prefix: <path>` — `.md` file without a numeric prefix (e.g. `readme.md`)
   - `[skip] empty: <path>` — file with only whitespace/frontmatter, no prompt content
   - Hidden files and directories (starting with `.`) are skipped silently

## Implementation Hints

### Relevant project context

The project is a single Rust crate at `ash/`. The key architectural boundary: the tree orchestration logic should be **independent of the ash script language** (parser, evaluator, AST). It only needs the **agent abstraction layer** in `ash/src/engine/`.

**Agent abstraction** (`ash/src/engine/`):
- `types.rs` — `ExecuteRequest { prompt, model, dir }` and `ExecuteResponse { stdout, stderr, exit_code }`
- `adapter.rs` — `Adapter` trait with `execute(&self, req: &ExecuteRequest) -> ExecuteResponse`
- `mod.rs` — global registry: `register(name, adapter)` / `get(name) -> Option<Arc<dyn Adapter>>`. Default agents (echo, opencode, claude-code, aider) are registered via `register_defaults()` in `main.rs`.
- `driver.rs` — per-agent CLI command builders (`OpenCodeDriver`, `ClaudeDriver`, etc.) wrapped by `LocalCliAdapter`

To dispatch a task, build an `ExecuteRequest`, look up the agent from the registry, and call `adapter.execute()`. This is the same path `eval/agent.rs::eval_agent_call()` uses internally.

**CLI entry** (`ash/src/main.rs`):
- No CLI framework — manual argument parsing in `run()`. Currently handles `--check` / `-c` / `--dry-run` flags and a positional file argument, falling back to stdin.
- `validate_agents()` manually scans a minimal `ash-project.yaml` for agent names.

**Existing defaults**:
- `eval/mod.rs`: `DEFAULT_AGENT = "echo"`
- `compact.rs`: `CompactConfig::new()` defaults to `mode="auto"`, `window="32000"`, `strategy="truncate"`

**What not to touch**: the parser (`parser.rs`), lexer (`lexer.rs`), AST (`ast.rs`), evaluator (`eval/`), and scope (`scope.rs`) are all ash language internals. The tree feature sits at the CLI level, above the agent layer.

### Frontmatter parsing

Hand-parsed, same approach as the existing `ash-project.yaml` parser in `main.rs:118-152`. No new dependencies needed — just extract `agent`, `model`, `compact`, `on_fail` keys from the `---`-delimited block at the top of a `.md` file.

### Testing

The project tests via `.ash` scripts in `ash/testdata/`. For the tree feature, Rust unit tests in `ash/tests/` for traversal ordering and frontmatter parsing would be more appropriate — create temp dirs with task tree fixtures and assert the resulting `Vec<Task>` order and fields.
