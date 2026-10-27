import { defineConfig } from 'vitest/config';
import solidPlugin from 'vite-plugin-solid';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
  plugins: [
    wasm(),
    solidPlugin(),
  ],
  test: {
    environment: 'jsdom',
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    setupFiles: ['src/test/setup.ts'],
    globals: true,
  },
  resolve: {
    alias: {
      '@': '/src',
    },
  },
});