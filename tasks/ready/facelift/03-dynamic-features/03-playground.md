# Playground Page — WASM Editor, REPL & DOM Contract

## Background

The WASM playground lets users write and run Ash scripts in the browser. It depends on two JavaScript files (`playground.js` and `eval-worker.js`) and a compiled WASM module from `ash-wasm/`. These JS files were moved to `public/js/` during Phase 01 scaffolding (with the Worker URL fix applied). The WASM files are copied to `public/wasm/` by the CI pipeline.

The playground's JavaScript uses `getElementById` for all DOM interactions. The new `/playground` Astro page must preserve every DOM ID from the original page so the scripts work without modification. Additionally, CSS classes for syntax highlighting, the editor overlay technique, terminal animation, toast notifications, and REPL styling are preserved in `globals.css` (from Phase 01).

## Intended Solution

### `/playground` Page (`src/pages/playground.astro`)

Static page using `BaseLayout` with `title="Playground"`. Renders the playground HTML structure with all required DOM IDs and loads the playground scripts.

### Required DOM IDs (Must Be Present)

Every element ID from the original `index.html` playground section must be present:

| Element ID         | Purpose                         | Type         |
| ------------------ | ------------------------------- | ------------ |
| `editor`           | CodeMirror/textarea mount point | `<textarea>` |
| `editor-highlight` | Syntax highlighting overlay     | `<div>`/`<pre>` |
| `output`           | Execution output display        | `<div>`      |
| `status`           | Execution status indicator      | `<span>`     |
| `wasm-status`      | WASM load status indicator      | `<span>`     |
| `script-tab`       | Script editor tab button        | `<button>`   |
| `repl-tab`         | REPL tab button                 | `<button>`   |
| `script-panel`     | Script editor panel container   | `<div>`      |
| `repl-panel`       | REPL panel container            | `<div>`      |
| `repl-prompt`      | REPL prompt display             | `<span>`     |
| `repl-input`       | REPL input field                | `<input>`    |
| `repl-output`      | REPL output display             | `<div>`      |
| `toast`            | Notification toast              | `<div>`      |

### Example Preload Elements (`.example-code` class)

The `highlightExamples()` function in `playground.js` (line 370) calls `document.querySelectorAll('.example-code')` and applies syntax highlighting. Elements with this class should be present on the page — they serve as pre-loaded example code that gets syntax-highlighted.

Add one or more hidden (or visible) `<pre><code class="example-code">` elements containing Ash example scripts. The scripts can be drawn from the default editor content or from known Ash examples. These elements are **not** present in the current `index.html` (the `highlightExamples()` is dead code currently), so the implementer must create them. Example:

```html
<pre style="display:none"><code class="example-code">#!js-echo:0.1.0

print "=== Ash Playground ==="

TASKS = ["Check login", "Test search", "Review footer"]
for TASK in TASKS {
	do "Navigate and ${TASK}"
	print "Task: ${TASK}"
	print "Result: ${stdout}"
}

print "Completed ${len(TASKS)} tasks"</code></pre>
```

### Page Structure

**Tab bar:** Two buttons: "Script" (`#script-tab`) and "REPL" (`#repl-tab`). Script tab starts active. Clicking switches visibility between the script panel and REPL panel.

**Script panel (`#script-panel`):**
- Editor area with textarea (`#editor`) and overlay div (`#editor-highlight`) for syntax highlighting. The textarea must have `spellcheck="false"`. The overlay technique: the textarea has transparent text color and a visible caret; the overlay div shows the syntax-highlighted version behind it. Both must share the same font, size, padding, and content.
- Output area (`#output`) for execution results.
- Status indicator (`#status`) and WASM status (`#wasm-status`) in a toolbar area.
- Toolbar with "Run" button (calls `runScript()`), "Load Example" button, "Clear" button.

**REPL panel (`#repl-panel`):**
- Output area (`#repl-output`) showing REPL history.
- Input area with prompt (`#repl-prompt`) displaying `> ` and input field (`#repl-input`).

**Toast (`#toast`):** Empty div at page bottom for notifications.

### Script Loading

At page bottom, load the playground scripts:

```html
<script src="/js/eval-worker.js"></script>
<script src="/js/playground.js"></script>
```

The worker is loaded as a separate file reference (not directly — `playground.js` creates a `new Worker('/js/eval-worker.js', ...)` with the absolute path fixed in Phase 01).

### Layout

- Desktop (≥768px): editor and output side by side.
- Mobile (<768px): editor and output stacked vertically.
- Use Tailwind responsive utilities (`flex flex-col md:flex-row`, etc.).
- Dark theme styling consistent with the rest of the site.

## Acceptance Criteria

1. All 13 DOM IDs from the table above are present on the `/playground` page.
2. The `.example-code` class is present on at least one `<code>` element with example Ash script content.
3. `playground.js` and `eval-worker.js` are loaded via `<script>` tags with correct `/js/` paths.
4. Script tab and REPL tab switch visibility between the two panels when clicked.
5. The editor textarea has `spellcheck="false"` attribute.
6. The "Run" button triggers script execution via the WASM playground.
7. Syntax highlighting works in the editor (tokens are colored per `globals.css` rules).
8. WASM status indicator shows loading status.
9. Page uses `BaseLayout` for shared shell.
10. Page is mobile-responsive: editor+output stacked on mobile, side-by-side on desktop.

## Implementation Hints

- The `playground.js` Worker URL was fixed in Phase 01 (line 283 changed to `/js/eval-worker.js`). Verify the moved file at `public/js/playground.js` has this change.
- The `eval-worker.js` at `public/js/eval-worker.js` is unchanged from the original.
- The WASM files (`public/wasm/*.wasm`, `public/wasm/*.js`) are copied by CI — they are NOT part of this task. The page should reference them as `wasm/ash_wasm.js` (the path `playground.js` uses to import the WASM module). Verify this path matches where CI copies the files.
- The editor overlay technique requires precise CSS: the textarea and overlay must have identical font (JetBrains Mono, 13px), identical padding, identical dimensions. The textarea has `color: transparent` and `caret-color: #e6edf3`. The overlay has `pointer-events: none`. These styles are in `globals.css` (Phase 01) via `.editor-wrap`, `.editor-highlight`, and `.editor-wrap textarea`.
- The tab-switching function `switchTab()` is defined in `playground.js`. The buttons need `onclick="switchTab('script')"` and `onclick="switchTab('repl')"` — same as the original HTML.
- The "Run" button: `onclick="runScript()"`. "Load Example": `onclick="loadExample()"`. "Clear": `onclick="clearEditor()"` and `onclick="clearOutput()"`.
- Terminal animation script (IntersectionObserver) is NOT needed on the playground page — it's used on `/docs`.
- Do NOT edit `playground.js` or `eval-worker.js` — the edits were done in Phase 01.
- The toast notification works via the `showToast()` function in `playground.js` — just ensure `#toast` div exists.
- Styling: use Tailwind wherever possible. The supplementary CSS in `globals.css` handles the editor overlay, syntax highlighting tokens, terminal animation, toast, and REPL prompt. Everything else is Tailwind.
