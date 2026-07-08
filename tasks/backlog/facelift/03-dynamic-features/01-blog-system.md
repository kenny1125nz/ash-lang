# Blog System — Listing, Posts & Recent Posts API

## Background

The site needs a blog backed by Cloudflare D1 storage, served via SSR Astro routes. Blog posts are created via `wrangler d1 execute --file` (CLI only, no admin UI). At launch, the `posts` table is empty — the blog page shows "No posts yet."

The D1 migration (`migrations/0001_initial.sql`) was created during Phase 01. The `posts` table schema is:

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

`author` is a free-text display field (e.g., "Sarah Chen") with no FK to the directors table.

## Intended Solution

Three SSR routes, all with `export const prerender = false`:

### `/blog` — Blog Listing (`src/pages/blog/index.astro`)

- Queries D1: `SELECT title, slug, author, published_at FROM posts ORDER BY published_at DESC`
- No pagination — all posts on one page.
- Each list item renders: title (linked to `/blog/[slug]`), author name, formatted date.
- **Empty state:** If query returns zero rows, display: "No posts yet." — centered, muted paragraph.
- **Error state:** If D1 query fails, return 500 with message "Unable to load posts. Please try again later." Log the error via `console.error`.
- **Malformed record:** If a row is missing required fields (`title`, `slug`, `author`, `published_at`), skip it in the listing and log a warning via `console.warn`.

### `/blog/[slug]` — Single Post (`src/pages/blog/[slug].astro`)

- Queries D1: `SELECT * FROM posts WHERE slug = ?` with the route param as the binding value.
- Renders: title (H1), author, formatted date, full body (markdown → HTML via `marked`).
- **Nonexistent slug:** `Astro.redirect('/404')` or `return new Response(null, { status: 404 })` with a 404 page.
- **Error state:** D1 query failure → 500 with "Unable to load posts. Please try again later." Log the error.
- **Malformed record:** Missing required fields → 500. Log a warning.

### `/api/recent-posts` — Recent Posts JSON Endpoint (`src/pages/api/recent-posts.ts`)

- GET endpoint. SSR.
- Queries D1: `SELECT title, slug, published_at FROM posts ORDER BY published_at DESC LIMIT 3`
- Returns JSON: `Content-Type: application/json` with array of `{ title, slug, published_at }`.
- Empty table returns `[]`.
- D1 query failure: returns 500 JSON `{ error: "Unable to load posts." }`. Log the error via `console.error`.
- This is NOT an Astro `.astro` page — it's a `.ts` file in `src/pages/api/`. Use Astro's API route pattern with a `GET` export function or the older `Astro.request` pattern.

### D1 Access Pattern

D1 bindings are exposed to SSR routes via `context.locals.runtime.env.DB`. In Astro API routes:

```ts
// src/pages/api/recent-posts.ts
export const prerender = false;

export async function GET({ locals }: { locals: { runtime: { env: { DB: D1Database } } } }) {
  const db = locals.runtime.env.DB;
  const { results } = await db.prepare('SELECT title, slug, published_at FROM posts ORDER BY published_at DESC LIMIT 3').all();
  return new Response(JSON.stringify(results ?? []), {
    headers: { 'Content-Type': 'application/json' },
  });
}
```

For `.astro` SSR pages, access `Astro.locals.runtime.env.DB` in the frontmatter:

```astro
---
export const prerender = false;
const db = Astro.locals.runtime.env.DB;
const { results } = await db.prepare('SELECT ...').all();
---
```

### `marked` Configuration

Use the `marked` library to render post `content` (markdown) to HTML:

```ts
import { marked } from 'marked';
const html = marked.parse(post.content);
```

Render the HTML with `@html` directive in Astro: `<article>{@html html}</article>` or `set:html={html}`.

### Formatted Dates

The `published_at` field is ISO 8601 text. Format for display using JavaScript's `new Date(published_at).toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })`.

### Blog Highlight Integration

The Homepage task created `BlogHighlight.astro` which fetches `/api/recent-posts`. That's it — no other integration needed. The BlogHighlight component handles the empty state correctly: if this endpoint returns `[]`, the homepage blog section stays hidden.

## Acceptance Criteria

1. `/blog` lists all posts from D1 ordered by `published_at DESC`, with title (linked), author, and date.
2. `/blog` shows "No posts yet." centered when the `posts` table is empty.
3. `/blog` returns 500 with "Unable to load posts. Please try again later." when D1 query fails.
4. `/blog/[slug]` renders a single post with title, author, date, and markdown content as HTML.
5. `/blog/[slug]` returns 404 for nonexistent slugs.
6. `/blog/[slug]` returns 500 with "Unable to load posts. Please try again later." when D1 query fails.
7. `/api/recent-posts` returns JSON array of latest 3 posts `[{ title, slug, published_at }]` with `Content-Type: application/json`.
8. `/api/recent-posts` returns `[]` when the `posts` table is empty.
9. `/api/recent-posts` returns 500 JSON `{ error: "..." }` when D1 query fails.
10. All three routes have `export const prerender = false`.
11. SSR routes build and deploy — they are packaged as Pages Functions by the Cloudflare adapter.

## Implementation Hints

- D1 binding name is `DB` as defined in `wrangler.toml` (`binding = "DB"`).
- Astro `.ts` API routes use `export async function GET(context)` or `export const GET = async (context) => ...`. Check the installed `@astrojs/cloudflare` adapter version for the exact `locals` shape — the D1 binding is at `context.locals.runtime.env.DB`.
- For `.astro` pages, D1 is accessed in the frontmatter `---` block via `Astro.locals.runtime.env.DB`.
- The `blog/[slug].astro` file uses `Astro.params.slug` to get the dynamic route parameter.
- The `marked` library is in `package.json` as `"marked": "^14.0"`. Import it with `import { marked } from 'marked'`.
- Blog posts use the `BaseLayout` with appropriate titles (e.g., `title={post.title}` for the post page, `title="Blog"` for the listing page).
- Create a `src/components/BlogCard.astro` component for the blog listing item — accepts `title`, `slug`, `author`, `published_at` props.
- For the empty state, use a conditional: `{results.length === 0 ? <p>No posts yet.</p> : ...}`.
- No pagination needed — list all posts on one page.
- No authentication, no admin UI — content insertion is manual via CLI.
