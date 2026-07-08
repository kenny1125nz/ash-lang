# Unify Documentation Across Publishing Channels

## Background

Ash publishes to four surfaces — npm (`@ash-lang/cli`), GitHub (repo root), a Cloudflare Pages website, and the VS Code Marketplace (`vscode-ash`). Each carries a README (or HTML landing page for the website) that describes the same underlying feature set. Currently these are written and maintained independently as standalone markdown files:

| Surface | File | Current audience |
|---------|------|------------------|
| GitHub | `README.md` | Developers evaluating adoption; full feature walkthrough |
| npm | `npm-package/README.md` | Users wanting to install and start immediately |
| VS Code | `ash-vscode/README.md` | Extension users wanting syntax/command reference |
| Website | `web-site/index.html` | Visitors seeking problem/solution framing |

This creates two problems:

1. **Synchronization fatigue.** When a feature is added or changed (e.g. the `using` clause in `.ash` scripts), the author must manually propagate that change across four files. Each surface drifts at a different rate, producing inconsistent and contradictory descriptions.

2. **Accuracy risk.** The existing READMEs were generated or assisted by AI and contain plausible-but-wrong statements. With four independent copies, catching and fixing each inaccuracy requires four separate corrections. There is no single source of truth to audit against.

The stated requirement: centralize documentation units covering ash features, specs, and examples; assemble the relevant subset into each target output based on that target's purpose.

## Intended Solution

### Architecture overview

```
docs/
├── units/                  # Single source of truth — one file per feature/concept
│   ├── what-is-ash.md
│   ├── install.md
│   ├── quick-start.md
│   ├── writing-tasks.md
│   ├── ... (feature units)
│   └── license.md
├── targets/                # Assembly configuration — one JSON file per output
│   ├── github.json
│   ├── npm.json
│   ├── vscode.json
│   └── website.json        # Markdown reference (NOT HTML — for website maintainers)
└── assemble.js             # Vanilla Node.js script — reads units, writes outputs
```

Outputs:
```
docs/units/*.md  ──[assemble.js + github.json]───→  README.md
docs/units/*.md  ──[assemble.js + npm.json]──────→  npm-package/README.md
docs/units/*.md  ──[assemble.js + vscode.json]───→  ash-vscode/README.md
docs/units/*.md  ──[assemble.js + website.json]──→  docs/website-reference.md
```

### 1. Content units (`docs/units/*.md`)

Each file covers exactly one feature or concept. Files are self-contained — they do not reference other unit files via relative paths (headings within the same file are fine).

JSON frontmatter (no YAML dependency):

```
---
{"id": "install", "title": "Installation"}
---

## npm

```sh
npm install -g @ash-lang/cli
```

## Prebuilt binaries

Download from [GitHub Releases](https://github.com/kenny1125nz/ash-lang/releases).
```

| Field | Required | Purpose |
|-------|----------|---------|
| `id` | yes | Unique identifier across all units. Must match `[a-z0-9-]+`. Matches the filename stem by convention. |
| `title` | yes | Human-readable label for this unit. Used in error messages (`unit "Installation" is missing...`) and by authors scanning the directory. Not injected into assembled output — the unit body carries its own headings. |
| (any other field) | no | Ignored. Reserved for future metadata. |

**No per-target field in the unit.** Which units go into which output is decided solely by the target config (section 2). The target config's `units` array is the single authority for both membership and ordering.

**Heading level convention — enforced by the assembler:** Unit body headings must start at `##` (h2), never `#` (h1). Each target config's `title` field provides the sole h1 heading for the assembled output. Units that use `#` will create structurally broken markdown with sibling h1 headings. The assembler validates this: any unit whose body (first non-whitespace line after frontmatter) starts with `# ` (h1) produces an error.

**Heading order convention — not enforced:** All headings within a unit should maintain correct nesting (h2 → h3 → h4, no skipping levels). This is a style rule for unit authors, not validated by the assembler (markdown renderers handle skipped levels gracefully).

The directory is flat — no subdirectories. ~18 files fit comfortably in one directory.

### 2. Target configuration (`docs/targets/*.json`)

One JSON file per output:

```json
{
  "output": "README.md",
  "title": "# Ash\n\n**Deterministic agent orchestration — structured task files, scriptable when you need more.**\n\n",
  "units": [
    "what-is-ash",
    "install",
    "quick-start",
    "writing-tasks",
    "scripting",
    "supported-agents",
    "building-from-source",
    "license"
  ],
  "footer": ""
}
```

