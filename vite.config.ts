import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

export default defineConfig({
  plugins: [vue()],
  assetsInclude: ['**/*.mp3'],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://192.168.0.189:8080',
        changeOrigin: true,
        secure: false,
        rewrite: (path) => path.replace(/^\/api/, '/api')
      }
    }
  },
  optimizeDeps: {
    exclude: [
      'opus-media-recorder',
      '@web-media/opus-decoder'
    ]
  },
  build: {
    // target: 'esnext',
    assetsInlineLimit: 0 // 禁用资源内联
  }
}) 