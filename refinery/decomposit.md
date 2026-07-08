
# Task Decomposition

Analyze the design or task from ${INPUT_FILE} and decompose it into a sequence of independently actionable tasks. The output must be structured as a valid ash [directory-mode](directory-mode.md) task tree under `tasks/backlog/_taskname`.

## Principles

- **Dependency ordering** — later tasks should not require modifying artifacts produced by earlier tasks. Each task must be self-contained and produce final output, not drafts that downstream tasks patch up.
- **Context window discipline** — each task's prompt must be small enough for a coding agent to process in a single context window. Split when a task would exceed reasonable limits. Err on the side of smaller, more focused tasks.
- **Verifiable completion** — every task must produce a concrete, testable outcome. No open-ended research tasks without a deliverable.

## Output Structure

Create the task tree at `tasks/backlog/` using numeric-prefix filenames. Each file is a markdown task prompt. Use subdirectories for grouped phases.

```
tasks/backlog/
├── 01-phase-name/
│   ├── 01-first-task.md
│   ├── 02-second-task.md
│   └── 03-third-task.md
├── 02-next-phase/
│   ├── 01-task.md
│   └── 02-task.md
└── 03-final-phase.md
```
Structure each file with these sections:

- **Background** — the problem, current state, why it matters
- **Intended Solution** — the design with concrete decisions, trade-offs, and rationale. .
- **Acceptance Criteria** — numbered, testable, specific conditions that define done
- **Implementation Hints** — project context: relevant modules, file paths, existing patterns, what NOT to touch. Do not prescribe struct names or step-by-step instructions.
  

## Dependency Rules

- A task may reference artifacts from any earlier task (lower numeric prefix).
- A task must not require modifying artifacts from any later task (higher numeric prefix).
- If a task needs an artifact that another task produces, the producer must have a lower prefix.

## Size Guidelines

Each task should address exactly one concern. Avoid tasks that are trivial mechanical steps with no design judgment (e.g. "create an empty file") — merge those into the task that needs them. Otherwise, use your judgment.
