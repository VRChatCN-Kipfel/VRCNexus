# AGENTS.md — VRCNexus 项目协作守则

> 本文件约束**任何**在本仓库工作的 AI 代理/会话/协作者。开始改代码前先读它。

## 项目是什么

VRCNexus = VRChat 本地自动化桌面应用（**Tauri 2 + Vue 3 + Rust**）。
不是网页应用！前端有 Tauri 环境校验，浏览器直开 dev 地址只会看到阻断页。

- 后端：Rust（src-tauri/src），VRChat API 客户端 / OSC / 三模式配置 / RPC 服务
- 前端：Vue 3 `<script setup>`（**纯 JS**，无 TypeScript、无 Tailwind）
- 数据源：VRCX 本地 sqlite 的活 auth cookie（用户无感认证）

## 🚨 红线（违反会出事，务必遵守）

### 1. 未授权素材永不入库
`frontend/public/bg*.jpg`（背景插画）**未取得作者授权**：
- 已被 `.gitignore` 绝对路径忽略 + 从 git 历史清理（filter-repo 重写过）
- **禁止** `git add -f` / 改路径绕过忽略 把它提交或推送
- 需要背景图时本地放置即可（文件存在就能加载，提交不含它）
- 新加素材同理：先确认授权，无授权一律走 ignore

### 2. 纯 JS 的 Vue 写法约束
- **禁止** `withDefaults(defineProps({...对象}))` —— 纯 JS（无 `lang="ts"`）下是非法组合，Vue 编译器直接报错、组件整块不渲染。用标准 `defineProps({...})` 带 default
- 不用 TS 语法、不用 Tailwind class（用 `style.css` 的类或内联样式）
- 改含 JS 模板字符串的 `.vue` 后，用 `node --check` 验证提取的 `<script>` 语法（历史教训：转义被吃 → 整段 JS 失效）

### 3. 原生 `<select>` 禁令
原生 `<select>` 的弹出列表由 **Windows 系统绘制**，CSS 无法玻璃化（深灰白底）。新增下拉/选择一律用自绘 `GlassSelect.vue`（已支持智能上下弹出）。

### 4. 玻璃效果性能
- 已回退 SVG 液态玻璃（GlassSurface / backdrop-filter: url(#filter)）——实测卡顿，**不要加回**
- 玻璃 = 半透明 tint + backdrop-filter: blur（≤12px）+ 内高光；背景用预烘焙模糊图（如有）
- 大面积实时 backdrop-filter + 动画会卡，动画元素避免叠在 blur 容器内

## Git 工作流

- 主仓库：`D:\VRCNexusTauri`（分支 `master`）
- **UI 工作树**：`C:\Users\lyric\Desktop\UI`（worktree，分支 `ui-desktop`）——外部 AI 改 UI 用这个
- 流程：在 worktree 改 → `git commit`（ui-desktop）→ 主仓库 `git merge ui-desktop` → `bun run tauri dev` 验证
- **禁止手动复制文件做同步**——一切走 git（worktree/merge）
- 改前先 `git status` / `git worktree list` 看有无别的会话在动；并行会话活跃时别碰其未提交文件
- 远端：`github.com/VRChatCN-Kipfel/VRCNexus`（public，**无 LICENSE**，勿擅自加）

## 常用命令

```bash
bun install                # 根依赖
cd frontend && bun install # 前端依赖
bun run tauri dev          # 开发(自动起 vite@4040 + 编 Rust)
bun run tauri build        # 打包(MSI/NSIS/exe)
```

- Rust 编译：cargo/rustc 在 `C:\Users\lyric\.cargo\bin`（PATH 可能需手动补）
- 前端 vite：端口 **4040 strictPort**（勿改，tauri.conf devUrl 对应）
- 需要跑 dev 时若 4040 被占/窗口消失：清理 vrcnexus/vite 进程后重启

## 设计语言（改 UI 前必读）

- 深色冷灰玻璃拟态（Slate 系）：变量在 `frontend/src/style.css :root`
  - `--glass` 冷蓝灰底、`--accent #94a3b8`、`--ink #e4eaf0`、`--bg0 #0d1117`
  - 已从紫色系全面迁到冷灰——**不要改回紫/高饱和**
- 动效：丝滑过渡（下拉弹性展开/视图微缩放/弹窗中心放大），已就位
- 可参考 `vue-bits`（本地克隆 `~/.openclaw/workspace/tmp/vue-bits`）抄纯 CSS 效果；勿引 gsap/motion-v/three
- 通知组件 `vue3-dynamic-island` 已装未接入，可按需用 `Island.success/error/info`

## 已知待办（别重复造）

- SessionsView 占位：SessionManager（Rust 进程管理/启动队列）未实现
- 布局预览 LayoutPreviewView 的 Session/收藏为假数据，待接真
- 默认视图临时指向 layout（App.vue `view = ref('layout')`），定稿后改回 home
