#!opencode:1.0

exec mkdir -p ./tmp

exec node workflows/defects/fetch-defects.js

RESULT = $(cat "./tmp/defects.json")
if RESULT == "[]" {
  exit 0
}

do @"workflows/defects/analyze-and-fix.md"

exec node workflows/defects/mark-fixed.js

exec node workflows/defects/deploy.js
