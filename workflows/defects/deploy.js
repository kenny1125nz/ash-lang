const { execSync } = require('child_process');

const siteDir = __dirname + '/../../web-site';

console.log('Building...');
execSync('npm run build', { cwd: siteDir, stdio: 'inherit' });

console.log('Deploying...');
execSync('npx wrangler pages deploy dist --branch=main --commit-dirty=true', { cwd: siteDir, stdio: 'inherit' });

console.log('Deployed.');
