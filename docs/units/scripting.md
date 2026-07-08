---
{"id": "scripting", "title": "Scripting with .ash Files"}
---

## Scripting with .ash Files

When you need more than one-shot prompts — chaining, conditionals, parallelism — write an `.ash` script:

```ash
#!opencode
do "Write a hello world program in Rust"
print stdout
```

```bash
ash hello.ash
```

### Agent shebang

Declare the agent with a shebang:

```ash
#!opencode:1.0

do "Review src/" with opencode
```

### Default agent

Set the default agent for all subsequent `do` calls with `use`:

```ash
use opencode
do "Review src/"                # uses opencode

use claude-code
do "Refactor the implementation" # uses claude-code
```

The agent can still be overridden per-call with `do "..." with <agent>`.

### Language overview

```ash
use opencode                             # set default agent

do "Review src/"                         # call an agent

fn rollback(FILE) {                      # functions
  exec git restore "${FILE}"
  do "Summarize what has been done"
}

for FILE in FILES {                      # loops, conditionals, retry
  try {
    do "Fix bugs in ${FILE}"
  } fail {
    print "failed on ${FILE}"
  } upto 3
}
```
