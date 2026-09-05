import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

// VRCNexus 是 Tauri 桌面应用：前端 dev server 仅作为 Tauri 窗口的内嵌页面源，
// 禁止把 5173 当普通网页直接打开使用（应用一律通过 bun run tauri dev / build 运行）。
// 端口固定 4040 + strictPort：被占用时直接报错，避免静默换端口导致 Tauri 连不上。
export default defineConfig({
  plugins: [vue()],
  server: {
    port: 4040,
    strictPort: true,
    host: '127.0.0.1', // 只绑本机，杜绝局域网被当网页访问
    watch: {
      ignored: ['**/src-tauri/target/**', '**/src-tauri/**'],
    },
  },
})
