# Research: Claude Code session integration

Read `05-implementation.md` to understand the `session { }` block feature.

Research [Claude Code's official documentation](https://docs.anthropic.com/en/docs/claude-code) to determine if it has any session/context continuation, then update `05-implementation.md`'s `### Per-agent behavior` section:

- Add a `#### claude-code` subsection
- Does it have `--continue`, `--resume`, `--session`, or similar flags?
- Any "projects", "conversations", or "resume" concept in the CLI?
- Any mechanism to carry context between invocations?
- If no session support: should driver silently ignore or warn? Any partial workaround?
