# Vision, Mission & Board Pages

## Background

Company content already exists in the repository but is invisible to the public:
- Vision statement: `workflows/board/vision.md` (2 lines)
- Mission statement with three pillars: `workflows/board/mision.md` (31 lines) — note the typo in the filename
- Five director profiles: `workflows/board/directors/sarah-chen.md`, `james-okonkwo.md`, `priya-nair.md`, `marcus-thorne.md`, `elena-vasquez.md`

These must be surfaced as public-facing pages at `/vision` and `/board`.

## Intended Solution

### `/vision` Page (`src/pages/vision.astro`)

Static page using `BaseLayout` with `title="Vision & Mission"`.

**Vision statement section** — displayed prominently as a hero block or large quote:
- Content from `workflows/board/vision.md`:
  - Title: "Orchestrating AI"
  - Body: "To be the definitive orchestration layer for human-AI collaboration, where every automated decision is legible, testable, and completely governed."
- Styled as a large, centered quote or hero block with distinct visual treatment (e.g., larger font, brand green accent, subtle background).

**Mission pillars section** — below the vision, three sections with full content from `workflows/board/mision.md`:

Each pillar (Own the Standard, Commoditize the Supplier, Eliminate Enterprise Liability) includes:
- Pillar title (H2)
- "The Strategic Moat" / "The Resilience Play" / "The Risk Mandate" subtitle
- BODY paragraph
- "The Board's Perspective:" section
- "Strategic Action:" section

Display options:
- Option A: Three full-width sections stacked vertically, each with all content rendered as markdown → HTML.
- Option B: Three cards using `MissionPillarCard.astro` (created in Homepage task) displaying the summary, with expandable/toggle for full content, or links to anchors below.

The page `<title>` and visible heading must display "Mission" (correct spelling), despite the source filename being `mision.md`.

**Do NOT rename the source file** `workflows/board/mision.md` — other workflows may reference it by that path.

### `/board` Page (`src/pages/board.astro`)

Static page using `BaseLayout` with `title="Board of Directors"`.

Lists all five directors from `workflows/board/directors/`. Each director card displays:
- **Name** (from the `# Name — Board Director Profile` heading)
- **Role** (from the `**Role:**` line in the markdown)
- **Background** (from the `## Background` section — rendered markdown to HTML)
- **Perspective** (from the `## Perspective` blockquote — rendered as a stylized quote)

Director cards are organized in a responsive grid:
- 1 column on mobile (< 640px)
- 2 columns at 640–1023px
- 3 columns at 1024px+

**Content reading strategy:** The director markdown files are at a fixed known path. There are two approaches:
1. **Build-time import:** Use Astro's `Astro.glob('./content/boss/*.md')` — but the files are outside `src/`.
2. **Hard-coded data:** Since there are exactly five directors with known filenames, import each file directly. Astro supports importing from outside `src/` using absolute paths or the `?raw` import query for text content, then parse the markdown frontmatter manually.
3. **Manual approach:** Create a data file or embed the director information. **But the spec says to render them verbatim from the source files.**

The simplest approach: use Node.js `fs.readFileSync` in the Astro frontmatter to read each markdown file at build time, parse the relevant sections, and pass the structured data to the template. Astro's `import.meta.env` / `process.cwd()` gives the project root for path resolution.

**DirectorCard component** (`src/components/DirectorCard.astro`): Accepts `name`, `role`, `background`, `perspective` as props. Renders a card with:
- Name (h3, brand green)
- Role (subtle, `text-text-secondary`)
- Background (rendered as HTML paragraphs, standard body text)
- Perspective (rendered as a blockquote with left border accent, italic or stylized)

The categories from `directors/directors.md` (Executive, Investor, Independent) are metadata for context — grouping by category on the page is optional and left to the implementer's judgment.

### Page Headers

Both pages have a page title (H1) at the top:
- `/vision`: "Vision & Mission"
- `/board`: "Board of Directors"

## Acceptance Criteria

1. `/vision` displays the vision statement ("Orchestrating AI" heading + body text) prominently at the top.
2. `/vision` displays full content from all three mission pillars in `mision.md` — "Own the Standard", "Commoditize the Supplier", "Eliminate Enterprise Liability".
3. `/vision` page `<title>` and visible H1 display "Mission" (correct spelling), not "Mision".
4. The file `workflows/board/mision.md` is NOT renamed.
5. `/board` displays all five directors: Sarah Chen, James Okonkwo, Priya Nair, Marcus Thorne, Elena Vasquez.
6. Each director card shows name, role, background text, and perspective quote.
7. Director cards use responsive grid: 1 column at <640px, 2 columns at 640–1023px, 3 columns at 1024px+.
8. Both pages use `BaseLayout` and have a consistent heading style.
9. Content is rendered verbatim from the source markdown files — no rewriting of director backgrounds or mission text.

## Implementation Hints

- Use Astro's top-level `---` frontmatter for data loading. `fs.readFileSync` and basic markdown parsing (split on `## ` headings) is sufficient — or use the `marked` library already in `package.json` dependencies.
- Path resolution: use `path.resolve(process.cwd(), '../../workflows/board/')` to reach outside `web-site/` — Astro allows this at build time for static pages.
- For parsing `## Background` and `## Perspective` from director files: split on `## ` headings, match sections by heading text. The Perspective is always a blockquote starting with `>`.
- The `MissionPillarCard.astro` component created during the Homepage task accepts `title` and `body` props. For the full `/vision` page, you may either reuse it with expanded content or create a separate full-pillar rendering section.
- Import `BaseLayout` from `../layouts/BaseLayout.astro`.
- All styling uses Tailwind utilities. Background: `bg-bg-primary`. Cards: `bg-bg-secondary border border-border rounded-lg p-6`.
- Perspective blockquote: `border-l-4 border-accent-green pl-4 italic text-text-secondary`.
