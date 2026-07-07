# @ash-lang/cli

Ash is a task runner for AI agents — a scripting language that composes AI agents into automated workflows. Drop markdown files in a folder, number them, and run. When you need loops, retries, or conditional logic — add an `.ash` script. Start simple. Grow as needed.

## Install

```sh
npm i -g @ash-lang/cli
```

On `npm install`, the correct platform binary is downloaded from GitHub Releases automatically.

## Usage

Replace `opencode` with the agent you have installed — opencode, claude-code, codex, gemini-cli, or kimi (echo is the built-in default when none is found, and these are the agents with built-in support):

```sh
ash --agent opencode ./tasks
```

Point Ash at a directory of numbered markdown tasks:

```
tasks/
├── 01-research.md
├── 02-implement/
│   ├── 00-init.ash
│   ├── 01-auth.md
│   ├── 02-api.md
│   └── 03-tests.md
├── 03-review.md
└── 04-deploy.md
```

```sh
ash ./tasks
```

Each `.md` file is one task sent to an AI agent. Files run in sorted order. When you need more than a single prompt — loops, retries, conditionals — use an `.ash` script:

```sh
ash path/to/workflow.ash
```

Or via `npx`:

```sh
npx @ash-lang/cli ./tasks
```

For `.ash` scripts, declare the agent with a shebang:

```ash
#!opencode:1.0

do "Review src/" with opencode
```

For `.md` tasks, set the agent in YAML frontmatter:

```markdown
---
agent: opencode
---

# Task Title

The prompt content goes here...
```

## Language

```ash
do "Review src/" with opencode      # call an agent

fn rollback(FILE) {                  # functions, composition
  exec git restore "${FILE}"
  do "Summarize what has been done for ${FILE} and save it to summary.md"
  exec git checkout -b "failure/${FILE}"
  exec git add summary.md
  exec git commit -m "fix failed for ${FILE}"
}

for FILE in FILES {                  # loop, conditionals, retry
  try {
    do "Fix bugs in ${FILE}"
  } fail {
    rollback(FILE)
  } upto 3
}
```

## Manual binary download

If the automatic download fails, grab the binary from [GitHub Releases](https://github.com/kenny1125nz/ash-lang/releases/latest) and place it alongside `ash.js` in the package directory.
