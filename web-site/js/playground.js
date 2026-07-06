const atomicNotify = Atomics.notify || Atomics.wake;
const ASH_KEYWORDS = new Set([
  'do','with','for','in','if','else','print','exec','try','evaluate',
  'accept','partial','fail','upto','fn','return','while','wait','within',
  'compact','exit','break','continue','subagent','using','and','or','not',
]);
const ASH_BUILTINS = new Set(['len','env']);

function escapeHtml(s) {
  return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
}

function highlightAsh(code) {
  const tokens = [];
  let i = 0;
  while (i < code.length) {
    if (code[i] === '"') {
      let j = i + 1, depth = 0;
      while (j < code.length) {
        if (code[j] === '"' && depth === 0) { j++; break; }
        if (code[j] === '$' && j + 1 < code.length && code[j + 1] === '{') depth++;
        if (code[j] === '}' && depth > 0) depth--;
        j++;
      }
      tokens.push({ t: 'str', v: code.slice(i, j) });
      i = j;
    } else if (code[i] === '#') {
      if (i === 0 || code[i - 1] === '\n') {
        let j = i;
        while (j < code.length && code[j] !== '\n') j++;
        tokens.push({ t: 'shebang', v: code.slice(i, j) });
        i = j;
      } else {
        let j = i;
        while (j < code.length && code[j] !== '\n') j++;
        tokens.push({ t: 'comment', v: code.slice(i, j) });
        i = j;
      }
    } else if (/[0-9]/.test(code[i])) {
      let j = i;
      while (j < code.length && /[0-9.]/.test(code[j])) j++;
      tokens.push({ t: 'num', v: code.slice(i, j) });
      i = j;
    } else if (code[i] === '$') {
      if (i + 1 < code.length && code[i + 1] === '{') {
        let j = i + 2, depth = 1;
        while (j < code.length && depth > 0) {
          if (code[j] === '{') depth++;
          if (code[j] === '}') depth--;
          j++;
        }
        tokens.push({ t: 'var', v: code.slice(i, j) });
        i = j;
      } else {
        let j = i + 1;
        while (j < code.length && /[a-zA-Z_0-9?]/.test(code[j])) j++;
        tokens.push({ t: 'var', v: code.slice(i, j) });
        i = j;
      }
    } else if (/[a-zA-Z_]/.test(code[i])) {
      let j = i;
      while (j < code.length && /[a-zA-Z_0-9]/.test(code[j])) j++;
      const word = code.slice(i, j);
      if (ASH_KEYWORDS.has(word)) tokens.push({ t: 'kw', v: word });
      else if (ASH_BUILTINS.has(word)) tokens.push({ t: 'builtin', v: word });
      else tokens.push({ t: 'var', v: word });
      i = j;
    } else if (/[=!<>+\-*/%]/.test(code[i])) {
      let j = i + 1;
      if (j < code.length && code[j] === '=') j++;
      tokens.push({ t: 'op', v: code.slice(i, j) });
      i = j;
    } else {
      tokens.push({ t: 'plain', v: code[i] });
      i++;
    }
  }
  return tokens.map(t => {
    if (t.t === 'plain') return escapeHtml(t.v);
    const cls = t.t === 'shebang' ? 'comment' : t.t === 'kw' ? 'keyword' : t.t === 'str' ? 'string' : t.t === 'num' ? 'number' : t.t === 'op' ? 'operator' : t.t === 'builtin' ? 'builtin' : 'variable';
    return '<span class="token-' + cls + '">' + escapeHtml(t.v) + '</span>';
  }).join('');
}

let evalWorker = null;
let stateView = null;
let resView = null;
let resData = null;
let workerReady = false;
let useBlocking = false;
let pendingAgentQueue = [];
let receivedChunks = false;

const statusEl = document.getElementById('status');
const outputEl = document.getElementById('output');
const editorEl = document.getElementById('editor');
const wasmStatusEl = document.getElementById('wasm-status');

function echoAgent(prompt) {
  return prompt;
}

async function processAgentQueue(queue) {
  if (!queue || queue.length === 0) return;
  statusEl.textContent = 'processing agent queue...';
  for (let i = 0; i < queue.length; i++) {
    const { prompt } = queue[i];
    const result = echoAgent(prompt);
    outputEl.textContent += result + '\n';
  }
  statusEl.textContent = 'done';
}

