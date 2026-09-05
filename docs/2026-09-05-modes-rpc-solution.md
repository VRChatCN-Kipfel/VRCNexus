# VRCNexus 三模式 + RPC 服务 + 设置弹窗 — 解决方案

> 日期：2026-09-05
> 状态：待评审
> 范围：架构方案（不涉及代码实施，实施分阶段见 §8）

---

## 1. 目标

1. 为 VRCNexus 定义 **本地 / 远程 / 服务** 三种运行模式，每模式独立配置。
2. 引入 **对外 RPC 服务层**，使「本体分身 / AI / 其他程序」可以远程调用 Core 能力。
3. GUI 新增 **设置弹窗**，在页面内可视化编辑模式与连接配置。
4. 配置支持 **环境变量覆盖配置文件**，避免敏感信息（token）落盘。

---

## 2. 概念模型

```
        ┌──────────────────────────────────────────┐
        │              VRCNexus Core                │
        │  认证 / 建房 / 收藏 / 会话 / OSC / 配置     │
        └──────┬──────────────────┬─────────────────┘
               │                  │
        ┌──────▼─────┐    ┌───────▼────────┐
        │  GUI 前端   │    │  RPC 服务层      │
        │ (Vue/Tauri) │    │ (HTTP/JSON)     │
        └──────┬─────┘    └───────┬────────┘
               │                  │
     ┌─────────▼─────────┐       │
     │ 本地直调 / 远程转发 │       │ ← 分身 / AI / 外部程序
     └───────────────────┘
```

**核心原则：Core 不感知调用者是谁。** GUI 直调、RPC 远程调用、未来 MCP 适配，都进同一个 Core。

---

## 3. 三模式定义

| 模式 | GUI 数据源 | RPC 服务 | 凭据来源 | 典型场景 |
|---|---|---|---|---|
| `local` | 本机 Core（现状） | 不开 | 本机 VRCX cookie | 单机自用（默认） |
| `service` | 本机 Core（照常） | **开**，对外提供 | 本机 VRCX cookie | 家里常驻机，分身/AI 连进来 |
| `remote` | **远端** Core（RPC 客户端转发） | 不开 | 远端 token（本机不读 cookie） | 人在外面，笔记本连家里那台 |

要点：
- **service 不等于无头**。GUI 照常跑，RPC 服务是"附加挂载"。无头（`--headless`）是以后的事，用命令行参数再说。
- **remote 模式本机不执行业务**，所有命令转发到远端 service。本机不需要 VRCX。
- 三者共享同一份 Core 代码与同一套命令方法。

---

## 4. 配置体系

### 4.1 优先级链（关键）

```
内置默认值  <  配置文件  <  环境变量（VRCN_*）  <  (未来 CLI 参数)
```

> 已定：环境变量前缀 **`VRCN_`**（2026-09-05 评审确认，非 VRCNEXUS_）

- GUI 设置弹窗读写的是**配置文件层**。
- 环境变量用于部署/临时覆盖，**不落盘**。
- 生效值需标明来源（default / file / env），设置弹窗中显示来源徽标。

### 4.2 配置文件

- 路径：`%APPDATA%\cn.lyric.vrcnexus\config.toml`（Tauri `app_config_dir`；支持 `VRCNEXUS_CONFIG` 环境变量指向自定义路径）
- 格式：**TOML**（与 Python 老版 `config.toml` 一脉相承）
- 由 Rust 侧统一读取/解析；前端通过新增 command（`settings_get` / `settings_set`）读写，**不直接碰文件**。

### 4.3 配置结构草案

```toml
# 当前模式
mode = "local"          # local | remote | service

[local]
# 无额外配置 —— 凭据默认走本机 VRCX cookie
# （可选覆盖：auth_source = "vrcx"）

[remote]
server_url = "http://127.0.0.1:4455"   # 远端 service 地址
token      = ""                        # 远端接入 token（建议走环境变量）

[service]
enabled    = true        # GUI 启动时是否自动挂载 RPC
bind       = "127.0.0.1" # 默认仅本机；跨机需改为局域网/隧道地址
port       = 4455
token      = ""          # 客户端接入 token；留空则启动时自动生成并提示
auth       = "challenge" # 认证开关：challenge（挑战应答，默认）/ off（关闭加密，仅限本机信任环境）
```

