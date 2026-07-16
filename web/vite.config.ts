import { defineConfig } from 'vite';

export default defineConfig({
  root: '.',
  publicDir: false,
  
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    target: 'esnext',
  },

  server: {
    port: 3003,
    cors: false,
    host: true,
  },

  optimizeDeps: {
    exclude: ['./pkg/ascii_canvas.js'],
  },

  // Ensure WASM files are served correctly
  assetsInclude: ['**/*.wasm'],
  
  // Worker configuration for potential future use
  worker: {
    format: 'es',
  },

  test: {
    environment: 'happy-dom',
    include: ['**/*.test.ts'],
  },
});
