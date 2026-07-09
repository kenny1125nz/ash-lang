# Documentation & Agents Pages

## Background

The current `index.html` contains a "Getting Started" section (`#getting-started`) and a "How Ash Works" section (`#how-it-works`). A separate `agents.html` lists all supported AI agents with configuration details.

These are documentation content, not marketing copy. Per the design's framing rule: **documentation content (commands, code examples, agent lists, configuration formats) is preserved verbatim and only restyled.** Framing/tone/marketing content (hero text, descriptions, CTAs) is rewritten to company voice, but that's handled by the Homepage task. This task deals with the preserved docs.

## Intended Solution

### `/docs` Page (`src/pages/docs.astro`)

Static page using `BaseLayout` with `title="Documentation"`. Contains content from `index.html`:

**Getting Started section** (from `#getting-started` in current `index.html`):
- Text: "Built-in support: opencode, claude-code, aider, codex, gemini-cli, kimi, and more. See [supported agents](/agents) for the full list and default configs."
- Code example: `ash --agent opencode ./tasks` with explanatory text preserved.
- Code example: `ash discover` for discovering new agents.
- YAML configuration example: `ash.yml` agent config block.
- Text about `tasks/` folder structure, `.md` task files, `.ash` script files.

All commands, code blocks, and YAML examples are preserved verbatim. The surrounding explanatory paragraphs are also preserved (they are documentation, not marketing framing).

**How Ash Works section** (from `#how-it-works` in current `index.html`):
- File tree diagram: `tasks/` folder structure with numbered `.md` files.
- Terminal animation showing `ash --agent opencode tasks/` and execution steps.
- The terminal animation must work: it uses `.terminal`, `.terminal-line`, and `@keyframes typeIn` CSS classes defined in `globals.css` (Phase 01). The IntersectionObserver script from the original `index.html` must also be included to trigger the animation on scroll.

**Syntax Reference section** (from current `index.html` syntax-ref section):
- Four code cards: Variables/Strings/Shell, Agent Calls, Control Flow, Functions & Composition.
- Shebang example: `#!opencode:1.0`.
- Frontmatter example: YAML `agent:` and `model:` in markdown frontmatter.

Content extraction approach: Read `web-site/index.html` (it still exists at implementation time — this task runs before the Scaffolding task's deletion of `index.html`). Extract relevant sections using their `id` attributes (`#getting-started`, `#how-it-works`) and the `.syntax-ref` class section. Convert the HTML to Astro-compatible markup using Tailwind classes.

**Important:** The scaffolding task deletes `web-site/index.html`. That's fine — this task, as part of Phase 02, runs AFTER Phase 01 in the implementation sequence. But if there's any issue with ordering, the content can also be sourced from reading the file inline in the task (the key content is described above).

### `/agents` Page (`src/pages/agents.astro`)

Static page using `BaseLayout` with `title="Supported Agents"`. Content from `agents.html` preserved verbatim.

The current `agents.html` is a full HTML page (~192 lines) with a header, nav, a big table of agents, and a footer. Extract just the content body (the agent table) and restyle with Tailwind:

- **Agent table:** A table listing supported agents with columns: Agent Name, Type, Default Model, Configuration Notes.
- The table data must match `agents.html` exactly. Preserve all agent entries, model names, and CLI flags.
- Use Tailwind table styling: `bg-bg-secondary`, `border-border`, alternating row subtle shading.
- For each agent, preserve any configuration YAML blocks shown below the table in `agents.html`.
- Links to agent homepages/documentation are preserved.

## Acceptance Criteria

1. `/docs` displays Getting Started content with all commands and code examples preserved verbatim from the original `index.html`.
2. `/docs` displays the terminal animation (`ash --agent opencode tasks/` with step-by-step execution lines) that animates on scroll into view.
3. `/docs` displays the syntax reference cards (Variables, Agent Calls, Control Flow, Functions) with all code examples preserved.
4. `/agents` displays the complete agent table with all entries from the original `agents.html`.
5. Both pages use Tailwind styling — no vanilla CSS classes from the old `style.css` are referenced.
6. Both pages use `BaseLayout` for shared shell.
7. Documentation content is not rewritten — only restyled.
8. `web-site/index.html` and `web-site/agents.html` are deleted after content extraction.

## Implementation Hints

- The source `web-site/index.html` and `web-site/agents.html` are available (not deleted by the Scaffolding task). Read them directly for content extraction.
- After extracting all content, delete `web-site/index.html` and `web-site/agents.html` — they are fully replaced by Astro pages.
- For the terminal animation: include the IntersectionObserver script inline (from `web-site/index.html` lines 281–299). This is a small vanilla JS snippet.
- The `.terminal`, `.terminal-line`, `@keyframes typeIn`, `.tp`, `.ag`, `.model`, `.ash-ext` classes/styles are in `globals.css` (from Phase 01).
- Code blocks: use `<pre><code>` with Tailwind styling (`bg-bg-tertiary rounded-lg p-4 overflow-x-auto font-mono text-sm`).
- The YAML config example block uses `bg-bg-tertiary` with monospace font.
- Table styling: `w-full border-collapse` with `border border-border` on cells, `bg-bg-secondary` on header row, alternating `bg-bg-primary`/`bg-bg-secondary` on data rows.
- All links that pointed to `index.html` sections (e.g., `index.html#getting-started`) should be updated to use the new routes (e.g., `/docs` or `/docs#getting-started`).
- Import `BaseLayout` from `../layouts/BaseLayout.astro`.