| Field | Required | Purpose |
|-------|----------|---------|
| `output` | yes | Path relative to repo root where the assembled markdown is written. |
| `title` | yes | Raw markdown prepended before any unit content. Contains the repo/package heading, tagline, and any target-specific framing. This is where audience tone is set — the `title` can be as long as needed to establish the right voice for the target. |
| `units` | yes | Ordered list of unit IDs to include. The list defines the final sequence — the assembler reads units in this order. Units not listed are excluded. Duplicate entries are an error. |
| `footer` | yes | Raw markdown appended after all unit content. Set to `""` (empty string) if no footer is needed. |

**On the `title` field format:** The `title` is a single JSON string with `\n` escape sequences for paragraph breaks. For short titles (1-2 lines) this is ergonomic. For longer framing text (3+ paragraphs), embedded escapes become visually noisy and error-prone. If a target needs substantial framing, the implementer may choose to put the content in a dedicated unit file instead (listed first in `units`) and keep the `title` short. Both approaches are valid — the assembler treats `title` and unit bodies identically during concatenation.

Target-to-output mapping:

| Config file | Output path | Purpose |
|-------------|-------------|---------|
| `docs/targets/github.json` | `README.md` | Full-spec GitHub README |
| `docs/targets/npm.json` | `npm-package/README.md` | Minimal install+usage for npm |
| `docs/targets/vscode.json` | `ash-vscode/README.md` | Extension features and commands |
| `docs/targets/website.json` | `docs/website-reference.md` | Feature reference for website maintainers |

Note: `docs/website-reference.md` is placed under `docs/`, not `web-site/`. Cloudflare Pages deploys the entire `web-site/` directory, so placing generated files there would publish internal documentation to the public website.

### 3. Assembly script (`docs/assemble.js`)

Vanilla Node.js (zero dependencies), matching the existing `npm-package/postinstall.js` style.

Behaviour:

1. Collects all errors into a list. The script never exits early on a single validation failure — every check contributes to the final error set.
2. Checks that `docs/units/` and `docs/targets/` directories exist. If either is missing, the error is collected (no raw Node.js crash) and the corresponding read step is skipped. For example, if `docs/units/` is missing, unit-level checks are skipped but target config parsing still proceeds for any targets that exist.
3. Reads all `docs/units/*.md` files (if the directory exists), extracting JSON frontmatter and body.
4. Reads all `docs/targets/*.json` files (if the directory exists), parsing each as JSON.
5. Runs all validation checks, collecting every error encountered:
   - Every unit has frontmatter delimiters (`---` opening and closing).
   - Every unit's frontmatter is valid JSON (`JSON.parse` succeeds).
   - Every unit has required fields `id` and `title` present and non-empty.
   - Every unit's `id` matches `[a-z0-9-]+` (no spaces, no special characters except hyphens).
   - No two units share the same `id`.
   - Every unit body's first non-whitespace line after frontmatter starts with `## ` (h2) — not `# ` (h1). An empty body (frontmatter only) is valid and skipped for this check.
   - Every target config file is valid JSON.
   - Every target config has required fields `output`, `title`, `units`, and `footer` present (all four are required; `footer` may be an empty string).
   - Every target config's `units` field is an array.
   - Every target config's `units` array has no duplicate entries.
   - Every unit ID listed in any target config's `units` array exists in `docs/units/`.
   - Every target config's `output` path has an existing parent directory. The assembler does not create directories.
6. If any errors were collected, prints all `error:` lines to stderr and exits 1. No files are written.
7. If validation passes:
   a. For each target config, assembles output: `config.title` + body of each listed unit (frontmatter stripped, joined with double newlines) + `config.footer`.
   b. Writes each output file, printing `wrote: <path>` to stdout for each file written.
   c. Checks for orphaned units (exist in `docs/units/` but not referenced by any target) and prints `warn: <message>` to stderr for each.
   d. Exits 0.

Error output example:

