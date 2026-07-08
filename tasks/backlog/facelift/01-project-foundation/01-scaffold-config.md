# Project Scaffolding — Configs, Styles, D1 Migration & File Operations

## Background

The Ash website (`web-site/`) is three static HTML pages served by `server.js` with vanilla CSS. There is no `package.json`, no framework, no build system, no backend storage. The site must be rebuilt on Astro + Tailwind CSS + Cloudflare D1/Pages Functions.

This task establishes the entire project skeleton: all configuration files, the CSS foundation, the D1 database schema, directory structure, file moves, and deletions. Every downstream task assumes this foundation is in place.

## Intended Solution

Create the following new files, verbatim or as specified:

### Configuration Files

**`web-site/package.json`** — new project root with Astro, Tailwind, Cloudflare, and Vitest dependencies:

```json
{
  "name": "ash-website",
  "private": true,
  "scripts": {
    "dev": "astro dev",
    "build": "astro build",
    "preview": "astro preview",
    "test": "vitest run"
  },
  "dependencies": {
    "astro": "^5.0",
    "@astrojs/cloudflare": "^12.0",
    "@astrojs/tailwind": "^6.0",
    "tailwindcss": "^3.4",
    "marked": "^14.0"
  },
  "devDependencies": {
    "vitest": "^3.0"
  }
}
```

**`web-site/astro.config.mjs`** — hybrid output with Cloudflare adapter and Tailwind integration:

```js
import { defineConfig } from 'astro/config';
import cloudflare from '@astrojs/cloudflare';
import tailwind from '@astrojs/tailwind';

export default defineConfig({
  output: 'hybrid',
  adapter: cloudflare(),
  integrations: [tailwind()],
  vite: {
    server: {
      headers: {
        'Cross-Origin-Opener-Policy': 'same-origin',
        'Cross-Origin-Embedder-Policy': 'credentialless',
      },
    },
  },
});
```

The adapter uses its default mode (produces Pages Functions output). COOP/COEP headers use `vite.server.headers` (not the deprecated top-level `server.headers`) for local dev WASM playground support.

**`web-site/tailwind.config.mjs`** — design tokens from the current dark theme:

```js
/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{astro,js,ts}'],
  theme: {
    extend: {
      colors: {
        bg: {
          primary: '#0d1117',
          secondary: '#161b22',
          tertiary: '#1c2333',
        },
        border: '#30363d',
        text: {
          primary: '#e6edf3',
          secondary: '#8b949e',
        },
        accent: {
          green: '#3fb950',
          blue: '#58a6ff',
          yellow: '#d29922',
          red: '#f78166',
        },
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'monospace'],
        sans: [
          '-apple-system', 'BlinkMacSystemFont',
          '"Segoe UI"', 'Roboto', '"Helvetica Neue"',
          'Arial', 'sans-serif',
        ],
      },
    },
  },
};
```

**`web-site/vitest.config.ts`** — test runner configuration:

```ts
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'node',
    include: ['tests/**/*.test.ts'],
  },
});
```

**`web-site/wrangler.toml`** — Cloudflare Pages + D1 binding:

```toml
name = "green-band-aooriwu"
pages_build_output_dir = "dist"
compatibility_date = "2026-01-01"

[[d1_databases]]
binding = "DB"
database_name = "ash-website-db"
database_id = "<REPLACE_WITH_OUTPUT_OF_wrangler_d1_create>"
```

The `database_id` placeholder must be replaced with the output of `wrangler d1 create ash-website-db --location web-site/`. This is a one-time manual step; CI relies on the committed ID matching the remote database.

### CSS Foundation

**`web-site/src/styles/globals.css`** — Tailwind directives plus supplementary CSS not replaced by Tailwind:

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

