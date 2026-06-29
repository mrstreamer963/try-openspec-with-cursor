import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import { resolve } from 'path';

export default defineConfig({
  plugins: [vue(), wasm(), topLevelAwait()],
  publicDir: resolve(__dirname, '../../content'),
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '@content': resolve(__dirname, '../../content'),
    },
  },
  worker: {
    format: 'es',
    plugins: () => [wasm()],
  },
  server: {
    port: 5173,
  },
});
