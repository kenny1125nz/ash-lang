# Task Definition: Improve Evaluation 

## Background

### The Problem

Ash scripts that use agentic evaluation loops rely on a fragile file-based mechanism to extract scores and check thresholds. The current refinery workflow (`refinery/refine-task.ash`) exemplifies this:

1. A produce agent writes `tmp/task-definition_${IDX}.md` — the convention is buried in prompt text, invisible in the script.
2. An assess agent is instructed (via prompt) to create `tmp/score.<N>` as an empty file whose filename encodes the numeric score — the convention is buried in prompt text, invisible in the script.
3. The script runs `ls tmp/score.* | head -1 | grep -oE '[0-9]+$'` to extract the score from the filename.
4. The script runs `test '${SCORE}' -ge ${THRESHOLD}` and checks the exit code.

This mechanism has three compounding failure modes, none of which are shallow shell bugs:

1. **Implicit side-effect contract.** The entire evaluation loop depends on conventions that exist only in plain-text prompt files. If an assess prompt creates `out/tmp/score.<N>` instead of `tmp/score.<N>`, the `ls` + `grep` pipeline silently finds nothing. There is no enforcement, no validation, and no error — just an infinite loop.

2. **No iteration context in the language.** Variables like `IDX` and `PREV_IDX` exist only because prompt files contain `${IDX}` and `${PREV_IDX}` interpolation markers. The values are set manually at the top of each iteration. If a new assess prompt is written without these markers, the scoring instructions produce garbage. The language offers no built-in understanding of "this is iteration N of M" — the user threads these manually through both the script body and every prompt file.

3. **Non-obvious truthiness and exit-code semantics.** The script uses `exec "test ..."` and then compares `$?` — an idiom that happens to work because the evaluation infrastructure sets `?` from the exec's exit code, but is opaque and easy to get wrong across different scripts.

In summary: the loop can work when all implicit conventions align perfectly, but even small deviations cause silent failure — not errors. The root cause is the absence of a score abstraction and evaluation-loop semantics in the language itself.

### Why This Matters

Evaluation is the mechanism that enables ash scripts to produce quality output through iterative refinement. Without a reliable evaluation primitive, scripts that use agentic evaluation either succeed by luck, loop forever, or are abandoned. The language needs evaluation as a first-class construct — not a fragile assembly of shell commands.

### Current State of the Language

- **`EvalTry`** exists: `try { ... } evaluate with { ... } accept/partial/fail { ... } upto N`. It provides a retry loop with 3-way categorical routing (accept/partial/fail) based on exit codes or truthiness. Its evaluator is inline ash code — it cannot invoke an external agent, function, or command to produce a score. Variables are set once in a scope that persists across iterations.
- **`BinaryTry`** exists: `try { ... } fail { ... } upto N`. Simpler retry loop, body-only, retries on failure or falsy return.
- **`evaluate`** is a reserved keyword but has no top-level statement parse path — it is only used as a suffix inside `try`.
- **No numeric score concept exists** in the value system. Scores are represented as integers but there is no scale convention, no threshold comparison built into the language, and no structured evaluation output format.
- **Prompt files capture iteration variables** via `${VAR}` interpolation (the file-path reader reads a `@"file.md"` and resolves interpolations against the current scope). This is how `IDX` and `PREV_IDX` reach prompt files today, but the language provides no built-in attempt counter.
- **`do` statements** support `with <agent> using <model> in <dir> compact <cfg>` clauses that specify how an agent call is dispatched. These same clause patterns are natural candidates for the evaluator clause of a new `evaluate` statement.

### Goal of This Task

**This is a high-level design task, not an implementation plan.** The objective is to identify the core abstractions, make the key design decisions, evaluate trade-offs, and produce a design that can be broken down into implementation tasks. The design should resolve the _what_ and the _why_ at a level where each decision is justified.

---

## Intended Solution

### Core Design: A First-Class `evaluate` Statement

The central design decision is to introduce a top-level `evaluate` statement that encapsulates the entire evaluation loop as a language construct. The language — not the user — owns the retry loop, threshold comparison, iteration context, and feedback propagation.

The targeted syntax reads:

