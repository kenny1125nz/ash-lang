export const prerender = false;

export async function GET({ locals }: { locals: { runtime: { env: { DB: D1Database } } } }) {
  try {
    const db = locals.runtime.env.DB;
    const { results } = await db.prepare('SELECT title, slug, published_at FROM posts ORDER BY published_at DESC LIMIT 3').all();
    return new Response(JSON.stringify(results ?? []), {
      headers: { 'Content-Type': 'application/json' },
    });
  } catch (e) {
    console.error('Failed to load recent posts:', e);
    return new Response(JSON.stringify({ error: 'Unable to load posts.' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' },
    });
  }
}
