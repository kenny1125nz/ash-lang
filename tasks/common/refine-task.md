# Task Definition Principles

refin/update task definition files of ${file} to be used by the directory-based orchestration engine (`ash <directory>`).

## File Structure

1. **Flat file for single tasks**
   If a task has no subtasks, use a single `N-name.md` file — no directory with one file inside.
   Example: `1-directory-orchestration.md`

2. **Directory for decomposed tasks**
   Use a folder (`N-name/`) only when the task breaks into multiple research/implementation files.
   Example: `2-session-context/` containing `01-opencode.md`, `02-claude-code.md`, ..., `05-implementation.md`

3. **Numeric prefix on everything**
   Files and directories are named `{NN}-{name}`, sorted together at each level by numeric prefix.
   Duplicate prefixes at the same level are an error — ash will report the conflict and exit.

## Content Shape

Task files use different shapes depending on their type. Pick one.

### Implementation tasks

Four fixed sections:

- **Background** — the problem being solved, current state, why it matters
- **Intended Solution** — design with code examples, tables, CLI invocations
- **Acceptance Criteria** — numbered, testable, specific (e.g., "prints `[ok]` after each task")
- **Implementation Hints** — project context for the coding agent. Describe relevant modules, file paths, existing patterns, and what NOT to touch. Do not prescribe struct names or step-by-step instructions. Follow existing codebase conventions for any new syntax.

### Research tasks

Three fixed parts:

- **Context** — read the implementation task file (name it) to understand what's being built
- **Research** — the agent's official documentation URL, specific questions to answer
- **Deliverable** — update the implementation task's per-topic subsection with concrete findings for integration