```
$ node docs/assemble.js
error: docs/units/ directory not found at /path/to/repo/docs/units/
error: unit "installer" listed in docs/targets/npm.json but no unit with that id exists in docs/units/
error: unit "what-is-ash" (docs/units/what-is-ash.md) is missing required field "title"
error: unit "mixed-case" (docs/units/mixed-case.md): id "MixedCase" does not match required pattern [a-z0-9-]+
error: unit "broken" (docs/units/broken.md): frontmatter is not valid JSON — Unexpected token '}' at position 14
error: duplicate unit id "install" found in docs/units/install.md and docs/units/install-alt.md
error: unit "bad-heading" (docs/units/bad-heading.md): body starts with h1 heading ("# "). Units must use h2+ (##).
error: docs/targets/npm.json: duplicate unit id "install" in units array
error: docs/targets/vscode.json: missing required field "footer"
error: output path "npm-package/README.md": parent directory "npm-package" does not exist
```

Success output example:

```
$ node docs/assemble.js
wrote: README.md
wrote: npm-package/README.md
wrote: ash-vscode/README.md
wrote: docs/website-reference.md
warn: unit "experimental-feature" (docs/units/experimental-feature.md) is not referenced by any target
```

**Why vanilla Node.js?** The project already has a vanilla Node.js script (`npm-package/postinstall.js`) with `require('fs')`/`require('path')`. No new runtime or package manager dependency. The JSON frontmatter avoids any external YAML parser.

### 4. Website handling

The website (`web-site/index.html`) is **not** auto-generated as HTML by the assembler — it is a fundamentally different format (HTML landing page with embedded WASM playground, CSS, JavaScript). Forcing markdown→HTML conversion adds a template engine, CSS concerns, and WASM embed awareness that the requirement does not ask for.

Instead, `docs/targets/website.json` assembles all feature unit content into `docs/website-reference.md` — a single markdown file under the `docs/` directory. When updating the website's HTML, the maintainer opens this file as the authoritative reference for feature descriptions. This ensures the website's copy stays consistent with the other surfaces without the assembler needing to understand HTML structure. The file lives outside `web-site/` to avoid being deployed by Cloudflare Pages alongside the production website.

This is a partial automation — the website maintainer still manually updates HTML, but now has one file to check instead of three. It addresses the original requirement's intent (website content stems from the same centralized units) without over-engineering into HTML generation, which would be scope creep.

### 5. Tone across audiences

The four targets serve different audiences with different expectations. However, the design deliberately uses the same unit body content across all targets, with per-target `title` and `footer` fields providing audience-specific framing.

| Target | Tone set by `title` | Tone carried by units |
|--------|---------------------|-----------------------|
| GitHub | Full-spec heading, "Deterministic agent orchestration" | Neutral documentation voice |
| npm | "Install and start in 30 seconds", encouraging | Neutral documentation voice |
| VS Code | "Extension for Ash agent shell scripts" | Neutral documentation voice |
| Website | (maintainer reference, not end-user facing) | Neutral documentation voice |

**Trade-off acknowledged:** This sacrifices per-audience optimization of unit content for the simplicity of a single version of truth. The alternative — per-target overrides for specific unit sections — would reintroduce the synchronization problem this system exists to solve. The unit authors should write in a neutral, direct documentation voice (modeled after `ash/docs/ash.md` — example-driven, minimal fluff). Per-target framing lives in the `title` and `footer` fields. For targets needing longer framing than a few lines, the framing can be placed in a dedicated unit file (listed first in `units`) instead of in the JSON `title` field.

### 6. Git and CI workflow

Generated files are **committed to the repository**. The assembler is run manually before committing doc changes:

```sh
node docs/assemble.js
```

CI adds a single verification step to the existing `build-and-test` job. **Critical:** the existing job sets `defaults.run.working-directory: ash`, so the doc-check step must explicitly override to the repo root:

```yaml
- name: Verify docs are in sync
  working-directory: ./
  run: |
    node docs/assemble.js
    git diff --exit-code README.md npm-package/README.md ash-vscode/README.md docs/website-reference.md
```

This catches drift — someone edits a unit but forgets to regenerate. The `working-directory: ./` override is necessary because `docs/` and the output paths are all relative to the repo root, not the `ash/` crate subdirectory.

This is a minimal addition to CI — one step in an existing job. No new workflow, no new job, no new trigger conditions.

### 7. Content migration inventory

Existing content is decomposed from source files into ~18 unit files:

