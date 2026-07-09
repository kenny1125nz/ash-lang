#!/usr/bin/env node
import { readFileSync, writeFileSync, unlinkSync, existsSync } from 'node:fs';
import { execSync } from 'node:child_process';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectDir = join(__dirname, '..');

const args = process.argv.slice(2);
if (args.length < 1 || args.includes('--help') || args.includes('-h')) {
  console.log(`Usage: node scripts/add-post.mjs <file.md> [options]

Options:
  --title <title>      Post title (default: first # heading in file)
  --slug <slug>        URL slug (default: derived from title)
  --author <name>      Author name (default: "Ash Team")
  --date <YYYY-MM-DD>  Publish date (default: today)
  --remote             Insert into remote D1 (default: local)
  --dry-run            Print SQL without executing

Example:
  node scripts/add-post.mjs posts/my-article.md
  node scripts/add-post.mjs posts/my-article.md --title "Hello World" --author "Jane"
  node scripts/add-post.mjs posts/my-article.md --remote`);
  process.exit(0);
}

const filePath = args[0];
if (!existsSync(filePath)) {
  console.error(`File not found: ${filePath}`);
  process.exit(1);
}

let content = readFileSync(filePath, 'utf-8').trim();
if (!content) {
  console.error('File is empty');
  process.exit(1);
}

const titleIdx = args.indexOf('--title');
let title = titleIdx >= 0 ? args[titleIdx + 1] : null;
const slugIdx = args.indexOf('--slug');
let slug = slugIdx >= 0 ? args[slugIdx + 1] : null;
const authorIdx = args.indexOf('--author');
const author = authorIdx >= 0 ? args[authorIdx + 1] : 'Ash Team';
const dateIdx = args.indexOf('--date');
const date = dateIdx >= 0 ? args[dateIdx + 1] : new Date().toISOString().slice(0, 10);
const remote = args.includes('--remote');
const dryRun = args.includes('--dry-run');

if (!title) {
  const match = content.match(/^#\s+(.+)$/m);
  title = match ? match[1].trim() : null;
}
if (!title) {
  console.error('No title found. Provide one with --title or add a "# Title" heading to the file.');
  process.exit(1);
}

if (!slug) {
  slug = title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 80);
}
if (!slug) {
  console.error('Could not derive a slug from the title. Provide one with --slug.');
  process.exit(1);
}

const escaped = content.replace(/'/g, "''");

const sql = `INSERT INTO posts (title, slug, content, author, published_at) VALUES ('${title.replace(/'/g, "''")}', '${slug.replace(/'/g, "''")}', '${escaped}', '${author.replace(/'/g, "''")}', '${date}');`;

if (dryRun) {
  console.log(`--dry-run — would insert with:\n  title:  ${title}\n  slug:   ${slug}\n  author: ${author}\n  date:   ${date}\n  size:   ${content.length} chars\n`);
  console.log('SQL:\n' + sql.slice(0, 200) + '...');
  process.exit(0);
}

const sqlFile = join(projectDir, '.tmp-add-post.sql');
writeFileSync(sqlFile, sql, 'utf-8');

try {
  const flag = remote ? '--remote' : '--local';
  const result = execSync(
    `npx wrangler d1 execute ash-website-db ${flag} --file="${sqlFile}"`,
    { cwd: projectDir, encoding: 'utf-8', stdio: 'pipe' },
  );
  console.log(`✓ Published: "${title}" (/${slug})`);
  if (result.includes('success')) {
    console.log('  Post inserted successfully.');
  }
} catch (err) {
  console.error('Failed to insert post:');
  console.error(err.stderr || err.message);
  process.exit(1);
} finally {
  if (existsSync(sqlFile)) unlinkSync(sqlFile);
}
