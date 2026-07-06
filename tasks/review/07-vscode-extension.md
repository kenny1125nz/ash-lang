# VSCode Extension

## Background

Ash scripts and task trees are just text files — but users edit them in VS Code. Without extension support, there's no syntax highlighting for `.ash` files, no way to run a script from the editor, and no integration with VS Code's build/task system. Every interaction requires switching to a terminal.

A VS Code extension closes this gap, making ash feel native to the editor. It also serves as the IDE distribution channel — alongside npm and GitHub Releases — giving users a one-click install from the VS Code Marketplace.

## Intended Solution

A VS Code extension (name: `ash-lang.vscode-ash`) with three capabilities:

1. **Syntax highlighting** — a TextMate grammar that tokenizes `.ash` files using the language's existing syntax (blocks, agent calls, string interpolation, functions, conditionals)
2. **Run and check commands** — `Ash: Run Script` executes the active editor file via `ash <file>`, and `Ash: Check Script` runs `ash --dry-run <file>`. Output appears in a VS Code output channel or integrated terminal.
3. **Task provider** (optional) — register `.ash` files as VS Code tasks so they appear in the "Run Task" picker

### Extension structure

```
ash-vscode/
├── package.json              # extension manifest
├── syntaxes/
│   └── ash.tmLanguage.json   # TextMate grammar
├── src/
│   └── extension.ts          # activate, register commands, resolve binary, spawn process
├── bin/                      # bundled binaries (one per platform, selected at runtime)
│   ├── linux-x64/ash
│   ├── darwin-x64/ash
│   ├── darwin-arm64/ash
│   └── win-x64/ash.exe
└── README.md
```

### Binary resolution strategy

| Execution context      | Binary source                                      |
| ---------------------- | -------------------------------------------------- |
| User has `ash` on PATH | Use PATH binary (no bundled binary needed)         |
| User installed via npm | PATH binary (from `npm i -g @ash-lang/cli`)        |
| No ash on PATH         | Fall back to bundled binary from `bin/<platform>/` |
| Bundled binary missing | Show error with install instructions               |

The extension tries PATH first, then falls back to the bundled binary matching `os.platform()` / `os.arch()`.

### Commands contributed

| Command           | Palette label       | Behavior                                   |
| ----------------- | ------------------- | ------------------------------------------ |
| `ash.runScript`   | `Ash: Run Script`   | Runs active editor's `.ash` file via `ash` |
| `ash.checkScript` | `Ash: Check Script` | Runs `ash --dry-run` on active editor      |
| `ash.stopScript`  | `Ash: Stop Script`  | Kills the running ash process              |

### Output channel

Output goes to a dedicated "Ash" output channel (not the integrated terminal) so it persists after execution and can be inspected without terminal scrollback. For commands that need TUI rendering (if ash ever gains one), fall back to the integrated terminal.

## Acceptance Criteria

1. **Syntax highlighting works for `.ash` files**
   - Opening any `.ash` file applies the ash grammar
   - Agent call syntax (`@agent`) is highlighted distinctly from normal text
   - Block delimiters (`{ }`), comments (`#`), and string interpolation (`${...}`) are tokenized correctly
   - Dark and light themes both look readable (use standard VS Code token scopes)

2. **`Ash: Run Script` executes the active `.ash` file**
   - Command appears in the Command Palette and editor context menu
   - Spawns `ash <filepath>` with `cwd` set to the file's directory
   - Output streams to the "Ash" output channel
   - Shows exit code and execution time when done

3. **`Ash: Check Script` runs syntax check**
   - Runs `ash --dry-run <filepath>`
   - Output shows which tasks would run without actually executing them
   - Output channel shows `[dry-run]` prefix on each line

4. **`Ash: Stop Script` kills the running process**
   - Only enabled when a script is actively running
   - Kills the child process (SIGTERM on unix, taskkill on Windows)

5. **Extension activates on `.ash` files**
   - Activation event: `onLanguage:ash` (defined in package.json)
   - Commands are registered on activation, not eagerly at startup

6. **Binary resolution falls back gracefully**
   - If ash is on PATH: uses it, no bundled binary needed
   - If not on PATH but bundled binary exists: uses bundled binary
   - If neither: shows error message with install instructions (npm, GitHub Releases)

## Implementation Hints

### Relevant project context

The extension is a **separate project** from the Rust crate at `ash/`. It lives at `vscode-ash/` at the repo root (parallel to `ash/`, `tasks/`, etc.) and does not touch the Rust codebase. It is a standard VS Code extension: Node.js, TypeScript, no framework dependencies.

**Distribution strategy** (`DISTRIBUTION.md`):
- VS Code extension bundling is part of the release workflow (after GitHub, after npm publish)
- Extension bundles platform binaries directly (unlike npm which downloads on install)
- Published via `vsce publish` from CI, uploaded to VS Code Marketplace

**Prerequisite tasks** (in ready):
- `04-github.md` — CI/CD pipeline that builds all platform binaries (the extension's `bin/` dir comes from these builds)
- `06-npm-package.md` — the PATH-resolved binary on user machines may come from npm global install

### Language grammar

- Create `syntaxes/ash.tmLanguage.json` — a TextMate grammar file
- The ash language has: agent calls (`@agent_name`), blocks (`{...}` with indentation), comments (`#`), string interpolation (`${...}`), keywords (`break`, `continue`, `return`, `if`, `for`, `else`), and function definitions (`fn name() {...}`)
- Use standard VS Code token scopes: `keyword.control`, `entity.name.function`, `string.interpolated`, `comment.line`
- Reference: TextMate grammar docs at <https://code.visualstudio.com/api/language-extensions/syntax-highlight-guide>

### Extension manifest (`package.json`)

- `activationEvents`: `["onLanguage:ash"]` (activates only when a `.ash` file is opened)
- `contributes.languages` — registers `.ash` file extension and `ash` language ID
- `contributes.grammars` — maps `ash` language to the TextMate grammar file
- `contributes.commands` — registers the three palette commands
- `main` — points to compiled `out/extension.js` (extension entry point)
- `engines.vscode` — minimum VS Code version (use `^1.85.0` or similar recent stable)

### Extension activation (`src/extension.ts`)

- `activate(context: vscode.ExtensionContext)` — the entry point
- Register all commands via `vscode.commands.registerCommand`
- Resolve ash binary path once at activation time (PATH check + fallback to bundled)
- Use `child_process.spawn()` to run ash, pipe stdout/stderr to the output channel
- Track the running process so `stopScript` can kill it

### Binary bundling

- Bundled binaries go in `bin/<platform>/ash` (or `ash.exe` on Windows)
- These are placed manually (or by CI) — the extension code only reads them
- `extension.ts` resolves `context.extensionPath + '/bin/' + platformDir + '/ash'`
- The CI build workflow (`github.md`) produces these binaries — the extension consuming task (integration) will wire them in

### What not to touch

- The Rust source code, `Cargo.toml`, build scripts, ash language grammar, and engine layer
- `DISTRIBUTION.md` — follow it, don't modify it
- CI workflows from `tasks/ready/04-github.md` — this task only creates the extension artifacts the CI will consume (the `vsce publish` step is in that task's release workflow)
