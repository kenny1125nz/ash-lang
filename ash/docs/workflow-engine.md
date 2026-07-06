# Markdown Workflow Engine

> A standalone tool that turns a markdown procedure document into an executed
> workflow. Write what needs to happen — numbered sections with prompts —
> and the engine runs each step through an LLM agent, passing state between them.
> No scripting language, no directory structure, no YAML. Just markdown.

---

## The core idea

A domain expert writes a procedure document:

````markdown
---
agent: opencode
model: sonnet
---

# Database Migration Guide

This workflow handles the full migration from MongoDB to PostgreSQL.
Backup the database before running.

## 1. Audit

> model: claude-sonnet-4

List every collection in the database. For each collection, document
the schema, indexes, and estimated row count. Note any embedded documents
that will need flattening for SQL.

## 2. Schema Design

Based on the audit results:
${stdout}

Design the PostgreSQL schema. Include CREATE TABLE statements
for every collection, handle relationships, and add indexes.
Output the full SQL DDL.

## 3. Migration Script

Write a Python migration script that reads from MongoDB and writes
to PostgreSQL using the schema above. Include:
- Batch processing for large collections
- Resume capability if interrupted
- Validation queries after migration

## 4. Validation

Run the migration on the test dataset and verify:
$(wc -l import.sql)

Check for:
- Row count matches between source and destination
- No truncated data
- Indexes are utilized
````

They run one command:

```bash
mwf run migration.md
```

The engine:
1. Parses the frontmatter (`agent: opencode`, `model: sonnet`)
2. Splits on `## N. Title` into sequential steps
3. Sends each step body as a prompt to the agent
4. Captures stdout from step N, makes it available as `${stdout}` in step N+1
5. Applies per-step overrides from blockquote directives (`> model: ...`)
6. Skips everything outside `##` sections (the intro is documentation)
7. Executes inline `$(cmd)` substitutions

No loops. No variables. No functions. No nesting.

The document is the program. The procedure is the execution.

---

## Why not ash?

Ash asks the user to learn a DSL:

```ash
for FILE in $(find src -name '*.ts') {
  do "Review ${FILE}" with opencode
}
```

This is simpler than bash, but still a **language**. Variables, string
interpolation, control flow, function syntax — it's a programming model
dressed in shell syntax.

The markdown workflow engine asks the user to write a procedure
document they'd write anyway. The only additions are:

1. **Numbered headings** — which they'd already use
2. **Frontmatter** — 3 lines of YAML at the top
3. **`${stdout}`** — a natural "use the results from the previous step"
4. **`$(cmd)`** — inline command output, same pattern ash already uses

The cognitive load is near-zero for anyone who has written a numbered
list of instructions before.

---

## Comparison

| | ash directory mode | ash scripts | markdown workflow |
|---|---|---|---|
| Author | Engineer structuring tasks | Engineer writing logic | Domain expert writing procedures |
| Unit | One `.md` file per task | `.ash` script file | One `.md` file per workflow |
| State | None shared between files | Variables, scope, `${stdout}` | `${stdout}` passes to next step |
| Control flow | Implicit by numeric order | `if`, `for`, `while`, `try` | Implicit by heading order |
| Learning curve | Must learn frontmatter format | Must learn ash DSL | Must learn 3 concepts |
| Best for | Large decomposed projects | Complex conditional pipelines | Straight-line procedures |

---

## Design decisions

### Why no variables?

Variables are where simplicity dies. As soon as you have `X = 42`, you
have mutation, scope, and the cognitive overhead of tracking state.
`${stdout}` is the only state-passing mechanism — it flows step to step
like a data pipe. If you need variables, use ash.

### Why no control flow?

`if`, `for`, `while` make workflows unpredictable. You can't read a
document top-to-bottom and understand what it does if there's branching.
Straight-line procedures are the 80% case. The 20% case (conditional
workflows) stays in ash.

### Why no loop/retry?

Same argument. If a step fails, the engine stops and reports the error.
The user decides whether to edit and restart, or re-run from the failure
point. The document is the record of what happened — adding retry logic
in the document obscures the audit trail.

### Why blockquote directives?

Per-step overrides (model, agent, on_fail) need to live somewhere.
Blockquotes are already a "meta" construct in markdown — readers parse
them as commentary, not content. They're visually distinct from the
procedure text and easy to scan.

```markdown
## 3. Migration Script

> model: claude-opus
> on_fail: continue

Write a Python migration script that reads from MongoDB...
```

The directive block is a visual annotation — the reader sees "this uses
a different model" at a glance, then reads the actual instructions.

### Why `${stdout}` instead of pipe syntax?

Pipe syntax (`|`) would look cleaner:

