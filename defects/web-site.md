# web site defects:

## doc link getting error
Error: Transform failed with 1 error:
/opt/apps/agents/ash/web-site/src/pages/docs/index.astro:42:2: ERROR: Unexpected "do"
12:00:05 [ERROR] Unexpected "do"
  Stack trace:
    at /opt/apps/agents/ash/web-site/src/pages/docs/index.astro:42:2
    [...] See full stack trace in the browser, or rerun with --verbose.
12:00:12 [ERROR] Unexpected "do"


## blog is not working
Failed to load posts: Error: D1_ERROR: no such table: posts: SQLITE_ERROR
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
    at async eval (/opt/apps/agents/ash/web-site/src/pages/blog/index.astro:26:19)

- I think we need a local db for developing phase

## planground is broken:
astro seems does not like embbeded ash code

Unexpected "do"
playground/index.astro:34:2
Open in editor
---
import BaseLayout from '../../layouts/BaseLayout.astro';
---
ion:relative;background:transparent;color:transparent;caret-color:#e6edf3;z-index:1;width:100%;height:100%;min-height:288px;border:none;outline:none;resize:none;padding:0;font-family:inherit;font-size:inherit;line-height:inherit;overflow:auto">#!js-echo:0.1.0

print "=== Ash Playground ==="
^
/opt/apps/agents/ash/web-site/src/pages/playground/index.astro:34:2: ERROR: Unexpected "do"
    at failureErrorWithLog (/opt/apps/agents/ash/web-site/node_modules/vite/node_modules/esbuild/lib/main.js:1467:15)
    at /opt/apps/agents/ash/web-site/node_modules/vite/node_modules/esbuild/lib/main.js:736:50
    at responseCallbacks.<computed> (/opt/apps/agents/ash/web-site/node_modules/vite/node_modules/esbuild/lib/main.js:603:9)
    at handleIncomingPacket (/opt/apps/agents/ash/web-site/node_modules/vite/node_modules/esbuild/lib/main.js:658:12)
    at Socket.readFromStdout (/opt/apps/agents/ash/web-site/node_modules/vite/node_modules/esbuild/lib/main.js:581:7)
    at Socket.emit (node:events:509:28)
    at Socket.emit (node:domain:489:12)
    at addChunk (node:internal/streams/readable:563:12)
    at readableAddChunkPushByteMode (node:internal/streams/readable:514:3)
    at Readable.push (node:internal/streams/readable:394:5)


## contact is not working
FailedToLoadModuleSSR
Could not import file.
Could not import ../../../layouts/BaseLayout.astro.
See Docs Reference
This is often caused by a typo in the import path. Please make sure the file exists.    