```
evaluate {
    <body statements>
} by <evaluator-clause>
   accept by <threshold-expr>
   upto <max-attempts-expr>
```

Where `<evaluator-clause>` is one of three forms:

- **Agent evaluator:** `@"prompt.md" with <agent> using <model>`
- **Function evaluator:** `fn_name(args...)`
- **Command evaluator:** `exec "<command>"`

#### Why This Syntax

The original requirement's design principle states that ash constructs should "be as natural as possible so people without coding experience can understand." This syntax follows that principle:

- **`evaluate { ... } by ...`** reads like English: "evaluate this work by this reviewer." The `by` preposition naturally introduces the evaluator, mirroring how humans think about peer review.
- **`accept by <N>`** reads like "accept it by score 85" — the threshold is the acceptance gate. Using `by` after `accept` is a deliberate choice: in English we say "accepted by a score of 85," not "accepted if score 85." This avoids introducing `if` or `when` which already have strong semantics in ash.
- **The body block uses `{ }`** consistently with all other ash compound statements (`if`, `for`, `while`, `try`). No new bracketing conventions.
- **Three evaluator forms** cover the three execution models ash already understands: agents (`do`), functions (`fn_name()`), and commands (`exec`). The user's mental model transfers directly — the same patterns they use elsewhere in ash work in evaluator position.

#### Why Only Three Evaluator Forms, Not Four

The original requirement's ideal target state shows three forms. A fourth — inline ash code as an evaluator — is not included because:
- **Function evaluators cover this use case.** An inline evaluation can be written as a function and invoked by name. This keeps the grammar simple (no `{ }` block after `by`, which would be ambiguous with the body block) and encourages reusable evaluation logic.
- **`EvalTry` already serves inline evaluation.** For inline ash code evaluators, `try { ... } evaluate with { ... }` is the right existing construct. Adding a fourth evaluator form to `evaluate` would blur the boundary between the two constructs without adding capability.

#### Relationship to Existing Constructs

The `evaluate` statement is a new construct that coexists with `EvalTry` and `BinaryTry`. They serve different use cases:

| Aspect                    | `EvalTry` (existing)                     | `evaluate` (new)                                                    |
| ------------------------- | ---------------------------------------- | ------------------------------------------------------------------- |
| Evaluator                 | Inline ash code                          | External agent / function / command                                 |
| Result model              | 3-way categorical (accept/partial/fail)  | Numeric score against threshold                                     |
| Score scale               | N/A                                      | 0–100 (fixed)                                                       |
| Feedback between attempts | Captured stdout via `$report`            | Extracted `FINDINGS:` text from evaluator output                    |
| Iteration counter         | None built-in                            | Language-provided (`$_attempt`, `$_max_attempts`)                   |
| Acceptance condition      | Exit code 0 or truthy expression         | Score >= threshold                                                  |
| Scope model               | One scope persists across all iterations | Fresh scope per iteration; working tree reverted between iterations |
| Return value              | Nil                                      | Final score as integer                                              |

Both `BinaryTry` and `EvalTry` are preserved unchanged. `EvalTry` is the right tool when evaluation logic is simple (a checksum, a grep, a quick inline test). `evaluate` is the right tool when evaluation requires an external agent, a function, or a script — especially when the evaluator needs to see the full context of what changed.

---

### Evaluator Contract: How Scores Are Produced

All three evaluator types produce a **numeric score on a fixed 0–100 scale**. The scale is not configurable — a single, predictable scale means scripts composed together can share score conventions without translation.

#### Agent Evaluator

An agentic evaluator receives a prompt that combines:
1. The user's evaluator prompt file (e.g., `@"panel/reviewer.md"`)
2. Language-injected scoring instructions (prepended in memory, not written to disk)
3. Change context — a diff showing what the body block changed (see Git Integration below)

The language-injected instructions ask the agent to output a structured format:

```
SCORE: <0-100 integer>
FINDINGS:
<actionable improvement feedback>
```

The agent's output is parsed. A line matching `SCORE:` followed by an integer is extracted as the score. The text after `FINDINGS:` is extracted as feedback for the next iteration's produce step.

