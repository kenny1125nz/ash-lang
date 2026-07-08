---
{"id": "parallel", "title": "Background and Parallel Execution"}
---

## Background and Parallel Execution

Run a statement in the background with `&`:

```ash
do "Long running analysis" &
print "main continues immediately"
wait
```

Run multiple statements in parallel with `wait { }`:

```ash
wait {
  do "Train model A"
  do "Train model B"
}
print "both models done"
```
