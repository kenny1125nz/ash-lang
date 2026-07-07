# Integration Test Set

## Background

Ash needs an integration test suite that validates its features against different coding agents (opencode, claude-code, aider). Currently there is no automated way to verify that ash works correctly with these agents. The directory orchestration feature allows task trees of markdown definitions and `.ash` scripts, providing a natural foundation for creating integration tests.

## Intended Solution

Create a test directory tree under `ash/testdata/integration/` that models a typical (simplified) software development workflow. The tree should exercise the core ash features:

- Directory-based task orchestration
- `.ash` script flow control (if/else, loops) within a tree
- Per-task frontmatter overrides (agent, model)
- Progress reporting and error handling

### Example test tree structure

```
ash/testdata/integration/
├── 01-requirements.md          # Define project requirements
├── 02-design.md                # Design system architecture
├── 03-implementation/
│   ├── 00-init-repo.ash        # Initialize git repo
│   ├── init.md                 # Project scaffolding (per-language, not a task)
│   ├── 01-module-a.md          # Implement module A
│   ├── 02-module-b.md          # Implement module B (CLI layer)
│   └── 03-integrate.md         # Integrate modules
├── 04-verify/
│   ├── 01-unit-test.md         # Run unit tests
│   └── 02-e2e-test.md          # Run end-to-end tests
└── 05-deploy.ash               # Deploy — commit and tag a release
```

The target project is determined by the cwd when ash is invoked (e.g., `ash --cwd ash/testdata/test-project/ ../integration/`). The cwd holds an initial primitive idea from the user (e.g., a bare scaffold or rough sketch). The task tree simulates the full development cycle: starting from this raw concept, each step refines it until the final task delivers a workable piece of software.

The modelled flow itself must be generic — it should not embed any knowledge of or assumptions about the specific idea in the cwd. The tree defines a process (requirements → design → implementation → verify → deploy → release), not a solution to the idea. This keeps the test reusable: run the same tree against a different cwd and it exercises the same workflow.

Each step hands off to the next by writing file(s) into the cwd (using relative paths). Each task references the output files of earlier steps as context. This creates a dependency chain where tasks are reminded of what was built before:

| Step | Reads | Writes |
|------|-------|--------|
| 01-requirements.md | `ideas.txt` | `REQUIREMENTS.md` |
| 02-design.md | `REQUIREMENTS.md` | `DESIGN.md` |
| 03-implementation/00-init-repo.ash | `DESIGN.md` | git repo |
| 03-implementation/01-* | `init.md`, `src/` (scaffold) | `src/*` |
| 04-verify/* | `src/*` | `tests/*` |
| 05-deploy.ash | `src/`, `tests/` | git tag `v1.0.0` |

Each task runs against a configurable agent (e.g., opencode, claude-code, aider). The test harness invokes `ash ash/testdata/integration/` and validates the output against expected patterns.

## Acceptance Criteria

1. **`ash testdata/integration/` executes the full tree in order**, producing output with `[1/N]` progress markers and `[ok]`/`[fail]` per task.

2. **Each core ash feature is covered by at least one test case**:
   - Directory orchestration (tree walk, numeric sort, depth-first descent)
   - `.ash` script flow control (if/else, loops) within a tree
   - Frontmatter-based agent override

3. **A `--dry-run` flag prints the expected execution plan** without actually dispatching tasks.

## Implementation Hints

### Relevant project context

- Tree orchestration logic lives at the CLI level in `ash/src/` (independent of parser/evaluator).
- Existing test fixtures are in `ash/testdata/` — place the integration tests alongside them.
- Frontmatter parsing follows the same hand-parsed approach as `ash-project.yaml` parsing in `main.rs:118-152`.

### What to cover

Model the test workflow after `tasks/done/1-directory-orchestration.md` acceptance criteria — the integration test should verify the exact behaviors specified there.

### What not to touch

Do not modify the engine agent abstraction, the parser, or the evaluator. The integration test set lives entirely in `ash/testdata/` and is driven by the existing CLI.
