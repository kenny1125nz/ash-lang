# Challenge Me

> A self-interrogation of ash's value proposition. Re-read before making decisions that
> expand scope, add syntax, or assume adoption. This file exists to keep the project
> honest.

---

## 1. What does ash actually do?

At its core, ash is a `for` loop around `Process::new("opencode").arg(prompt)`. A 3-line
bash script does the same thing:

```bash
for FILE in $(find src -name '*.ts'); do
  opencode run "Review $FILE"
done
```

Ash adds:

- **A parser** — validates syntax before execution
- **A scope system** — variables, `${stdout}`, `${stderr}`, `$?`
- **Built-in retry** — `try { } fail { } upto N`
- **Session management** — `--continue` flag plumbing
- **An agent registry** — `with opencode` resolved through config
- **A directory walker** — numbered markdown files executed in order

Question: is the parser + scope + retry + session surface large enough to justify learning
a new language, or is it a thin wrapper that would be better served as flags on a CLI tool?

---

## 2. The deterministic workflow paradox

Ash's value proposition is "deterministic execution with intelligence fused in." The
structure is deterministic — step order, conditions, retries. But every step bottoms out
at `do "prompt"` — a stochastic LLM call. Two runs of the same script produce different
code changes.

The retry machinery (`upto 3`) compounds this: it adds stochastic retries on top of
stochastic results. The workflow looks rigorous; the output isn't.

Counterpoint: this is the same model as human delegation. A manager's process is
deterministic (assign, review, accept); the worker's output is not. The value is in
encoding the process, not guaranteeing the outcome.

---

## 3. Bash vs ash

| | Bash | Ash |
|---|---|---|
| Learning curve | 35 years of tutorials, Stack Overflow, LLM training data | 260-line getting-started doc |
| Determinism | Yes (shell commands) | Partially (LLM calls are stochastic) |
| State passing | `$?`, stdout pipes | `${stdout}`, `${stderr}`, `$?`, scoped variables |
| Error handling | `set -e`, `trap`, `||` | `try { } fail { } upto N` |
| Readability | Terse, arcane to non-engineers | Verbose, structured, few special characters |
| Audience | Engineers | ? |
| Install base | Every Unix system | Only where someone compiled it |

Ash's bet: readability and structure beat ubiquity and compatibility. A 100-step bash
script is incomprehensible; a 100-step ash script can be read top-to-bottom by someone
who doesn't write code. If that bet is wrong, ash has no reason to exist.

---

## 4. The real competition isn't bash — it's no visual tools

n8n, Zapier, Make, Retool — these are the tools that claim "automation for everyone."
They're visual, drag-and-drop, require zero syntax learning. Ash asks users to write
text files with numbered prefixes and YAML frontmatter.

But visual tools have a fundamental problem: they convert a canvas into JSON or YAML
under the hood. That JSON is unreadable garbage. The visual surface is the *only* way
to interact with the workflow. Lose the tool, lose the workflow.

Ash inverts this: the text file IS the canonical form. It's version-controllable,
diffable, searchable, and reviewable in any text editor. A visual builder that targets
ash would be a renderer, not a replacement — export nodes on a canvas as a `.md` file,
run with `ash`. The text survives the tool.

Moreover, text is LLM-native. Every agent produces text. Every workflow consumes text.
Generating markdown from a natural language description and executing it directly —
without format conversion — is a capability visual tools can't match without adding a
text serialization layer.

Counterpoint: visual tools already have millions of users who chose drag-and-drop over
scripting. The fact that text is "more correct" doesn't mean it's more adopted.

---

## 5. Agent abstraction: the half-step

Scripts still name agents directly:

```ash
do "fix this" with opencode
do "review this" with claude-code
```

This ties the workflow to specific tools. The user writing the workflow should think in
terms of *what needs to be done*, not *which binary to invoke*. The config file
(`ash-project.yaml`) maps names to binaries — but the names are still tool names, not
capability names.

A capability layer would look like:

```ash
do "fix this" with bug-fixer
do "review this" with reviewer
```

