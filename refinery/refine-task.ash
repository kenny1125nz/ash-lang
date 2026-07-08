#!opencode:1.0

INPUT_FILE = $1
if len(INPUT_FILE) == 0 {
  print "usage: ash refine-task.ash <requirement.md>"
  exit 1
}

THRESHOLD = 85
MAX = 8

exec "mkdir -p tmp && rm -f tmp/score.* tmp/assessment-report.md"

for ATTEMPT in range(1, MAX + 1) {
  PREV_IDX = ATTEMPT - 1
  IDX = ATTEMPT
  do @"refinery/prompt-produce.md" using deepseek/deepseek-v4-pro 

  exec "rm -f tmp/score.*"
  do @"refinery/prompt-assess.md" using deepseek/deepseek-v4-pro   
  SCORE = $(ls tmp/score.* 2>/dev/null | head -1 | grep -oE '[0-9]+$' )  
  exec test '${SCORE}' -ge ${THRESHOLD}
  if $? == 0 {    
    exec "cp tmp/task-definition_${ATTEMPT}.md tasks/ready/refined-task.md"
    exec "cat tasks/ready/refined-task.md"
    exit 0
  }
  print "Attempt ${ATTEMPT}: score=${SCORE}, threshold=${THRESHOLD} pass=${$?}"
}
exit 1
