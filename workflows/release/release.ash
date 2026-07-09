tag = ${1}

include "workflows/release/prepare.ash"

do @"workflows/release/generate-release-notes.md" with opencode

include "workflows/release/publish.ash"
