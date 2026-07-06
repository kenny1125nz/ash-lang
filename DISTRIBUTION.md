# Distribution Plan

## 1. Build Pipeline

One CI workflow builds the `ash` binary for all targets:

| Platform | Target |
|----------|--------|
| Linux x64 | `x86_64-unknown-linux-gnu` |
| macOS x64 | `x86_64-apple-darwin` |
| macOS ARM | `aarch64-apple-darwin` |
| Windows x64 | `x86_64-pc-windows-msvc` |

Output: `ash` (linux/mac) or `ash.exe` (windows) per platform.

## 2. npm Package

**Package name:** `@ash-lang/cli`

**Approach:** One meta-package with a `postinstall.js` script.

```
@ash-lang/cli/
  package.json          # name, bin, postinstall
  postinstall.js         # detect platform → download correct binary from GitHub Releases
  ash.js                 # shim: spawns the downloaded binary (used by "bin" field)
  README.md
```

- `bin: { "ash": "ash.js" }` → `npx ash run script.ash`
- Binary downloaded on `npm install` to a `.bin/` sibling
- No platform sub-packages (keeps it simple — single package, lazy download)

**Publish:** `npm publish` from CI after build.

## 3. VS Code Extension

**Name:** `ash-lang.vscode-ash`

```
vscode-ash/
  package.json           # extension manifest
  syntaxes/
    ash.tmLanguage.json  # TextMate grammar from existing tokenizer
  src/
    extension.ts         # activate: register commands, resolve binary, spawn process
  bin/                   # bundled binaries (one per platform, selected at runtime)
    linux-x64/ash
    darwin-x64/ash
    darwin-arm64/ash
    win-x64/ash.exe
  README.md
```

**Features:**
- Syntax highlighting for `.ash` files
- Command: "Ash: Run Script" → runs active editor file
- Command: "Ash: Check Script" → `--dry-run` mode
- Task provider (optional): `.ash` files as build tasks
- Output shown in VS Code output channel or integrated terminal

**Publish:** `vsce publish` from CI, uploaded to VS Code Marketplace.

## 4. Release Workflow

```
[tag push] → Build matrix (4 platforms) → 
  1. Upload binaries to GitHub Release
  2. npm publish @ash-lang/cli
  3. vsce publish vscode-ash/
```

- Binary lives in GitHub Releases (single source of truth)
- npm `postinstall.js` fetches from the latest GitHub Release
- VS Code extension bundles binaries directly (or also fetches on first activation)

## 5. Future

- Language Server (LSP) for richer IDE support
- Homebrew formula (`brew install ash-lang`)
- Docker image
