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

### Language overview

```ash
do "Review src/" with opencode      # call an agent

fn rollback(FILE) {                  # functions
  exec git restore "${FILE}"
  do "Summarize what has been done"
}

for FILE in FILES {                  # loops, conditionals, retry
  try {
    do "Fix bugs in ${FILE}"
  } fail {
    print "failed on ${FILE}"
  } upto 3
}
```
