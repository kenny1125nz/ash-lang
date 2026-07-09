# Shared Layout Components — BaseLayout, Nav, Footer

## Background

The existing website has copy-pasted `<header>`, `<nav>`, and `<footer>` elements in every HTML file. Astro's layout system eliminates this duplication. These three components are used by every page in the site.

The site is entirely dark-themed (no light/dark toggle). All colors come from the Tailwind theme extensions defined in `tailwind.config.mjs` during Phase 01.

## Intended Solution

### BaseLayout (`src/layouts/BaseLayout.astro`)

The shared shell for every page. Receives a `title` prop for the `<title>` tag and renders `<slot />` for page content.

Structure:
```
<!DOCTYPE html> → <html lang="en"> → <head> (meta, fonts, stylesheet, title) → <body> (Nav, <main><slot /></main>, Footer)
```

`<head>` must include:
- `<meta charset="UTF-8">`
- `<meta name="viewport" content="width=device-width, initial-scale=1.0">`
- `<title>` using the `title` prop suffixed with ` — Ash` (e.g., `"Home — Ash"`)
- Google Fonts preconnect and stylesheet for JetBrains Mono (same as current `index.html`)
- Optional: favicon link to `/favicon.ico`

The `<body>` has `bg-bg-primary text-text-primary font-sans` as minimum base classes.

The `<main>` element wraps `</slot>` with responsive horizontal padding (e.g., `px-4 md:px-6 lg:px-8`).

### Nav (`src/components/Nav.astro`)

Responsive navigation bar with logo and hamburger toggle.

**Logo:** Links to `/`. Displays "ash" in brand green (`text-accent-green`), bold, monospace or styled font.

**Primary nav items (horizontal above 768px, hidden below):**

| Label      | Link        | Sub-items (desktop dropdown or simple)                      |
| ---------- | ----------- | ----------------------------------------------------------- |
| Home       | `/`         | —                                                           |
| About      | —           | `/vision` (Vision & Mission), `/board` (Board of Directors) |
| Docs       | —           | `/docs` (Getting Started), `/agents` (Agents), `/reference` (Reference) |
| Blog       | `/blog`     | —                                                           |
| Playground | `/playground` | —                                                        |
| Contact    | `/contact`  | —                                                           |

Desktop: horizontal link bar with dropdowns for About and Docs groups. Or simpler: flat horizontal links for each top-level item, with "About" and "Docs" linking to their most representative page and sub-links shown on hover.

**Mobile (< 768px):** Hamburger icon button. On click, reveals a vertical menu overlay or expanding panel with all links and sub-items. Minimum 44×44px tap target for the hamburger button and each nav link.

**GitHub link:** External link to `https://github.com/kenny1125nz/ash-lang` with `target="_blank"` and `rel="noopener noreferrer"`. Present in both desktop and mobile nav.

**Active state:** Highlight the current page's nav item using a CSS class or Astro's `Astro.url.pathname` check. Active items use brand green color.

### Footer (`src/components/Footer.astro`)

Dark background (`bg-bg-secondary`), border-top (`border-border`). Contains:

1. Company name: "Ash" — prominent, maybe larger font with brand green.
2. Navigation links row: Home, Vision & Mission, Board, Docs, Agents, Reference, Blog, Playground, Contact.
3. GitHub link (external, same URL as nav).
4. Copyright: `© {current year} Ash. All rights reserved.`

Footer is a shared component rendered by `BaseLayout`. No props needed unless dynamic year is injected.

## Acceptance Criteria

1. `BaseLayout.astro` renders a complete HTML document with shared `<head>` (meta tags, fonts, title) wrapping a `<main>` slot area.
2. `Nav.astro` shows a horizontal link bar at ≥768px viewport width.
3. `Nav.astro` collapses to a hamburger menu at <768px viewport width with all links accessible.
4. All hamburger menu interactive elements and nav links have minimum 44×44px tap targets on mobile.
5. The GitHub external link is present in both desktop and mobile navigation.
6. `Footer.astro` displays company name, nav links, GitHub link, and copyright year.
7. The entire page uses dark theme colors from `tailwind.config.mjs` (background, text, accents).
8. All components compile without errors when imported by a simple test page.

## Implementation Hints

- Build with Tailwind utility classes exclusively — no custom CSS beyond what's in `globals.css`.
- Use `Astro.props.title` in BaseLayout for the page title.
- Mobile hamburger: use a `<script>` block in Nav.astro (or an inline `onclick` + CSS class toggle). No JavaScript framework. Keep it simple — a `hidden`/`block` toggle on a menu div is sufficient.
- For desktop dropdowns on "About" and "Docs", CSS `:hover` with `group`/`group-hover` Tailwind classes is preferred over JavaScript.
- Active nav state: check `Astro.url.pathname` and conditionally apply `text-accent-green` or a border-bottom style.
- Brand green (`#3fb950`) maps to `accent-green` in Tailwind config.
- Use `new Date().getFullYear()` for the dynamic copyright year in Footer (inline in the Astro template).
- Follow Astro conventions: `---` frontmatter fence for component logic, HTML below.
- The `globals.css` file already exists from Phase 01 — do not modify it.
