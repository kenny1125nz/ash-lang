# NPM Package

## Background

Ash needs to be installable via npm for users who prefer `npm i -g @ash-lang/cli` over direct binary downloads. Currently the project has no `package.json`, no shim scripts, no npm publishing setup. The distribution plan in `DISTRIBUTION.md` describes a meta-package approach where a `postinstall.js` script downloads the correct platform binary from GitHub Releases — but none of the scaffolding exists yet.

Without npm distribution, the only way to get ash is cloning the repo and compiling from source, which limits adoption. npm is the primary package manager for JavaScript/TypeScript developers and a common entry point for CLI tools.

## Intended Solution

A single npm package `@ash-lang/cli` that acts as a meta-package. It never bundles binaries — instead, `npm install` triggers a `postinstall.js` script that detects the user's platform and downloads the prebuilt binary from the latest GitHub Release.

### Package structure

```
npm-package/
├── package.json          # name, bin, postinstall, metadata
├── postinstall.js         # detect platform → fetch binary from GitHub Releases
├── ash.js                 # shim: spawns the downloaded binary (used by "bin" field)
└── README.md              # install instructions, basic usage
```

### package.json shape

```json
{
  "name": "@ash-lang/cli",
  "version": "0.1.0",
  "description": "Ash CLI — multi-agent orchestration",
  "bin": {
    "ash": "ash.js"
  },
  "scripts": {
    "postinstall": "node postinstall.js"
  },
  "files": [
    "ash.js",
    "postinstall.js",
    "README.md"
  ],
  "repository": "kenny1125nz/ash-lang",
  "license": "MIT"
}
```

### postinstall.js behavior

| Platform | Download target |
|----------|----------------|
| linux x64 | `ash-linux-x64` |
| macos x64 | `ash-darwin-x64` |
| macos arm64 | `ash-darwin-arm64` |
| windows x64 | `ash-win-x64.exe` |

- Detect platform via `os.arch()` and `os.platform()`
- Fetch from `https://github.com/kenny1125nz/ash-lang/releases/latest/download/<filename>`
- Save to the same directory as the shim script (`ash.js`)
- Set executable permission (`chmod +x`) on unix
- On download failure, print a warning (don't fail the install — the user can download manually)

### ash.js shim

A minimal Node.js script that:
- Resolves the binary path (same directory as the shim)
- On Windows, append `.exe`
- Spawn the binary with `child_process.spawn`, forwarding `process.argv.slice(2)` and stdio

```js
#!/usr/bin/env node
const { spawn } = require('child_process');
const path = require('path');
const bin = path.join(__dirname, process.platform === 'win32' ? 'ash-win-x64.exe' : 'ash');
spawn(bin, process.argv.slice(2), { stdio: 'inherit' }).on('exit', process.exit);
```

### CLI invocation

```
npm i -g @ash-lang/cli
ash run tasks/ready/99-tidyups.ash
npx @ash-lang/cli run tasks/ready/99-tidyups.ash
```

## Acceptance Criteria

1. **`package.json` is valid and publishable**
   - `bin.ash` points to `ash.js`
   - `postinstall` script runs `node postinstall.js`
   - `files` array includes only the npm artifacts (no Rust source, no tasks)

2. **`postinstall.js` downloads the correct binary**
   - Detects platform correctly on linux-x64, darwin-x64, darwin-arm64, win-x64
   - Downloads from GitHub Releases `latest` URL
   - Sets `+x` on unix (no-op on Windows)
   - Prints clear status messages during download
   - Gracefully handles missing network or 404 (warning, non-fatal)

3. **`ash.js` shim spawns the binary correctly**
   - Forwards all CLI arguments unchanged
   - Forwards stdin, stdout, stderr
   - Exits with the same exit code as the binary

4. **Global install works end-to-end**
   - `npm i -g @ash-lang/cli` completes without errors
   - Running `ash` on the PATH invokes the ash binary
   - `ash` with no arguments behaves identically to running the binary directly

5. **`npx` invocation works**
   - `npx @ash-lang/cli <args>` downloads and runs without permanent install
   - Behaves identically to global install invocation

## Implementation Hints

### Relevant project context

**Repository structure:**
- This is a Rust CLI crate at `ash/`. The npm package is independent — it does not touch the Rust codebase.
- `DISTRIBUTION.md` describes the full distribution strategy and serves as the spec.
- `tasks/ready/04-github.md` is a prerequisite task covering CI/CD and GitHub Releases. The `postinstall.js` download URL depends on GitHub Releases existing.

**Existing npm infrastructure:**
- None. No `package.json`, no npm scripts, no Node.js files exist anywhere in the repo.

**Where to create files:**
- The npm package likely goes in a `npm-package/` or `npm/` directory at the repo root (parallel to `ash/`, `tasks/`, etc.), or as a top-level package. Match the structure from `DISTRIBUTION.md`.

### Node.js considerations

- Use only Node.js built-in modules (`https`, `child_process`, `fs`, `path`, `os`) — no npm dependencies in the meta-package itself.
- `postinstall.js` should handle `HTTP 302` redirects (GitHub Releases uses redirects for `latest` URLs).
- The download filename pattern should match the CI Release asset naming (see `tasks/ready/04-github.md`).
- On Windows, the binary name includes `.exe`; on unix, it does not.

### Publishing workflow

- npm publishing happens in CI (part of the `github.md` release workflow), not manually.
- Use `npm publish --access public` since the package is scoped (`@ash-lang/cli`).
- The npm package version should track the ash binary version.

### What not to touch

- The Rust source code, `Cargo.toml`, build scripts, and test suite are unchanged.
- `DISTRIBUTION.md` is the spec — follow it, don't modify it.
- CI workflows (from `tasks/ready/04-github.md`) are owned by that task; this task only creates the npm publishing artifacts the CI will consume.