### 4.4 环境变量对应表

| 环境变量 | 覆盖目标 |
|---|---|
| `VRCN_MODE` | `mode` |
| `VRCN_REMOTE_SERVER_URL` | `remote.server_url` |
| `VRCN_REMOTE_TOKEN` | `remote.token` |
| `VRCN_SERVICE_ENABLED` | `service.enabled` |
| `VRCN_SERVICE_BIND` | `service.bind` |
| `VRCN_SERVICE_PORT` | `service.port` |
| `VRCN_SERVICE_TOKEN` | `service.token` |
| `VRCN_SERVICE_AUTH` | `service.auth`（`challenge` / `off`） |
| `VRCN_CONFIG` | 配置文件路径本身 |

> 认证开关默认 `challenge`（安全默认）。切 `off` 时 RPC 无鉴权裸奔——仅允许 `bind=127.0.0.1` 时保存生效；bind 为外部地址时拒绝 off 或强警告（见 §5.5）。

> Token 推荐只走环境变量或由程序生成后存于受限权限位置；配置文件中的 token 字段保留但标注风险。

---

## 5. RPC 服务层

### 5.1 风格决策

**不做 REST/JSON-RPC 二选一，选「方法中心 + HTTP/JSON」**（✅ 2026-09-05 评审确认）：

- `commands.rs` 的 8 个方法天然是"方法"（建房、查收藏、发聊天框），方法中心零转换成本。
- 与未来 MCP 工具调用（`{name, arguments}`）**语义同构**，届时只需薄适配层，Core 不动。
- curl 可直接调试；不引 gRPC/tarpc 等重框架。

**传输分两轨（不冲突）**：
- **同步请求** → HTTP `POST /rpc`
- **服务端事件推送**（建房完成、VRChat 状态变化、会话事件）→ 后续可选 WebSocket `/ws` 或 SSE。v1 不做，接口预留。

### 5.2 端点草案

> **认证方案（✅ 2026-09-05 定稿）**：带 session 的挑战应答——登录单次握手，后续带 session token 直通。见 §5.5。

```
POST /rpc/auth/challenge     → 200 {challenge}                     # ① 取挑战
POST /rpc/auth/verify        → 200 {session_token, expires_at}     # ③ 验证+发 session
POST /rpc                    → 200 {ok, data} / 401/400            # ④ 业务（带 X-VRCN-Session）
GET  /health                 → 200 {status: "ok"}                  # 存活探针（无鉴权）
GET  /rpc/methods            → 200 [方法清单]                      # 方法自省
Content-Type: application/json
X-VRCN-Session: <session token>          # ④ 起所有业务请求携带

- `GET /health` → 存活探针（无鉴权或仅轻量）
- `GET /rpc/methods` → 列出可用方法（给分身做自省，调试友好）

### 5.3 方法清单（v1 = 现有 8 命令 1:1）

| RPC method | 对应现有 command |
|---|---|
| `auth.status` | `auth_status` |
| `auth.logout` | `auth_logout` |
| `groups.list` | `list_groups` |
| `favorites.groups` | `favorites_groups` |
| `favorites.worlds` | `favorites_worlds` |
| `instance.create` | `create_instance` |
| `world.resolve` | `resolve_world` |
| `chatbox.send` | `send_chatbox` |

### 5.4 安全基线

- 默认 `bind = 127.0.0.1`；**跨机必须显式改 bind + 配强 token**。
- token 必填：无 token 或签名错误 → 401。日志不落 token/签名。
- 建房/聊天框为敏感操作，未来可加方法级权限（先记 TODO）。
- service 与 GUI 共存时，RPC 共享同一个 `AppState`（同一份认证会话），不另起炉灶。

### 5.5 认证方案（✅ 2026-09-05 定稿：带 session 的挑战应答）

> 用户原倾向「歪门邪道」（XOR），后经讨论确认最佳形态：**challenge-response 登录 + 短时 session token**。
> 技术事实：固定 key 的 XOR 可逆，只能防日志/肉眼，不充当唯一防线；主 token 通过握手推导、永不上线。

**流程（登录单次握手 → 后续 session 直通）**

```text
客户端                                  服务端
  │ ① POST /rpc/auth/challenge           │
  │─────────────────────────────────────>│ 生成随机 challenge（60s 过期）
  │ ← 200 {challenge}                    │
  │ ② resp = SHA256(token + challenge)   │
  │ ③ POST /rpc/auth/verify              │
  │    {challenge, response}             │
  │─────────────────────────────────────>│ 验证通过 → 签发 session token（随机、24h）
  │ ← 200 {session_token, expires_at}    │
  │ ④ 之后所有请求带 X-VRCN-Session      │
  │    POST /rpc  {method, params}       │
  │─────────────────────────────────────>│ 只校验 session（快）
  │ ← 200 {ok, data}  /  401 明确错误    │