**Design decision — no "first integer" fallback for agent output.** Agentic output — especially from LLM-based agents — frequently contains integers unrelated to scoring: step counts, line numbers, token counts, character counts, cost estimates. A fallback to "first standalone integer line" would silently pick up the wrong number. An evaluator that fails to follow the output format should fail loudly so it gets fixed, not produce a random score.

**Design decision — evaluator prompt augmentation vs. user prompts.** The language injects the scoring format instructions rather than requiring the user to write them. This makes the scoring contract visible at the language level, not buried in prompt text. However, it creates a known interaction: if the user's evaluator prompt already contains scoring instructions (e.g., "Output your score as a number between 0 and 100"), the injected instructions may conflict. The severity depends on the content:
- **Lenient conflict** (mild): two sets of scoring instructions appear, the LLM sees both and follows the later one. This produces a valid score but with extra noise.
- **Hard conflict**: the user's prompt asks for `Score: 85/100` (with `/100` suffix or different formatting) while the injected instructions ask for `SCORE: 85`. The parser rejects the user's format as unparseable, surfacing an error.

This trade-off favors surfacing problems: a misaligned evaluator prompt fails immediately rather than silently producing the wrong score. When users write evaluator prompts for the `evaluate` statement, they should omit their own scoring format instructions and let the language provide them. A guideline should document this.

#### Function Evaluator

A function evaluator's **return value** is the score:
- `Int` values are used directly.
- `Float` values are rounded to the nearest integer.
- Any other type errors immediately.

No output parsing is needed — the function returns a typed value through ash's normal function-return mechanism. The function's stdout is captured as the evaluator output (available as `$_evaluator_output`).

#### Command Evaluator

A command evaluator runs a shell command. Stdout is parsed for the score. The parsing strategy is different from agent evaluators because command stdout is typically programmatic, not verbose natural language:

- First attempt: find a line matching `SCORE:` followed by an integer (same pattern as agent evaluator). This allows commands to emit structured output when they choose to.
- Fallback: if no `SCORE:` line is found, parse the first standalone integer line. This handles simple commands like `python eval.py` that just print a number.
- If neither works, error immediately.

**Design decision — exit codes are not scores.** Unix exit codes encode process status categories (0=success, 1=general error, 130=SIGINT, 137=SIGKILL). An exit code of 137 does not mean "score 137" — it means the evaluator was killed. Command exit codes indicate runtime health, not evaluation quality. These domains should remain separate.

#### Score Validation

After extraction, the score is validated against [0, 100] inclusive. Values outside this range produce an error immediately — no clamping, no silent adjustment. An out-of-range score indicates an evaluator bug and should be fixed.

---

### Scope Model: Fresh Scope Per Iteration, Working Tree Reverted

Each iteration of the evaluate loop pushes a new scope and pops it before the next iteration begins. This is distinct from `EvalTry`, which pushes one scope and keeps it across all iterations.

**Working tree state is reverted between iterations.** Each iteration starts from the state of the working tree as it was before the evaluate block began execution. Side effects from a previous iteration's body block (files created, modified, or deleted) do NOT leak into the next iteration. This makes each iteration a fresh attempt from the same baseline, rather than a cumulative patch on top of potentially flawed previous output.

If the user wants cumulative refinement where iteration N builds on iteration N-1's output, they model it explicitly by having the body block read previous output files — the evaluate statement provides the `FINDINGS:` feedback as the bridge between iterations, not raw filesystem state.

**Phase 1 limitation:** Full implementation of working tree reversion requires git integration (phase 2, see Git Integration below). In phase 1, program variable scope is properly isolated per iteration, but filesystem isolation may be best-effort. This is acknowledged as an implementation gap to be closed in the git sandboxing follow-up.

#### Variables Set in Each Iteration's Scope

Within each iteration's scope, the language sets these variables automatically:

| Variable             | Type    | Value                                                                               |
| -------------------- | ------- | ----------------------------------------------------------------------------------- |
| `$_attempt`          | Integer | Current attempt number, 1-indexed                                                   |
| `$_max_attempts`     | Integer | Total allowed attempts                                                              |
| `$_feedback`         | String  | Evaluation findings from the previous iteration (empty string on iteration 1)       |
| `$_evaluator_output` | String  | Full stdout from the evaluator (set **after** the evaluator runs; undefined before) |

