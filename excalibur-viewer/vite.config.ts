import { defineConfig } from 'vite'
import { resolve } from 'path'

export default defineConfig({
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '@/engine': resolve(__dirname, 'src/engine'),
      '@/data': resolve(__dirname, 'src/data'),
      '@/terrain': resolve(__dirname, 'src/terrain'),
      '@/entities': resolve(__dirname, 'src/entities'),
      '@/overlays': resolve(__dirname, 'src/overlays'),
      '@/camera': resolve(__dirname, 'src/camera'),
      '@/utils': resolve(__dirname, 'src/utils'),
    },
  },
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:54321',
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'public/index.html'),
      },
    },
  },
})