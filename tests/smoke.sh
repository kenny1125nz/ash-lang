#!/bin/bash
set -euo pipefail

ASH="./target/debug/ash"
ASHD="./target/debug/ashd"
TEST_DIR="/tmp/ashd-smoke-test"
WATCH_DIR="$TEST_DIR/incoming"

echo "=== ashd smoke test ==="

# Cleanup from previous runs
rm -rf "$TEST_DIR"
mkdir -p "$WATCH_DIR"

# Create minimal ashd config
cat > "$TEST_DIR/ashd.yml" << 'EOF'
daemon:
  ws_listen: "127.0.0.1:9877"
  grace_period_secs: 5
sources:
  - type: folder
    name: smoke-test
    path: /tmp/ashd-smoke-test/incoming
    workflows:
      - path: /tmp/ashd-smoke-test/workflow.ash
EOF

# Create a minimal workflow script
cat > "$TEST_DIR/workflow.ash" << 'EOF'
echo "hello from smoke test"
EOF

# Start ashd in background
ASHD_CONFIG="$TEST_DIR/ashd.yml" "$ASHD" &
ASHD_PID=$!
sleep 2

# Verify ashd is listening
if ! kill -0 $ASHD_PID 2>/dev/null; then
    echo "FAIL: ashd failed to start"
    exit 1
fi
echo "PASS: ashd started (pid=$ASHD_PID)"

# Verify ashd --check works
ASHD_CONFIG="$TEST_DIR/ashd.yml" "$ASHD" --check
echo "PASS: ashd --check succeeded"

# Create a test event
EVENT_DIR="$WATCH_DIR/event-001"
mkdir -p "$EVENT_DIR"
cp "$TEST_DIR/workflow.ash" "$EVENT_DIR/test.ash"

# Wait for ashd to claim the event
sleep 3

# Check that event was claimed (moved to .processing)
if [ -d "$WATCH_DIR/.processing/event-001" ]; then
    echo "PASS: event claimed by ashd"
else
    echo "FAIL: event not claimed"
    exit 1
fi

# Wait for completion
sleep 10

# Check that event was completed (moved to .done)
if [ -d "$WATCH_DIR/.done/event-001" ]; then
    echo "PASS: event completed"
elif [ -d "$WATCH_DIR/.done/failed/event-001" ]; then
    echo "PASS: event completed (with failure status)"
else
    echo "FAIL: event not completed"
    exit 1
fi

# Clean shutdown
kill $ASHD_PID 2>/dev/null || true
wait $ASHD_PID 2>/dev/null || true

echo "=== All smoke tests passed ==="