Body statements can reference these via standard `${VAR}` interpolation in prompt files and string literals. No special injection machinery is needed — the existing scope system handles it.

The underscore prefix convention (`$_attempt`, not `attempt`) signals that these are language-managed variables, not user-defined. This prevents collision with user variables named `attempt`, `feedback`, or `score` and makes their origin clear when reading scripts.

#### Variables Set in the Parent Scope After Completion

After the loop completes (acceptance or exhaustion), the following variables are written to the parent scope:

| Variable             | Type    | Value on acceptance                    | Value on exhaustion                    |
| -------------------- | ------- | -------------------------------------- | -------------------------------------- |
| `$score`             | Integer | The accepted score (>= threshold)      | The last attempted score (< threshold) |
| `$accepted`          | Boolean | `true`                                 | `false`                                |
| `$_evaluator_output` | String  | Full stdout of the accepting evaluator | Full stdout of the last evaluator      |

Code after the evaluate block can inspect these to decide what to do next — branch on `$accepted`, use `$score` in expressions, or log `$_evaluator_output`.

**Design decision — `$score` and `$accepted` use no underscore prefix.** Unlike the per-iteration variables, the output variables are explicitly part of the evaluate statement's public contract — they are the mechanism by which scripts interact with the evaluate result. Reusing common names like `$score` (vs. `$_eval_score`) reads naturally: `if $accepted { print "Passed with score $score" }`. Users should be aware that `$score` in the parent scope is overwritten by the evaluate statement when the loop completes, but not during the loop's iterations.

#### Nested Evaluate Blocks

