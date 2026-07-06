const https = require('https');
const fs = require('fs');
const path = require('path');
const os = require('os');

const PLATFORM_MAP = {
  'linux-x64': 'ash-linux-x64',
  'darwin-x64': 'ash-darwin-x64',
  'darwin-arm64': 'ash-darwin-arm64',
  'win32-x64': 'ash-win-x64.exe',
};

const platform = `${os.platform()}-${os.arch()}`;
const remoteName = PLATFORM_MAP[platform];

if (!remoteName) {
  console.warn(`[@ash-lang/cli] Unsupported platform: ${platform}. Skipping binary download.`);
  process.exit(0);
}

const localName = process.platform === 'win32' ? 'ash-win-x64.exe' : 'ash';
const dest = path.join(__dirname, localName);
const url = `https://github.com/kenny1125nz/ash-lang/releases/latest/download/${remoteName}`;

console.log(`[@ash-lang/cli] Downloading ${remoteName} from GitHub Releases...`);

function download(targetUrl, followCount) {
  followCount = followCount || 0;
  if (followCount > 5) {
    console.warn('[@ash-lang/cli] Too many redirects. Skipping binary download.');
    console.warn('[@ash-lang/cli] Download the binary manually from https://github.com/kenny1125nz/ash-lang/releases/latest');
    process.exit(0);
  }

  https.get(targetUrl, (res) => {
    if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
      const redirect = res.headers.location;
      const isAbsolute = /^https?:\/\//i.test(redirect);
      download(isAbsolute ? redirect : new URL(redirect, url).href, followCount + 1);
      return;
    }

    if (res.statusCode !== 200) {
      console.warn(`[@ash-lang/cli] GitHub Releases returned HTTP ${res.statusCode}. Skipping binary download.`);
      console.warn('[@ash-lang/cli] Download the binary manually from https://github.com/kenny1125nz/ash-lang/releases/latest');
      process.exit(0);
    }

    const file = fs.createWriteStream(dest, { mode: 0o755 });
    res.pipe(file);
    file.on('finish', () => {
      file.close(() => {
        if (process.platform !== 'win32') {
          fs.chmodSync(dest, 0o755);
        }
        console.log(`[@ash-lang/cli] Binary installed: ${localName}`);
      });
    });
  }).on('error', (err) => {
    console.warn(`[@ash-lang/cli] Download failed: ${err.message}`);
    console.warn('[@ash-lang/cli] Download the binary manually from https://github.com/kenny1125nz/ash-lang/releases/latest');
    if (fs.existsSync(dest)) {
      try { fs.unlinkSync(dest); } catch (_) {}
    }
    process.exit(0);
  });
}

download(url);
