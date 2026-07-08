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

### Directory orchestration

If the `@` path points to a **directory** rather than a file, the task tree walker runs instead — the same mechanism used by `ash tasks/`:

```ash
do @"tasks/" with opencode
```

This walks the directory, discovers numbered `.md` and `.ash` files, and executes them in sorted order using the specified agent. Each `.md` file becomes a standalone task sent to the agent. Each `.ash` file is executed as a script with access to the same evaluator scope.

This lets scripts recursively compose task directories:

```ash
do @"review/" with opencode
do @"fix/" with opencode using sonnet
if $? == 0 {
  do @"deploy/" with opencode
}
```

Individual task files can override the agent and model via YAML frontmatter, just like in directory mode.
