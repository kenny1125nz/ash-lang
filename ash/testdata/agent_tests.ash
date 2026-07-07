# ============================================================================
# ash agentic behavior — comprehensive test suite
# Default agent is "echo" (from shebang in test_all.ash)
# Explicit `with echo` is used only to validate the explicit-agent path.
# ============================================================================

# ---------------------------------------------------------------------------
# 1. Basic agent call (uses shebang default agent)
# ---------------------------------------------------------------------------
print "=== 1. Basic agent call (default) ==="
do "basic echo call"
print "  stdout: ${stdout}"

# ---------------------------------------------------------------------------
# 2. Explicit agent via `with <agent>` clause
# ---------------------------------------------------------------------------
print "=== 2. Explicit with echo ==="
do "explicit agent" with echo
print "  stdout: ${stdout}"

# ---------------------------------------------------------------------------
# 3. stdout / stderr / $? variables after call
# ---------------------------------------------------------------------------
print "=== 3. Scope variables ==="
do "scope check"
print "  stdout: ${stdout}"
print "  stderr: '${stderr}'"
if $? {
  print "  FAIL: exit code should be 0"
} else {
  print "  exit code is 0"
}

# ---------------------------------------------------------------------------
# 4. Agent with model (using clause)
# ---------------------------------------------------------------------------
print "=== 4. using clause ==="
do "model test" with echo using "gpt-4"
print "  stdout: ${stdout}"

# ---------------------------------------------------------------------------
# 5. Agent with compact clause
# ---------------------------------------------------------------------------
print "=== 5. compact clause ==="
do "compact test" compact "truncate 100"
print "  stdout: ${stdout}"

# ---------------------------------------------------------------------------
# 6. Agent with directory (in clause)
# ---------------------------------------------------------------------------
print "=== 6. in clause ==="
do "dir test" in "."
print "  stdout: ${stdout}"

# ---------------------------------------------------------------------------
# 7. All options combined (explicit agent + using + in + compact)
# ---------------------------------------------------------------------------
print "=== 7. All options combined ==="
do "all together" with echo using "sonnet" in "/tmp" compact "summarize"
print "  stdout: ${stdout}"

# ---------------------------------------------------------------------------
# 8. try/fail — success path
# ---------------------------------------------------------------------------
print "=== 8. try/fail (success) ==="
try {
  do "try success"
  print "  try body: ${stdout}"
} fail {
  print "  FAIL: should not reach fail block"
} upto 1

# ---------------------------------------------------------------------------
# 9. try/fail — failure path (nonexistent agent)
# ---------------------------------------------------------------------------
print "=== 9. try/fail (failure) ==="
try {
  do "should fail" with nonexistent-agent-xyz
  print "  FAIL: should have errored"
} fail {
  print "  fail block reached (expected)"
} upto 1

# ---------------------------------------------------------------------------
# 10. Background agent call (& postfix)
# ---------------------------------------------------------------------------
print "=== 10. Background agent ==="
do "background task" &
print "  main continues while agent runs in background"
wait
print "  background done"

# ---------------------------------------------------------------------------
# 11. Multiple background agent calls
# ---------------------------------------------------------------------------
print "=== 11. Multiple background agents ==="
do "bg one" &
do "bg two" &
do "bg three" &
wait
print "  all background agents done"

# ---------------------------------------------------------------------------
# 12. Parallel wait block with agents
# ---------------------------------------------------------------------------
print "=== 12. Parallel agents ==="
wait {
  do "parallel A"
  do "parallel B"
}
print "  parallel agents done"

# ---------------------------------------------------------------------------
# 13. Agent inside a function
# ---------------------------------------------------------------------------
print "=== 13. Agent in function ==="
fn run_agent(MSG) {
  do MSG
  return stdout
}
RES = run_agent("function call")
print "  returned: ${RES}"

# ---------------------------------------------------------------------------
# 14. Agent inside for loop
# ---------------------------------------------------------------------------
print "=== 14. Agent in for loop ==="
for ITEM in "first
second
third" {
  do "loop: ${ITEM}"
  print "  got: ${stdout}"
}

# ---------------------------------------------------------------------------
# 15. Agent inside while loop
# ---------------------------------------------------------------------------
print "=== 15. Agent in while loop ==="
I = 0
while I < 3 {
  do "while iter ${I}"
  print "  iter ${I}: ${stdout}"
  I = I + 1
}

