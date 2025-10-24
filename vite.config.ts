import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path' // <--- 【新增】导入 Node.js 的 path 模块

// https://vite.dev/config/
export default defineConfig({
  // 【保留】原有的 plugins 配置
  plugins: [vue()],

  // 【新增】resolve.alias 配置，用于设置路径别名
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@cutie/cpu-pipeline': path.resolve(__dirname, './packages/cpu-pipeline/src/index.ts'),
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