| Unit ID | Source material | Included in configs |
|---------|----------------|---------------------|
| `what-is-ash` | README.md L3–7 | github |
| `install` | README.md L28–32, npm-package/README.md L7–11 | github, npm |
| `quick-start` | README.md L34–82, npm-package/README.md L15–49 | github, npm |
| `writing-tasks` | README.md L85–121 | github |
| `scripting` | README.md L123–136 | github, npm |
| `supported-agents` | README.md L144–151, npm-package/README.md L17 | github, npm |
| `building-from-source` | README.md L153–161 | github |
| `license` | README.md L163–165 | github |
| `directory-mode` | ash/docs/ash.md (directory-based orchestration section) | github |
| `ash-scripts` | ash/docs/ash.md (ash script language section) | github |
| `variables` | ash/docs/ash.md (variables + strings) | github |
| `control-flow` | ash/docs/ash.md (control flow) | github |
| `functions` | ash/docs/ash.md (functions) | github |
| `builtins` | ash/docs/ash.md (built-in tools) | github |
| `sessions` | ash/docs/ash.md (session blocks) | github |
| `parallel` | ash/docs/ash.md (background & parallel) | github |
| `file-prompts` | ash/docs/ash.md (file-based prompts) | github |
| `repl` | ash/docs/ash.md (REPL) | github, npm |
| `vscode-extension` | ash-vscode/README.md (all) | vscode |

The "Included in configs" column is a recommendation, not a rule — the JSON configs are authoritative.

Note on `scripting` in npm: The npm README's audience wants a brief language overview, not a full reference. The `scripting` unit must therefore be written as a concise overview suitable for both GitHub (as an intro to the language section) and npm (as the sole language content). The detailed language feature units (`variables`, `control-flow`, `functions`, etc.) are GitHub-only and provide the full reference depth.

The `website.json` config should include all units (or a generous subset) to serve as the comprehensive reference for website maintainers.

### Trade-offs and rationale

| Decision | Rationale |
|----------|-----------|
| **JSON frontmatter, not YAML** | Node.js stdlib has `JSON.parse`. No external parser dependency. The frontmatter shape (2 flat fields) is trivially JSON. Consistent with `package.json` and `tsconfig.json` already in the repo. |
| **Only `id` and `title` required in frontmatter** | Minimum viable metadata. `title` is a human-readable label (used in error messages and by authors), not injected into output. Ordering is determined by the target config's `units` array — the single authority. |
| **Target config `units` array determines order** | Simple and explicit. The assembler reads units in the listed order, no sorting logic needed. The array is the single decision point for both membership and sequence. |
| **No per-target field in unit frontmatter** | Avoids split-brain: if a unit says `targets: [npm]` but `github.json` lists it too, which wins? The target config is the sole authority. |
| **Collect all errors, never exit early** | Better developer experience. Running the assembler after bulk changes surfaces all problems at once, including directory-not-found errors alongside content-level issues. Consistent with "Surface, Don't Hide" — surface everything wrong in a single run. |
| **Orphaned units are a warning, not an error** | A unit might be created before a target config is updated (forward compatibility). Warning makes the situation visible without blocking valid work. |
| **Assembler prints `wrote: <path>` to stdout on success** | "Surface, Don't Hide" — the assembler mutates files on disk and must announce what it did. Running the assembler and seeing no output would be disorienting. |
| **h1 heading in unit body is an error** | Structural correctness. An h1 in a unit creates a sibling h1 alongside the target title's h1, producing broken document structure. Detecting this is a single-line check on the body's first heading. |
| **`id` format validated by assembler** | `[a-z0-9-]+` is checked at assembly time. Prevents filenames with spaces or special characters from being used as IDs, which would break cross-referencing. |
| **`footer` is a required field (may be `""`)** | Avoids `undefined` in string concatenation. Every target config must explicitly state its footer, even if empty. |
| **Duplicate IDs within a `units` array are an error** | Duplicate entries are almost certainly a copy-paste mistake. Catching this prevents silently including the same content twice. |
| **Missing `docs/units/` or `docs/targets/` is a collected error** | Produces a clean error message instead of a raw Node.js crash. Collected alongside other errors so the user sees all problems at once, not just the first. |
| **`website-reference.md` under `docs/`, not `web-site/`** | Cloudflare Pages deploys `web-site/` in its entirety. Placing generated files there would publish internal documentation to the public website. |
| **Flat unit directory** | ~18 files fit comfortably. No need for subdirectories, no recursive walk, no path-matching complexity. Can add subdirs later if it grows beyond ~40 files. |
| **Generated outputs committed to repo** | Keeps the repo self-contained. A contributor cloning the repo sees the canonical READMEs without running the assembler. CI verifies freshness. |
| **No website HTML generation** | Different format, different structure. Forcing markdown→HTML adds a template engine, CSS concerns, and WASM embed logic — none of which the requirement asks for. `docs/website-reference.md` provides an authoritative reference without the complexity of HTML generation. |
| **Neutral documentation voice in units** | Sacrifices per-audience tone optimization for simplicity. Per-target `title`/`footer` provide audience-specific framing. Long framing can live in a dedicated unit file instead of the JSON `title` string. |
| **`ash/docs/*.md` untouched** | Those files serve a different audience (Rust contributors, architecture decisions). Not in scope for user-facing feature docs. |

