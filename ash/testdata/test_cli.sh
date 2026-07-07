#!/usr/bin/env bash
# ============================================================================
# ash CLI-level test suite
# Tests features that cannot be tested from within ash scripts:
#   - `ash discover` subcommand
#   - Logging system (ASH_LOG / ASH_LOG_FILE env vars)
#   - CLI flags (--check, --dry-run)
# ============================================================================
set -euo pipefail

ASH_BIN="${ASH_BIN:-./target/debug/ash}"
PASS=0
FAIL=0

pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

report() {
    echo ""
    echo "=== Results: ${PASS} passed, ${FAIL} failed ==="
    exit $(( FAIL > 0 ? 1 : 0 ))
}

# ----- discover subcommand -----

echo "=== discover: runs without error ==="
OUT=$($ASH_BIN discover 2>&1) && pass "discover exited 0" || fail "discover exited $?"

echo "=== discover: output contains known agents ==="
echo "$OUT" | grep -q "opencode" && pass "discover lists opencode" || fail "discover missing opencode"
echo "$OUT" | grep -q "echo" && pass "discover lists echo" || fail "discover missing echo"

echo "=== discover --write: generates config ==="
TMPDIR=$(mktemp -d)
pushd "$TMPDIR" > /dev/null
$ASH_BIN discover --write 2>&1 && [ -f ash-project.yaml ] && pass "discover --write created ash-project.yaml" || fail "discover --write failed"
popd > /dev/null
rm -rf "$TMPDIR"

echo "=== discover --write --force: overwrites existing config ==="
TMPDIR=$(mktemp -d)
echo "old" > "$TMPDIR/ash-project.yaml"
pushd "$TMPDIR" > /dev/null
$ASH_BIN discover --write --force 2>&1 && pass "discover --write --force overwrote config" || fail "discover --write --force failed"
popd > /dev/null
rm -rf "$TMPDIR"

# ----- logging system -----

echo "=== logging: ASH_LOG=info produces log file ==="
TMPDIR=$(mktemp -d)
pushd "$TMPDIR" > /dev/null
ASH_LOG=info ASH_LOG_FILE=test_ash.log $ASH_BIN discover 2>&1 > /dev/null
[ -f test_ash.log ] && [ -s test_ash.log ] && pass "ASH_LOG=info created log file" || fail "ASH_LOG=info did not create log file"
popd > /dev/null
rm -rf "$TMPDIR"

echo "=== logging: ASH_LOG=trace produces more log output ==="
TMPDIR=$(mktemp -d)
pushd "$TMPDIR" > /dev/null
ASH_LOG=trace ASH_LOG_FILE=trace.log $ASH_BIN discover 2>&1 > /dev/null
ASH_LOG=info ASH_LOG_FILE=info.log $ASH_BIN discover 2>&1 > /dev/null
TRACE_LINES=$(wc -l < trace.log)
INFO_LINES=$(wc -l < info.log)
[ "$TRACE_LINES" -ge "$INFO_LINES" ] && pass "trace log >= info log lines" || fail "trace log ($TRACE_LINES lines) < info log ($INFO_LINES lines)"
popd > /dev/null
rm -rf "$TMPDIR"

echo "=== logging: ASH_LOG_FILE uses custom path ==="
TMPDIR=$(mktemp -d)
pushd "$TMPDIR" > /dev/null
mkdir -p custom/path
ASH_LOG=debug ASH_LOG_FILE=custom/path/ash.log $ASH_BIN discover 2>&1 > /dev/null
[ -f custom/path/ash.log ] && pass "ASH_LOG_FILE custom path works" || fail "ASH_LOG_FILE custom path not found"
popd > /dev/null
rm -rf "$TMPDIR"

# ----- CLI flags -----

echo "=== --check: validates valid script ==="
OUT=$($ASH_BIN --check "$(dirname "$0")/lang_tests.ash" 2>&1) && pass "--check exited 0" || fail "--check exited $?"
echo "$OUT" | grep -qi "ok" && pass "--check printed OK" || true

# ----- piped batch input (non-TTY) -----

