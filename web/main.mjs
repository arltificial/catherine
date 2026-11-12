// Purpose: the browser glue (openspec change `add-wasm-web-view`) — the
// one module that instantiates the wasm boundary and installs it as
// `window.catherineWasm` before the app shell loads.
//
// Responsibilities: run the wasm-bindgen `init()` (fetching the .wasm
// relative to this module), adapt the generated exports to the
// [`window.catherineWasm`] surface `src/wasm.ts` expects, then import the
// app. No framework, no bundler-time coupling beyond this file: the app
// never imports `pkg/` directly.
//
// Build: `just web-build` bundles this file plus `src/app.ts` into
// `app.js`, which `index.html` loads as a module.

import init, { check, analyze, max_document_bytes } from "./pkg/catherine_wasm.js";

await init("./pkg/catherine_wasm_bg.wasm");

window.catherineWasm = {
  check,
  analyze,
  max_document_bytes: () => max_document_bytes(),
};

await import("./src/app.ts");
