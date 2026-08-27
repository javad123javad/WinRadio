import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    target: process.env.TAURI_PLATFORM === 'windows' ? 'chrome97' : 'safari13',
    minify: !process.env.TAURI_DEBUG && 'esbuild',
    sourcemap: !!process.env.TAURI_DEBUG,
  },
  server: {
    port: 1420,
    strictPort: true,
  },
})