Each evaluate statement manages its own iteration scopes independently. An inner evaluate writes its final variables to its surrounding scope (which is the outer evaluate's current iteration scope). Scope nesting naturally isolates iteration variables between nested loops. The per-iteration working tree reversion applies to the innermost evaluate block — outer evaluate iterations see the compound effect of each complete inner evaluate run.

---

### `FINDINGS:` Parsing Specification

The `FINDINGS:` delimiter is the mechanism that extracts actionable improvement feedback from evaluator output for the next iteration. The following parsing rules resolve edge cases:

1. **Case sensitivity:** Case-insensitive. `FINDINGS:`, `Findings:`, and `findings:` are all recognized. (Rationale: LLM output varies in capitalization; case sensitivity here would cause silent extraction failures for no benefit. Unlike `SCORE:` which must be precise to avoid false positives, `FINDINGS:` content is human-readable feedback — false negatives are worse than false positives.)

2. **Line anchoring:** `FINDINGS:` must appear at the start of a line (after optional whitespace). Mid-line occurrences like `see FINDINGS: below` are not recognized. (Rationale: inline usage is unlikely in LLM output and matching it would cause confusion with partial text matches.)

3. **Multiple occurrences:** The first occurrence wins. If evaluator output contains multiple `FINDINGS:` sections, only the first one is extracted. (Rationale: if the evaluator generated multiple findings sections, the first should be the primary one; extracting all would produce garbled concatenation.)

4. **Partial matches rejected:** `KEY_FINDINGS:`, `FINDINGS_SUMMARY:`, `FINDINGS_APPENDIX:` and similar compound words are NOT matched. Only standalone `FINDINGS:` (case-insensitive, word boundary) is recognized. (Rationale: partial matching would silently capture the wrong section.)

5. **End delimiter:** The findings text extends to the end of the evaluator output. There is no explicit end marker — everything after the `FINDINGS:` line up to the end of stdout is captured as findings. (Rationale: evaluator output is structured as `SCORE:` header, then `FINDINGS:` header, then body. The `FINDINGS:` section is the terminal section of the output format. Adding an end marker would require the language-injected format instructions to specify one, increasing the prompt injection surface.)

#### Why `FINDINGS:` Extraction Instead of Full Evaluator Output

Passing the full evaluator output as feedback to the produce agent would include: scoring instructions, the score value, `git diff` output (potentially thousands of lines), and the evaluator's chain-of-thought reasoning. Passing all of this overwhelms the produce agent with noise. The `FINDINGS:` delimiter identifies the signal within the noise — the actionable improvement directions that the produce agent should address.

#### When `FINDINGS:` Is Absent

If no `FINDINGS:` section is found in the evaluator output, the full evaluator output is used as fallback feedback. This is a degradation (the produce agent receives noise) but not an error — an evaluator that produces unstructured output still provides some context that may help. The produce agent should be instructed to focus on specific actionable items and ignore boilerplate.

---

### Return Value and Truthiness

The evaluate statement's return value is the final score as an integer (`Value::Int`). This is true regardless of whether the outcome is acceptance or exhaustion.

**The acceptance indicator is `$accepted`, not the return value's truthiness.** `Value::Int(0)` is falsy in ash, so a score of 0 on exhaustion would make `if evaluate { ... }` falsy — but a score of 0 on acceptance would produce the same falsy result despite being a "pass." This is confusing and error-prone. The correct mechanism for branching on acceptance vs. exhaustion is to inspect `$accepted`:

```
evaluate { ... } by @"reviewer.md" with <agent> accept by 85 upto 5
if $accepted {
    print "Passed with score $score"
} else {
    print "Failed with score $score"
}
```

Returning the score as the statement's value serves composability — the score can be used in expressions (e.g., `total = total + evaluate { ... }`). It is NOT intended as an acceptance check mechanism. The `$accepted` variable is the canonical indicator.

---

### `upto` Semantics

`upto N` means **N total attempts** (not N retries), matching the existing semantics of `BinaryTry` and `EvalTry`. `upto 3` produces at most 3 iterations numbered 1 through 3. This consistency avoids off-by-one confusion across construct types.

---

### Three Outcomes: Acceptance, Exhaustion, Error

1. **Acceptance** — score >= threshold on any attempt. The loop terminates immediately. The accepted score, evaluator output, and `$accepted = true` are written to the parent scope. Body side effects from the accepting iteration are preserved on disk (they produced the accepted result).

2. **Exhaustion** — all N attempts complete without reaching the threshold. The last score, last evaluator output, and `$accepted = false` are written to the parent scope. Body side effects from the final attempt are preserved on disk (the user may want to inspect what was produced despite the score being below threshold). No rollback occurs on exhaustion — the user can decide what to do with the last attempt's output.

3. **Error** — body execution crashes, evaluator invocation fails, or score extraction fails. The error propagates immediately, surfacing the problem. There is no silent retry.

---

### Error Handling: Surface, Don't Hide

The design follows a strict "Surface, Don't Hide" discipline. The following are **errors that propagate immediately** — the evaluate loop aborts, no silent retry is attempted:

- **Body execution failure** — any statement in the body block crashes. A broken produce step won't fix itself on retry.
- **Evaluator invocation failure** — agent engine unreachable, command not found, function not defined. These are not transient; retrying won't fix them.
- **Score extraction failure** — no parseable `SCORE:` line in agent output, command stdout not an integer, function returns wrong type. A broken evaluator should not loop silently.
- **Score out of range** — values outside [0, 100]. Indicates an evaluator bug.

This is different from `EvalTry`, which catches body errors and retries. The `evaluate` statement assumes that failure in the produce step indicates a genuine problem, not a transient issue — agents don't "crash" intermittently; they produce output or fail consistently. If the user wants error-retry behavior, they can wrap the evaluate block in a `BinaryTry`.

---

### Git Integration: Two-Phase Approach

The original requirement envisions git-based sandboxing — branch creation, change tracking, commit-on-acceptance, merge-back — as a way to isolate the evaluate loop's effects and provide change context. This is a complementary capability designed in two phases:

#### Phase 1 (this task): Read-Only Change Visibility

The evaluator (agent type only) receives a diff of unstaged working-tree changes as part of its prompt. This gives the evaluator context about what the body block changed without requiring any git operations:
- Run `git diff` (unstaged changes only)
- Exclude `.ash/` directories — this is the runtime metadata directory used by the ash tool itself (for config, telemetry, session state). Changes under `.ash/` are artifacts of ash's own operation, not part of the evaluation target.
- Append the diff to the evaluator's prompt as change context

If `git diff` fails (not a git repo, git not installed), evaluation proceeds without diff context — graceful degradation, not an error. The evaluator still has the body's output and task context; it just can't inspect exact line-level changes.

**Why `git diff` for the evaluator only, not the produce step.** The produce agent's job is to create or modify work based on a task definition and prior feedback. Adding `git diff` to the produce agent's prompt would overwhelm it with the same information the evaluator uses to judge quality — the produce agent should focus on the task, not on analyzing its own diffs. If the produce agent needs to understand what changed between iterations, the `$_feedback` variable from the evaluator is the appropriate channel — the evaluator describes what changed and what needs improvement.

#### Phase 2 (separate task): Git Sandboxing

The full sandboxing approach — branch-per-evaluate, commit artifacts, merge on acceptance, branch cleanup on exhaustion — is a significant feature that interacts with:
- Repository state management (switching branches, handling dirty working trees)
- Commit message conventions
- Conflict resolution when merging back
- Nested evaluate loops (branch stacking)
- Per-iteration working tree reversion (providing the mechanism for the scope model's isolation guarantee)

This is designed as a separate feature because:
1. It has independent value — git sandboxing could be useful even without the evaluate statement (e.g., for general script isolation).
2. It introduces complexity that would obscure the core evaluate design.
3. The original requirement itself calls for further breakdown.

The phase 1 read-only `git diff` provides immediate value while full sandboxing is designed separately.

---

### Open Design Questions

These are areas where the design intentionally does not prescribe a solution — they need analysis in follow-up tasks:

1. **Evaluator timeout** — agents can run for minutes or hours. If an evaluator hangs, the evaluate loop blocks indefinitely. Should there be a timeout clause (`upto 5 minute`)? How does timeout interact with retry — is a timed-out evaluation treated as a failed attempt or propagated as an error?

2. **Large diffs and agent context windows** — a `git diff` for a large code change could exceed the agent's context window. Should there be a size limit with truncation? A warning when the diff is large?

3. **Interaction with `compact` subagents** — the `do ... compact <cfg>` mechanism alters how agents process context. If a body step uses compact mode, the evaluate cycle still works but the produce agent may have changed its behavior.

4. **Interaction with `session` blocks** — sessions carry state across agent calls. If an evaluate body runs inside a session, the evaluator call within the same evaluate statement may or may not be part of that session — this depends on whether the evaluator runs in the session context or outside it.

5. **Multi-step body rollback** — if a body block has multiple steps and step 3 crashes, steps 1 and 2 have already changed files on disk. Should the evaluate loop roll back these changes before retrying? This is a general retry-scope problem that also affects `EvalTry` and `BinaryTry`. Phase 2 git sandboxing would provide the mechanism for this.

---

### What This Task Covers vs. What Needs Further Breakdown

**This task resolves the core design:**
- The `evaluate` statement concept and syntax
- Three evaluator types (agent, function, command) with justification for excluding a fourth
- Fixed 0–100 scoring scale
- Evaluator output format contract (`SCORE:` / `FINDINGS:`)
- Complete `FINDINGS:` parsing specification (case sensitivity, line anchoring, multiple occurrences, partial matches, end delimiter)
- Named output variables (`$score`, `$accepted`, `$_evaluator_output`) and per-iteration variables (`$_attempt`, `$_max_attempts`, `$_feedback`)
- Per-iteration scope model with working tree reversion semantics
- `upto` semantics matching existing constructs
- Acceptance, exhaustion, and error outcomes with explicit side-effect semantics
- Return value vs. acceptance indicator distinction
- Error handling discipline (surface, don't hide)
- Read-only `git diff` for evaluator context with `.ash/` exclusion rationale
- Evaluator prompt augmentation conflict analysis

**Follow-up tasks (in order):**
1. Implementation: parser, AST node, evaluator loop → concrete working feature
2. Implementation: git sandboxing (branch, commit, merge, cleanup, working tree reversion)
3. Analysis: evaluator timeout, context window management, compact/session interaction
4. Documentation: evaluator prompt writing guide (how to write a compatible evaluator prompt, format contract examples)

---

## Acceptance Criteria

1. The design defines a `evaluate` statement that encapsulates an evaluation loop with a body block, an evaluator clause, a threshold, and a maximum attempt count.

2. The design supports three evaluator types — agent (prompt file + agent + model), function (existing fn call), and command (shell exec) — with clear specification of how each produces a numeric score on a 0–100 scale. The exclusion of a fourth inline-code form is justified.

3. The evaluator output format contract (`SCORE:` / `FINDINGS:`) is specified and the rationale for each parser behavior (strict single-integer extraction for agents, structured-then-fallback for commands, direct return for functions) is stated.

4. The complete `FINDINGS:` parsing specification addresses: case insensitivity, line-start anchoring, first-occurrence precedence, partial match rejection, and end-of-output delimiters. Each rule includes its rationale.

5. The per-iteration scope model is specified: fresh scope each iteration, working tree reverted between iterations, all language-provided variables (`$_attempt`, `$_max_attempts`, `$_feedback`, `$_evaluator_output`) named with their types and values.

6. The parent-scope output variables (`$score`, `$accepted`, `$_evaluator_output`) are named, typed, and their values for both acceptance and exhaustion outcomes are specified.

7. The three loop outcomes (acceptance, exhaustion, error) are defined with clear semantics for what happens in each case: what is returned, what variables are set in the parent scope, and what side effects are preserved or rolled back.

8. The return value (final score as `Value::Int`) is distinguished from the acceptance indicator (`$accepted` as `Value::Bool`). The truthiness pitfall is explicitly called out with a usage example.

9. The `upto N` semantics match existing `BinaryTry` and `EvalTry` (N total attempts, not retries).

10. The design explicitly states which failure modes propagate as errors immediately (body crash, evaluator invocation failure, score extraction failure, out-of-range score) and justifies why silent retry is not appropriate for each.

11. The read-only `git diff` integration for agent evaluators is specified: what is diffed (unstaged changes), what is excluded (`.ash/` directories with rationale), and what happens when git is unavailable (graceful degradation).

12. The feedback propagation mechanism (`FINDINGS:` extraction) is specified with its rationale — why extraction is preferred over passing full evaluator output, and what happens when `FINDINGS:` is absent (full output as fallback).

13. The evaluator prompt augmentation conflict is analyzed and the trade-off stated: language-provided format instructions may conflict with user prompts, users should omit their own format instructions, misaligned prompts fail loudly.

14. The relationship between the new `evaluate` statement and existing `EvalTry`/`BinaryTry` is clarified — they coexist, serving different use cases, with a comparison table.

15. The full git sandboxing feature (branch/commit/merge/cleanup, working tree reversion) is identified as a separate follow-up task with its justification and list of interactions it must address.

16. Open design questions are listed with their scope — timeout, context window size, compact/session interaction, multi-step rollback.

17. A follow-up task breakdown is provided in dependency order.

18. The design does not prescribe implementation details — no Rust file paths, no AST variant names, no parser method names, no regex patterns, no line-number references, no enum variant names. Implementation decisions belong to the implementation task. Implementation Hints provide architectural context, not implementation recipes.

19. The scope semantic boundary between program state (variables) and disk state (filesystem side effects) is explicitly addressed: per-iteration scope isolation for variables, working tree reversion for filesystem state, with acknowledgement of the phase 1 implementation gap for filesystem isolation.

20. All findings from the v6 assessment report are addressed.

---

## Implementation Hints

### Architectural Context

The evaluate statement touches the same architectural layers as every other ash language construct. Understanding the existing patterns at each layer provides the context needed to fit this feature in naturally.

### Language Layer

The syntax for the evaluate statement combines patterns from two existing constructs:

- **Compound statement structure** from `try` — a keyword followed by `{ body }` followed by optional suffix clauses. The evaluate statement follows this same structure: `evaluate { body } by ... accept by ... upto ...`. Study how `try` dispatches between `BinaryTry` and `EvalTry` based on the next keyword after the body block — the same kind of contextual dispatch applies when parsing `evaluate`'s evaluator clause and `accept by` suffix.

- **Agent dispatch clauses** from `do` — the `with <agent> using <model> in <dir> compact <cfg>` clause sequence. The agent-type evaluator reuses these exact clause patterns: the evaluator is `@"prompt.md" with <agent> using <model>`, parsed with the same logic that `do` uses for its agent configuration.

All keywords required (`evaluate`, `accept`, `with`, `using`) are already registered. The `by` keyword after `evaluate` and after `accept` is recognized contextually (like how `with` and `using` are contextual within `do` parsing), not as a globally reserved keyword.

The evaluator clause is an expression — `@"file.md" with <agent>` is identical to how a `do` statement starts, `fn_name(args)` is identical to how function calls appear everywhere, and `exec "cmd"` is identical to how standalone `exec` statements work. No new expression forms are needed.

### Evaluator Layer

The evaluate statement's runtime behavior is a retry loop with per-iteration scope management, evaluator dispatch, score extraction, and threshold comparison. Three existing patterns provide the foundation:

- **Retry loop structure** — the existing `BinaryTry` and `EvalTry` evaluators implement looping with max-attempt guards, iteration counting, and outcome determination. The evaluate loop follows the same shape but adds score extraction and threshold comparison between the body and the iteration wrap-up.

- **Scope lifecycle** — the existing evaluator already has push/pop scope operations and variable get/set methods. The evaluate statement pushes a fresh scope at the top of each iteration and pops it at the bottom, with language-managed variables set into the new scope before the body runs. Variable propagation to the parent scope happens after the loop terminates — the parent scope (not the final iteration scope) receives `$score`, `$accepted`, and `$_evaluator_output`.

- **Agent call dispatch** — invoking an agent as the evaluator reuses the existing agent-call mechanism directly. The prompt augmentation (scoring instructions, git diff context) happens in the evaluate evaluator before the agent call, by prepending text to the prompt in memory.

- **Command execution** — invoking a command as the evaluator reuses the existing shell executor, with stdout captured for score parsing.

- **Function call** — invoking a function as the evaluator reuses the existing function-call mechanism, with the return value used as the score.

Each evaluator type may need a distinct score-extraction strategy. Agent output requires strict `SCORE:` line parsing (no numeric fallback). Command output uses structured-then-fallback parsing. Function evaluators use the typed return value directly.

### Interpolation and Prompt Files

The existing `${VAR}` interpolation system (which resolves variables against the current scope in file contents and string literals) is the mechanism that makes iteration context available in body prompt files. When a body block contains `@"task.md"`, the file reader resolves `${_attempt}`, `${_max_attempts}`, and `${_feedback}` from the iteration's scope automatically. No new injection machinery is needed.

The language-injected scoring-format instructions for agent evaluators are prepended to the evaluator's prompt **in memory**. They are NOT written to any prompt file. This avoids polluting the user's prompt files and makes the injection transient.

### Project-Wide Concerns

- **Agent name collection** — a pre-flight validation step discovers agent names referenced throughout a script. The evaluate statement's body and evaluator clause may reference agents; this discovery step must include them.

- **Test coverage** — ash tests cover parsing round-trips (parse string, serialize, re-parse) and execution behavior. The evaluate statement needs test coverage in both categories, following existing patterns in the test data files.

### Patterns to Avoid Confusing

- The `evaluate` keyword as a statement starter (new) vs. `evaluate with` as a `try` suffix (existing `EvalTry`). These are separate parse paths that should not be conflated or merged.

### What NOT to Touch

The following existing components are NOT modified by this feature and should remain unchanged:

- **`EvalTry` and `BinaryTry`** — their parsing, AST representation, and evaluation remain identical. They coexist with the new `evaluate` statement as separate constructs serving different use cases.
- **The `do` statement** — its agent-clause parsing (`with`/`using`/`in`/`compact`) is reused by reference (the same parsing logic), not modified or extended.
- **The interpolation system** — `${VAR}` resolution works through scope and needs no changes.
- **The scope implementation** — push/pop/get/set semantics are sufficient; no new scope primitives are required.
- **The integer value representation** — scores are integers, represented as the existing integer type. No new value variant is needed.
- **Prompt files in `refinery/` and user scripts** — these are user-level artifacts, not part of the language. The evaluate statement does not depend on any specific prompt file structure.
- **The exit code variable (`$?`)** — the evaluate statement's `$accepted` variable is a separate mechanism. Exit code semantics for commands remain unchanged.