# ---------------------------------------------------------------------------
# 16. Interpolated prompt
# ---------------------------------------------------------------------------
print "=== 16. Interpolated prompt ==="
TOPIC = "interpolation"
do "testing ${TOPIC} in prompt"
print "  stdout: ${stdout}"

# ---------------------------------------------------------------------------
# 17. Sequential calls (scope overwrites)
# ---------------------------------------------------------------------------
print "=== 17. Sequential calls ==="
do "call one"
print "  (1) ${stdout}"
do "call two"
print "  (2) ${stdout}"

# ---------------------------------------------------------------------------
# 18. Variable as prompt (VarRef)
# ---------------------------------------------------------------------------
print "=== 18. VarRef prompt ==="
MSG = "variable prompt"
do MSG
print "  stdout: ${stdout}"

# ---------------------------------------------------------------------------
# 19. Backward compat: `with subagent <name>` uses shebang default
# ---------------------------------------------------------------------------
print "=== 19. with subagent (backward compat) ==="
do "subagent compat" with subagent foo
print "  stdout: ${stdout}"

# ---------------------------------------------------------------------------
# 20. Explicit `with <agent> subagent <name>`
# ---------------------------------------------------------------------------
print "=== 20. with + subagent (explicit) ==="
do "explicit subagent" with echo subagent my-cap
print "  stdout: ${stdout}"

# ---------------------------------------------------------------------------
# 21. Agent in binary try
# ---------------------------------------------------------------------------
print "=== 21. Agent in binary try ==="
try {
  do "binary try body"
  print "  try body: ${stdout}"
} upto 1

# ---------------------------------------------------------------------------
# 22. Eval try partial — reject → accept chain
# ---------------------------------------------------------------------------
print "=== 22. Eval try partial (reject → accept) ==="
ATTEMPT = 0
try {
  ATTEMPT = ATTEMPT + 1
} evaluate with {
  ATTEMPT >= 3
} accept {
  print "  accepted on attempt ${ATTEMPT}"
} partial {
  print "  partial: attempt ${ATTEMPT}, retrying"
} upto 5

# ---------------------------------------------------------------------------
# 23. Eval try partial — accept on first attempt (partial does NOT run)
# ---------------------------------------------------------------------------
print "=== 23. Eval try partial (accept first) ==="
try {
  X = 42
} evaluate with {
  X == 42
} accept {
  print "  accepted first try"
} partial {
  print "  FAIL: partial should NOT run on accept"
} upto 1

# ---------------------------------------------------------------------------
# 24. Eval try — all attempts fail (falsy expression = partial, not fail)
# ---------------------------------------------------------------------------
print "=== 24. Eval try all fail (falsy = partial) ==="
COUNT = 0
try {
  COUNT = COUNT + 1
} evaluate with {
  COUNT > 10
} accept {
  print "  FAIL: should not be accepted"
} partial {
  print "  partial: attempt ${COUNT}"
} fail {
  print "  FAIL: fail block should not run for falsy expression"
} upto 3
print "  exhausted after ${COUNT} attempts (expected)"

# ---------------------------------------------------------------------------
# 25. Eval try — exit 2+ triggers fail block (not partial)
# ---------------------------------------------------------------------------
print "=== 25. Eval try exit 2+ (fail path) ==="
try {
  X = 1
} evaluate with {
  exec sh -c "exit 2"
} accept {
  print "  FAIL: should not accept"
} partial {
  print "  FAIL: exit 2+ is fail, not partial"
} fail {
  print "  fail block reached (exit 2+ correctly)"
} upto 1

# ---------------------------------------------------------------------------
# 26. Eval try with agent call + partial
# ---------------------------------------------------------------------------
print "=== 26. Eval try + agent + partial ==="
N = 0
try {
  do "eval agent call ${N}"
  N = N + 1
} evaluate with {
  N >= 2
} accept {
  print "  agent accepted: ${stdout}"
} partial {
  print "  partial: agent returned ${stdout}"
} upto 3

# ---------------------------------------------------------------------------
# 27. Eval try with function call in evaluate block
# ---------------------------------------------------------------------------
print "=== 27. Eval try + function call ==="
fn is_even(N) {
  return N % 2 == 0
}
VAL = 1
try {
  VAL = VAL + 1
} evaluate with {
  is_even(VAL)
} accept {
  print "  accepted (${VAL} is even)"
} partial {
  print "  partial: ${VAL} is odd, retrying"
} upto 5

