---
{"id": "directory-mode", "title": "Directory Mode"}
---

## Directory Mode

Pass a directory to run task files in numeric-prefix order:

```bash
ash ./tasks
ash ./tasks --dry-run
```

```
tasks/
├── 01-intro.md
├── 02-types/
│   ├── 01-token.md
│   ├── 02-value.md
│   └── 03-ast.md
└── 03-conclusion.md
```

Execution order: `01-intro.md` → `02-types/01-token.md` → `02-types/02-value.md` → `02-types/03-ast.md` → `03-conclusion.md`

Files are executed when they have a numeric prefix (`01-`, `02-`, etc.). Files with duplicate prefixes at the same level are reported as errors.

Markdown files can set per-task settings with YAML frontmatter:

```markdown
---
agent: opencode
model: sonnet
on_fail: continue
---

# Task Title

The prompt content for the agent goes here...
```
