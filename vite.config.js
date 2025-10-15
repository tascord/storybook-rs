import { defineConfig } from 'vite';

export default defineConfig({
  optimizeDeps: {
    exclude: ['example/pkg'],
  },
  server: {
    fs: {
      // Allow serving files from the example/pkg directory
      allow: ['..'],
    },
  },
});
