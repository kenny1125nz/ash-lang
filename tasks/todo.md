# Feature to do list

## session/context
Currently, ash's agent jobs are independent one shot requests, for workflows requires knowlege of prior steps, it depends on the agent's exploring capability, which makes the workflow less token efficient. For some AI agent, it provide capability to cotinue on previous session (like opencode), but there is still caveat that , 
- we need option to turn the continue flag on and off as part of the ash language , we can't keep it on  and use one session for all tasks
- Concurrent requests might cause contention 

the challenges here are :
- we need provide native session/context control keywords in the ash language
- different agent might have different session/context controlling mechanism, some might not support at all (like claude code)
- the impact to the compact behavior,  for opencode, compact outside of session has no value  

## Agent Discovery
To make ash ready to use right after unbox, it need provide agent discovery and self config capability. 


## Directory Based orchestration
Besides script based orchestration, a more simple and natural way of organizing tasks is using folders and files, which naturally form a tree, perfect aligned with task decomposition
With folloing file structure, we should be able to create another application ( or build it ast part of ash) to just load the tree and interate over the files send to agent one by one.
although we need figure out how to specify things like agent/model to use, compacting behavior
```
── tasks
    ├── 1-types
    │   ├── 01-token.md
    │   ├── 02-value.md
    │   ├── 03-ast.md
    │   ├── 04-compact.md
    │   └── 05-scope.md
    ├── 2-transforms
    │   ├── 01-lexer.md
    │   ├── 02-interp.md
    │   └── 03-parser.md
    └── 6-advanced
        ├── 01-try-blocks.md
        ├── 02-wait-bg.md
        └── 04-integration.md

```

## packaged for NPM install


## packages for VScode extention


## GitHub