# ---------------------------------------------------------------------------
# 28. Conditional agent call
# ---------------------------------------------------------------------------
print "=== 28. Conditional agent ==="
SHOULD_RUN = true
if SHOULD_RUN {
  do "conditional call"
  print "  ran: ${stdout}"
}
if not SHOULD_RUN {
  print "  FAIL: should not run"
}

# ---------------------------------------------------------------------------
# 29. Agent in nested blocks
# ---------------------------------------------------------------------------
print "=== 29. Agent in nested blocks ==="
if true {
  if true {
    do "nested block"
    print "  nested: ${stdout}"
  }
}

# ---------------------------------------------------------------------------
# 30. Invalid dir (should fail validation)
# ---------------------------------------------------------------------------
print "=== 30. Invalid dir ==="
try {
  do "bad dir" in "/nonexistent/path/xyz"
  print "  FAIL: should have errored"
} fail {
  print "  invalid dir correctly rejected"
} upto 1

# ---------------------------------------------------------------------------
# 31. Compound result from multiple calls
# ---------------------------------------------------------------------------
print "=== 31. Compound result ==="
do "part one"
A = stdout
do "part two"
B = stdout
print "  combined: ${A}${B}"

# ---------------------------------------------------------------------------
# 32. Expression prompt (string + var concatenation)
# ---------------------------------------------------------------------------
print "=== 32. Expression prompt ==="
SUFFIX = "expr suffix"
do "testing " + SUFFIX
print "  stdout: ${stdout}"

# ---------------------------------------------------------------------------
# 33. Exit code isolation: exec → agent resets $?
# ---------------------------------------------------------------------------
print "=== 33. Exit code isolation ==="
exec false
print "  after false: ${?}"
do "exit reset"
print "  after agent: ${?}"

# ---------------------------------------------------------------------------
# 34. Empty prompt
# ---------------------------------------------------------------------------
print "=== 34. Empty prompt ==="
do ""
print "  stdout: '${stdout}'"

# ---------------------------------------------------------------------------
# 35. Agent call result directly compared
# ---------------------------------------------------------------------------
print "=== 36. Session block — multiple do calls ==="
session {
  do "session call one"
  print "  (1) ${stdout}"
  do "session call two"
  print "  (2) ${stdout}"
  do "session call three"
  print "  (3) ${stdout}"
}
print "  session block completed"

print "=== 37. Session block — do outside session is not inside ==="
session {
  do "inside session"
  print "  inside: ${stdout}"
}
do "outside session"
print "  outside: ${stdout}"

print "=== 38. Session block — nested sessions error ==="
try {
  session {
    session {
      do "should not run"
    }
  }
  print "  FAIL: nested session should have errored"
} fail {
  print "  nested session correctly rejected"
} upto 1

print "=== 39. Session block with compact ==="
session {
  do "session with compact" compact "truncate 100"
  print "  stdout: ${stdout}"
}

print "=== 40. Session block inside if ==="
FLAG = true
if FLAG {
  session {
    do "conditional session"
    print "  conditional: ${stdout}"
  }
}

print "=== 41. Session block with using clause ==="
session {
  do "session model" using "sonnet"
  print "  stdout: ${stdout}"
}

print "=== 42. Session block with explicit agent ==="
session {
  do "explicit echo" with echo
  print "  stdout: ${stdout}"
}

print "=== 43. @file interpolation — undefined variables preserved ==="
VAR1 = "hello"
# VAR2 intentionally not set — should stay as ${VAR2}
do @"testdata/prompt.md"
print "  preserved: ${stdout}"

print "=== 44. @file interpolation — multiple variables ==="
VAR1 = "hello"
VAR2 = "world"
do @"testdata/prompt.md"
print "  resolved: ${stdout}"

print "=== 45. Session with @file prompt ==="
VAR1 = "inside"
VAR2 = "session"
session {
  do @"testdata/prompt.md"
  print "  resolved: ${stdout}"
}

print "=== 46. Session toggle — begin/end with do calls ==="
session begin
do "toggle session call one"
print "  (1) ${stdout}"
do "toggle session call two"
print "  (2) ${stdout}"
session end
print "  session toggle completed"

print "=== 47. Session toggle — isolation from outside ==="
session begin
do "inside toggle session"
print "  inside: ${stdout}"
session end
do "outside toggle session"
print "  outside: ${stdout}"

print "=== 48. Session toggle — nested begin error ==="
try {
  session begin
  session begin
  print "  FAIL: nested session begin should have errored"
} fail {
  print "  nested session begin correctly rejected"
  session end
} upto 1

