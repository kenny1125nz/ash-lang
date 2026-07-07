let ashWasm = null;
let useBlocking = false;
let agentQueue = [];
const decoder = new TextDecoder();

try {
  new SharedArrayBuffer(4);
  useBlocking = true;
} catch (_) {}

const REQ_SIZE = 65536;
const RES_SIZE = 524288;

const stateBuf = useBlocking ? new SharedArrayBuffer(4) : null;
const reqBuf = useBlocking ? new SharedArrayBuffer(REQ_SIZE) : null;
const resBuf = useBlocking ? new SharedArrayBuffer(RES_SIZE) : null;
const state = useBlocking ? new Int32Array(stateBuf) : null;
const reqView = useBlocking ? new Uint8Array(reqBuf) : null;
const resView = useBlocking ? new Uint8Array(resBuf) : null;
const reqData = useBlocking ? new DataView(reqBuf) : null;
const resData = useBlocking ? new DataView(resBuf) : null;

const encoder = new TextEncoder();

function agentCallback(prompt, model) {
  if (!useBlocking) {
    agentQueue.push({ prompt, model });
    return '[queued] page-agent call queued (non-blocking mode)';
  }

  try {
    const encoded = encoder.encode(prompt);
    if (encoded.length + 4 > REQ_SIZE) {
      return 'Error: prompt too large for shared buffer';
    }
    reqData.setInt32(0, encoded.length, true);
    reqView.set(encoded, 4);

    state[0] = 1;
    self.postMessage({ type: 'agent', prompt, model });

    while (state[0] === 1) {
      Atomics.wait(state, 0, 1, 5000);
    }

    if (state[0] === 2) {
      const len = resData.getInt32(0, true);
      const sharedSlice = new Uint8Array(resBuf, 4, len);
      const result = decoder.decode(new Uint8Array(sharedSlice));
      state[0] = 0;
      self.postMessage({ type: 'chunk', text: result + '\n' });
      return result;
    }

    state[0] = 0;
    return '';
  } catch (e) {
    console.warn('[agentCallback]', e.message);
    state[0] = 0;
    return '[agentCallback error] ' + String(e);
  }
}

function outputCallback(chunk) {
  self.postMessage({ type: 'chunk', text: chunk });
}

self.onmessage = function (e) {
  if (e.data.type === 'run' && ashWasm) {
    agentQueue = [];
    const output = ashWasm.run(e.data.script);
    self.postMessage({
      type: 'output_done',
      full: output,
      agentQueue: useBlocking ? null : agentQueue,
    });
  }
  if (e.data.type === 'repl_init' && ashWasm) {
    try {
      ashWasm.repl_init();
      self.postMessage({ type: 'repl_ready' });
    } catch (e) {
      self.postMessage({ type: 'repl_result', output: 'Error in repl_init: ' + e.message });
    }
  }
  if (e.data.type === 'repl_eval' && ashWasm) {
    try {
      const result = ashWasm.repl_eval(e.data.line);
      self.postMessage({ type: 'repl_result', output: result || '' });
    } catch (e) {
      self.postMessage({ type: 'repl_result', output: 'Error: ' + e.message });
    }
  }
};

async function init() {
  try {
    const mod = await import('../wasm/ash.js');
    await mod.default();
    ashWasm = { run: mod.run, parse: mod.parse, repl_init: mod.repl_init, repl_eval: mod.repl_eval };
    mod.register_agent_callback(agentCallback);
    if (mod.set_output_callback) {
      mod.set_output_callback(outputCallback);
    }
    self.postMessage({ type: 'worker_ready', blocking: useBlocking, stateBuf, reqBuf, resBuf });
  } catch (e) {
    self.postMessage({ type: 'output_done', full: 'Worker init error: ' + e.message });
  }
}

init();
