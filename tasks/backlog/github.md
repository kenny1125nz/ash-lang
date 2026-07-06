# GitHub

## Background

Ash has no public GitHub presence. The repository exists locally with a single commit, but lacks the artifacts expected of an open-source project: no README, no license, no contributing guide, no CI/CD pipeline. Without these, the project cannot be shared, discovered, or trusted by users — and the distribution plan described in `DISTRIBUTION.md` has no automation to build, test, or publish releases.

Setting up the GitHub repo with standard open-source signals and a CI/CD pipeline is a prerequisite for npm publishing, VS Code extension publishing, and community adoption.

## Intended Solution

Create a public GitHub repository for ash with the following components:

1. **Repository setup** — README (what ash is, quick start, supported agents), MIT license, CONTRIBUTING.md, `.gitignore` (already exists with `target/`)
2. **CI pipeline** — a GitHub Actions workflow that runs on every push and PR:
   - `cargo build` and `cargo test` on ubuntu-latest
   - Matrix build for all distribution targets (linux-x64, macos-x64, macos-arm64, windows-x64) on tag push
   - Upload compiled binaries as GitHub Release artifacts
3. **Release workflow** — on version tag push (e.g. `v0.1.0`):
   - Build all platform binaries
   - Create GitHub Release with binaries attached
   - (Future hook points for `npm publish` and `vsce publish`)

### Repository structure after setup

```
ash/                   # repo root (Rust workspace or single crate)
├── .github/
│   └── workflows/
│       └── ci.yml     # build, test, release
├── .gitignore         # (already exists)
├── LICENSE            # MIT
├── README.md
├── CONTRIBUTING.md
├── ash/
│   ├── Cargo.toml
│   └── src/
├── tasks/
└── DISTRIBUTION.md    # (already exists)
```

## Acceptance Criteria

1. **GitHub repo is public and contains standard files**
   - `README.md` with project description, quick install, basic usage
   - `LICENSE` (MIT)
   - `CONTRIBUTING.md` with setup instructions and CLA/DCO if needed

2. **CI runs on push and PR**
   - Workflow triggers on `push` to main and `pull_request`
   - Runs `cargo build` and `cargo test` on ubuntu-latest
   - Build failures and test failures are reported as CI status checks

3. **Release workflow builds all distribution targets**
   - Triggered by pushing a version tag (`v*`)
   - Builds `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`
   - Creates a GitHub Release with all four binaries attached
   - Release body includes a changelog or commit summary

4. **CI passes on the current codebase**
   - `cargo build` succeeds (currently only depends on `regex`)
   - `cargo test` passes (any existing tests)

## Implementation Hints

### Relevant project context

The project is a single Rust crate at `ash/`. Builds with `cargo build` and `cargo test` from that directory. The only dependency is `regex`.

**Existing build configuration:**
- `ash/Cargo.toml` — package name `ash`, version `0.1.0`, edition 2021, depends on `regex = "1"`
- `.gitignore` — only `target/`

**Distribution plan:**
- `DISTRIBUTION.md` — describes the full pipeline: GitHub Releases as the single source of truth for binaries, npm `postinstall.js` downloading from Releases, VS Code extension bundling binaries. The GitHub CI workflow is the foundation this depends on.

**No existing CI or GitHub integration:**
- No `.github/` directory
- No remote configured (repo is local-only with one commit)

### CI workflow guidelines

- Place workflow at `.github/workflows/ci.yml`
- Use `actions/checkout@v4` for checkout
- Use `actions-rs/toolchain@v1` or just `rustup` for Rust toolchain setup
- Use `softprops/action-gh-release@v2` for creating releases
- Test job: run `cargo build --verbose && cargo test --verbose` inside `ash/` directory (set `working-directory` or `cwd`)
- Release job: `cargo build --release --target <triple>` for each platform, upload artifacts with `actions/upload-artifact@v4`, then attach to release

### What not to touch

The Rust source code, `Cargo.toml`, `.gitignore`, and task files are unchanged — this is purely repository infrastructure.
