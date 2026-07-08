#!opencode:1.0

INPUT_FILE = $1
if len(INPUT_FILE) == 0 {
  print "usage: ash decomposit.ash <task.md>"
  exit 1
}

do @"refinery/decomposit.md" using deepseek/deepseek-v4-pro 
