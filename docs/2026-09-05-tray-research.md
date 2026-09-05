# VRCNexus 系统托盘（System Tray）调研方案

> 日期：2026-09-05
> 状态：调研完成（subagent 深度调研 + 复核）
> 范围：仅方案，未实施

## 1. 需求背景

VRCNexus（Tauri 2 + Rust + Vue3）有三模式（local/remote/service）。托盘需求场景：
- service 模式下 GUI 常驻后台，最小化到托盘不占任务栏
- 托盘菜单快捷操作（建房/看状态/退出）
- 后台驻留指示（区别于"关了窗口=退了程序"）

## 2. Tauri 2 托盘官方方案

Tauri 2 把托盘从 `tauri::system_tray` 改为 **`tauri::tray::TrayIconBuilder`**（内置，无需额外插件）。要点：

### 2.1 启用 feature
```toml
# src-tauri/Cargo.toml
tauri = { version = "2", features = ["tray-icon"] }
```

### 2.2 最小实现（Rust 侧）

```rust
// lib.rs setup 中
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::menu::{Menu, MenuItem};
use tauri::Manager;

// 构建菜单
let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
let create = MenuItem::with_id(app, "create", "建房…", true, None::<&str>)?;
let menu = Menu::with_items(app, &[&create, &quit])?;

let tray = TrayIconBuilder::with_id("main-tray")
    .icon(app.default_window_icon().unwrap().clone())
    .menu(&menu)
    .show_menu_on_left_click(true) // Windows 左键也弹菜单；false 则左键触发事件
    .on_menu_event(|app, event| match event.id.as_ref() {
        "quit" => app.exit(0),
        "create" => { /* emit 到前端或建房 */ }
        _ => {}
    })
    .on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
            // 左键单击 → 显示主窗口
            let app = tray.app_handle();
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
    })
    .build(app)?;
```

### 2.3 图标资源
- Windows 托盘用 `.ico`（多尺寸，至少 16x16/32x32），Tauri 构建时从 `tauri.conf.json` 的 `bundle.icon` 生成
- `app.default_window_icon()` 取的是窗口图标，可直接复用作托盘图标；想要专门小图标可 `.icon(path)` 单独指定

### 2.4 tauri.conf.json 配置
```json
{
  "app": {
    "trayIcon": { "id": "main-tray", "iconPath": "icons/icon.ico" }
  }
}
```
> 注：trayIcon 可在配置里静态声明，但**动态菜单/事件仍需 Rust 代码**。推荐代码方式（灵活）。

## 3. Windows 特有注意

- **最小化到托盘 vs 关闭到托盘**：Tauri 2 需自己处理窗口事件
  - 关闭到托盘：监听 `RunEvent::ExitRequested` → `api.prevent_exit()` + 隐藏窗口
  - 最小化到托盘：监听窗口 minimize 事件 → hide()
- **explorer.exe 重启后托盘图标消失**：Windows 的 `TaskbarCreated` 消息——Tauri 2 的 tray-icon crate 已处理（v2.1+），但旧版本有 bug；保持 tauri 2.11.x 即可。已知 issue：重启 explorer 后图标可能短暂消失，官方在 wry/tray-icon 层已修复
- **左键/右键**：Windows 默认**右键**出菜单；左键默认无动作（除非 `show_menu_on_left_click(true)` 或监听 Click 事件）。托盘图标左键单击恢复窗口是常见需求，需手动处理（见上例）
- **通知气泡**：托盘本身无 balloon API；要用 `tauri-plugin-notification`（Windows 走 toast）

## 4. 与三模式结合建议

| 模式 | 托盘行为 |
|---|---|
| local | 最小化到托盘可选；关闭=退出 |
| service | **推荐常驻托盘**：关闭按钮→最小化到托盘（程序继续跑 RPC），托盘菜单显示"服务运行中"状态 + 退出 |
| remote | 同 local（GUI 只是客户端） |

- **service 模式关闭窗口不退出**：`RunEvent::ExitRequested` 里如果 mode=service 且 auth 开着 RPC，`prevent_exit()` + hide 窗口 + 托盘提示
- **与未来 headless**：headless 时窗口根本不建，托盘可有可无——service 常驻更多靠 RPC 健康检查而非托盘。托盘是"有人看着机器时"的交互层

## 5. 风险与坑

1. **关闭到托盘 vs 真正退出**要设计清楚，否则用户困惑"关了还在跑"
2. 托盘图标多实例冲突（重复启动）——可考虑 `tauri-plugin-single-instance`
3. debug 模式下 `default_window_icon()` 可能为空（icons 未嵌入）→ 需 `icon()` 显式给
4. 菜单文案中文无问题（UTF-8），但 Windows 托盘 tooltip 建议短

## 6. 参考路线

1. Cargo.toml 加 `features = ["tray-icon"]`
2. lib.rs setup 建 TrayIconBuilder + 菜单（退出/建房/显示主窗）
3. 窗口关闭行为：service 模式 → hide + prevent_exit；否则正常退出
4. （可选）前端监听托盘菜单 emit 的事件做建房弹窗
5. （可选）single-instance 插件防多开

## 7. 参考链接
- Tauri 2 tray 官方: https://v2.tauri.app/learn/system-tray/
- tray-icon crate: https://crates.io/crates/tray-icon
- Tauri 菜单 API: https://v2.tauri.app/learn/menu/
- window-state 插件(记忆窗口位置): https://v2.tauri.app/plugin/window-state/
