# Web Site Facelift — Task Definition v6

## Background

The Ash website (`web-site/`) is three static HTML pages served by a standalone `server.js` with vanilla CSS. It speaks in a "project introduction" voice, has no company identity, no blog, no feedback collection, and minimal responsive design. There is no framework, no build system, no `package.json`, no backend storage.

Company content already exists in the repo but is invisible to the public: a vision statement (`workflows/board/vision.md`), a mission statement with three pillars (`workflows/board/mision.md`), and five director profiles (`workflows/board/directors/*.md`).

The site deploys to Cloudflare Pages (`green-band-aooriwu`) via `wrangler pages deploy web-site` in the `deploy-website` CI job. The `build-and-test` CI job runs `docs/assemble.js` on push to master and commits generated files (including `web-site/reference.html`).

### Original requirements (from `ideas/web-site-facelift.md`)
1. Introduce a proper UI and styles framework
2. Re-tone from "project introduction" to "commercial company's home page that runs an open-source project"
3. Make the site mobile-device friendly
4. Surface vision, mission, and board members on the website
5. Add a blog — shift from static to storage-backed (preferably Cloudflare Workers with storage)
6. Add user feedback collection

### Improvements over v5
v6 resolves 9 issues from `tmp/assessment-report.md` (2 critical, 5 major, 2 minor) while preserving the architecture, content, and key decisions from v5. Changes from v5 are enumerated in §20.

## Intended Solution

### 1. Technology Stack

**Astro** `^5.0` with `output: 'hybrid'` + **Tailwind CSS v3.4** + **Cloudflare D1** for storage + **Pages Functions** (via `@astrojs/cloudflare` adapter `^12.0`) for SSR routes.

**Why Astro:**
- Shared components (header, nav, footer) without shipping unnecessary client JS. The only browser JavaScript is the playground (`js/playground.js`, `js/eval-worker.js`).
- `@astrojs/cloudflare` with `output: 'hybrid'` means most pages are statically generated at build time; only blog listing, blog posts, and the feedback API endpoint use on-demand SSR. D1 bindings are exposed to SSR routes via `context.locals.runtime.env.DB`.
- `astro dev` uses Miniflare internally to provide local D1 access.

**Why Tailwind CSS v3:**
- Utility classes provide systematic responsive design across breakpoints (`sm:`, `md:`, `lg:`, `xl:`).
- Dark theme (`dark:` variant) maps cleanly to the existing color palette.
- v3 is stable with `@astrojs/tailwind ^6.0`. Tailwind v4 is a breaking-change release with a CSS-first config paradigm still evolving in the Astro ecosystem.

**Why D1 + Pages Functions (not a separate Worker):**
- Blog serving and feedback storage are lightweight CRUD. D1 is SQLite-compatible and integrates directly with Pages Functions.
- No separate Worker deploy. The adapter surfaces D1 bindings natively.
- Collocated in one project — no split deploy pipeline.

**What NOT to choose:**
- **React/Next.js:** A content site with one dynamic page type and one form endpoint gains nothing from a full React framework. Astro ships zero JS to the client by default.
- **11ty + separate Worker:** Viable for static generation but splits the project into two deploy units. Astro's hybrid mode keeps everything in one.
- **Plain HTML + Tailwind:** Duplicates header/footer across pages, no SSR for blog/feedback.

### 2. Directory Structure

```
web-site/
  astro.config.mjs
  tailwind.config.mjs
  vitest.config.ts            # NEW
  package.json                # NEW — created from scratch (scripts: dev, build, preview, test)
  wrangler.toml               # NEW — full template provided in §18
  public/
    _headers                  # MOVED from web-site/_headers (wildcard, safe to move unchanged)
    js/                       # MOVED from web-site/js/ (playground.js, eval-worker.js)
    wasm/                     # NEW destination for WASM output (from ash-wasm)
    favicon.ico
  src/
    pages/
      index.astro             # Homepage — static (client-side blog fetch)
      vision.astro            # Vision & Mission — static
      board.astro             # Board of Directors — static
      docs.astro              # Getting Started (from index.html) — static
      agents.astro            # Agents table (from agents.html) — static
      playground.astro        # Playground — static
      reference.astro         # Feature reference — static (reads generated fragment)
      contact.astro           # Contact & Feedback form — static
      blog/
        index.astro           # Blog listing — SSR
        [slug].astro          # Blog post — SSR
      api/
        feedback.ts           # Feedback endpoint — SSR
        recent-posts.ts       # Recent posts JSON endpoint — SSR (for homepage client fetch)
    layouts/
      BaseLayout.astro        # Shared layout: head, header, nav, footer
    components/
      Nav.astro               # Responsive nav with hamburger
      Footer.astro
      BlogCard.astro
      DirectorCard.astro
      FeedbackForm.astro
      MissionPillarCard.astro
      BlogHighlight.astro     # Client-side island: fetches /api/recent-posts
    styles/
      globals.css             # Tailwind directives + supplementary CSS
    content/
      reference-content.html  # Generated by assemble.js, read at Astro build time
  migrations/
    0001_initial.sql          # D1 migration: posts + feedback tables
  tests/
    feedback.test.ts          # Integration tests for /api/feedback
```

**wrangler.toml location:** `web-site/wrangler.toml`. See §18 for the full template.

**Playground JS files:** `web-site/js/playground.js` and `web-site/js/eval-worker.js` are moved to `web-site/public/js/`. One line in `playground.js` is edited — the Worker URL at line 283 is changed from `'js/eval-worker.js'` to `'/js/eval-worker.js'` (absolute path). Without this change, the relative path resolves to `/playground/js/eval-worker.js` on the new `/playground` route but the file is served at `/js/eval-worker.js`. See §10 for full details.

### 3. Site Architecture — Pages and Routes

