import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import { resolve } from 'path';

export default defineConfig({
  base: './',
  plugins: [vue(), wasm(), topLevelAwait()],
  publicDir: resolve(__dirname, '../../content'),
  resolve: {
    alias: {
      '@idle-colony/client': resolve(__dirname, '../../packages/client/src'),
    },
  },
  worker: {
    format: 'es',
    plugins: () => [wasm()],
  },
  server: {
    port: 5173,
  },
  optimizeDeps: {
    include: ['pixi.js', 'parse-svg-path'],
  },
});
