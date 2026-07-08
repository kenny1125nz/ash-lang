---
{"id": "control-flow", "title": "Control Flow"}
---

## Control Flow

All bodies use `{ }`:

### If / else

```ash
if $? == 0 {
  print "build passed"
} else if SCORE > 0.8 {
  print "good enough"
} else {
  print "fail"
  exit 1
}
```

### For loops

```ash
for FILE in FILES {
  exec eslint FILE
}
```

### While loops

```ash
while RETRIES < 3 {
  do "Fix remaining issues"
  RETRIES = RETRIES + 1
}
```

### Try blocks

Binary try — retries on failure, runs an optional fail block:

```ash
try {
  do "Deploy to staging"
} fail {
  print "deployment failed, rolling back"
} upto 3
```

Eval try — evaluates a condition after each attempt:

```ash
try {
  do "Generate report"
} evaluate with {
  SCORE >= 85
} accept {
  print "quality threshold met"
} partial {
  print "attempt ${ATTEMPT} below threshold, retrying"
} fail {
  print "unexpected error"
} upto 5
```

| Clause | Runs when |
|--------|-----------|
| `accept` | Evaluator returns truthy / `$?` == 0 |
| `partial` | Evaluator returns falsy / `$?` == 1 |
| `fail` | Body errors or evaluator `$?` >= 2 |
| `upto N` | Maximum retry count |

### Evaluate blocks

`evaluate` is a top-level statement that runs a body, evaluates the result using an external evaluator (agent, function, or command), and retries until a numeric score threshold is met or attempts are exhausted.

```ash
evaluate {
  do "Write a blog post about Rust"
} by @"reviewer.md" with opencode
   accept by 85
   upto 5
```

The `by` clause selects the evaluator — one of three forms:

| Evaluator | Syntax | Description |
|-----------|--------|-------------|
| Agent | `@"prompt.md" [with <agent>] [using <model>]` | An agent reviews the output and produces a score |
| Function | `fn_name(args...)` | A user-defined function returns the score |
| Command | `exec "<command>"` | A shell command outputs the score |

The agent evaluator receives language-injected scoring instructions so prompts don't need to specify the output format. The agent must output:

```
SCORE: <0-100 integer>
FINDINGS:
<actionable improvement feedback>
```

#### Per-iteration variables

Within each iteration's body, these variables are automatically set:

| Variable | Type | Description |
|----------|------|-------------|
| `$_attempt` | Int | Current attempt number, 1-indexed |
| `$_max_attempts` | Int | Total allowed attempts |
| `$_feedback` | String | Findings from the previous iteration (empty on attempt 1) |

#### Post-loop variables

After the evaluate block completes, these variables are set in the parent scope:

| Variable | Type | On acceptance | On exhaustion |
|----------|------|---------------|---------------|
| `$score` | Int | The accepted score (>= threshold) | The last attempted score |
| `$accepted` | Bool | `true` | `false` |
| `$_evaluator_output` | String | Full evaluator stdout from the accepting run | Full evaluator stdout from the last run |

#### Outcomes

- **Acceptance** — score >= threshold on any attempt. The loop terminates immediately. Side effects from the accepting iteration are preserved.
- **Exhaustion** — all attempts complete without reaching the threshold. The last iteration's side effects are preserved for inspection.
- **Error** — body crash, evaluator failure, or score extraction failure propagates immediately. No silent retry.

#### Example: branching on acceptance

```ash
evaluate {
  do "Write documentation"
} by score_fn()
   accept by 80
   upto 5

if $accepted {
  print "Approved with score $score"
  exec deploy docs/
} else {
  print "Rejected (score $score), check output"
  exit 1
}
```