| Route               | Page                    | Source content                                           | Mode                            |
| ------------------- | ----------------------- | -------------------------------------------------------- | ------------------------------- |
| `/`                 | Home                    | Company copy (§4)                                        | Static + client-side blog fetch |
| `/vision`           | Vision & Mission        | `workflows/board/vision.md`, `workflows/board/mision.md` | Static                          |
| `/board`            | Board of Directors      | `workflows/board/directors/*.md`                         | Static                          |
| `/blog`             | Blog listing            | D1 `posts` table                                         | SSR                             |
| `/blog/[slug]`      | Blog post               | D1 `posts` table                                         | SSR                             |
| `/contact`          | Contact & Feedback form | New form + company info                                  | Static                          |
| `/api/feedback`     | Feedback endpoint       | POST → writes to D1                                      | SSR                             |
| `/api/recent-posts` | Recent posts JSON       | GET → reads D1 `posts` (latest 3)                        | SSR                             |
| `/docs`             | Getting Started         | Current `index.html` getting-started section             | Static                          |
| `/agents`           | Agents table            | Current `agents.html`                                    | Static                          |
| `/reference`        | Feature reference       | Generated fragment from `docs/assemble.js`               | Static                          |
| `/playground`       | Script editor + REPL    | WASM playground, `js/playground.js`                      | Static                          |

**Blog highlight architecture:** The homepage is fully static with `prerender = true`. The blog highlight section is a client-side component (`BlogHighlight.astro`) that renders an empty container `<div id="blog-highlight">` and includes a small `<script>` that `fetch`es `/api/recent-posts`. If the response contains posts, the script renders them into the container. If the response is empty or the fetch fails, the container remains invisible (no heading, no "no posts" message). This approach:
- Avoids the architectural contradiction of "static with SSR data fetch"
- Avoids experimental server islands (requiring opt-in flags)
- Adds a lightweight SSR API endpoint (`/api/recent-posts`) that is independently testable
- Accepts that an empty blog at launch shows no blog section — this is a deliberate product decision, not a defect. The empty state is correct: no posts means nothing to show.

**Navigation:** Primary nav items: Home | About (Vision, Board) | Docs (Getting Started, Agents, Reference) | Blog | Playground | Contact. On mobile (< 768px): hamburger menu. Logo links to `/`.

### 4. Homepage Design and Copy

The homepage communicates "a company exists behind this open-source tool." Section order and copy are specified here — the implementer uses this verbatim or makes minor edits for fit; does not invent new positioning.

**Section 1 — Hero:**
> # Orchestrating AI
> Ash is the company behind the open-source Ash orchestration runtime — the definitive orchestration layer for human-AI collaboration, where every automated decision is legible, testable, and completely governed.

Two CTA buttons: "Learn About Ash" → `/docs`, "Try the Playground" → `/playground`.

**Section 2 — Mission Pillars (three cards):**
- Card 1: **Own the Standard** — Establish our text-native format as the universal system of record for AI orchestration, creating switching costs through workflow IP encoded in `.ash` scripts.
- Card 2: **Commoditize the Supplier** — Decouple enterprise value from underlying AI models. Our platform-agnostic architecture lets organizations hot-swap any LLM provider without touching their workflows.
- Card 3: **Eliminate Enterprise Liability** — Transform unpredictable, stochastic AI behavior into auditable, deterministic business processes with full audit trails, state control, and governed execution.

Each card includes a link: "Read our full mission" → `/vision`.

**Section 3 — Product Teaser (condensed from current index.html):**
Three cards: Deterministic Control (sequencing/branching/retries), Agent-Agnostic (swap providers without touching workflows), Workflows as Code (commit, version, share, re-run). One heading + one sentence each. Links to `/docs`.

**Section 4 — Board Teaser:**
> ## Our Leadership
> Ash is guided by a board of directors with deep expertise in developer tools, enterprise architecture, and open-source communities.
>
> **Sarah Chen** — Open-Source & Community Veteran · **James Okonkwo** — DevTools GTM Expert · **Priya Nair** — Investor Director, Framework Capital · **Marcus Thorne** — Co-Founder & CEO · **Dr. Elena Vasquez** — Enterprise Architecture Leader
>
> [Meet the board → `/board`]

**Section 5 — Blog Highlight:**
- Client-side component fetches `/api/recent-posts` (returns JSON array of `{title, slug, published_at}` for latest 3 posts).
- If the array is empty: the container div is never made visible (no heading, no "no posts" message).
- If the fetch fails: the container stays hidden (fail gracefully).
- On slow connections: no loading indicator or skeleton — content either appears or doesn't. This is a deliberate trade-off: adding loading UX adds complexity beyond the scope of a static homepage module. Acceptable for a marketing site with a secondary blog section.

**Section 6 — Footer:**
Company name ("Ash"), navigation links (Home, Vision, Board, Docs, Agents, Reference, Blog, Playground, Contact), copyright year, GitHub link.

### 5. Vision, Mission, and Board Pages

**Vision & Mission (`/vision`):**
- Renders content from `workflows/board/vision.md` and `workflows/board/mision.md` verbatim.
- Vision statement displayed prominently at the top, styled as a large quote or hero block.
- Below, the three mission pillars in full — all content from `mision.md` rendered as-is.
- The source filename is `mision.md` (a typo in the repo). **Do not rename it** — other workflows may reference it. The page `<title>` and heading displays "Mission" (correct spelling).

**Board (`/board`):**
- Lists all five directors from `workflows/board/directors/`: Sarah Chen, James Okonkwo, Priya Nair, Marcus Thorne, Elena Vasquez.
- Each director card: name, role/title, Background section, and Perspective section as provided in their markdown file.
- Layout: 1 column on mobile (< 640px), 2 columns at 640–1023px, 3 columns at 1024px+.
- The categories from `directors/directors.md` (Executive, Investor, Independent) are metadata for context; grouping by category is optional.

### 6. Blog System

**Architecture:** Cloudflare D1 with a `posts` table, served via SSR Astro routes.

**D1 `posts` table:**

