# Homepage — Company Landing Page

## Background

The current `index.html` is a "project introduction" page for a developer tool. The new homepage must reposition as a **commercial company's home page that runs an open-source project**. This is the most important page on the site — it sets the tone for everything else.

Copy, section order, and CTAs are fully specified below and must not be improvised. The implementer may make minor wording adjustments for layout fit but must not invent new positioning, slogans, or value propositions.

## Intended Solution

A static Astro page (`src/pages/index.astro`) using `BaseLayout` with `title="Home"`. Contains six sections in order:

### Section 1 — Hero

Full-width dark background (or subtle gradient). Content:

> # Orchestrating AI
>
> Ash is the company behind the open-source Ash orchestration runtime — the definitive orchestration layer for human-AI collaboration, where every automated decision is legible, testable, and completely governed.

Two CTA buttons side by side:
1. **"Learn About Ash"** → links to `/docs` (primary, brand green background, white/light text — `btn-primary` equivalent)
2. **"Try the Playground"** → links to `/playground` (secondary, outlined or muted)

The heading uses a large responsive type scale (e.g., `text-4xl md:text-5xl lg:text-6xl`). Subtitle body text uses `text-text-secondary`.

### Section 2 — Mission Pillars

Three cards displayed in a row (stack vertically on mobile). Each card:

| Card | Title                    | Body (verbatim)                                                                                                                                                                                                   |
| ---- | ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1    | **Own the Standard**     | Establish our text-native format as the universal system of record for AI orchestration, creating switching costs through workflow IP encoded in `.ash` scripts.                                                    |
| 2    | **Commoditize the Supplier** | Decouple enterprise value from underlying AI models. Our platform-agnostic architecture lets organizations hot-swap any LLM provider without touching their workflows.                                         |
| 3    | **Eliminate Enterprise Liability** | Transform unpredictable, stochastic AI behavior into auditable, deterministic business processes with full audit trails, state control, and governed execution.                                     |

Each card includes a link at the bottom: "Read our full mission" → `/vision`.

Design: cards with subtle background (`bg-bg-secondary`), border (`border-border`), rounded corners, padding. Icon/emoji optional per card (not specified — implementer may add a visual element or leave as text-only). Use `MissionPillarCard.astro` component for reusability (the `/vision` page also uses these cards in full).

### Section 3 — Product Teaser

Condensed from the current `index.html` compare/pain-points section. Three cards:

| Card | Title                 | Body                                                                                                                                                                            |
| ---- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | Deterministic Control | Sequencing, branching, retries, and conditional logic run at CPU speed. AI time is spent only on content where judgment matters.                                                |
| 2    | Agent-Agnostic        | Swap AI providers without touching your workflows. Configure once. Use OpenCode, Claude Code, Aider, or any CLI-based tool.                                                     |
| 3    | Workflows as Code     | Commit, version, share, and re-run. Workflow IP lives in files — not buried in chat threads. Review, audit, and publish like any other software artifact.                       |

Each card: one heading + one sentence. Link at bottom: "Learn more →" or "Read the docs →" pointing to `/docs`.

### Section 4 — Board Teaser

Text section, no cards. Verbatim copy:

> ## Our Leadership
>
> Ash is guided by a board of directors with deep expertise in developer tools, enterprise architecture, and open-source communities.
>
> **Sarah Chen** — Open-Source & Community Veteran · **James Okonkwo** — DevTools GTM Expert · **Priya Nair** — Investor Director, Framework Capital · **Marcus Thorne** — Co-Founder & CEO · **Dr. Elena Vasquez** — Enterprise Architecture Leader
>
> [Meet the board →](/board)

The director names should be visually distinct (e.g., bold, accent color). The link is a standard CTA to `/board`.

### Section 5 — Blog Highlight

Client-side component (`src/components/BlogHighlight.astro`). Renders:

```html
<div id="blog-highlight" style="display:none">
  <h2>Latest from the Blog</h2>
  <div id="blog-highlight-posts"></div>
</div>
<script>
  fetch('/api/recent-posts')
    .then(r => r.json())
    .then(posts => {
      if (!Array.isArray(posts) || posts.length === 0) return;
      var container = document.getElementById('blog-highlight');
      var postsEl = document.getElementById('blog-highlight-posts');
      posts.forEach(p => {
        var a = document.createElement('a');
        a.href = '/blog/' + p.slug;
        a.textContent = p.title;
        // render post card/row with title, date
      });
      container.style.display = 'block';
    })
    .catch(() => { /* fail silently — container stays hidden */ });
</script>
```

Key behaviors:
- Container starts with `display:none` — invisible by default.
- If `/api/recent-posts` returns an empty array `[]`, the container stays hidden (no heading "no posts" message).
- If the fetch fails (network error, 500), the container stays hidden — fail gracefully.
- If posts are returned, the container becomes visible and renders linked post titles (and optional dates).
- No loading indicator, no skeleton screens — content either appears or doesn't. This is a deliberate trade-off for a simple static homepage.

The `/api/recent-posts` endpoint is built in a downstream task (Blog System). For now, the BlogHighlight component simply expects it to exist. During local development before the blog task is done, the fetch will fail → container stays hidden → no visible issue.

### Section 6 — Footer

Rendered by `BaseLayout` via `Footer.astro` (built in Phase 01). No additional homepage-specific footer content.

### Between-Section Spacing

Use consistent vertical rhythm. Typical spacing: `py-16 md:py-24` per section, with `border-b border-border` optionally separating sections. Do not crowd sections together — the page should feel spacious on desktop.

## Acceptance Criteria

1. Page renders at `/` (static, prerendered) with all six sections in order: Hero, Mission Pillars, Product Teaser, Board Teaser, Blog Highlight, Footer.
2. Hero contains the exact heading "Orchestrating AI" and the specified subheading text (verbatim or with only minor layout-fit edits).
3. Two CTA buttons are present: "Learn About Ash" → `/docs`, "Try the Playground" → `/playground`.
4. Three mission pillar cards contain the exact body text specified above, each with a "Read our full mission" → `/vision` link.
5. Three product teaser cards contain the specified headings and body text, each with a link to `/docs`.
6. Board teaser lists all five director names and links to `/board`.
7. Blog highlight section is invisible when `/api/recent-posts` returns `[]` or the fetch fails.
8. Blog highlight section displays post titles as links to `/blog/[slug]` when posts are returned.
9. Page is mobile-responsive: single-column on mobile, cards reflow to multi-column on larger screens.
10. Page uses `BaseLayout` — no duplicate `<head>`, `<header>`, or `<footer>` markup.

## Implementation Hints

- Import and use `BaseLayout` from `../layouts/BaseLayout.astro`.
- Create `src/components/MissionPillarCard.astro` for the mission pillar cards. Accept `title` and `body` as props. This component is also used on the Vision page.
- Create `src/components/BlogHighlight.astro` for the client-side blog fetch. This is an Astro component with an inline `<script>` tag — no framework, no external JS.
- The blog fetch URL is `/api/recent-posts`. This endpoint will be created in the Blog System task (03-dynamic-features/01). Use that exact path.
- Use Tailwind responsive grid utilities: `grid grid-cols-1 md:grid-cols-3` for card rows.
- Brand green (`accent-green`) for primary CTAs, links, and heading accents.
- The hero subtext should use `text-text-secondary` for legibility on the dark background.
- Copy should be used verbatim — do not rewrite the company value proposition.
- The blog highlight `<script>` uses vanilla JS only — no JSX, no reactivity framework.
- `BaseLayout`, `Nav`, `Footer` are already built — import and use them, do not recreate.
