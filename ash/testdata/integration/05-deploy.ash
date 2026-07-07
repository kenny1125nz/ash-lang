#!echo:1.0

# Scripted deployment task — exercises advanced ash features
print "=== Deploy to Staging ==="

# Session block for grouped operations
session {
  print "  Building release artifacts..."
  print "  Module A: built"
  print "  Module B: built"
  print "  Integration: verified"
}

# Binary try/fail block
try {
  print "  Running pre-deploy checks..."
  print "  All checks passed"
} fail {
  print "  Pre-deploy checks failed"
} upto 1

# Within toggle for deployment directory context
within "/tmp" {
  CWD = $(pwd)
  print "  Deploying from: ${CWD}"
  print "  Artifacts staged successfully"
}

# For loop over services with array
for S in ["web", "api", "worker"] {
  print "  Service ${S}: started"
}

# Boolean expression in condition
HEALTHY = true
STABLE = true
if HEALTHY and STABLE {
  print "  System status: healthy and stable"
}

# Exit cleanly
print "=== Deployment complete ==="