| Column         | Type    | Constraints                        |
| -------------- | ------- | ---------------------------------- |
| `id`           | INTEGER | PRIMARY KEY AUTOINCREMENT          |
| `title`        | TEXT    | NOT NULL                           |
| `slug`         | TEXT    | NOT NULL UNIQUE                    |
| `content`      | TEXT    | NOT NULL (markdown)                |
| `author`       | TEXT    | NOT NULL                           |
| `published_at` | TEXT    | NOT NULL (ISO 8601)                |
| `created_at`   | TEXT    | NOT NULL DEFAULT (datetime('now')) |
| `updated_at`   | TEXT    | NOT NULL DEFAULT (datetime('now')) |

- **`author` convention:** Use full names (e.g., "Sarah Chen", not "sarah"). Free-text display field, no FK to directors.

**`/blog` route (SSR, `prerender = false`):**
- Lists posts ordered by `published_at DESC`.
- Each list item: title (linked to `/blog/[slug]`), author, published date.
- No pagination — show all posts on one page.

**`/blog/[slug]` route (SSR, `prerender = false`):**
- Renders a single post. Markdown content rendered to HTML via `marked`.
- Displays: title, author, published date, full body.
- Nonexistent slug: returns 404.

**`/api/recent-posts` route (SSR, `prerender = false`):**
- GET endpoint. Returns JSON array of `{ title, slug, published_at }` for the 3 most recent posts.
- Empty table returns `[]`.

**Empty-state and error handling:**
- Empty blog listing: "No posts yet." — centered, muted paragraph.
- Homepage blog highlight: container div rendered but never made visible — no heading, no "no posts" message.
- `/blog/[slug]` nonexistent slug: Astro returns 404 via `return new Response(null, { status: 404 })`.
- D1 query failure (any SSR route): return 500 with user-facing message "Unable to load posts. Please try again later." Log the error via `console.error`.
- Malformed post record (missing required fields): skip in listing (500 for single post view). Log a warning.

**Content creation:** Blog posts are inserted via `wrangler d1 execute` using the `--file` flag with a prepared SQL file (avoiding shell quoting issues with markdown). Example:
```
wrangler d1 execute ash-website-db --file insert-post.sql
```
Where `insert-post.sql` contains the INSERT statement. No admin UI, no authentication, no protected endpoint.

**Launch state:** The D1 `posts` table starts empty after migration. The blog page will display "No posts yet." — this is technically correct. If posts should be live at launch, insert them via `wrangler d1 execute --file` after migration. The migration itself is data-structure only; seed data is a separate operational step, not part of the migration.

### 7. assemble.js and the Reference Page

**Current behavior:** `docs/assemble.js` auto-discovers targets from `docs/targets/*.json`, reads `docs/units/*.md`, applies templates, and writes output. The `website-html.json` target uses `docs/templates/reference.html` (a full HTML document) to produce `web-site/reference.html`, which is committed by the `build-and-test` CI job.

**Problem:** The full-HTML template cannot be used as a fragment in an Astro page.

**Solution — add a new target and fragment template (assemble.js itself is untouched):**

1. Create `docs/templates/reference-fragment.html` containing only the content body — no `<html>`, `<head>`, `<body>`, navigation, or footer:

```html
<div class="ref-hero">
  {{HERO}}
</div>
<div class="ref-content">
  {{CONTENT}}
</div>
```

2. Create `docs/targets/website-fragment.json`:

```json
{
  "output": "web-site/src/content/reference-content.html",
  "title": "# Ash — Feature Reference\n\nComplete feature reference for Ash agent shell scripts.\n",
  "units": [
    "what-is-ash", "install", "quick-start", "writing-tasks", "scripting",
    "directory-mode", "ash-scripts", "variables", "control-flow", "functions",
    "builtins", "sessions", "parallel", "file-prompts", "repl",
    "supported-agents", "building-from-source", "license", "vscode-extension"
  ],
  "template": "docs/templates/reference-fragment.html"
}
```

3. The **original** `website-html.json` target and `docs/templates/reference.html` template remain unchanged — they continue to produce `web-site/reference.html` for the GitHub README reference.

4. The Astro reference page (`src/pages/reference.astro`) reads the committed `src/content/reference-content.html` at build time via `Astro.glob` or direct `fs` import (Astro allows `import` of HTML fragments in `.astro` files). This is a static page.

5. **CI pipeline updates for the fragment:**
   - In `build-and-test` job: the `git add` step adds `${{ github.workspace }}/web-site/src/content/reference-content.html` to the commit list.
   - In `deploy-website` job: `node docs/assemble.js` runs before `npm run build` to ensure fragment freshness.

### 8. Contact & Feedback

**Form (`/contact`):**
- Fields: name (text, required), email (email, required), message (textarea, required), category (select: General, Bug Report, Feature Request, Other).
- Client-side validation: all required fields must be non-empty; email must contain `@`. Errors displayed inline next to the field.
- On valid submit: POST JSON `{ name, email, message, category }` to `/api/feedback`.
- Success: show "Thank you for your feedback." message, reset the form.
- The form is a static HTML component with inline `<script>` for validation and fetch. No JavaScript framework needed.

**Feedback API endpoint (`/api/feedback.ts`, SSR):**
- Accepts POST with JSON body.
- Request body size is limited to **64 KB**. Larger payloads return 413 with `{ error: "Request body too large." }`. This is a trivial defense — consistent with the acknowledged lack of comprehensive anti-spam measures.
- Server-side validation mirrors client:
  - name: non-empty string, trimmed length > 0
  - email: non-empty string, contains `@`
  - message: non-empty string, trimmed length > 0
  - category: must be one of `["General", "Bug Report", "Feature Request", "Other"]`
  - Invalid → 400 with `{ error: "descriptive message" }`. The descriptive message states which field failed and why (e.g., `"Email must contain @"`).
- D1 write failure → 500 with `{ error: "Unable to save feedback. Please try again later." }`. Log the error.
- D1 connection/timeout → 503 with `{ error: "Service temporarily unavailable. Please try again later." }`.
- Successful write → 200 with `{ ok: true }`.

