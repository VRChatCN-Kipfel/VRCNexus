# VRCNexus

> AI-native VRChat 管理与自动化平台 —— 本地优先的桌面应用（Tauri + Vue 3 + Rust）

VRCNexus 是一个面向 VRChat 玩家的**本地自动化控制台**：读取你的 VRCX 登录态与收藏，快速建房、管理多开 Session、推送 OSC 聊天消息，全部在桌面完成，无需打开网页控制台。

## ✨ 功能

- **🏠 建房**：从收藏夹/世界 ID 一键创建群组房/公开房/好友房/私密房，支持区域（jp/usw/use/eu）、排队、年龄门控
- **💎 世界收藏**：读取 VRChat 云端收藏夹（分组目录 + 网格浏览 + 搜索），点选世界直接预填建房
- **📡 Session 管理**：多开实例的启动队列与状态管理（进程对账/恢复，Rust 后端实现中）
- **💬 OSC 聊天**：向指定 VRChat 实例的 OSC 端口推送 /chatbox/input 消息
- **🔐 三模式架构**：`local`（本机直驱）/ `service`（本机 Core + 对外 RPC）/ `remote`（GUI 连远端服务），带挑战应答认证
- **🖥️ 玻璃拟态 UI**：纯 CSS 液态玻璃设计系统 + 自绘下拉 + 丝滑动效，深色冷灰主题

## 🚀 快速开始

### 环境要求

- [Rust](https://rustup.rs/)（1.98+，含 cargo）
- [Bun](https://bun.sh/)（1.4+）或 Node 20+
- Windows 10/11（Tauri 2 + WebView2，系统自带）
- VRChat 账号需已登录 [VRCX](https://github.com/vrcx-project/VRCX)（用于读取活 auth cookie）

### 开发运行

```bash
bun install                 # 安装根依赖(@tauri-apps/cli)
cd frontend && bun install  # 安装前端依赖
bun run tauri dev           # 启动桌面应用(开发模式, 自动起 vite + 编译 Rust)
```

> 注意：VRCNexus 是 **Tauri 桌面应用**，不是网页。`main.js` 内置环境校验——直接用浏览器打开 dev 地址会显示阻断页而非应用。

### 打包

```bash
bun run tauri build         # 产出安装包(MSI/NSIS)与便携 exe
```

## 🏗️ 项目结构

```
VRCNexusTauri/
├── frontend/               # Vue 3 + Vite 前端
│   ├── src/
│   │   ├── App.vue         # 应用壳：导航 + 视图切换 + 背景
│   │   ├── style.css       # 玻璃拟态设计系统(CSS 变量 + 组件类 + 动画)
│   │   ├── lib/api.js      # Rust command 调用封装
│   │   ├── components/     # 自绘组件(GlassSelect 玻璃下拉等)
│   │   └── views/          # Home/Favorites/Sessions/Chatbox/Settings/LayoutPreview
│   └── public/             # 静态资源(背景图需本地自备, 见下)
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── commands.rs     # Tauri command 入口(前端 invoke)
│       ├── vrchat.rs       # VRChat API 客户端(cookie/建房/收藏)
│       ├── osc.rs          # OSC /chatbox/input 发送
│       ├── config.rs       # 三模式配置系统(TOML + 环境变量)
│       ├── rpc.rs          # RPC 服务(axum, service 模式)
│       ├── state.rs        # 全局状态(Mutex 共享)
│       └── lib.rs          # Builder 组装
└── docs/                   # 架构/设计文档
```

## 🔑 认证原理

VRCNexus **不需要你的 VRChat 密码**：它从 VRCX 的本地 SQLite（`%APPDATA%\VRCX\VRCX.sqlite3` 的 `cookies` 表）读取**当前有效的 auth cookie** 作为凭证。VRCX 运行中会实时刷新该 cookie，因此保持 VRCX 登录即可。

- 认证源：VRCX cookie → `/auth/user` 验证 → 缓存
- 所有请求**不走系统代理**（直连 api.vrchat.cloud，避免代理出口导致 cookie 失效）

## 📜 授权与素材状态

> ⚠️ **本仓库当前未选择开源许可证（无 LICENSE 文件）**——保留所有权利，待决定后再补充。
>
> 项目内背景插画（`frontend/public/bg.jpg`）**未取得作者授权**，已通过 `.gitignore` 排除在仓库外（含历史清理）。如需使用请自行获取授权并放入该路径；不要提交未授权素材。

## 🧩 相关

- [VRChatCN-Kipfel 社区](https://github.com/VRChatCN-Kipfel) —— 中文 VRChat 开发者社区
- 前身：Python FastAPI 版（本地归档，未入仓库）