## Acceptance Criteria

1. **`docs/units/` exists** with at least 15 markdown files, each covering one feature or concept. No file is empty. Every unit body starts with a `##`-level heading (no `#` headings).

2. **Each unit file begins with a JSON frontmatter block** delimited by `---` lines. Required fields `id` and `title` are present and non-empty. `id` matches `[a-z0-9-]+`. No YAML frontmatter anywhere.

3. **No two unit files share the same `id`.** Enforced by the assembler (error, exit 1).

4. **`docs/targets/github.json`** exists with a `units` array covering the full feature set. Running `node docs/assemble.js` produces `README.md` that contains a heading for every feature described in the current `README.md` (install, quick start, writing tasks, scripting, supported agents, building from source, license).

5. **`docs/targets/npm.json`** exists with a minimal `units` subset. Running the assembler produces `npm-package/README.md` that covers install, quick usage, and a brief language overview — matching the scope of the current `npm-package/README.md` (no build-from-source, no full language reference).

6. **`docs/targets/vscode.json`** exists with `units` covering extension features, commands, and CLI relationship. Running the assembler produces `ash-vscode/README.md` that covers syntax highlighting, run/check/stop commands, and agent configuration — matching the scope of the current `ash-vscode/README.md`.

7. **`docs/targets/website.json`** exists, assembles all units (or a generous subset), and writes `docs/website-reference.md` as a markdown reference file. The file is **not** placed inside `web-site/`.

8. **`node docs/assemble.js` exits 0**, prints a `wrote: <path>` line to stdout for each output file written, and produces all four output files.

9. **`node docs/assemble.js` exits 1** and prints one or more `error:` lines to stderr when any of the following occur:
   - `docs/units/` or `docs/targets/` directory does not exist.
   - A target config references a unit ID that does not exist.
   - A unit is missing a required frontmatter field (`id`, `title`).
   - A unit's `id` does not match `[a-z0-9-]+`.
   - A unit's frontmatter contains invalid JSON.
   - A unit has no frontmatter delimiters at all (no `---` block).
   - Duplicate `id` values are found across unit files.
   - A unit body starts with an h1 heading (`# `) instead of h2 (`## `).
   - A target config is not valid JSON or is missing required fields (`output`, `title`, `units`, `footer`).
   - A target config's `units` field is not an array or contains duplicate entries.
   - A target output path's parent directory does not exist.

10. **`node docs/assemble.js` exits 0 but prints `warn:` lines to stderr** when a unit file in `docs/units/` is not referenced by any target config. The warning does not block assembly.

11. **CI verification step** exists in `.github/workflows/ci.yml` (within the existing `build-and-test` job, with `working-directory: ./` override) that runs `node docs/assemble.js && git diff --exit-code README.md npm-package/README.md ash-vscode/README.md docs/website-reference.md`.

12. **The three README outputs** (`README.md`, `npm-package/README.md`, `ash-vscode/README.md`) contain a section for every feature and command described in the corresponding current standalone file. Content may be reorganized and reworded, but no feature, command, or concept present in the current files is omitted.

13. **`npm-package/package.json` `"files"` array** still includes `"README.md"` — the generated output is at the same path. No change needed.

14. **No code changes** to the following, except the CI step addition: `ash/` Rust source, `ash-wasm/`, `ash-vscode/src/`, `web-site/` HTML/CSS/JS, `npm-package/postinstall.js`, `refinery/`, `tasks/`.

## Implementation Hints

### Relevant file paths

| Path | Role |
|------|------|
| `docs/` | **New** root for the centralized doc system |
| `docs/units/*.md` | **New** content unit files with JSON frontmatter |
| `docs/targets/*.json` | **New** per-target assembly configs (github, npm, vscode, website) |
| `docs/assemble.js` | **New** vanilla Node.js assembly script |
| `docs/website-reference.md` | **New generated file** — reference for website maintainers (NOT in web-site/) |
| `README.md` | **Regenerated** — output for GitHub |
| `npm-package/README.md` | **Regenerated** — output for npm |
| `ash-vscode/README.md` | **Regenerated** — output for VS Code |
| `.github/workflows/ci.yml` | **Modified** — add doc-verification step (with `working-directory: ./`) to `build-and-test` job |
| `npm-package/package.json` | **Review only** — verify `"README.md"` remains in `"files"` array |

