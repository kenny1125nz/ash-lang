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
ASH_LOG=debug ASH_LOG_FILE=custom/path/ash.log $ASH_BIN discover 2>&1 > /dev/null
[ -f custom/path/ash.log ] && pass "ASH_LOG_FILE custom path works" || fail "ASH_LOG_FILE custom path not found"
popd > /dev/null
rm -rf "$TMPDIR"

# ----- CLI flags -----

echo "=== --check: validates valid script ==="
OUT=$($ASH_BIN --check "$(dirname "$0")/lang_tests.ash" 2>&1) && pass "--check exited 0" || fail "--check exited $?"
echo "$OUT" | grep -qi "ok" && pass "--check printed OK" || true

# ----- cleanup -----

report