**D1 `feedback` table:**

| Column       | Type    | Constraints                        |
| ------------ | ------- | ---------------------------------- |
| `id`         | INTEGER | PRIMARY KEY AUTOINCREMENT          |
| `name`       | TEXT    | NOT NULL                           |
| `email`      | TEXT    | NOT NULL                           |
| `message`    | TEXT    | NOT NULL                           |
| `category`   | TEXT    | NOT NULL                           |
| `created_at` | TEXT    | NOT NULL DEFAULT (datetime('now')) |

**Known limitation (acknowledged, not addressed):** The feedback endpoint has no authentication, CAPTCHA, or rate limiting. This is acceptable for initial launch; anti-spam measures are considered out of scope for this task.

**Retrieval:** Feedback is queryable via `wrangler d1 execute "SELECT * FROM feedback ORDER BY created_at DESC"`. No admin UI is in scope.

### 9. Content Migration — Section-by-Section Mapping

The current `index.html` is a monolithic page whose sections map to different new pages. The mapping uses element `id` attributes where present for robustness:

| Current `index.html` section (identified by)                            | Destination                                                        |
| ----------------------------------------------------------------------- | ------------------------------------------------------------------ |
| `<head>` meta, fonts, stylesheet link                                   | `BaseLayout.astro` (shared head)                                   |
| `<header>` + `<nav>`                                                    | `Nav.astro` (shared component)                                     |
| `<section class="hero">` (project intro)                                | Replaced by new company hero on `/` (§4 Section 1)                 |
| `<section class="compare">` (pain cards)                                | Condensed into product teaser on `/` (§4 Section 3)                |
| `<section class="pain-points" id="how-it-works">`                       | Condensed into product teaser or moved to `/docs`                  |
| Getting-started section (`<section>` containing install/usage commands) | `/docs` — content preserved verbatim, layout restyled              |
| `<section id="playground">`                                             | `/playground` — DOM IDs preserved per §10, layout restyled         |
| Agent-specific documentation paragraphs                                 | `/docs`                                                            |
| `<footer>`                                                              | `Footer.astro` (shared component)                                  |
| Inline terminal animation `<script>`                                    | `/playground` (it animates the terminal in the playground context) |
| `<script src="js/playground.js">`                                       | `/playground` only                                                 |
| `<script src="js/eval-worker.js">` reference                            | `/playground` only                                                 |

**Framing rule:** Documentation content (commands, code examples, agent lists, configuration formats) is preserved verbatim and only restyled. Framing/tone/marketing content (hero text, descriptions, CTAs) is rewritten to company voice using copy from §4.

**Agents page (`agents.html`):** Content is documentation. Preserved verbatim, restyled using Tailwind on the `/agents` page.

### 10. Playground — DOM Contract and JS Modifications

The WASM playground scripts depend on specific DOM element IDs. These IDs **must** be preserved:

| Element ID         | Purpose                         |
| ------------------ | ------------------------------- |
| `editor`           | CodeMirror/textarea mount point |
| `editor-highlight` | Syntax highlighting overlay     |
| `output`           | Execution output display        |
| `status`           | Execution status indicator      |
| `wasm-status`      | WASM load status indicator      |
| `script-tab`       | Script editor tab button        |
| `repl-tab`         | REPL tab button                 |
| `script-panel`     | Script editor panel container   |
| `repl-panel`       | REPL panel container            |
| `repl-prompt`      | REPL prompt display             |
| `repl-input`       | REPL input field                |
| `repl-output`      | REPL output display             |
| `toast`            | Notification toast              |

Additionally, the `.example-code` CSS class must be preserved on click-to-load examples (used at `playground.js:370`). The implementer must examine the current `index.html` to identify which elements carry `.example-code` and reproduce them on the `/playground` page.

**Worker URL fix:** `playground.js` line 283 currently reads:
```javascript
evalWorker = new Worker('js/eval-worker.js', { type: 'module' });
```
On the current site, the playground is served from `/` (root), so the relative path `js/eval-worker.js` resolves to `/js/eval-worker.js`. On the new site, the playground is at `/playground`, where the same relative path resolves to `/playground/js/eval-worker.js` — which does not exist. **The fix is a one-line edit:** change the Worker URL to `/js/eval-worker.js` (absolute path).

**Modified files:** `playground.js` line 283 is edited. No other changes to either JS file.

**No other CSS selectors in `playground.js` need updating.** The script uses exclusively `getElementById` and `querySelectorAll('.example-code')`. The HTML structure around the playground can be completely reorganized with Tailwind as long as:
1. All IDs from the table above are present.
2. The `.example-code` class is on example-preload elements (reproduced from current `index.html`).
3. CSS for syntax highlighting tokens (`.token-keyword`, `.token-string`, `.token-builtin`, `.token-number`, `.token-operator`, `.token-comment`, `.token-variable`, `.token-shebang`) is preserved in the supplementary CSS.
4. The editor overlay technique (`.editor-highlight`, `.editor-wrap textarea` positioning/transparency) is preserved in the supplementary CSS.

### 11. Design Tokens

These values become Tailwind theme extensions in `tailwind.config.mjs`:

| CSS variable              | Value             | Tailwind use                          |
| ------------------------- | ----------------- | ------------------------------------- |
| `--bg`                    | `#0d1117`         | `colors.bg.primary`                   |
| `--bg2`                   | `#161b22`         | `colors.bg.secondary`                 |
| `--bg3`                   | `#1c2333`         | `colors.bg.tertiary`                  |
| `--border`                | `#30363d`         | `colors.border`                       |
| `--text`                  | `#e6edf3`         | `colors.text.primary`                 |
| `--text2`                 | `#8b949e`         | `colors.text.secondary`               |
| `--accent` (brand blue)   | `#58a6ff`         | `colors.accent.blue`                  |
| `--accent2` (brand green) | `#3fb950`         | `colors.accent.green` (primary brand) |
| `--accent3` (yellow)      | `#d29922`         | `colors.accent.yellow`                |
| `--accent4` (red)         | `#f78166`         | `colors.accent.red`                   |
| `--font-mono`             | JetBrains Mono    | `fontFamily.mono`                     |
| `--font-sans`             | System font stack | `fontFamily.sans`                     |

