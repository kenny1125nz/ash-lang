# Toggle forms for `session` and `within`

## Background

Currently, ash supports `session { }` and `within <path> { }` as curly-brace block constructs. Inside `.ash` scripts, these work well — the user writes the entire block including its body in one go.

However, this block-only form is awkward in two contexts:

1. **REPL (interactive mode)** — a user wants to start a session, run commands one at a time, then end it. Typing the entire block in one go defeats the purpose of interactive exploration. A toggle form (`session begin` / `session end`) lets them open a session, run multiple `do` calls across several lines, and close it.

2. **Long scripts** — in `.ash` files, deeply nested blocks become hard to read. Toggle forms allow flattening: a `within begin` at the top of the file and `within end` at the bottom, with all the logic in between, without needing to indent everything.

## Intended Solution

Add `begin` / `end` toggle forms for `session` and `within`. Both forms (block and toggle) share the same evaluator state — the toggle simply sets/clears the same flags the block form sets on enter/exit.

### Syntax

**Session:**
```ash
session begin
do "task one" with opencode
do "task two" with opencode
session end
```

**Within (working directory):**
```ash
within begin "/project/src"
do "fix the login bug"
exec cargo build
within end
```

Block forms remain unchanged and compose with toggles:
```ash
session begin
  do "setup" with opencode
  within begin "/tmp"
    do "build in tmp"
  within end
session end
```

### Tokens

`begin` and `end` become reserved keywords to avoid ambiguity with variable/function names. They only trigger special parsing when immediately following `session` or `within`.

### Parser dispatch

```rust
"session" => {
    if peek_is_ident("begin")  → parse SessionToggle { active: true }
    if peek_is_ident("end")    → parse SessionToggle { active: false }
    else                       → parse SessionBlock { body }
}

"within" => {
    if peek_is_ident("begin")  → parse WithinToggle { active: true, path }
    if peek_is_ident("end")    → parse WithinToggle { active: false }
    else                       → parse DirBlock { dir, body }
}
```

### AST nodes

```rust
pub struct SessionToggle {
    pub pos: Pos,
    pub active: bool,      // true = begin, false = end
}

pub struct WithinToggle {
    pub pos: Pos,
    pub active: bool,      // true = begin, false = end
    pub path: Option<Box<Node>>,  // Some for begin, None for end
}
```

### Evaluator

- `eval_session_toggle`: if `active`, calls the same logic as entering a `session { }` block (increment `session_depth`, error if already > 0). If `!active`, decrements, error if already 0.
- `eval_within_toggle`: if `active`, resolves path, pushes current directory, sets new directory. If `!active`, pops and restores previous directory, error if none active.

## Acceptance Criteria

1. **`session begin` / `session end` toggle works in scripts**
   - `session begin` sets `session_depth` to 1, `session end` resets it
   - `do` calls between begin/end pass `--continue` to the agent

2. **`session begin` inside an open session is an error**
   - Same as nested `session { }` blocks — runtime error

3. **`session end` without an open session is an error**
   - Runtime error: "session end without matching begin"

4. **`within begin <path>` / `within end` toggle works in scripts**
   - Changes working directory for subsequent calls, restores on `within end`
   - `within end` without `within begin` is an error

5. **Toggles and blocks can be mixed**
   - `session begin` then `session end` then `session { ... }` works — state is independent per invocation

6. **Block forms are unchanged**
   - `session { ... }` and `within <path> { ... }` continue to work as before

7. **`begin` and `end` are valid as identifiers outside toggle context**
   - `begin = 1` or `end = 2` parses as variable assignment when not after `session`/`within`

## Implementation Hints

### Relevant project context

**Parser dispatch:**
- `parser.rs:136-170` — `parse_statement()` matches keyword strings in `tok.literal`. `session` and `within` already dispatch to their respective parse methods. Extend each to check the next token for `begin`/`end`.

**Tokenizer:**
- `token.rs:101-112` — keyword table. Add `"begin"` and `"end"` so they're recognized as `TkIdent` keywords. The parser distinguishes toggle usage via context (immediately after `session`/`within` tokens).

**AST:**
- `ast.rs` — add `SessionToggle { pos, active }` and `WithinToggle { pos, active, path }` structs, plus `Node::SessionToggle` and `Node::WithinToggle` variants.
- Add `pos()` match arms for both.

**Evaluator:**
- `eval/mod.rs` — add `eval_session_toggle()` and `eval_within_toggle()` methods. Dispatch them from `eval_statement()`.
- The existing `eval_session_block()` and `eval_dir_block()` (for `within`) contain the enter/exit logic — extract the inner increment/decrement or push/pop logic into reusable helpers, or duplicate it in the toggle evaluator.

**collect_agent_names:**
- `main.rs:9-92` — add `Node::SessionToggle` and `Node::WithinToggle` match arms (no-op since toggles don't contain agent calls).

### What not to touch

The engine layer, session flag propagation to drivers, and directory switching logic are correct — toggles just need to trigger the same evaluator state changes that blocks already do.
