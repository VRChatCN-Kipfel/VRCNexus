//! VRCNexus Tauri 后端
//! 路线 B：Rust 原生重写（不依赖 Python 后端）
//! 模块：vrchat(API) / osc / state / commands

mod commands;
mod osc;
mod state;
mod vrchat;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .setup(|app| {
            let _ = app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .build(),
            );
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
