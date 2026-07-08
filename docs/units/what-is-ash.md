---
{"id": "what-is-ash", "title": "What is Ash?"}
---

## What is Ash?

Ash is a task runner for AI agents — deterministic, repeatable, no scripting required. Point Ash at a directory of markdown files and it walks the tree in sorted order, sending each file to your configured AI agent. One task per file.

```
tasks/
├── 1-plan/
│   ├── 01-requirements.md
│   └── 02-architecture.md
├── 2-implement/
│   ├── 01-auth.md
│   ├── 02-api.md
│   └── 03-tests.md
└── 3-review/
    └── 01-code-review.md

→ ash tasks/
```

When you need more than one-shot prompts — chaining, conditionals, parallelism — write an `.ash` script. Start simple. Grow as needed.