print "=== 49. Session toggle + block mixed ==="
session begin
do "toggle before block"
print "  toggle: ${stdout}"
session end
session {
  do "block after toggle"
  print "  block: ${stdout}"
}
print "  mixed usage works"

print "=== 50. Within toggle — begin/end with do calls ==="
CWD_BEFORE = $(pwd)
within begin "/tmp"
do "do inside within toggle"
print "  (1) ${stdout}"
CWD_INSIDE = $(pwd)
within end
CWD_AFTER = $(pwd)
print "  cwd before: ${CWD_BEFORE}"
print "  cwd inside: ${CWD_INSIDE}"
print "  cwd after: ${CWD_AFTER}"

print "=== 51. Within toggle — begin/end with session ==="
session begin
within begin "/tmp"
do "session + within toggle"
print "  combined: ${stdout}"
within end
session end
print "  session + within toggle works"

print "=== 52. Within toggle — begin with non-existent path ==="
try {
  within begin "/nonexistent/path/xyz"
  print "  FAIL: should have errored"
} fail {
  print "  within begin with bad path correctly rejected"
} upto 1

print "=== 53. Within toggle — nested (stack-based, should work) ==="
CWD_ROOT = $(pwd)
within begin "/tmp"
CWD_L1 = $(pwd)
within begin "/var"
CWD_L2 = $(pwd)
within end
CWD_L1_RESTORED = $(pwd)
within end
CWD_ROOT_RESTORED = $(pwd)
print "  root: ${CWD_ROOT}"
print "  level1: ${CWD_L1}"
print "  level2: ${CWD_L2}"
print "  after pop1 (should be L1): ${CWD_L1_RESTORED}"
print "  after pop2 (should be root): ${CWD_ROOT_RESTORED}"
print "  nested within toggle works"

print "=== 54. Session toggle — begin inside session block is error ==="
try {
  session {
    session begin
    print "  FAIL: should have errored"
  }
} fail {
  print "  session begin inside session block correctly rejected"
} upto 1

print "=== 55. Session toggle — block inside begin is error ==="
try {
  session begin
  session {
    do "should not run"
  }
  print "  FAIL: nested session should have errored"
} fail {
  print "  block inside toggle correctly rejected"
  session end
} upto 1

# ---------------------------------------------------------------------------
# 56. Within toggle — do inside with in clause override
# ---------------------------------------------------------------------------
print "=== 56. Within toggle + in clause ==="
within begin "/tmp"
do "in clause override" in "/"
print "  within toggle + in: ${stdout}"
within end
print "  within toggle + in works"

# ---------------------------------------------------------------------------
# 57. Session toggle — multiple begin/end pairs independent
# ---------------------------------------------------------------------------
print "=== 57. Session toggle — independent pairs ==="
session begin
do "first pair call"
print "  (1) ${stdout}"
session end
print "  (gap)"
session begin
do "second pair call"
print "  (2) ${stdout}"
session end
print "  independent session toggles work"

# ---------------------------------------------------------------------------
# 58. Mixed within toggle and block
# ---------------------------------------------------------------------------
print "=== 58. Mixed within toggle + block ==="
within begin "/tmp"
do "inside toggle"
print "  toggle: ${stdout}"
within end
within "/tmp" {
  do "inside block"
  print "  block: ${stdout}"
}
print "  mixed within works"

# ---------------------------------------------------------------------------
# 59. Session toggle — do with using clause
# ---------------------------------------------------------------------------
print "=== 59. Session toggle + using ==="
session begin
do "toggle with model" using "sonnet"
print "  stdout: ${stdout}"
session end
print "  session + using works"

# ---------------------------------------------------------------------------
# 60. Session toggle — do with compact clause
# ---------------------------------------------------------------------------
print "=== 60. Session toggle + compact ==="
session begin
do "toggle with compact" compact "truncate 100"
print "  stdout: ${stdout}"
session end
print "  session + compact works"

# ---------------------------------------------------------------------------
# 61. Eval try — error variable set on body failure
# ---------------------------------------------------------------------------
print "=== 61. Error variable on body failure ==="
try {
  exec sh -c "exit 1"
} evaluate with {
  false
} accept {
  print "  FAIL: body failed, should not accept"
} partial {
  print "  partial: error=${error}"
} fail {
  print "  fail: error=${error}"
} upto 2
print "  error was: ${error}"
