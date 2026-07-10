#!echo:1.0
#!compact mode=on strategy=truncate window=32000

# ============================================================================
# ash language — master test runner
# ============================================================================
#
# Run:    ash ash/testdata/test_all.ash
#
# Integration tests (directory-based orchestration) are run separately:
#   ash ash/testdata/integration/
#
# CLI-level tests:
#   bash ash/testdata/test_cli.sh
# ============================================================================

include "lang_tests.ash"
include "agent_tests.ash"

exit 0
