# Research: Aider discovery

Read `05-implementation.md` to understand the agent discovery feature.

Research [Aider's official documentation](https://aider.chat/docs) to understand how it can be detected and probed, then update `04-implementation.md`'s `#### aider` subsection with:

- Binary name and common install paths (pip, etc.)
- How to detect it's installed (PATH lookup, `which aider`)
- How to probe capabilities: `--version`, `--help`, or other discovery flags
- What capabilities to report (model support, session-like chat history, etc.)
