//! VRCNexus Tauri 后端
//! 路线 B：Rust 原生重写（不依赖 Python 后端）
//! 模块：vrchat(API) / osc / state / commands / config / rpc

mod commands;
mod config;
mod osc;
mod rpc;
mod state;
mod vrchat;

use state::AppState;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let core = Arc::new(AppState::new());

    tauri::Builder::default()
        .manage(core.clone())
        .setup(move |app| {
            let _ = app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .build(),
            );

            // service 模式：附加 RPC 服务（GUI 照常跑），独立线程不阻塞主循环
            if core.config.mode() == "service" {
                let core_for_rpc = Arc::clone(&core);
                std::thread::spawn(move || {
                    if let Err(e) = rpc::serve(core_for_rpc) {
                        log::error!("RPC 服务启动失败: {e}");
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth_status,
            commands::auth_logout,
            commands::list_groups,
            commands::favorites_groups,
            commands::favorites_worlds,
            commands::create_instance,
            commands::resolve_world,
            commands::send_chatbox,
            commands::settings_get,
            commands::settings_set,
            commands::app_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
