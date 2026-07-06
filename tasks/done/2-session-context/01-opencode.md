# Research: opencode session integration

Read `05-implementation.md` to understand the `session { }` block feature.

Research [opencode's official documentation](https://opencode.ai) to understand its session/context model, then update `05-implementation.md`'s `### Per-agent behavior` section:

- Add or Update `#### opencode` subsection with the concrete integration details
- What CLI flags control session behavior? (`--continue`, `--new-session`, `--session <id>`, etc.)
- How are sessions identified — working directory, explicit ID, or something else?
- Are sessions persisted? Can concurrent sessions coexist?
- How does compact interact with sessions?
- How should `OpenCodeDriver::build_command()` be updated to support `session { }` blocks?