The brand green (`#3fb950`) is the primary brand color (logo, primary CTAs, links, interactive accents). Blue (`#58a6ff`) is a secondary accent.

**CSS not replaced by Tailwind** (lives in `src/styles/globals.css` after the `@tailwind` directives):
- `.token-*` syntax highlighting colors
- `.editor-highlight`, `.editor-wrap textarea` positioning/transparency
- `.terminal`, `.terminal-line`, `@keyframes typeIn`
- `.toast`, `.toast.show`, `.toast.error`
- `.repl-prompt-line`

Everything else (layout, typography, spacing, colors, responsive breakpoints) uses Tailwind utilities.

The entire site is dark-themed. No light/dark toggle.

### 12. Mobile Responsiveness

Using Tailwind breakpoints: `sm:` 640px, `md:` 768px, `lg:` 1024px, `xl:` 1280px.

- **Navigation:** Horizontal links above 768px; hamburger menu below 768px. Minimum tap target 44×44px.
- **Homepage hero:** Single-column stacked below 768px; wider layout above.
- **Board cards:** 1 column below 640px, 2 columns at 640–1023px, 3 columns at 1024px+.
- **Blog listing:** Single column at all sizes.
- **Playground:** Editor + output side by side above 768px; stacked vertically below 768px.
- **Forms:** Full-width inputs on mobile, labels above inputs. Base font size minimum 16px on inputs to prevent iOS zoom on focus.
- **Typography:** Heading sizes scale down with responsive classes (e.g., `text-3xl lg:text-5xl`).
- **No horizontal overflow** at any viewport width from 320px to 1920px.

### 13. Testing Strategy

**Integration tests only.** The testing scope is deliberately narrow — this is a marketing/content site, not a SaaS platform:

**Integration tests (Vitest, in `web-site/tests/`):**
1. **Feedback API endpoint (`tests/feedback.test.ts`):**
   - Valid submission → 200, `{ ok: true }`
   - Missing name → 400, `{ error: "..." }`
   - Missing email → 400, `{ error: "..." }`
   - Email without `@` → 400, `{ error: "..." }`
   - Missing message → 400, `{ error: "..." }`
   - Invalid category → 400, `{ error: "..." }`
   - Use `describe`/`it` blocks. Mock the D1 binding. Since the `@astrojs/cloudflare` runtime API isn't available outside `astro dev`'s Miniflare, the test imports and directly invokes the validation/business-logic portion of the handler (extracted as a pure function), or uses a simple in-memory mock for the DB binding. The goal is to test that validation rules produce correct status codes and error messages.

**What is NOT tested (visual review sufficient):**
- Blog SSR routes — no business logic beyond D1 read/write; tested via manual deployment smoke check.
- Page rendering / E2E — visual review catches missing imports, broken Tailwind, and build failures.
- Playground smoke — WASM execution is tested by the Rust `ash-wasm` crate.
- Visual design accuracy — colors, spacing, typography.
- Board content accuracy — source files rendered verbatim.

**Test infrastructure:**
- `web-site/tests/` directory with Vitest config (`vitest.config.ts`).
- `package.json` scripts: `"test": "vitest run"`.
- `deploy-website` CI runs `npm test` before `npm run build`. Test failure blocks deploy.
- For local dev: `npm test` runs against the project's Vitest configuration.

### 14. Local Development Setup

**Prerequisites:** Node.js 18+, `wrangler` CLI, Rust + wasm-pack (for WASM build).

**One-time setup:**
```bash
# Create D1 database locally
wrangler d1 create ash-website-db --location web-site/

# Apply initial migration
wrangler d1 migrations apply ash-website-db --local --config web-site/wrangler.toml

# Install dependencies
cd web-site && npm install
```

**Development loop:**
```bash
cd web-site
npm run dev    # astro dev with Miniflare D1
```

The `@astrojs/cloudflare` adapter uses Miniflare to provide local D1 access during `astro dev`. The D1 binding from `wrangler.toml` is picked up automatically. If automatic binding detection does not work, verify the adapter version supports Miniflare D1 simulation and that `wrangler.toml` contains a valid `[[d1_databases]]` binding. Run `wrangler d1 execute ash-website-db --local --command "SELECT 1"` to confirm local D1 is accessible.

**COOP/COEP headers in local dev:** The production `_headers` file is a Cloudflare Pages-specific mechanism — `astro dev` does not read it. To enable the WASM playground (which requires `SharedArrayBuffer`) during local development, configure the Vite dev server in `astro.config.mjs`:

