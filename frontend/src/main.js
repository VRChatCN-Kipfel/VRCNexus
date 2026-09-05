// VRCNexus 入口：创建 Vue 应用 + Tauri 环境校验
// 非 Tauri 环境(浏览器直开)显示阻断页——本项目是桌面应用，不提供网页版

import { createApp } from 'vue'
import './style.css'
import App from './App.vue'

// ===== 代码级校验：VRCNexus 是 Tauri 桌面应用，禁止作为普通网页直接打开 =====
// 检测是否运行在 Tauri WebView（有 @tauri-apps/api 的 window.__TAURI_INTERNALS__）。
// 浏览器里直接 vite dev / 静态打开时不存在该标记 → 阻断渲染并提示。
const isTauri = typeof window !== 'undefined' && !!window.__TAURI_INTERNALS__

if (!isTauri) {
  document.documentElement.innerHTML = ''
  document.body.style.cssText = 'margin:0;display:flex;align-items:center;justify-content:center;height:100vh;background:#070a14;color:#eef1f8;font-family:system-ui,sans-serif;'
  const box = document.createElement('div')
  box.style.cssText = 'text-align:center;max-width:520px;padding:32px;border:1px solid rgba(255,255,255,.15);border-radius:18px;background:rgba(255,255,255,.05);'
  box.innerHTML =
    '<div style="font-size:34px;margin-bottom:12px">🖥️</div>' +
    '<h2 style="margin:0 0 10px;font-size:20px">VRCNexus 是桌面应用</h2>' +
    '<p style="color:#9aa4bc;font-size:14px;line-height:1.7;margin:0">' +
    '不能直接在浏览器里打开。请通过 Tauri 运行：<br>' +
    '<code style="background:rgba(255,255,255,.1);padding:2px 8px;border-radius:6px;font-size:13px">bun run tauri dev</code>' +
    '（或已打包的 VRCNexus.exe）</p>'
  document.body.appendChild(box)
} else {
  createApp(App).mount('#app')
}
