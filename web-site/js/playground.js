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

// --- Shared worker state ---
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

// --- REPL state ---
let replReady = false;
let replBuffer = [];
let replHistory = [];
const replPromptEl = document.getElementById('repl-prompt');
const replInputEl = document.getElementById('repl-input');
const replOutputEl = document.getElementById('repl-output');
const scriptTabEl = document.getElementById('script-tab');
const replTabEl = document.getElementById('repl-tab');
const scriptPanelEl = document.getElementById('script-panel');
const replPanelEl = document.getElementById('repl-panel');

function braceDepth(line) {
  let open = 0;
  for (const ch of line) {
    if (ch === '{') open++;
    if (ch === '}') open--;
  }
  return open;
}

function switchTab(mode) {
  if (mode === 'repl') {
    scriptTabEl.classList.remove('active');
    replTabEl.classList.add('active');
    scriptPanelEl.style.display = 'none';
    replPanelEl.style.display = 'flex';
    replInputEl.focus();
    if (workerReady && !replReady) {
      replInit();
    }
  } else {
    replTabEl.classList.remove('active');
    scriptTabEl.classList.add('active');
    replPanelEl.style.display = 'none';
    scriptPanelEl.style.display = '';
    editorEl.focus();
  }
}

function replInit() {
  evalWorker.postMessage({ type: 'repl_init' });
}

function replEvalLine(line) {
  replOutputEl.innerHTML += '<span class="repl-prompt-line">&gt; ' + escapeHtml(line) + '</span><br>';
  replHistory.push(line);
  evalWorker.postMessage({ type: 'repl_eval', line });
}

function replSubmit() {
  if (!workerReady) { showToast('WASM is still loading...', true); return; }

  const line = replInputEl.value;
  replInputEl.value = '';

  if (replBuffer.length === 0 && line.trim().startsWith('.') && line.trim().split(' ')[0] === '.help') {
    replOutputEl.innerHTML += '<span class="repl-prompt-line">&gt; .help</span><br>';
    replOutputEl.innerHTML += '.help &mdash; Show this help<br>';
    replOutputEl.innerHTML += '.clear &mdash; Clear all variables<br>';
    replOutputEl.innerHTML += '.vars &mdash; List all variables<br>';
    replOutputEl.innerHTML += '.exit &mdash; Exit REPL<br>';
    replOutputEl.scrollTop = replOutputEl.scrollHeight;
    return;
  }

  replBuffer.push(line);
  const depth = replBuffer.reduce((sum, l) => sum + braceDepth(l), 0);

  if (depth > 0) {
    replPromptEl.textContent = '... ';
    return;
  }

  const full = replBuffer.join('\n');
  replBuffer = [];
  replPromptEl.textContent = '> ';
  replEvalLine(full);
}

function replScrollBottom() {
  replOutputEl.scrollTop = replOutputEl.scrollHeight;
}

// --- Worker message handler ---
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
      if (replPanelEl.style.display === 'flex' && !replReady) {
        replInit();
      }
      return;
    }

    if (msg.type === 'worker_error') {
      wasmStatusEl.textContent = 'WASM error: ' + msg.msg;
      wasmStatusEl.style.color = 'var(--accent4)';
      return;
    }

    if (msg.type === 'repl_ready') {
      replReady = true;
      return;
    }

    if (msg.type === 'repl_result') {
      if (msg.output) {
        replOutputEl.innerHTML += escapeHtml(msg.output).replace(/\n/g, '<br>') + '<br>';
        replScrollBottom();
      }
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

// --- WASM init ---
async function initWasm() {
  try {
    evalWorker = new Worker('js/eval-worker.js', { type: 'module' });
    setupWorkerHandlers();
    wasmStatusEl.textContent = 'loading WASM...';
    wasmStatusEl.style.color = 'var(--accent3)';
  } catch (e) {
    wasmStatusEl.textContent = 'WASM unavailable';
    wasmStatusEl.style.color = 'var(--accent4)';
  }
}

// --- Script runner ---
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

// --- Editor sync ---
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

// --- REPL input key binding ---
function setupReplInput() {
  if (!replInputEl) return;
  replInputEl.addEventListener('keydown', function (e) {
    if (e.key === 'Enter') {
      e.preventDefault();
      replSubmit();
    }
  });
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
setupReplInput();
highlightExamples();
initWasm();