```

**要点**

- **单次验证**：登录握手一次，之后 session 直通，无需每请求签名
- **错误返回明确**：各阶段独立——握手失败 `401 auth failed` / session 过期 `401 session expired` / 方法不存在 `400 unknown method` / 业务失败 `{ok:false, error}`
- **主 token 永不上线**：仅以 `SHA256(token+challenge)` 形式出现在握手，抓包逆不回
- **泄露可止血**：session token 被偷 → 24h 过期 + 可主动吊销；主 token 不受影响
- **XOR 可叠**：session token 传输时可选叠 XOR 混淆层（防日志显眼）；泄了也只是短命 session
- challenge 有效期 60s；session 默认 24h（可配置）；可加 session 吊销表（v2）

**认证开关（2026-09-05 补充）**

- `service.auth = "challenge"`（默认）：走上面完整握手；需要配置 token
- `service.auth = "off"`：关闭认证与加密，`POST /rpc` 裸调、无 session 校验
  - 适用：纯本机信任环境（如 `bind=127.0.0.1` + 本机分身），省去握手；调试期方便
  - 红线：**bind 为外部地址（非 127.0.0.1）时不允许 off**——保存配置即拦截并提示，防止 token-less 服务暴露到网络
  - off 时 `GET /health`、`/rpc/methods` 行为不变；所有业务方法免鉴权直接执行
  - GUI 设置弹窗中：认证开关切 off 时若 bind 非本机，前端即提示并阻止保存（后端双保险校验）

---

## 6. GUI 设置弹窗

### 6.1 入口

- 左侧导航栏新增「设置」项（⚙ 齿轮），或底栏齿轮按钮（待定，见 §9）。
- 点击后**页面内打开 modal**（沿用现有玻璃拟态样式，非新窗口）。

### 6.2 弹窗布局

```
┌─ 设置 ────────────────────────────────┐
│ ┌────────┬──────────────────────────┐ │
│ │ 通用     │  当前模式: ○本地 ○远程 ○服务 │ │
│ │ 远程连接  │                          │ │
│ │ 服务     │  [模式对应的表单]           │ │
│ │ 关于     │                          │ │
│ └────────┴──────────────────────────┘ │
│        [恢复默认]        [取消] [保存]  │
└───────────────────────────────────────┘
```

- 左侧小节：通用（模式选择）/ 远程连接 / 服务 / 关于
- 切换模式 radio 时，右侧表单切换为对应模式配置段
- 每个字段旁显示来源徽标：`默认` / `配置文件` / `环境变量`（env 覆盖时字段只读并提示）

### 6.3 表单字段

| 小节 | 字段 | 说明 |
|---|---|---|
| 通用 | 模式（local/remote/service） | radio 三选一 |
| 远程连接 | server_url / token | remote 模式必填；token 可留空 → 提示走环境变量 |
| 服务 | 启用开关 / bind / port / token / **认证开关（challenge/off）** | token 旁带「生成随机」按钮；认证 off 时 bind 必须为本机 |
| 关于 | 版本 / 配置路径 / 当前生效值预览 | 只读 |

### 6.4 行为

- **保存**：`settings_set` → Rust 写回配置文件（merge，不整文件覆盖）。
- **应用**：v1 提示「部分配置重启后生效」；remote/service 切换涉及前端数据源，需重启或 v2 热切换（见 §7 前端抽象，若做 api 抽象层则 remote 可热切）。
- **恢复默认**：清除自定义项回默认值（不删 env）。

---

## 7. 代码改造点

### 7.1 Rust 侧（src-tauri）

```
src/
├── core/                  # ← 新增：与 Tauri 解绑的业务层
│   ├── mod.rs
│   ├── state.rs           # AppState 从 src/state.rs 迁入（去掉 tauri::State 依赖）
│   ├── api.rs             # （可选）Api 再封装，保持现 vrchat.rs 不动
│   └── commands.rs        # 命令逻辑从 src/commands.rs 迁入（纯函数）
├── rpc/                   # ← 新增：RPC 服务层
│   ├── mod.rs             # axum router：/rpc /health /rpc/methods
│   ├── server.rs          # 启动/停止（tokio task 持有 AppState handle）
│   └── dispatch.rs        # method → core 命令 的映射表
├── config.rs              # ← 新增：配置读取（默认<文件<env 优先级解析）
├── commands.rs            # 变薄：tauri::command 壳 → 调 core（或直接迁空）
├── lib.rs                 # manage(core) + 按 mode 决定是否挂 rpc
├── state.rs               # 删除/迁移
├── vrchat.rs / osc.rs     # 不动（本就是纯业务）
```

新增依赖：`axum`、`tokio`、`toml`（+ `config` crate 可选）。

### 7.2 前端（frontend/src）

```
src/
├── lib/
│   └── api.js             # ← 新增：统一调用入口
│                          #    call(method, params):
│                          #      mode==='remote' → fetch(`{server_url}/rpc`)
│                          #      否则 → invoke(method)
├── views/
│   └── SettingsView.vue   # ← 新增：设置弹窗（modal）
├── App.vue                # nav 加「设置」项；挂载 SettingsView
├── HomeView.vue 等        # 各 view 的 invoke 改走 lib/api.js
```

> 前端抽象 `api.js` 是关键：remote 模式 = 换数据源，UI 组件零改动。这也让"远程模式热切换"成为可能（v2）。

---

## 8. 分阶段实施

| 阶段 | 内容 | 验收 |
|---|---|---|
| **P0** | `config.rs`：优先级解析 + TOML 读写 | `VRCNEXUS_SERVICE_PORT=9999` 覆盖配置文件生效 |
| **P1** | Core 解绑 + service 模式 RPC 跑通 | GUI 开着，`curl POST /rpc` 调 `instance.create` 成功 |
| **P2** | remote 模式：前端 `api.js` + RPC 客户端 | 两台机器/两实例互通；GUI 经远端建房成功 |
| **P3** | 设置弹窗 UI（读/写/来源徽标） | 弹窗改配置 → 重启后生效 |
| **P4** | （可选）WebSocket 事件推送 | 服务端建房完成主动推给订阅方 |
| **P5** | （可选）MCP 适配层 | OpenClaw 直接调 VRCNexus 建房 |

每阶段独立可交付、可回滚，不阻塞主线功能开发。

---

## 9. 评审决议（2026-09-05）

| # | 问题 | 决议 |
|---|---|---|
| 1 | RPC 风格 | ✅ **方法中心**（非 REST/JSON-RPC 二选一） |
| 2 | 环境变量前缀 | ✅ **`VRCN_`**（非 VRCNEXUS_） |
| 3 | token 传输 | ✅ **带 session 的挑战应答**（challenge-response 登录 + 短时 session；XOR 可选混淆层）；另加 **认证开关** `service.auth = challenge/off`（默认 challenge；off 仅限 bind=127.0.0.1，跨机拒绝） |
| 4 | 设置入口 | ✅ 底栏齿轮图标（导航五项已满） |
| 5 | 服务端口 | ✅ 4455 |
| 6 | 配置文件格式 | ✅ TOML |
| 7 | remote 热切换 | ✅ v1 统一"重启生效"，先跑通再优化 |
