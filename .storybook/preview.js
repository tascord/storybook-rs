// Preview configuration for Rust WASM components
// This file initializes the WASM module and sets up the rendering environment

import init, { register_all_stories } from '../example/pkg/example.js';

// Initialize WASM module before stories load
let wasmInitialized = false;

export async function loadGlobalSetup() {
  if (!wasmInitialized) {
    await init();
    register_all_stories();
    wasmInitialized = true;
  }
}

// Call setup before rendering
loadGlobalSetup();

export const parameters = {
  actions: { argTypesRegex: '^on[A-Z].*' },
  controls: {
    matchers: {
      color: /(background|color)$/i,
      date: /Date$/,
    },
  },
};
