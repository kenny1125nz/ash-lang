const { execSync } = require('child_process');
const fs = require('fs');

const path = './tmp/fixed-ids.txt';
if (!fs.existsSync(path)) process.exit(0);

const ids = fs.readFileSync(path, 'utf-8').trim().split('\n').filter(Boolean);
for (const id of ids) {
  const cmd = `npx wrangler d1 execute ash-website-db --remote --command="UPDATE feedback SET status = 'fixed' WHERE id = ${id}"`;
  execSync(cmd, { cwd: __dirname + '/../../web-site', stdio: 'pipe' });
  console.log('Fixed #' + id);
}
fs.rmSync(path, { force: true });
