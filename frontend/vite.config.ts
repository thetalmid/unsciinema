import { defineConfig } from 'vite';
import solid from '@solidjs/vite-plugin';

// Static client app. Deploys at `/` by default; for a nested base run:
//   npx vite build --base /nested/path/
// All asset/worker/WASM URLs are relative (import.meta.url / BASE_URL), so no
// absolute paths or localhost calls appear anywhere in the build.
export default defineConfig({
  base: process.env.VITE_BASE ?? '/',
  plugins: [solid()],
  publicDir: '../public',
  build: {
    target: 'esnext',
    assetsInlineLimit: 0,
    outDir: 'dist',
    emptyOutDir: true,
    // Keep the generated WASM glue as an ES module; the worker imports it too.
    modulePreload: { polyfill: false },
  },
  worker: {
    format: 'es',
  },
  server: {
    host: true,
    port: 5173,
    allowedHosts: [".e2b.app"],
  },
  preview: {
    host: true,
    port: 4173,
    // Explicit host allow for the Arena preview proxy (mission §11: allow the
    // known host, do not disable host checks wholesale). Localhost and
    // 127.0.0.1 remain allowed by Vite itself.
    allowedHosts: [".e2b.app"],
  },
});
