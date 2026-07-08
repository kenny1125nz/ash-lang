---
{"id": "repl", "title": "REPL"}
---

## REPL

Run `ash` with no arguments to enter interactive mode:

```bash
$ ash
ash> NAME = "world"
ash> print "hello ${NAME}"
hello world
```

Commands: `.help`, `.clear`, `.vars`, `.exit`. Up/down arrows navigate history. Multi-line blocks (`if`, `for`, `session`) auto-detect continuation.
