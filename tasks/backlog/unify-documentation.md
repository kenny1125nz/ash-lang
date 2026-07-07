# Unify documentation across website, GitHub README, npm README, and VS Code README

The four docs currently have inconsistent taglines, agent lists, folder numbering conventions, config formats, and section organization. They should follow a unified structure adapted to each medium.

## Proposed unified structure

Each document follows this flow, adapted to its audience:

```
1. TAGLINE (unified across all)
2. Install
3. Quick Start (--agent + folder structure)
4. Agent Configuration (built-in list → ash discover → --agent → ash.yml)
5. Writing Tasks (.md frontmatter → .ash shebang)
6. Language Examples (varies by audience depth)
7. Advanced (CLI flags, REPL, etc. — only where relevant)
8. Appendix (build from source, license, manual download — per doc)
```

## Current inconsistencies

| Aspect | GitHub README | NPM README | Website | VS Code README |
|--------|--------------|-----------|---------|---------------|
| Tagline | "Deterministic agent orchestration..." | "Task runner for AI agents..." | "Task runner for AI agents" | No tagline |
| Agent list | aider (outdated), no codex/gemini/kimi | Correct | Correct | Duplicate lines |
| Folder numbering | `1-plan/` style | `01-plan/` style | Flat `01-research.md` | None |
| Config format | `default_agent:` (old) | `--agent` only | `ash.yml` agents block | Both old and new |
| CLI flags | Has section | None | None | None |
| `ash discover` | Not mentioned | Not mentioned | Yes | Yes |

## Key changes needed

1. Unify tagline phrase across all four
2. Update GitHub README agent table + folder numbering to match others
3. Remove duplicate agent lines in VS Code README
4. Move `--agent` into the `ash ./tasks` line in NPM README
5. Add `ash discover` to NPM and GitHub READMEs

## Files

- `/opt/apps/agents/ash/README.md` — GitHub repo README
- `/opt/apps/agents/ash/npm-package/README.md` — npm package README
- `/opt/apps/agents/ash/ash-vscode/README.md` — VS Code extension README
- `/opt/apps/agents/ash/web-site/index.html` — website landing page
