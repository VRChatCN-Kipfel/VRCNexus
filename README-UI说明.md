# VRCNexus 前端 UI 代码包（含全部文字/文案）

Tauri 桌面应用（Vue 3 + Vite）的前端 UI 源码。**UI 结构与界面文字（中文）全部内嵌在 .vue / .js 文件里**（无独立 i18n 文件，改文字直接改这些文件）。

## 与真实项目的对应（改完覆盖回去）

- 真实工程：`D:\VRCNexusTauri\`（git 仓库，frontend/ + src-tauri/）
- 本包 = `D:\VRCNexusTauri\frontend\src\` 的**同路径拷贝** + `public/bg.jpg`
- 改完把文件按同路径覆盖回 `D:\VRCNexusTauri\frontend\src\` 即可；dev 窗口热更新即时生效
- 运行：根目录 `bun run tauri dev`（Tauri 桌面应用，不能纯浏览器开；main.js 有环境校验）

## 文件说明

- `main.js` — 入口 + Tauri 环境校验（非 Tauri 显示阻断页，勿删）
- `App.vue` — 应用壳：左侧药丸导航 + 主视图切换（当前默认视图临时指向 layout 预览）
- `style.css` — 玻璃拟态设计系统 v2（CSS 变量在 :root：深紫灰玻璃 --glass、亮紫 --accent #8b5cf6、白字 --ink；.glass/.g-btn/.g-input/.g-pill/.spot/.bg-scene 等）
- `lib/api.js` — 前端 API 封装（调 Rust command 的封装层）
- `views/HomeView.vue` — 建房页（真功能）
- `views/FavoritesView.vue` — 收藏页（真功能）
- `views/SessionsView.vue` — 会话页（占位，SessionManager 后端未做）
- `views/ChatboxView.vue` — OSC 聊天页（真功能）
- `views/SettingsView.vue` — 设置页（新加，真功能）
- `views/LayoutPreviewView.vue` — 布局预览（重点）：双栏控制台（左 Session+OSC / 右 世界收藏+建房记录+建房工具栏），Session/收藏为假数据，建房工具栏为真功能
- `public/bg.jpg` — 背景图（暗色森林插画，CSS 模糊 + 深色遮罩）

## 技术约束（重要）

- Vue 3 `<script setup>`，**无 Tailwind**（用 style.css 类或内联样式，不能用 w-full 等）
- 文字颜色/阴影体系：白字 + 深色投影；hover 发光只 hover 触发
- 通知组件已装：`vue3-dynamic-island@1.0.6`（全局挂载后可 `Island.success/error/info(title, subtitle)` 弹灵动岛，替代 alert/内联提示）
- 参考：`C:\Users\lyric\.openclaw\workspace\tmp\vue-bits`（vue-bits 动效组件克隆，技能 vue-bits-translator 有转译表，优先用已有轮子）
- Rust 后端 command：auth_status/logout、list_groups、favorites_groups/worlds、create_instance、resolve_world、send_chatbox、settings_get/set、app_mode（经 `invoke()` 调用）

## 当前待改/方向

- 双栏控制台（LayoutPreviewView）仍在预览，底部留白问题需最终确认
- 布局定稿后：把假数据换真数据、把 4 个旧视图功能合并进总控台、清理临时预览入口
- 界面文字（中文）如需调整/润色直接改各 .vue 的模板文本