Where `ash-project.yaml` maps `bug-fixer → opencode`, `reviewer → claude-code`.
Switching providers is a config change. The script describes the workflow; the config
describes the execution environment.

Current state: not implemented. The `with` clause still maps to agent names, not
capabilities.

---

## 6. The actual user

Ash targets "everyone, not just IT engineers." But the domain expert who writes their
own workflow scripts is a rare crossover. More likely:

- **An engineer** codifies a workflow that a domain expert *approves*
- **An engineer** uses ash to orchestrate their own multi-agent CI tasks
- **An AI agent** generates ash scripts from natural language descriptions

If the real user is an engineer (or an LLM generating code for an engineer), then:

- The DSL can afford more expressiveness
- Readability matters for *review* (the domain expert approves), not *authoring*
- Agent abstraction matters for portability across machines, not accessibility

Open question: does the data support any of these user profiles, or are we guessing?

---

## 7. The "big players can absorb this" problem

| Player | Ash feature they could add in one sprint |
|--------|------------------------------------------|
| Claude Code | File-based task runner, `--retry` flag |
| OpenCode | Batch mode, piping prompts |
| Copilot | Already deeper IDE integration |
| Aider | Already has session restore |

Ash's moat: cross-agent orchestration. None of these tools can say "use opencode for
generation and Claude Code for review in one pipeline." This is a real gap — but it
only matters if users actually run multiple agents. Most pick one and stick with it.

Counterpoint: the gap will widen as more AI coding agents emerge. Each new agent is
another integration point ash handles in config. Each competing tool has to build that
integration themselves.

---

## 8. Where the real value is

Stripping away the DSL, three components are genuinely novel:

1. **The `.md` task file format** — Markdown with YAML frontmatter as an AI task
   descriptor. Universally readable, editor-agnostic, diff-friendly. No other agent
   tool has a task format a non-programmer can read.

2. **The directory tree walker** — Numbered files executed in order, prefix conflict
   detection, mixed `.md` and `.ash` entries. This is a build-system model for agent
   orchestration that no AI tool currently does.

3. **The agent adapter registry** — `ash-project.yaml` as a single config point for
   all agent CLI tools. Adding a new agent is a config change, not a recompile.
   `ash discover` auto-generates the config.

The expression language (variables, loops, functions, tries) is the weakest layer —
it's rewriting what bash does in a smaller, less tested grammar. If the directory
walker and task format could be invoked without the `.ash` scripting language, the
value proposition would be clearer.

---

## 9. The path not taken: markdown workflow engine

A separate tool that executes a single `.md` file by treating `##` headings as
sequential steps:

````markdown
---
agent: opencode
---

# Migration Guide

## 1. Research
Trace all files related to the login system.

## 2. Plan
Based on: ${stdout}
Create a step-by-step migration plan.

## 3. Implement
Execute the plan. Apply changes to each file.
````

No scripting language. No variables (beyond `${stdout}`). No control flow. No
directory structure. The document IS the program.

This targets the 80% case — straight-line procedures — with near-zero cognitive
load. The 20% case (loops, conditionals, parallel execution) stays in `.ash` scripts.

Design doc: [workflow-engine.md](workflow-engine.md)

---

## 10. Current defense summary

| Attack | Defense |
|--------|---------|
| "It's just bash with a parser" | Readability is the point — bash is write-only for non-engineers |
| "LLM calls are non-deterministic" | Same as human delegation — encode the process, not the outcome |
| "Visual tools already do this" | Text is canonical, diffable, LLM-native — visual tools convert to garbage text |
| "Big players can absorb this" | Cross-agent orchestration is the moat — no single tool can offer it |
| "The target user doesn't exist" | Unclear — need data. Engineer-authored, domain-expert-reviewed seems most plausible |
| "The expression language is redundant" | The task format + tree walker are the real value; the `.ash` language is the 20% escape hatch |

---

## Revisit triggers

Reread this document when:

- Considering adding a new syntax construct
- Debating whether a feature belongs in ash or a separate tool
- Wondering why adoption isn't faster
- Deciding whether to pivot toward the markdown workflow engine
- Feeling defensive about bash comparisons
