const { execSync } = require('child_process');
const fs = require('fs');

fs.rmSync('./tmp/defects.json', { force: true });

const stdout = execSync(
  'npx wrangler d1 execute ash-website-db --remote --json --command="SELECT id, name, email, category, message, created_at FROM feedback WHERE status = \'open\' ORDER BY created_at DESC"',
  { cwd: __dirname + '/../../web-site', encoding: 'utf-8', timeout: 15000 }
);

const data = JSON.parse(stdout);
const results = data[0]?.results || [];

fs.writeFileSync('./tmp/defects.json', JSON.stringify(results, null, 2));
console.log(results.length > 0 ? results.length + ' defects found' : 'NO_DEFECTS');
