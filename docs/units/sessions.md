---
{"id": "sessions", "title": "Session Blocks"}
---

## Session Blocks

Keep multiple `do` calls in a shared agent session:

```ash
session {
  do "Implement the token types"
  do "Implement the value system"
  do "Refactor the AST"
}
```

A toggle form (`begin` / `end`) spans non-contiguous code without nesting:

```ash
session begin
do "Research approach"
do "Draft prototype"
session end

# ... intervening code ...

session begin
do "Finalize implementation"
session end
```

Mix toggle and block forms as long as they are not nested. `session begin` errs if a session is already active. `session end` errs without a matching `begin`.