```js
export default defineConfig({
  // ...
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

The `public/_headers` file provides these headers in production; the `vite.server.headers` config covers local dev. (Astro 5 uses `vite.server.headers`, not the deprecated top-level `server.headers`.)

### 15. CI Deployment Pipeline

The `deploy-website` job in `.github/workflows/ci.yml` is updated to:

```yaml
deploy-website:
  name: Deploy Website
  if: github.ref_type == 'tag'
  needs: github-release
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - uses: dtolnay/rust-toolchain@stable
      with:
        targets: wasm32-unknown-unknown

    - name: Install wasm-pack
      run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

    - name: Build WASM
      run: |
        cd ash-wasm
        wasm-pack build --target web --release
        mkdir -p ../web-site/public/wasm
        cp pkg/*.wasm ../web-site/public/wasm/
        cp pkg/*.js ../web-site/public/wasm/

    - name: Generate docs
      run: |
        npm install --prefix docs
        node docs/assemble.js

    - name: Setup Node
      uses: actions/setup-node@v4
      with:
        node-version: 20

    - name: Apply D1 migrations
      env:
        CLOUDFLARE_API_TOKEN: ${{ secrets.CLOUDFLARE_API_TOKEN }}
        CLOUDFLARE_ACCOUNT_ID: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
      run: |
        npm install -g wrangler
        wrangler d1 migrations apply ash-website-db --config web-site/wrangler.toml

    - name: Install and test
      working-directory: web-site
      run: |
        npm ci
        npm test

    - name: Build Astro
      working-directory: web-site
      run: npm run build

    - name: Deploy to Cloudflare Pages
      env:
        CLOUDFLARE_API_TOKEN: ${{ secrets.CLOUDFLARE_API_TOKEN }}
        CLOUDFLARE_ACCOUNT_ID: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
      run: |
        npm install -g wrangler
        wrangler pages deploy web-site/dist --project-name=green-band-aooriwu --branch main
```

Key changes from current CI:
1. WASM output: `web-site/wasm/` → `web-site/public/wasm/` (Astro serves `public/` at root; playground scripts reference `wasm/ash_wasm.js` → resolves to `public/wasm/ash_wasm.js`)
2. `docs/assemble.js` runs in `deploy-website` (was only in `build-and-test`)
3. D1 migration step added (before build)
4. `npm ci && npm test && npm run build` in `web-site/`
5. Deploy directory: `web-site/dist/` (Astro output) instead of `web-site/`

**`build-and-test` CI update:** Add the fragment path to the `git add` step:
```
git add "${{ github.workspace }}/web-site/src/content/reference-content.html"
```
(Added to the existing list of five files at `ci.yml:32-36`.)

### 16. What Stays Unchanged (Out of Scope)

- `ash/` — Rust core, parser, evaluator, runtime
- `ash-wasm/` — WASM compilation logic (only the output copy destination in CI changes)
- `docs/assemble.js` — reference doc generation logic (new templates/targets are additive, assemble.js itself is untouched)
- `docs/units/*.md` — documentation source files
- `docs/targets/website-html.json` — existing target continues to produce `web-site/reference.html` for GitHub
- `docs/templates/reference.html` — existing full-HTML template, still needed
- `docs/package.json` — existing dependency on `marked`
- `web-site/js/playground.js` — **except line 283** (Worker URL change from `'js/eval-worker.js'` to `'/js/eval-worker.js'`). All other logic unchanged.
- `web-site/js/eval-worker.js` — completely untouched
- `web-site/_headers` — content unchanged (wildcard `/*` rule, safe for all new routes), moved to `web-site/public/_headers`
- `npm-package/` and `ash-vscode/` — npm package and VS Code extension
- `workflows/board/` filenames — especially `mision.md` (typo preserved)
- Repository license and contribution model

### 17. Files to Delete

- `web-site/index.html` — replaced by Astro pages
- `web-site/agents.html` — replaced by Astro page at `/agents`
- `web-site/server.js` — replaced by `astro dev`
- `web-site/.cloudflareignore` — no longer needed (Astro builds into `dist/`)
- `web-site/css/style.css` — replaced by Tailwind + supplementary CSS in `src/styles/globals.css`

Note: `web-site/reference.html` keeps being generated by assemble.js for the GitHub target and committed to the repo. It is no longer the website's source of truth for the reference page, but the committed file is kept for downstream consumers (README links, GitHub browsing).

**Files moved (not deleted, not edited):**
- `web-site/js/playground.js` → `web-site/public/js/playground.js` (line 283 edited)
- `web-site/js/eval-worker.js` → `web-site/public/js/eval-worker.js`
- `web-site/_headers` → `web-site/public/_headers`

### 18. New Files to Create

- `web-site/package.json` — with these scripts and dependencies:

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

- `web-site/vitest.config.ts`:

```ts
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'node',
    include: ['tests/**/*.test.ts'],
  },
});
```

Tests run in `node` environment (no DOM needed — feedback API validation is pure logic). The `include` pattern scopes Vitest to the `tests/` directory.

- `web-site/astro.config.mjs`:

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

The adapter is configured without an explicit `mode` option — the default behavior produces Pages Functions output. **If the installed adapter version requires an explicit mode,** the implementer verifies the correct option against that version's documentation at implementation time rather than guessing here. COOP/COEP headers use `vite.server.headers` (Astro 5 / Vite compatible), not the deprecated top-level `server.headers`.

- `web-site/tailwind.config.mjs`:

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

- `web-site/wrangler.toml`:

```toml
name = "green-band-aooriwu"
pages_build_output_dir = "dist"
compatibility_date = "2026-01-01"

