import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path' // <--- 【新增】导入 Node.js 的 path 模块

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],

  // 仅保留项目内部 alias；front-cpu 走 node_modules（npm 包）
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },

  // 【新增】Tauri 推荐的服务器配置
  // prevent vite from obscuring rust errors
  clearScreen: false,
  // tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1421, // 这是一个 Tauri 常用的默认端口，你可以换成别的
    strictPort: true,
  },
})
