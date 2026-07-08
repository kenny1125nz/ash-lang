---
{"id": "quick-start", "title": "Quick Start"}
---

## Quick Start

### Configure your agent

Create `ash.yml` in your project root:

```yaml
default_agent: opencode
```

Or set it per-run:

```bash
ash --agent opencode tasks/
```

### Run your first project

```
my-project/
├── ash.yml
└── tasks/
    ├── 1-init/
    │   └── 01-setup.md
    └── 2-feature/
        └── 01-add-login.md
```

```bash
ash my-project/tasks/
```

Ash prints each task and its result as the agent completes it. Tasks that return a non-zero exit code are marked as failures.

### Skip failures, keep going

```bash
ash --continue-on-error tasks/
```

### Validate without running

```bash
ash --check tasks/
```

### See what would run

```bash
ash --dry-run tasks/
```
