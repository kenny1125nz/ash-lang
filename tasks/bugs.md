exec  is applying absolute path to the command
> exec "cp tmp/task-definition_2.md tasks/ready/../ashco/ideas/event-driven.md"
bash: line 1: /opt/apps/agents/ash/cp tmp/task-definition_2.md tasks/ready/../ashco/ideas/event-driven.md: No such file or directory

this should have been covered and detected by tests
