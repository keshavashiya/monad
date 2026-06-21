import { defineConfig } from 'vite';
import path from 'node:path';

// Vite loads this config as native ESM, where `__dirname` is not defined.
// `import.meta.dirname` is available on Node 20.11+ (which Vite 8 requires).
const here = import.meta.dirname;

export default defineConfig({
  base: './',
  build: {
    outDir: 'dist',
    assetsInlineLimit: 0,
  },
  // Handle WASM files from the firmware build
  optimizeDeps: {
    exclude: ['monad-wasm'],
  },
  server: {
    fs: {
      allow: [
        // Project root (console/)
        path.resolve(here, '.'),
        // Firmware WASM output (outside project root)
        path.resolve(here, '../firmware/pkg'),
      ],
    },
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
});
