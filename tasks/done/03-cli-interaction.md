# CLI REPL — interactive mode

## Background

Currently, when `ash` is invoked with no arguments and stdin is piped, it reads all input as a single script and executes it. This works for scripting but is awkward for exploration and ad-hoc use — users must write entire scripts even for a single `do` call or to test a variable.

A REPL (Read-Eval-Print Loop), like Python's interactive mode, would let users type ash statements one at a time and see results immediately.

## Intended Solution

When `ash` is invoked with no arguments and stdin is a terminal (TTY), enter REPL mode instead of reading a script:

```bash
$ ash
ash> NAME = "world"
ash> print "hello ${NAME}"
hello world
ash> do "explain closures" with opencode using sonnet
[agent output streams here...]
ash> exit
```

### Line accumulation for block constructs

Ash uses `{ }` blocks extensively (`if`, `for`, `while`, `fn`, `try`, `session`, `within`, `wait`). In a REPL, users shouldn't have to close the block on the same line — the REPL should keep reading lines until the block is complete.

**Brace counting.** After each line, count `{` and `}` (ignoring those inside strings and comments via a quick pass through the lexer). If there are more opens than closes, show a continuation prompt (`... `) and read the next line.

```bash
ash> if true {
...   print "inside if"
...   session {
...     do "task with session"
...   }
... }
inside if
task with session
```

Precedence of unfinished constructions:
1. Unclosed `{ }` blocks — always triggers continuation
2. Trailing `\` — manual line continuation (alternative for long single statements)

```bash
ash> do "explain closures in JavaScript \
... with examples" with opencode
```

**Ctrl-C cancels multi-line input.** Discard accumulated input, return to `ash> `.

### Multi-line prompts

| State | Prompt |
|-------|--------|
| New line | `ash> ` |
| Inside unclosed `{ }` | `... ` |
| After trailing `\` | `... ` |

### Session and working directory

The REPL benefits from two enhanced constructs (see `enhance-session-within.md`):

**Session toggle** — start and end a session across multiple lines without a block:

```bash
ash> session begin
ash> do "implement token types" with opencode
[agent runs within session]
ash> do "implement value system" with opencode
[agent reuses session context]
ash> session end
ash> do "one-shot task" with opencode
[agent runs without session]
```

**Within toggle** — change working directory across multiple lines without a block:

```bash
ash> within begin "/project/src"
ash> do "fix the login bug"
ash> exec cargo build
ash> within end
```

These toggle forms (`session begin/end`, `within begin/end`) and their block forms (`session { }`, `within <path> { }`) all work in the REPL identically to how they work in `.ash` scripts.

### REPL commands

Built-in dot-commands available in the REPL:

| Command | Description |
|---------|-------------|
| `exit` | Exit the REPL (same as ash `exit` statement) |
| `.help` | Print available commands and usage |
| `.clear` | Clear the current scope (reset all variables) |
| `.vars` | List all variables and their current values |

## Acceptance Criteria

1. **`ash` with no args and TTY enters REPL mode**
   - `isatty(stdin)` returns true → REPL, prints `ash> ` prompt
   - `isatty(stdin)` returns false → existing batch mode (reads all stdin as script)

2. **Each line is evaluated independently**
   - `print "one"` prints `one` immediately
   - `print "two"` on next line prints `two`

3. **Variables persist across lines**
   - `X = 10` followed by `print X` prints `10`

4. **Expression results are printed**
   - `2 + 2` prints `4`
   - Statements (`print`, `do`, `if`, etc.) do NOT print a result

5. **Brace-aware line accumulation**
   - Typing `if true {` shows continuation prompt `... `
   - Each subsequent line is accumulated until closing `}` matches
   - Nested blocks work: `session {` inside `if {` waits for both to close
   - The fully accumulated input is parsed and evaluated as one statement

6. **Trailing `\` continues lines**
   - `do "long prompt \` followed by Enter shows `... ` and accumulates the next line

7. **Ctrl-C cancels multi-line input**
   - Mid-block Ctrl-C discards accumulated input, returns to `ash> `

8. **Ctrl-D (EOF) exits cleanly**
   - Exits with code 0

9. **Error recovery — bad input doesn't crash the REPL**
   - Parse errors print the error and return to `ash> `
   - Eval errors print the error and return to `ash> `

10. **`session begin/end` and `within begin/end` work in REPL**
    - Same behavior as in `.ash` scripts (see `enhance-session-within.md`)

11. **Piped stdin still works**
    - `echo "print 42" | ash` executes as a batch script (no prompt printed)

## Implementation Hints

### Relevant project context

**Current stdin handling:**
- `main.rs:243-252` — when no positional arg, reads all stdin via `read_to_string()`, then parses and evaluates as a single script. This is the batch mode path.

**TTY detection:**
- Rust's `std::io::IsTerminal` trait (stabilized in Rust 1.70) detects if stdin is a terminal.

**REPL loop structure:**
- A new module `ash/src/repl.rs` with a `run_repl()` function that:
  - Prints `ash> ` prompt, reads a line with `std::io::stdin().read_line()`
  - Counts `{`/`}` with a minimal lexer pass (skip string/comment content)
  - If more opens than closes, show `... ` prompt, read next line, accumulate
  - Trailing `\` handling: strip the backslash and newline, continue reading
  - Ctrl-C handling: set a signal handler or catch the interrupt
  - Parses accumulated input with `parse_str()`, evaluates with `Evaluator`
  - Prints expression results, reports errors without exiting
  - Loops until Ctrl-D or `exit`

**Brace counting:**
- The simplest approach: pass the accumulated buffer through `lexer::tokenize()`. Count `TkLBrace` (`{`) and `TkRBrace` (`}`). If counts differ, continue reading.
- Edge cases: braces inside strings (`"{"`) and comments (`// {}`) must be ignored. The lexer already handles this — just filter the token kinds.

**Evaluator reuse:**
- `eval/mod.rs:72-100` — `Evaluator` holds scope, compact config, default agent/model. Create one `Evaluator` per REPL session so variables accumulate.
- The evaluator's `eval_statement()` dispatches all statement types including the new `SessionToggle` and `WithinToggle` from `enhance-session-within.md`.

**Dependencies:**
- The `enhance-session-within.md` task must be completed first — the REPL relies on `session begin/end` and `within begin/end` for interactive session/directory workflows.

### What not to touch

The parser, lexer, evaluator, and engine layers are correct — the REPL just needs to detect brace balance and call them in a loop.
