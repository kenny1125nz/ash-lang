---
{"id": "directory-mode", "title": "Directory Mode"}
---

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

Same-prefix files at the same level form a parallel group — all tasks with the same prefix run concurrently:

```
tasks/
├── 01-research.md
├── 02-parse-data.md
├── 02-analyze-results.md    ← runs in parallel with 02-parse-data.md
├── 02-summarize.md          ← also part of the parallel group
└── 03-implement.md          ← waits for all of 02-* to finish
```

By default, ash prompts for confirmation: *"N parallel group(s) detected. Run them in parallel? [y/N]"*. Pass `--yes` to skip the prompt and always allow. In non-interactive mode (CI, piped stdin), parallel groups produce an error unless `--yes` is given.

A file and a subdirectory with the same prefix form a combined parallel group — the file and the entire subdirectory walk run concurrently on separate threads.

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
| `--yes` / `-y` | Allow parallel execution without confirmation prompt |
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