```markdown
## 2. Plan

Use the output from the previous step to create a plan.

> input: ${prev}
```

But `${stdout}` is explicit about what it is — stdout from the last step.
It's the same pattern ash uses, so users who graduate from workflows to
ash scripts don't need to relearn state passing.

---

## Execution model

```
parse frontmatter → agent, model
split on "## N. Title" → ordered steps
for each step:
  1. resolve ${stdout} → replace with previous step's output
  2. resolve $(cmd) → replace with command output
  3. strip blockquote directives
  4. build prompt from remaining body text
  5. invoke agent with prompt
  6. capture stdout, stderr, exit code
  7. if exit ≠ 0 and on_fail = "stop" → halt
  8. continue to next step
```

On completion the engine reports: N steps, M passed, K failed. Failed
steps include stderr in the report. The original markdown document is
preserved unmodified — no output is written into it by default.

---

## Step directives

| Directive | Values | Description |
|-----------|--------|-------------|
| `model` | model name | Override the AI model for this step |
| `agent` | agent name | Override the agent for this step |
| `on_fail` | `stop` (default), `continue` | Control whether to halt on failure |
| `compact` | compact directive | Context compacting for this step |

All directives are optional. Missing directives inherit from the
frontmatter or from defaults.

---

## Templating within steps

Two replacements are supported:

| Pattern | Source | Example |
|---------|--------|---------|
| `${stdout}` | Previous step's stdout | `Based on: ${stdout}` |
| `$(cmd)` | Shell command output | `Files: $(find src -name '*.ts')` |

Literals `$` that should not be substituted are written as `\$`.

---

## Invocation

```bash
mwf run migration.md                    # execute all steps
mwf run migration.md --from 3           # resume from step 3
mwf run migration.md --dry-run          # print steps without executing
mwf run migration.md --agent claude-code:sonnet   # override default
```

Resume mode is important: a 10-step workflow that fails at step 7
shouldn't require re-running the first 6 expensive LLM calls. Resume
skips completed steps and picks up from the failure point.

---

## What's not in scope

- **Variables** — use ash for state beyond `${stdout}`
- **Loops** — copy-paste sections if you need repeat steps (the document
  is the loop unrolling)
- **Conditionals** — use ash for branching workflows
- **Parallel steps** — intentionally sequential; parallel execution is
  unpredictable and hurts auditability
- **Sub-workflows** — include a sub-workflow file if you need composition
  (a step body that references another .md file)
- **Human-in-the-loop** — add a step directive like `> pause: true` later
  if the user needs approval gates between steps

---

## Why this works as a separate tool

1. **Zero overlap with ash's engine.** The executor is a simple loop: for
   each section, call the agent adapter. It doesn't need the ash evaluator,
   scope, parser, or tree walker.

2. **Independent surface area.** This tool has its own config, its own
   invocation, and its own output format. It doesn't share state with ash.

3. **Different user, different entry point.** People find this via
   "automate my workflow with AI" — not "scripting language for agents."

4. **Thin wrapper.** The bulk of the work is already done:
   - Agent adapters + driver registry (`engine/`)
   - Agent discovery (`engine/discovery.rs`)
   - `ash-project.yaml` config format
   - Markdown frontmatter parser (`tree.rs` already does this)
   - `${stdout}` interpolation (`interpolation.rs`)

   The only new code needed is: split a markdown document on `##` headings,
   strip blockquote directives, and run a for-loop.

---

## Relationship to ash

```
ash → scripting language + directory orchestration
mwf → procedure documents → agent calls

Both use the same engine (adapters, drivers, config, discovery).
Both register agents from ash-project.yaml.
ash is for engineers. mwf is for domain experts.
```

The tools are siblings sharing a core library. Nothing in ash changes.
The workflow engine is a minimal new binary — likely 200-300 lines of
code on top of the existing engine.

---

## Open questions

1. **Resume semantics.** If step 3 fails and the user runs `--from 3`,
   does `${stdout}` contain the output from step 2 (which already ran
   successfully) or is it empty? Need to persist step outputs between runs.

2. **Output file.** Should a `--output result.md` flag produce a copy of
   the workflow document with step outputs appended under each section?

3. **Section references.** Should later sections be able to reference
   specific earlier sections by number? `${stdout:2}`? That adds
   complexity but enables more sophisticated workflows.

4. **Naming.** `mwf` is a placeholder. Alternatives: `proc` (procedure),
   `step` (steps), `flow`, `runmd`. The name should be one syllable,
   terminal-friendly, and imply "workflow" without overlapping with ash.

5. **Should it share the ash-project.yaml format?** Yes. One config file
   for both tools. Adding a new agent to `ash-project.yaml` makes it
   available to both ash scripts and markdown workflows.
