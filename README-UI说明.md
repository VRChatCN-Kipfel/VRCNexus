# VRCNexus 前端 UI 代码包（UI + 文字 + 动画）

Tauri 桌面应用（Vue 3 + Vite，**纯 JS 无 TS 无 Tailwind**）的前端 UI 源码。
**界面结构、中文文字、动画效果全部内嵌在 .vue / .js / .css 里**，无独立 i18n。

## 与真实项目对应

- 真实工程：`D:\VRCNexusTauri\`（git 主仓库，frontend/ + src-tauri/）
- 本文件夹是它的 **git worktree**（分支 `ui-desktop`），等于完整项目副本
- **改完提交**：`git add -A && git commit -m "说明"`（在 ui-desktop 分支），主仓库合并由负责人处理
- 运行预览：根目录 `bun run tauri dev`（Tauri 桌面应用，非纯浏览器；main.js 有环境校验）
- 参考图：`C:\Users\lyric\Downloads\QQ20260905-175222.png`（目标视觉：双栏玻璃控制台）

## 文件说明（全部在 frontend/src/）

- `main.js` — 入口 + Tauri 环境校验（非 Tauri 显示阻断页，勿删）
- `App.vue` — 应用壳：左侧药丸导航 + 主视图切换 + 背景场景 + 视图过渡动画
- `style.css` — 玻璃拟态设计系统 v2：CSS 变量(:root) + 公共组件类 + 常驻动画(keyframes)
- `lib/api.js` — 前端 API 封装（调 Rust command）
- `components/GlassSelect.vue` — 自绘玻璃下拉（替代原生 select，智能上下弹出+弹性动画）
- `views/HomeView.vue` — 建房页（真功能）
- `views/FavoritesView.vue` — 收藏页（真功能）
- `views/SessionsView.vue` — 会话页（占位）
- `views/ChatboxView.vue` — OSC 聊天页（真功能）
- `views/SettingsView.vue` — 设置弹窗（真功能）
- `views/LayoutPreviewView.vue` — 双栏控制台预览（重点）：左 25% Session+OSC / 右 75% 世界收藏大卡+最近记录+底部建房工具栏 / 最右窄托盘
- `public/bg.jpg` — 背景图（暗色森林插画，CSS 模糊+深色遮罩）

## 技术约束（重要）

- **纯 JS**：`<script setup>` 不用 TS；**`defineProps` 用对象式，禁止 `withDefaults(defineProps({...}))`**（纯 JS 下非法组合会编译崩）
- **无 Tailwind**：用 style.css 类或内联样式
- 动效：可参考 vue-bits（克隆 `C:\Users\lyric\.openclaw\workspace\tmp\vue-bits`），**优先纯 CSS 可抄的**，勿引 gsap/motion-v/three
- 原生 `<select>` 弹出列表系统绘制无法玻璃化 → 已全部换 GlassSelect
- 通知组件已装 `vue3-dynamic-island`（未接入，可按需用 `Island.success/error/info`）
- Rust command：auth_status/logout、list_groups、favorites_groups/worlds、create_instance、resolve_world、send_chatbox、settings_get/set、app_mode

## 当前状态（2026-09-05 21:2x）

- ✅ GlassSelect 自绘玻璃下拉（全项目替换原生 select）
- ✅ 丝滑过渡动画：下拉弹性展开 / 视图切换微缩放 / 设置弹窗中心放大
- ✅ 双栏控制台预览布局（LayoutPreviewView）
- ❌ 已回退：GlassSurface SVG 液态玻璃（实测太卡，勿再加回）
- 待办方向：布局定稿后假数据换真、4 视图功能合并进总控台、Session 列表接真实会话
