#!echo:1.0

# Scripted implementation task — exercises ash flow control features
print "=== Module B: Scripted Implementation ==="

MODULE = "tasker-cli"
VERSION = "1.0.0"
print "Building module ${MODULE} v${VERSION}"

# If/else flow control
STATUS = "ready"
if STATUS == "ready" {
  print "  Status: ready — generating CLI layer"
} else {
  print "  Status: not ready — skipped"
}

# For loop over array of components
for C in ["parser", "renderer", "router"] {
  print "  Component: ${C}"
}

# While loop with counter
I = 0
while I < 3 {
  print "  Iteration ${I}"
  I = I + 1
}

# Function definition and call
fn module_name(PREFIX, NAME) {
  return PREFIX + "-" + NAME
}
FULL = module_name("ash", MODULE)
print "  Resolved name: ${FULL}"

# Nested if/else with function call
fn is_even(N) {
  return N % 2 == 0
}
SCORE = 42
if is_even(SCORE) {
  print "  Score ${SCORE} is even — all checks pass"
} else {
  print "  Score ${SCORE} is odd — review needed"
}

# Exit cleanly
print "=== Module B complete ==="
