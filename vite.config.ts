import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  root: 'web',
  publicDir: false,
  
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    target: 'esnext',
    minify: 'terser',
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'web/index.html'),
      },
    },
  },

  server: {
    port: 3000,
    open: true,
    cors: true,
  },

  optimizeDeps: {
    exclude: ['../pkg/ascii_canvas.js'],
  },

  resolve: {
    alias: {
      '@': resolve(__dirname, 'web'),
    },
  },

  // Ensure WASM files are served correctly
  assetsInclude: ['**/*.wasm'],
  
  // Worker configuration for potential future use
  worker: {
    format: 'es',
  },
});
