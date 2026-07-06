# Research: Aider session integration

Read `05-implementation.md` to understand the `session { }` block feature.

Research [Aider's official documentation](https://aider.chat/docs) to understand its session/chat model, then update `05-implementation.md`'s `### Per-agent behavior` section:

- Add an `#### aider` subsection
- How does `--chat-history` work? Can it resume prior sessions?
- Does aider have `--continue`, `--resume`, `--load <file>`, `--new-chat` flags?
- How does `--message` / `--msg` interact with existing chat history?
- How does aider scope sessions — per directory, per file?
- How should `AiderDriver::build_command()` be updated for `session { }` blocks?
