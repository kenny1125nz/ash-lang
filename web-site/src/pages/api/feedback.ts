import { validateFeedback } from '../../lib/feedback';

export const prerender = false;

export async function POST({ request, locals }: { request: Request; locals: { runtime: { env: { DB: D1Database } } } }) {
  const contentLength = request.headers.get('content-length');
  if (contentLength && parseInt(contentLength) > 65536) {
    return new Response(JSON.stringify({ error: 'Request body too large.' }), {
      status: 413,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  let body: unknown;
  try {
    const text = await request.text();
    if (new TextEncoder().encode(text).length > 65536) {
      return new Response(JSON.stringify({ error: 'Request body too large.' }), {
        status: 413,
        headers: { 'Content-Type': 'application/json' },
      });
    }
    body = JSON.parse(text);
  } catch {
    return new Response(JSON.stringify({ error: 'Name is required.' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const result = validateFeedback(body);
  if (!result.valid) {
    return new Response(JSON.stringify({ error: result.error }), {
      status: result.status,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  try {
    const db = locals.runtime.env.DB;
    await db.prepare(
      'INSERT INTO feedback (name, email, message, category) VALUES (?, ?, ?, ?)'
    ).bind(result.data.name, result.data.email, result.data.message, result.data.category).run();
  } catch (e) {
    console.error('D1 write failed:', e);
    return new Response(JSON.stringify({ error: 'Unable to save feedback. Please try again later.' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  return new Response(JSON.stringify({ ok: true }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  });
}
