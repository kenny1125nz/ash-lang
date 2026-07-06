# Research: opencode discovery

Read `05-implementation.md` to understand the agent discovery feature.

Research [opencode's official documentation](https://opencode.ai) to understand how it can be detected and probed, then update `04-implementation.md`'s `#### opencode` subsection with:

- Binary name and common install paths (npm, cargo, etc.)
- How to detect it's installed (PATH lookup, `which opencode`)
- How to probe capabilities: `--version`, `--help`, or other discovery flags
- What capabilities to report (model support, session support, etc.)