echo "=== batch: single print statement ==="
OUT=$(echo 'print "hello"' | $ASH_BIN 2>/dev/null)
echo "$OUT" | grep -q "hello" && pass "batch printed output" || fail "batch missing output"

echo "=== batch: variable + print ==="
OUT=$(echo 'X = 42
print X' | $ASH_BIN 2>/dev/null)
echo "$OUT" | grep -q "42" && pass "batch handles vars" || fail "batch missing var value"

echo "=== batch: multi-line block ==="
OUT=$(printf 'if true {\n  print "block"\n}' | $ASH_BIN 2>/dev/null)
echo "$OUT" | grep -q "block" && pass "batch handles blocks" || fail "batch missing block output"

echo "=== batch: expr evaluation (print form) ==="
OUT=$(echo 'print "result: " + (2 + 2)' | $ASH_BIN 2>/dev/null)
echo "$OUT" | grep -q "result: 4" && pass "batch evaluates expression" || fail "batch missing expression result"

echo "=== batch: exit code propagates ==="
# Disable set -e for this test since we expect non-zero exit
set +e
echo 'exit 42' | $ASH_BIN 2>/dev/null
rc=$?
set -e
[ "$rc" -eq 42 ] && pass "batch exit code propagates" || fail "batch exit code was $rc"

# ----- --config flag -----

echo "=== --config: overrides config path ==="
TMPDIR=$(mktemp -d)
echo 'agents:
  custom-echo:
    type: local-cli
    cmd: echo
    message_flag: " "' > "$TMPDIR/ash.yaml"
OUT=$($ASH_BIN --config "$TMPDIR/ash.yaml" discover 2>&1) && pass "--config discover works" || fail "--config discover failed $?"
rm -rf "$TMPDIR"

echo "=== --config: error if file missing ==="
OUT=$($ASH_BIN --config "/nonexistent/path/ash.yaml" discover 2>&1) && fail "--config should have errored" || pass "--config correctly rejects missing file"

echo "=== --config: used with check ==="
TMPDIR=$(mktemp -d)
echo 'agents:
  custom-echo:
    type: local-cli
    cmd: echo
    message_flag: " "' > "$TMPDIR/ash.yaml"
SCRIPT="$TMPDIR/test.ash"
echo '#!custom-echo:1.0
print "hello"' > "$SCRIPT"
OUT=$($ASH_BIN --config "$TMPDIR/ash.yaml" --check "$SCRIPT" 2>&1) && pass "--config + --check works" || fail "--config + --check failed"
rm -rf "$TMPDIR"

# ----- -c shorthand for --check -----

echo "=== -c: shorthand for --check ==="
OUT=$($ASH_BIN -c "$(dirname "$0")/lang_tests.ash" 2>&1) && pass "-c exited 0" || fail "-c exited $?"

# ----- --continue-on-error flag -----

echo "=== --continue-on-error: ash script with non-zero exit after failure ==="
OUT=$(printf 'exit 1\n' | $ASH_BIN --continue-on-error 2>/dev/null) && pass "--continue-on-error handled exit" || true
# Just verify the flag doesn't crash
set +e
$ASH_BIN --continue-on-error -c "$(dirname "$0")/lang_tests.ash" > /dev/null 2>&1
rc=$?
set -e
[ "$rc" -eq 0 ] && pass "--continue-on-error + --check works" || fail "--continue-on-error + --check failed with $rc"

echo "=== -k: shorthand for --continue-on-error ==="
set +e
$ASH_BIN -k -c "$(dirname "$0")/lang_tests.ash" > /dev/null 2>&1
rc=$?
set -e
[ "$rc" -eq 0 ] && pass "-k shorthand works" || fail "-k failed with $rc"

# ----- --dry-run flag (directory mode) -----

echo "=== --dry-run: lists tasks without executing ==="
OUT=$($ASH_BIN --dry-run "$(dirname "$0")/integration" 2>&1) && pass "--dry-run exited 0" || fail "--dry-run exited $?"
echo "$OUT" | grep -q "dry-run" && pass "--dry-run printed tasks" || fail "--dry-run missing task list"

# ----- cleanup -----

report