function setupWorkerHandlers() {
  evalWorker.addEventListener('message', async (e) => {
    const msg = e.data;

    if (msg.type === 'worker_ready') {
      useBlocking = msg.blocking;
      wasmStatusEl.textContent = useBlocking ? 'WASM loaded (blocking)' : 'WASM loaded (non-blocking)';
      wasmStatusEl.style.color = 'var(--accent2)';
      if (useBlocking && msg.stateBuf) {
        stateView = new Int32Array(msg.stateBuf);
        resView = new Uint8Array(msg.resBuf);
        resData = new DataView(msg.resBuf);
      }
      workerReady = true;
      return;
    }

    if (msg.type === 'chunk') {
      outputEl.textContent += msg.text;
      receivedChunks = true;
      return;
    }

    if (msg.type === 'output_done') {
      if (!receivedChunks) {
        outputEl.textContent = msg.full || '(no output)';
      }
      statusEl.textContent = 'done';
      if (msg.agentQueue && msg.agentQueue.length > 0) {
        pendingAgentQueue = msg.agentQueue;
        outputEl.textContent += '\n\n' + pendingAgentQueue.length + ' agent call(s) queued (non-blocking mode).';
      }
      if (pendingAgentQueue.length > 0) {
        await processAgentQueue(pendingAgentQueue);
        pendingAgentQueue = [];
      }
      receivedChunks = false;
      return;
    }

    if (msg.type === 'agent' && useBlocking) {
      const encoder = new TextEncoder();
      const resultStr = echoAgent(msg.prompt);
      const encoded = encoder.encode(resultStr);
      if (encoded.length + 4 <= resView.length) {
        resData.setInt32(0, encoded.length, true);
        resView.set(encoded, 4);
      } else {
        const fallback = encoder.encode('[Result too large]');
        resData.setInt32(0, fallback.length, true);
        resView.set(fallback, 4);
      }
      Atomics.store(stateView, 0, 2);
      atomicNotify(stateView, 0, 1);
    }
  });
}

async function initWasm() {
  try {
    evalWorker = new Worker('../wasm/eval-worker.js', { type: 'module' });
    setupWorkerHandlers();
    wasmStatusEl.textContent = 'loading WASM...';
    wasmStatusEl.style.color = 'var(--accent3)';
  } catch (e) {
    wasmStatusEl.textContent = 'WASM unavailable';
    wasmStatusEl.style.color = 'var(--accent4)';
  }
}

window.runScript = async function () {
  const code = editorEl.value.trim();
  if (!code) { showToast('Enter a script first', true); return; }
  if (!workerReady) { showToast('WASM is still loading...', true); return; }

  outputEl.textContent = '';
  statusEl.textContent = 'running...';
  pendingAgentQueue = [];
  receivedChunks = false;
  evalWorker.postMessage({ type: 'run', script: code });
};

window.clearEditor = function () {
  editorEl.value = '';
  syncEditor();
};

window.clearOutput = function () {
  outputEl.textContent = 'Click "Run" to execute the script.';
  statusEl.textContent = 'ready';
};

function syncEditor() {
  const highlight = document.getElementById('editor-highlight');
  if (highlight) {
    highlight.innerHTML = highlightAsh(editorEl.value) + '\n';
    highlight.scrollTop = editorEl.scrollTop;
    highlight.scrollLeft = editorEl.scrollLeft;
  }
}

window.loadExample = function () {
  editorEl.value = `#!js-echo:0.1.0

# Ash language demo — variables, loops, conditionals, agent calls
print "=== Ash Playground ==="
print ""

TASKS = ["Check the login page", "Test the search feature", "Review the footer links"]
print "Running \${len(TASKS)} automated checks..."
print ""

for TASK in TASKS {
\tdo "Navigate to the site and \${TASK}"
\tprint "Task: \${TASK}"
\tprint "Result: \${stdout}"
\tprint ""
}

print "--- Summary ---"
print "Completed \${len(TASKS)} checks"
print "All done"`;
  syncEditor();
  clearOutput();
};

function setupEditor() {
  if (!editorEl || !document.getElementById('editor-highlight')) return;
  editorEl.addEventListener('input', syncEditor);
  editorEl.addEventListener('scroll', syncEditor);
  syncEditor();
}

function highlightExamples() {
  document.querySelectorAll('.example-code').forEach(el => {
    const code = el.textContent || '';
    el.innerHTML = highlightAsh(code);
  });
}

function showToast(msg, isError) {
  const toast = document.getElementById('toast');
  toast.textContent = msg;
  toast.className = 'toast' + (isError ? ' error' : '');
  toast.classList.add('show');
  setTimeout(() => toast.classList.remove('show'), 3000);
}

setupEditor();
highlightExamples();
initWasm();