### Existing patterns to follow

- **Vanilla Node.js**: The assembler uses only `require('fs')` and `require('path')`. Same pattern as `npm-package/postinstall.js`. Zero dependencies. No `npm install` needed.
- **Error style**: Print `error: <message>` to stderr, collect all errors including directory-not-found, exit 1. Never exit early on the first error — surface everything at once. Print `warn: <message>` for non-blocking issues. Print `wrote: <path>` to stdout on success.
- **Content unit prose**: Model after `ash/docs/ash.md` — direct, example-driven, minimal fluff. Write in neutral documentation voice — avoid marketing language (that belongs in target config `title` fields or dedicated framing units). Every unit body opens with a `##` heading matching the unit's `title` field.
- **Frontmatter extraction**: Split file on the first two `---` lines. Content between them is the JSON string. Lines before the first `---` are body content (included in assembly). JSON parse errors produce hard validation failures. If the file does not contain two `---` lines, it's an error (no frontmatter).
- **Unit file naming**: Stem matches `id` by convention: `install.md` → `"id": "install"`. Not enforced by the assembler (the `id` field is authoritative), but following the convention makes the directory scannable.

### What NOT to touch

- `ash/` — Rust source, Cargo.toml, `ash/docs/*.md` (architecture/contributor docs are separate)
- `ash-vscode/src/` — TypeScript extension source, grammar, icons, `package.json`
- `web-site/` — `index.html`, `css/`, `js/`, `_headers`, `wasm/` artifacts, Cloudflare config. Do **not** add generated files to this directory.
- `refinery/` — task refinement pipeline
- `tasks/` — development task definitions
- `npm-package/package.json` — except reviewing `"files"` array, do not change
- `npm-package/postinstall.js` — binary download logic
- `npm-package/ash.js` — CLI shim
- `.github/workflows/ci.yml` job structure — add only one step within `build-and-test` with `working-directory: ./`, do not restructure jobs or triggers
- `ash-wasm/` — WASM crate
- `DISTRIBUTION.md` — distribution plan (out of scope)
- `CONTRIBUTING.md` — contributor guide (out of scope)

### Edge cases the design handles

- **Missing `docs/units/` or `docs/targets/`**: Error collected alongside all other validation errors. The script continues checking whatever it can (e.g., if `units/` is missing but `targets/` exists, target configs are still validated). No raw Node.js crash.
- **Empty `units` array in target config**: Valid — produces output with only `title` + `footer`.
- **Unit file with no frontmatter**: Error — `id` and `title` cannot be extracted. The assembler detects missing `---` delimiters.
- **Unit file with malformed JSON in frontmatter**: Error — file path and `JSON.parse` error message printed.
- **Unit body using `# ` heading (h1)**: Error — the assembler checks the first non-whitespace line after frontmatter. An entirely empty body is valid and skips this check.
- **Unit `id` with invalid characters**: Error — `[a-z0-9-]+` enforced. Uppercase, spaces, underscores, or special characters are rejected.
- **Duplicate unit ID in `units` array**: Error — `duplicate unit id "<id>" in units array` printed for each occurrence.
- **Missing `footer` field in target config**: Error — `footer` is a required field (empty string is valid).
- **Extra/unrecognized fields in frontmatter**: Silently ignored. Forward compatibility.
- **Target `output` path pointing to a missing directory**: Error with a clear message. The assembler does not create directories.
- **Unit file with content before the opening `---`**: Treated as body content (included in assembled output). Valid pattern if needed.
- **Multi-line JSON frontmatter**: `JSON.parse` handles this natively.
- **Unicode/emoji in unit body**: Survives assembly unchanged. `fs.readFileSync` with `utf-8` handles this.
- **CI step `working-directory`**: Must be explicitly set to `./` (repo root). The existing `build-and-test` job defaults to `ash/`. Without the override, `node docs/assemble.js` would fail because `docs/` is at the repo root.
- **Generated file in `web-site/`**: Must not happen. The `website.json` output path is `docs/website-reference.md`, not inside `web-site/`. The What NOT to Touch list calls this out.
