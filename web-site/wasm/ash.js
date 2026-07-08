import * as bg from './ash_wasm_bg.js';

async function init() {
  const response = await fetch(new URL('ash_wasm_bg.wasm', import.meta.url));
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, { './ash_wasm_bg.js': bg });
  bg.__wbg_set_wasm(instance.exports);
  instance.exports.__wbindgen_start();
}

const { parse, run, repl_init, repl_eval, register_agent_callback, set_output_callback } = bg;

export { init as default, parse, run, repl_init, repl_eval, register_agent_callback, set_output_callback };
