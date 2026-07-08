# Improve Evaluation Score

In following typical  evaluation cycles,  the evaluation result score is implemented in a very odd way, score.file-> extract score -> stdout -> exec test -> $? check,
And what's worse is it was not working the exit condition never triggered.

```ash
  for ATTEMPT in range(1, MAX + 1) {
    PREV_IDX = ATTEMPT - 1
    IDX = ATTEMPT
    do @"refinery/prompt-produce.md" using deepseek/deepseek-v4-flash
    do @"refinery/prompt-assess.md" using deepseek/deepseek-v4-pro
    exec "ls tmp/score.* | head -1 | grep -oE '[0-9]+$'"
    exec rm tmp/score.*
    SCORE = stdout

    exec "test '${SCORE}' -ge ${THRESHOLD}"
    print "Attempt ${ATTEMPT}: score=${SCORE}, threshold=${THRESHOLD} ${$?}"
    if $? == 0 {
      exec "cp tmp/task-definition_${ATTEMPT}.md tasks/ready/refined-task.md"
      exec "cat tasks/ready/refined-task.md"
      exit 0
    }
  }
```

the evaluation rely on a score file to be generated as part of prompt-assess.md.  this means implicit requirement of how assess prompt should be written in other scenarios.  unless we inject extra prompt (of generating score file, but then what's the score range, 0-10, 0-100?) to the assessment call to the agent

Same with the IDX/PREV_IDX , need be coded in prompt files. which is not idea

we need a more elegant way of doing evaluations , there was a attempt to model that by try {} evaluate with {} xxx.  it is found hard to use, and lack of accessable "index" in retries, it is against the most important design princple of ash language, it need be as natural as possible so people without coding experience can understand it ( does not have write it, as it will be largely done by coding agents).


## A few things to consider

## Analyze key entities/artifacts involved.
Agents work differently from function calls, the Outcome is combination of returned  value ( standard Output/Error + exit code)  and  more importantly the impact it exerted (files it created or updated).
The target of evaluation is most likely against the impact ( the changed part), utilising of hand-off files is a compromise. we need find a better way to identify the changes. git seem to be a good candidate, but we dont want mess around current repository ( or at least not current branch)

Evaluation itself could be  determinstic code ( prefered) or agentic driven,  while the output of evalution should have both a numeric score to check against acceptance threshold and a findings/report as part of the input of next cyle's for improvement.

## Ideal Target State
An ideal way of expressing the cyle should be 
```

# agent based
evaluate {
  do "xxx" with opencode
  ...
  # could be multiple steps
} by @"evaluator.md" with claude-code using deepseek/deepseek-v4-pro
 accept by 85 upto 8
 
# function based
fn evaluate(){
  return score
}

evaluate {
  do "xxx" with opencode
  ...
  # could be multiple steps
} by evaluate()
 accept by 85 upto 8

# command based
evaluate {
  do "xxx" with opencode
  ...
  # could be multiple steps
} by exec "python scripts/evaluator.py"
 accept by 85 upto 8
```

This implies:
- evaluation score scale 0-100
- the by clause can be followed by 1 coded evaluation logic run as os command , 2 functional call,3 agentic call
- for coded evaluation and functional call, it can be return value of the function,  exit code of command. or formated standard output
- if it's agentic call, we might need inject extra prompt to ask for scoring

## Sandbox based on git branching
as mentioned ealier, git could be a good candiate to track changes. we could integrate with git to :
- create a local branch like ash/uuid before all the operations, and keep the original branch name before 
- assume the do "xxx" will apply changes in the branch, not commited
- the evaluation step will need inject prompt about scoring and give a bit of context of git, so agent can use git diff to identify the changes for review
- after evaluation step, changes and evaluation report can be commited, 
- if the score is higher than threshold, the application merge changes back to previsou branch, and delete ash/uuid branch
- if the score is lower than threshhold, we need inject prompt to the do "xxx" operations(s) to add the evaluation findings/report


## Goal of this Task
The Goal of this task is not for a implementaion ready design, it's to identify a high level solution to the problem,  and it will likely requires further break down.