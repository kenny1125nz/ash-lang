# Reference Page — Fragment Template, Target & Astro Page

## Background

`docs/assemble.js` auto-discovers targets from `docs/targets/*.json`, reads `docs/units/*.md`, applies templates, and writes output. The current `website-html.json` target produces a full HTML document at `web-site/reference.html` using `docs/templates/reference.html` as its template.

**Problem:** A full HTML document template cannot be used as a fragment inside an Astro page. The reference page needs the content body only — no `<html>`, `<head>`, `<body>`, navigation, or footer.

**Solution:** Add a new target and fragment template alongside the existing ones. `assemble.js` itself is never touched — it auto-discovers the new target from its directory scan. The existing `website-html.json` target and `reference.html` template remain unchanged and continue producing the full-HTML version for GitHub/README consumers.

## Intended Solution

### Step 1: Create Fragment Template

**`docs/templates/reference-fragment.html`:**

```html
<div class="ref-hero">
  {{HERO}}
</div>
<div class="ref-content">
  {{CONTENT}}
</div>
```

Only the body content — no `<html>`, `<head>`, `<body>`, `<header>`, `<nav>`, `<footer>`, or `<style>` tag. The `{{HERO}}` and `{{CONTENT}}` placeholders are substituted by `assemble.js` with the title markdown rendered as HTML and the concatenated unit content rendered as HTML, respectively.

### Step 2: Create New Target

**`docs/targets/website-fragment.json`:**

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

This mirrors `website-html.json` but outputs to `web-site/src/content/reference-content.html` using the fragment template. The `assemble.js` script auto-creates missing parent directories via its `validateOutputPaths` function.

### Step 3: Create Astro Reference Page

**`src/pages/reference.astro`** — static page using `BaseLayout` with `title="Feature Reference"`.

Reads the committed fragment at build time. Since `src/content/reference-content.html` is a static HTML fragment, import it as raw text and inject with `@html`:

```astro
---
import BaseLayout from '../layouts/BaseLayout.astro';
import fragment from '../content/reference-content.html?raw';
---

<BaseLayout title="Feature Reference">
  <div class="container mx-auto px-4 py-12">
    {@html fragment}
  </div>
</BaseLayout>
```

The fragment contains `{{HERO}}` and `{{CONTENT}}` replaced by actual HTML at generation time, so the page renders the full reference content.

**Important:** `@html` renders unescaped HTML. The content comes from `docs/assemble.js` which uses `marked` to render trusted markdown source files (not user input), so this is safe.

### Step 4: CI Fragment Commit

The `build-and-test` CI job must add the new fragment to its commit list. Add this line to the existing `git add` block in `.github/workflows/ci.yml`:

```yaml
git add "${{ github.workspace }}/web-site/src/content/reference-content.html"
```

This ensures the fragment is committed to the repo alongside the other generated docs. Wait — the CI job is handled in a downstream task (04-deployment/01-ci-pipeline.md). The reference page task does NOT modify CI — it only ensures the fragment can be generated and consumed. The CI update is the downstream task's responsibility.

## Acceptance Criteria

1. `docs/templates/reference-fragment.html` exists with exactly the content shown above (no HTML boilerplate).
2. `docs/targets/website-fragment.json` exists with the specified `output`, `title`, `units`, and `template` fields.
3. Running `node docs/assemble.js` produces `web-site/src/content/reference-content.html` with the rendered reference content.
4. The original `docs/targets/website-html.json` and `docs/templates/reference.html` are unchanged.
5. `docs/assemble.js` itself is untouched — it auto-discovers the new target.
6. `/reference` page renders the full feature reference within `BaseLayout`.
7. The reference page is a static (prerendered) page.

## Implementation Hints

- The fragment template is deliberately minimal — only two `<div>` containers with placeholders. The CSS classes `.ref-hero` and `.ref-content` can be styled with Tailwind in the Astro page or left unstyled if Tailwind utilities handle it.
- The `assemble.js` output path is `web-site/src/content/reference-content.html`. The directory `web-site/src/content/` exists (created in Phase 01 scaffolding). `assemble.js`'s `validateOutputPaths` auto-creates directories, so even if the directory is missing, it will be created.
- The existing `website-html.json` and `reference.html` files are NOT to be modified. The fragment system is purely additive.
- Astro's `?raw` import suffix imports a file as a plain string. This is a standard Vite feature available in Astro. Alternative: use `fs.readFileSync` in the frontmatter if `?raw` doesn't work for content outside `src/`.
- The CSS classes `.ref-hero` and `.ref-content` in the fragment: either add minimal Tailwind-equivalent styles in a `<style>` block in `reference.astro`, or leave them as semantic wrappers and style the containing elements with Tailwind. The original `reference.html` template has `.ref-hero` and `.ref-content` styling — replicate the visual appearance with Tailwind.
- The reference page should match the visual style of the current reference page (dark theme, code blocks, headings hierarchy) but using Tailwind instead of the old `style.css` `<style>` tag.
- Do NOT edit `docs/assemble.js`, `docs/targets/website-html.json`, or `docs/templates/reference.html`.