[[d1_databases]]
binding = "DB"
database_name = "ash-website-db"
database_id = "<REPLACE_WITH_OUTPUT_OF_wrangler_d1_create>"
```

The `database_id` is populated once via `wrangler d1 create ash-website-db --location web-site/`, then committed. This is a one-time manual step — CI relies on the committed ID matching the remote database. If the D1 database is ever recreated, the ID must be updated in this file.

- `web-site/migrations/0001_initial.sql` — D1 migration:

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

- `web-site/src/` — all Astro components, pages, layouts per §2.
- `web-site/tests/feedback.test.ts` — integration tests per §13.
- `docs/templates/reference-fragment.html` — fragment template per §7.
- `docs/targets/website-fragment.json` — new target per §7.

### 19. Key Decisions (Don't Revisit)

- Astro (not Next.js, not 11ty, not plain HTML)
- Tailwind CSS v3.4 (not v4, not a component library)
- D1 + Pages Functions (not a separate Worker API)
- `wrangler.toml` at `web-site/` level (not repo root)
- `output: 'hybrid'` (not `'server'` or `'static'`)
- `@astrojs/cloudflare` adapter without explicit `mode` (default behavior; verify if required by installed version)
- Homepage blog highlight: client-side fetch to `/api/recent-posts` (not server islands, not server-rendered)
- Homepage copy: supplied in §4 — use verbatim or make minor edits for fit; do not invent new positioning
- Blog content insertion: CLI only via `--file` flag (no admin UI)
- assemble.js: untouched; new template + target added
- `mision.md` typo: preserved in filename, corrected in display
- Testing: integration tests for feedback API only (no E2E, no blog SSR tests)
- COOP/COEP: `public/_headers` for production, `vite.server.headers` for local dev
- Worker URL fix: edit `playground.js` line 283 only — change to absolute path
- Request body size limit: 64 KB for feedback endpoint (trivial defense, not a substitute for anti-spam)
- Launch blog state: empty table after migration; seed data is a separate operational step

### 20. Changes From v5 (for Reviewers)

These are substantive changes from `tmp/task-definition_5.md`. Numbered by assessment report finding:

1. **(Finding #1 — Worker URL path):** §2, §10, and §16 updated. `playground.js` line 283 **must** be edited from `'js/eval-worker.js'` to `'/js/eval-worker.js'` (absolute path). The v5 claim that the relative path "resolves correctly when both files are in `public/js/`" was incorrect for the new `/playground` route. The "What NOT to touch" constraint is relaxed to allow this one-line edit.

2. **(Finding #2 — Adapter mode):** §18 `astro.config.mjs` changed from `cloudflare({ mode: 'directory' })` to `cloudflare()`. The explicit `mode` is removed; the adapter's default behavior produces the correct Pages Functions output format. If the installed version requires an explicit mode, the implementer verifies at implementation time.

3. **(Finding #3 — server.headers):** §14 and §18 changed from top-level `server.headers` to `vite.server.headers`. Astro 5 removed the top-level `server.headers` config option. The Vite-level equivalent is the correct approach.

4. **(Finding #4 — tailwind.config.mjs):** §18 now provides a complete `tailwind.config.mjs` template with all color tokens, font families, and content paths from §11. The implementer no longer needs to infer the Tailwind theme nesting structure.

5. **(Finding #5 — vitest config):** §18 now provides `vitest.config.ts` with `environment: 'node'` and an explicit `include` pattern. Tests are scoped to the `tests/` directory.

6. **(Finding #6 — _headers audit):** §2 notes that `_headers` uses a wildcard `/*` rule, making it safe to move unchanged to `public/_headers` regardless of new route structure. The file content is documented: COOP/COEP isolation headers only, no route-specific rules.

7. **(Finding #7 — D1 DB ID):** §18 adds a note that the `database_id` is a one-time manual step. CI relies on the committed ID. If the D1 database is recreated, the ID must be updated in `wrangler.toml`.

8. **(Finding #8 — Blog SSG path):** §3 and §4.5 updated to explicitly acknowledge the empty blog launch state and the invisible homepage blog section as a deliberate product decision, not a defect.

9. **(Finding #9 — marked duplication):** Acknowledged as unlikely to cause issues. Both `docs/package.json` (assemble.js) and `web-site/package.json` (blog SSR) use `marked`. Version drift is a theoretical risk. The `marked` dependency in `web-site/package.json` is kept because blog SSR routes need it at runtime in the Pages Function.

10. **(Finding #10 — No seed data):** §6 adds an explicit "Launch state" note. The D1 migration is data-structure only. Seed blog posts are a separate operational step using `wrangler d1 execute --file`. This is intentional — the task does not prescribe launch content.

11. **(Finding #11 — Loading state):** §4.5 adds a note that the blog highlight has no loading indicator. This is a deliberate trade-off for keeping the homepage fully static with a minimal client-side script.

12. **(Finding #12 — Example preload elements):** §10 adds an instruction for the implementer to examine the current `index.html` to locate `.example-code` elements and reproduce them on the `/playground` page.

13. **(Finding #13 — Miniflare D1 binding):** §14 adds a troubleshooting note: verify the adapter version supports Miniflare D1 simulation, and use `wrangler d1 execute --local --command` to confirm local D1 accessibility.

14. **(Finding #14 — Request size limit):** §8 adds a 64 KB request body size limit to the feedback endpoint. Payloads exceeding 64 KB return 413. This is a trivial defense consistent with the acknowledged lack of comprehensive anti-spam measures.

15. **(Finding #15 — Navigation naming):** No change. The shift from "Getting Started" to comprehensive "Docs" (with Agents and Reference as sub-items) is intentional and acknowledged.

Additional structural changes from v5:
- §2: `public/js/` path and the `playground.js` edit documented explicitly.
- §10: Renamed from "Playground — DOM Contract" to "Playground — DOM Contract and JS Modifications". Worker URL fix documented.
- §13: Removed blog SSR integration tests and Playwright E2E smoke tests entirely (same as v5, not changed further).
- §16: "What Stays Unchanged" updated to reflect the `playground.js` line 283 exception.
- §18: Added `vitest.config.ts`, `tailwind.config.mjs` templates. `astro.config.mjs` updated for `vite.server.headers` and adapter defaults.
- §19: Key decisions updated to reflect new decisions on Worker URL, adapter mode, request size limit, and launch state.

## Acceptance Criteria

1. **Framework builds**: `npm run build` in `web-site/` produces deployable output in `web-site/dist/`. Project uses Astro `^5.0` with `output: 'hybrid'` and Tailwind CSS `^3.4`. No raw HTML files with copy-pasted headers/footers remain. A `package.json` exists with `dev`, `build`, `preview`, and `test` scripts.

2. **Company homepage**: Landing page (`/`) presents Ash as a company running an open-source project. Hero references "Orchestrating AI." Homepage includes, in order: hero (with two CTAs), three mission pillar cards, product teaser, board director names, and footer. Blog highlight appears only when `/api/recent-posts` returns posts (client-side fetch, empty div otherwise — no heading, no "no posts" message). Blog fetch failure is silently handled (container stays hidden).

3. **Vision, Mission, and Board pages**: `/vision` displays the vision statement prominently and all three mission pillar sections with full text from `workflows/board/vision.md` and `workflows/board/mision.md`. `/board` lists all five directors with name, role, background, and perspective from `workflows/board/directors/*.md`. The page `<title>` and heading on `/vision` display "Mission" (not "Mision").

4. **Mobile responsive**: Site is usable from 320px to 1920px. Navigation collapses to hamburger below 768px. Board cards: 1 col / 2 cols / 3 cols at breakpoints. No horizontal overflow. Tap targets minimum 44×44px. Form inputs minimum 16px font size.

5. **Blog serves and displays posts**: `/blog` lists published posts from D1 ordered by `published_at DESC`. `/blog/[slug]` renders individual posts with title, date, author, and markdown body rendered as HTML. Empty blog shows "No posts yet." Nonexistent slug returns 404. D1 query failure returns 500 with user-facing message.

6. **User feedback collection**: `/contact` has a form with name, email, message, and category fields. Valid submission POSTs JSON to `/api/feedback`, stores in D1 `feedback` table, returns `{ ok: true }`. Invalid submission returns 400 with `{ error: "..." }` describing the failed field. Request bodies exceeding 64 KB return 413. D1 failure returns 500 or 503 with descriptive error. Feedback is retrievable via `wrangler d1 execute` query.

7. **Product content preserved**: Agents table (from `agents.html`), getting-started instructions (from `index.html`), and full feature reference (from `docs/assemble.js` via fragment template) are present at `/agents`, `/docs`, and `/reference`. Documentation content is restyled with Tailwind, not rewritten.

8. **Playground functions**: WASM playground loads and runs Ash scripts. All DOM IDs from §10 are present. `.example-code` class preserved on example-load elements. Syntax highlighting works. Script and REPL tabs switch correctly. WASM playground works in local dev (COOP/COEP headers served by Vite). Worker loads `eval-worker.js` correctly at `/playground` route (uses absolute path `/js/eval-worker.js`).

9. **Deployment deploys**: `deploy-website` CI job builds WASM → generates docs (assemble.js) → applies D1 migrations → runs `npm test` → builds Astro → deploys `web-site/dist/` to Cloudflare Pages `green-band-aooriwu`. COOP/COEP headers present in deployed output via `public/_headers`.

10. **Tests pass**: `npm test` in `web-site/` runs feedback API integration tests (valid submission → 200, missing fields → 400 with error messages). Tests pass before deploy.

11. **Local dev loop works**: `npm run dev` in `web-site/` starts `astro dev` with blog SSR routes backed by local D1 (Miniflare). Developers can test blog listings, blog posts, and feedback submission locally. WASM playground works locally with COOP/COEP headers.

12. **No regressions in generated docs**: `docs/assemble.js` continues to produce `web-site/reference.html` (full HTML via existing `website-html.json` target) and the new `web-site/src/content/reference-content.html` (fragment via new `website-fragment.json` target). Both are committed by CI.

13. **`mision.md` filename unchanged**: The typo in `workflows/board/mision.md` is not corrected. The page heading on `/vision` displays "Mission" (correct spelling).

## Implementation Hints

### Project context
- Website lives under `web-site/`. This is the working directory for most changes.
- Cloudflare Pages project: `green-band-aooriwu`. The D1 database will be `ash-website-db`.
- CI jobs: `build-and-test` (push to master, generates docs + commits them), `deploy-website` (tags, deploys). Both need updates.
- Board content: `workflows/board/vision.md` (2 lines), `workflows/board/mision.md` (31 lines, 3 pillars), five director files in `workflows/board/directors/` (~30 lines each), one metadata file `directors.md`.
- WASM build: `wasm-pack build --target web --release` in `ash-wasm/`. Current output: `web-site/wasm/`. New output: `web-site/public/wasm/`.
- `docs/assemble.js` uses `marked` for markdown-to-HTML, reads `docs/targets/*.json`, writes to configured output paths. It's a CommonJS script with a `validateOutputPaths` function that auto-creates missing parent directories.
- `docs/package.json` has `marked` as a dependency (`"type": "commonjs"` implied).
- Current `web-site/server.js` is an ES module. It serves static files at port 3000 with COOP/COEP headers. This entire file is deleted — `astro dev` replaces it.
- `web-site/_headers` contains only a wildcard `/*` rule for COOP/COEP headers. Safe to move to `public/_headers` unchanged.

### Existing patterns
- Dark theme colors are defined in `web-site/css/style.css` as CSS custom properties.
- Brand green (`#3fb950`) is the primary brand color — logo, CTAs, links.
- Brand blue (`#58a6ff`) is a secondary accent.
- Code highlighting uses token classes: `.token-keyword`, `.token-string`, `.token-builtin`, `.token-number`, `.token-operator`, `.token-comment`, `.token-variable`, `.token-shebang`. Keep the CSS for these in the supplementary stylesheet.
- `web-site/_headers` sets COOP/COEP globally for SharedArrayBuffer support. Content unchanged, moved to `web-site/public/_headers`.
- `playground.js` creates a Worker: `new Worker('/js/eval-worker.js', { type: 'module' })` (after the one-line edit from relative to absolute path). Both JS files must be present in `public/js/`.
- Current `index.html` contains `.example-code` elements for pre-loadable playground examples. Reproduce these on the `/playground` Astro page.

### What NOT to touch
- `ash/` (Rust core)
- `ash-wasm/` (WASM compilation — only the output copy destination changes in CI)
- `docs/assemble.js` (generation logic — additive templates/targets only)
- `docs/package.json` (existing dependency on `marked`)
- `docs/units/*.md` (documentation source)
- `docs/templates/reference.html` (existing full-HTML template, still needed)
- `docs/targets/website-html.json` (existing target, still needed)
- `web-site/js/playground.js` — **except line 283** (change `'js/eval-worker.js'` to `'/js/eval-worker.js'`). All other logic unchanged.
- `web-site/js/eval-worker.js` — completely untouched.
- `web-site/_headers` — move to `public/_headers`, do not edit content.
- `npm-package/` and `ash-vscode/`
- `workflows/board/` filenames
- Repository license and contribution model