/* Syntax highlighting tokens */
.token-keyword { color: #58a6ff; }
.token-string { color: #3fb950; }
.token-builtin { color: #d29922; }
.token-number { color: #f78166; }
.token-operator { color: #e6edf3; }
.token-comment { color: #8b949e; font-style: italic; }
.token-variable { color: #e6edf3; }
.token-shebang { color: #8b949e; }

/* Editor overlay — transparent textarea over syntax-highlighted pre */
.editor-wrap { position: relative; }
.editor-highlight {
  position: absolute;
  top: 0; left: 0;
  width: 100%; height: 100%;
  padding: inherit;
  margin: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
  overflow: hidden;
  pointer-events: none;
  font-family: inherit;
  font-size: inherit;
  line-height: inherit;
  color: transparent;
  background: transparent;
  border: none;
}
.editor-wrap textarea {
  position: relative;
  background: transparent;
  color: transparent;
  caret-color: #e6edf3;
  z-index: 1;
}

/* Terminal animation */
.terminal { font-family: 'JetBrains Mono', monospace; }
.terminal-line { opacity: 0; }
.terminal.visible .terminal-line { animation: typeIn 0.4s ease forwards; }
@keyframes typeIn {
  to { opacity: 1; }
}

/* Toast notification */
.toast {
  position: fixed; bottom: 24px; right: 24px;
  background: var(--bg2); border: 1px solid #30363d;
  color: #e6edf3; padding: 12px 20px; border-radius: 8px;
  font-size: 14px; z-index: 100; opacity: 0;
  transform: translateY(8px); transition: opacity 0.3s, transform 0.3s;
  pointer-events: none;
}
.toast.show { opacity: 1; transform: translateY(0); }
.toast.error { border-color: #f78166; }

/* REPL prompt */
.repl-prompt-line { color: var(--text2); }
```

These styles use `var(--*)` for colors that are also defined in the Tailwind theme. Values referenced by `playground.js` and `eval-worker.js` — do not change the class names.

### D1 Migration

**`web-site/migrations/0001_initial.sql`** — initial database schema:

```sql
CREATE TABLE IF NOT EXISTS posts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL,
  slug TEXT NOT NULL UNIQUE,
  content TEXT NOT NULL,
  author TEXT NOT NULL,
  published_at TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS feedback (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  email TEXT NOT NULL,
  message TEXT NOT NULL,
  category TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Directory Structure

Create these directories (empty, awaiting downstream tasks):

```
web-site/public/js/
web-site/public/wasm/
web-site/src/pages/blog/
web-site/src/pages/api/
web-site/src/layouts/
web-site/src/components/
web-site/src/styles/       (globals.css written above)
web-site/src/content/
web-site/migrations/        (0001_initial.sql written above)
web-site/tests/
```

### File Moves (with Edits)

**Move `web-site/js/playground.js` → `web-site/public/js/playground.js`** with one edit at line 283: change `'js/eval-worker.js'` to `'/js/eval-worker.js'` (absolute path). The current relative path resolves to `/playground/js/eval-worker.js` on the new `/playground` route but the file is served at `/js/eval-worker.js`. No other changes to this file.

**Move `web-site/js/eval-worker.js` → `web-site/public/js/eval-worker.js`** — no edits. This file is completely untouched.

**Move `web-site/_headers` → `web-site/public/_headers`** — no edits. Contains a wildcard `/*` rule for COOP/COEP headers, safe for all new routes.

### Files to Delete

- `web-site/server.js` — replaced by `astro dev`
- `web-site/.cloudflareignore` — no longer needed (Astro builds into `dist/`)
- `web-site/css/style.css` — replaced by Tailwind + globals.css

Do NOT delete `web-site/index.html` or `web-site/agents.html` — their content is extracted by downstream tasks (Docs & Agents page) before deletion.

**Do not delete** `web-site/reference.html` — it is generated by `docs/assemble.js` and committed for GitHub consumers.

**Do not delete** `web-site/wasm/` (populated by CI). The CI pipeline's WASM output destination changes to `web-site/public/wasm/` in a downstream task.

## Acceptance Criteria

1. `npm install` succeeds in `web-site/` with all dependencies resolving.
2. `npm run dev` starts `astro dev` without config errors and serves on a local port.
3. `npm test` runs Vitest and finds no tests (empty `tests/` directory — no-op, exit code 0).
4. `src/styles/globals.css` contains `@tailwind base; @tailwind components; @tailwind utilities;` followed by all supplementary CSS.
5. `public/js/playground.js` contains Worker URL `'/js/eval-worker.js'` (absolute path) at the line previously reading `'js/eval-worker.js'`.
6. `public/js/eval-worker.js` is identical to the original `web-site/js/eval-worker.js`.
7. `public/_headers` is identical to the original `web-site/_headers`.
8. The three old files (`server.js`, `.cloudflareignore`, `css/style.css`) are deleted from `web-site/`. `index.html` and `agents.html` are preserved for downstream content extraction.
9. `migrations/0001_initial.sql` contains `CREATE TABLE IF NOT EXISTS posts` and `CREATE TABLE IF NOT EXISTS feedback`.
10. `wrangler.toml` contains `name = "green-band-aooriwu"`, `pages_build_output_dir = "dist"`, and one `[[d1_databases]]` block with `binding = "DB"`.

## Implementation Hints

- Work exclusively in `web-site/`.
- All configuration file content is provided verbatim above — copy it exactly.
- The `playground.js` edit is precisely: line 283, change `'js/eval-worker.js'` to `'/js/eval-worker.js'`. Find it by searching for `new Worker(`.
- When moving `playground.js` and `eval-worker.js`, check that the original `web-site/js/` directory is empty after the moves and can be removed.
- The `database_id` in `wrangler.toml` is a placeholder. Leave it as-is; it gets populated by a manual operation after `wrangler d1 create`.
- `web-site/reference.html` and `web-site/wasm/` must NOT be deleted.
- Do not install npm packages or run anything beyond what's needed to create the files — `npm install` will be run later.
