#!/usr/bin/env node
const { spawn } = require('child_process');
const path = require('path');
const bin = path.join(__dirname, process.platform === 'win32' ? 'ash-win-x64.exe' : 'ash');
spawn(bin, process.argv.slice(2), { stdio: 'inherit' }).on('exit', process.exit);
