# Contact & Feedback — Form Page, API Endpoint & Tests

## Background

The site needs a contact form and a server-side feedback collection endpoint. The form captures user feedback (general inquiries, bug reports, feature requests). Data is stored in Cloudflare D1 and retrievable via `wrangler d1 execute` queries. No admin UI is in scope.

The D1 `feedback` table was created in Phase 01 (`migrations/0001_initial.sql`):

| Column       | Type    | Constraints                        |
| ------------ | ------- | ---------------------------------- |
| `id`         | INTEGER | PRIMARY KEY AUTOINCREMENT          |
| `name`       | TEXT    | NOT NULL                           |
| `email`      | TEXT    | NOT NULL                           |
| `message`    | TEXT    | NOT NULL                           |
| `category`   | TEXT    | NOT NULL                           |
| `created_at` | TEXT    | NOT NULL DEFAULT (datetime('now')) |

The endpoint has **no authentication, CAPTCHA, or rate limiting**. This is an acknowledged limitation — anti-spam is out of scope for the initial launch. A 64 KB request body size limit is the only defense.

## Intended Solution

### `/contact` Page (`src/pages/contact.astro`)

Static page using `BaseLayout` with `title="Contact"`. Contains:

**Company info section** (above the form):
- Brief text: "Have questions about Ash, found a bug, or want to share feedback? We'd love to hear from you."
- Optional: email address, GitHub link, or other contact channels.

**Feedback form** using `FeedbackForm.astro` component:

Fields:
- **Name:** `<input type="text" name="name" required>` — text, required
- **Email:** `<input type="email" name="email" required>` — email, required
- **Category:** `<select name="category" required>` with options: "General", "Bug Report", "Feature Request", "Other"
- **Message:** `<textarea name="message" required>` — textarea, required

Client-side validation (inline `<script>` in the component):
- All required fields must be non-empty (trimmed).
- Email must contain `@`.
- Errors displayed inline next to the offending field (e.g., `<span class="text-accent-red text-sm">Email is required.</span>`).
- Valid submission: `POST` JSON `{ name, email, message, category }` to `/api/feedback` via `fetch`.
- On success (`{ ok: true }`): show "Thank you for your feedback." message, reset form.
- On error (non-200): show error message from response or a generic "Something went wrong." message.

Styling:
- Form inputs: full-width on mobile, labels above inputs.
- Base font size minimum 16px on inputs (prevents iOS zoom on focus) — use Tailwind's `text-base` which maps to 16px.
- Input styles: dark background (`bg-bg-secondary`), border (`border-border`), focus ring with brand green.
- Submit button: brand green background (`bg-accent-green`), white/light text, full-width on mobile, auto-width on desktop.

### `/api/feedback` Endpoint (`src/pages/api/feedback.ts`)

SSR API route. Accepts POST with JSON body. Returns JSON responses.

**Request body size limit:** 64 KB. Read the body and check its byte length. If > 65536 bytes, return `413` with `{ error: "Request body too large." }`.

**Server-side validation** (mirrors client validation):
| Check | Error code | Error message |
|-------|------------|---------------|
| `name`: non-empty string, trimmed length > 0 | 400 | `"Name is required."` |
| `email`: non-empty string, contains `@` | 400 | `"Email is required."` or `"Email must contain @."` |
| `message`: non-empty string, trimmed length > 0 | 400 | `"Message is required."` |
| `category`: must be one of `["General", "Bug Report", "Feature Request", "Other"]` | 400 | `"Invalid category."` |

Validation order: check each field in the order above. Return the FIRST error encountered.

**D1 write:** `INSERT INTO feedback (name, email, message, category) VALUES (?, ?, ?, ?)`

**Error handling:**
- D1 write failure → 500 with `{ error: "Unable to save feedback. Please try again later." }`. Log the actual error via `console.error`.
- D1 connection/timeout → 503 with `{ error: "Service temporarily unavailable. Please try again later." }`.
- Successful write → 200 with `{ ok: true }`.

