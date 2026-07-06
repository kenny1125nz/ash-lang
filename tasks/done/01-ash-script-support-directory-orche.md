# Support .ash script files in directory orchestration

## Background

Currently, directory-based orchestration only recognizes `.md` files as task files. The tree traversal engine walks the directory tree, identifies `.md` files with numeric prefixes, and dispatches them as agent prompts.

However, ash also has its own scripting language (`.ash` files) which can orchestrate multiple agent calls, set variables, use control flow, retry logic, session blocks, and more. Without `.ash` script support in the tree, users cannot leverage these language features in directory-orchestrated workflows — they're limited to plain markdown prompts.

Supporting `.ash` files in the tree unlocks full ash language capabilities within directory-orchestrated workflows.

## Intended Solution

The tree traversal should also recognize `.ash` files with numeric prefixes (e.g., `01_setup.ash`, `02_build.ash`) alongside `.md` files. When a `.ash` file is encountered, it should be parsed and evaluated as an ash script using the existing interpreter, not treated as a raw prompt.

```
tasks/
├── 01_intro.md            # Markdown prompt (existing behavior)
├── 02_setup.ash           # Ash script — parsed and evaluated
├── 03_build.ash           # Ash script — can use variables from prior steps
└── 04_outro.md            # Markdown prompt
```

### Execution model

- `.md` files: read as prompt text, dispatched to the configured agent via `ExecuteRequest` (existing behavior)
- `.ash` files: parsed by the existing parser, evaluated by the existing evaluator. Variables set in a `.ash` file persist for subsequent tasks (both `.md` and `.ash`) — similar to how `include` works in `.ash` scripts.

This means earlier steps can set up state that later steps consume:

```
02_setup.ash:
  TARGET = "src/auth/login.ts"
  MODEL = "sonnet"

03_build.ash:
  do "Refactor ${TARGET}" with opencode using MODEL
```

### Task file eligibility update

Files with numeric prefix and either `.md` or `.ash` extension are eligible. The `is_task_file` check in `tree.rs` must be broadened.

## Acceptance Criteria

1. **`.ash` files with numeric prefix are recognized as tasks**
   - `01_build.ash` is included in the task list, `readme.ash` is skipped (no prefix)

2. **`.ash` tasks are parsed and evaluated as ash scripts**
   - The script's `do` calls, variables, control flow, etc. all execute normally
   - The script's shebang (`#!agent:version`) sets the default agent for that script

3. **Variables set in a `.ash` script persist for subsequent tasks**
   - `VAR = "hello"` in `01_setup.ash` is available as `${VAR}` in `02_task.md`

4. **`.md` and `.ash` files are sorted together by numeric prefix**
   - `01_intro.md` → `02_setup.ash` → `03_build.md` (order respects prefix, not extension)

5. **Existing `.md` behavior is unchanged**
   - Markdown files continue to work as before — frontmatter parsed, prompt dispatched to agent

6. **Mixed extensions at the same level with same prefix are a conflict**
   - `01_task.md` and `01_task.ash` conflict (same prefix) and produce an error

7. **`--dry-run` shows `.ash` tasks**
   - Prints file path with type indicator (e.g., `agent=echo` for default)

## Implementation Hints

### Relevant project context

**Where task file filtering happens:**
- `tree.rs:32-38` — `is_task_file()` currently checks only for `.md` extension with a numeric prefix. Must also accept `.ash`.

**Where tasks are executed:**
- `tree.rs:246-348` — `run_tree()` reads task content, dispatches to agent. Currently all tasks go through the same path: read file → parse frontmatter → build `ExecuteRequest` → call `engine::get(agent).execute()`.
- For `.ash` files, this path doesn't apply — instead, the file should be parsed with `ash::parser::parse_str()` and evaluated with an `Evaluator`.

**Existing ash script execution:**
- `main.rs:204-242` — the CLI script mode: `parse_str(&src)` → `validate_agents()` → `Evaluator::new()` → `eval.eval_script(&script)`. This is the pattern to follow for `.ash` files in tree mode.
- `eval/mod.rs:85-100` — `Evaluator::new()` initializes with empty scope, default agent, etc. For tree mode, the evaluator should persist across tasks so variables accumulate.

**Key design decisions:**
- The `Evaluator` (or just its `global_scope`) should be shared across all tasks in the tree — `.md` tasks should also have access to variables set by prior `.ash` tasks.
- Each `.md` task still dispatches via `ExecuteRequest` (unchanged), but the prompt interpolation already resolves `${VAR}` from scope via the `eval_fp` change made earlier.
- Task execution could switch between two modes: "prompt mode" for `.md` and "script mode" for `.ash`, with a shared evaluator/scope.

### What not to touch

The parser, lexer, AST, and existing evaluator logic are all correct — they just need to be called from the tree orchestration loop for `.ash` files.
