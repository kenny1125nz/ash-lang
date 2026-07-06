# Prompt File Interpolation

## Background

Ash supports reading external files as agent prompts via the `@file` syntax:
```ash
do @tasks/instructions.md with opencode
```

However, the file content is sent to the agent as-is. If the file contains `${VAR}` placeholders, they are not resolved — the agent receives literal `${VAR}` text instead of the variable's value. This prevents composing prompts from reusable templates with dynamic values.

Example of what doesn't work today:
```ash
target = "1-directory-orchestration.md"
do @tasks/common/refine-task.md with opencode
```

Where `refine-task.md` contains `update the task definition file ${target}...`. The agent sees `${target}` literally instead of `1-directory-orchestration.md`.

## Intended Solution

When file content is read via `@file`, resolve `${VAR}` placeholders against the current ash scope before sending to the agent. `$(cmd)` placeholders should also be resolved for consistency.

This is the same interpolation logic already applied to inline strings and text blocks — the file content should be treated no differently.

```ash
target = "1-directory-orchestration.md"
do @tasks/common/refine-task.md with opencode
```

The agent receives: `update the task definition file 1-directory-orchestration.md...`

## Acceptance Criteria

1. **`@file` content resolves `${VAR}` from scope**
   - `x = "world"; do @test.md` where `test.md` contains `hello ${x}` sends `hello world` to the agent

2. **`@file` content resolves `$(cmd)`**
   - `do @test.md` where `test.md` contains `host: $(hostname)` sends the resolved hostname

3. **Undefined variables are preserved**
   - `${UNDEFINED}` in file content stays as `${UNDEFINED}` (same behavior as inline strings)

4. **No scope pollution**
   - Variables resolved from the file content do not leak into or overwrite the current scope

5. **Unchanged: multiple variables work**
   - `x = "a"; y = "b"; do @test.md` where `test.md` contains `${x} and ${y}` sends `a and b`

## Implementation Hints

### Relevant project context

**Where `@file` is evaluated:**
- `eval/expr.rs:264-278` — `eval_fp()` reads the file and returns raw content as `Value::String`. No interpolation is performed.
- `ast.rs:244-247` — `FilePath { pos, path: Box<Node> }` — the path itself can be an expression (variable, string, etc.)

**Where interpolation already happens:**
- `eval/expr.rs:151-182` — `resolve_interpolations(value, interps)` resolves `${VAR}` from the evaluator's `current_scope` and `$(cmd)` via the `Executor`.
- `eval/expr.rs:141-148` — `eval_string()` and `eval_text_block()` both call `resolve_interpolations()`
- `interpolation.rs:8-42` — `Interpolation::resolve()` uses a regex to find `${VAR}` and `$(cmd)` patterns and replace them via closures

**The fix:** In `eval_fp()`, after reading the file content, call `self.resolve_interpolations(&content, &[])` (empty interps list triggers the regex-based path that handles raw `${...}` text). The existing `resolve_interpolations` method already handles the regex path internally when `interps` is empty — it falls through to `Interpolation::resolve()`.

### What not to touch

The interpolation module itself (`interpolation.rs`) works correctly — it just needs to be called from the right place.
