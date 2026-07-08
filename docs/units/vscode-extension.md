---
{"id": "vscode-extension", "title": "VS Code Extension"}
---

## VS Code Extension

Syntax highlighting, check, and run commands for Ash (`.ash`) agent shell scripts.

The extension provides language tooling only — the Ash runtime is a separate CLI. Install it via npm or download from GitHub Releases:

```sh
npm i -g @ash-lang/cli
```

### Features

- Syntax highlighting for `.ash` files — variables, strings, control flow, agent calls
- Run script — executes the current `.ash` file with the ash runtime
- Check script — validates syntax without executing

### Commands

| Command | Title | Description |
|---------|-------|-------------|
| `ash.runScript` | Ash: Run Script | Run the active `.ash` file |
| `ash.checkScript` | Ash: Check Script | Validate syntax of the active `.ash` file |
| `ash.stopScript` | Ash: Stop Script | Stop a running script |

Run from the Command Palette (`Ctrl+Shift+P`) or right-click an `.ash` file in the editor.
