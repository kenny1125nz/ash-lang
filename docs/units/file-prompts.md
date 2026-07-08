---
{"id": "file-prompts", "title": "File-based Prompts"}
---

## File-based Prompts (`@file`)

Load a prompt from a file with variable interpolation:

```ash
do @"path/to/prompt.md"
```

The file is read, `${VAR}` placeholders are resolved, and the result is sent to the agent.

`@<path>` loads the prompt from a file relative to the script:

```ash
do @skills/refactor.md with opencode
do @skills/review.md with opencode using sonnet
do @'play_step${n}.md' with opencode
do @${DIR}/prompts/task.md with opencode
```
