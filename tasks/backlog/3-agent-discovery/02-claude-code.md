# Research: Claude Code discovery

Read `05-implementation.md` to understand the agent discovery feature.

Research [Claude Code's official documentation](https://docs.anthropic.com/en/docs/claude-code) to understand how it can be detected and probed, then update `04-implementation.md`'s `#### claude-code` subsection with:

- Binary name and common install paths (npm, etc.)
- How to detect it's installed (PATH lookup, `which claude`)
- How to probe capabilities: `--version`, `--help`, or other discovery flags
- What capabilities to report (model support, session support: none, etc.)
