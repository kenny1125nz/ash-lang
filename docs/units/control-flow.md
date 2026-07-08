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