**Implementation pattern:**

```ts
export const prerender = false;

export async function POST({ request, locals }: { request: Request; locals: { runtime: { env: { DB: D1Database } } } }) {
  // 1. Check body size (Content-Length header or byte length)
  // 2. Parse JSON
  // 3. Validate fields in order — return first error as 400
  // 4. Insert into D1
  // 5. Return { ok: true } or error
}
```

### Tests (`web-site/tests/feedback.test.ts`)

Integration tests for the feedback API validation logic. Since the `@astrojs/cloudflare` runtime isn't available outside `astro dev`'s Miniflare, extract validation as a pure function that the endpoint calls, then test it directly with mock D1.

**Approach:** Export a `validateFeedback(body)` function from a shared module (e.g., `src/lib/feedback.ts`) that performs validation and returns `{ valid: true, data }` or `{ valid: false, status, error }`. The API endpoint calls this function, then writes to D1 on success. Tests call `validateFeedback()` directly — no network, no D1.

**Test cases** (use `describe`/`it` blocks):

1. Valid submission → returns `{ valid: true, data: { name, email, message, category } }`
2. Missing name → returns `{ valid: false, status: 400, error: "Name is required." }`
3. Empty name (whitespace only) → returns 400, "Name is required."
4. Missing email → returns 400, "Email is required."
5. Email without `@` → returns 400, "Email must contain @."
6. Missing message → returns 400, "Message is required."
7. Empty message (whitespace only) → returns 400, "Message is required."
8. Invalid category (e.g., "Spam") → returns 400, "Invalid category."

**Test infrastructure:**
- Vitest config already exists (`vitest.config.ts` from Phase 01).
- `npm test` runs `vitest run` which finds `tests/**/*.test.ts`.
- Tests run in `node` environment (no DOM needed).

## Acceptance Criteria

1. `/contact` displays a form with name, email, category, and message fields.
2. Client-side validation prevents submission when fields are empty or email lacks `@`. Inline errors are shown.
3. Valid submission sends POST to `/api/feedback` and shows "Thank you for your feedback." on success.
4. `/api/feedback` returns 200 `{ ok: true }` for valid submissions and inserts into D1 `feedback` table.
5. `/api/feedback` returns 400 with descriptive error for each missing/invalid field (name, email, message, category).
6. `/api/feedback` returns 413 `{ error: "Request body too large." }` for payloads exceeding 64 KB.
7. `/api/feedback` returns 500 or 503 with user-facing error when D1 is unavailable.
8. All form inputs have minimum 16px font size (no iOS zoom on focus).
9. `npm test` passes — all 8 test cases pass.
10. Test file exports a pure `validateFeedback()` function; tests import and call it directly.

## Implementation Hints

- Create `src/components/FeedbackForm.astro` for the form. Include an inline `<script>` for client-side validation and fetch — no framework, no external JS.
- Create `src/lib/feedback.ts` for the validation logic exported as a pure function. Both the API endpoint and tests import from this module.
- Form inputs: use `text-base` class for 16px font size. Add `focus:ring-2 focus:ring-accent-green focus:border-accent-green focus:outline-none` for focus styling.
- Body size check: read `request.headers.get('content-length')` first. If missing, read the body into a buffer and check `body.byteLength`. Or use a `TransformStream` approach for streaming size limits.
- D1 insertion: wrap in try/catch. Distinguish between "D1 error" (500) and "D1 timeout/unavailable" (503) if detectable — otherwise default to 500.
- The validation function should return a discriminated union: `{ valid: true, data: FeedbackInput } | { valid: false, status: number, error: string }`.
- Tests use Vitest's `describe`/`it`/`expect` API. No mocking library needed — the validation function is pure.
- The FeedbackForm component should clear the form and show the success message by manipulating the DOM (vanilla JS, no framework).
- Do not implement CAPTCHA, rate limiting, or authentication. The 64 KB limit is the only anti-abuse measure.
