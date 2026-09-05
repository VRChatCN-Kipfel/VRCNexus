//! 全局状态：API 客户端、认证用户、配置
//! 用 Mutex 包住，跨 command / RPC 共享。不依赖 tauri（纯 Rust，可被 GUI 与 rpc 层共同持有）。

use crate::config::ConfigStore;
use crate::vrchat::{self, Api, User};
use std::sync::Mutex;

pub struct AppState {
    /// 当前 API 客户端（None = 未认证）
    pub api: Mutex<Option<Api>>,
    /// 当前用户
    pub user: Mutex<Option<User>>,
    /// 认证来源
    pub auth_source: Mutex<Option<String>>,
    /// 三模式配置
    pub config: ConfigStore,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            api: Mutex::new(None),
            user: Mutex::new(None),
            auth_source: Mutex::new(None),
            config: ConfigStore::new(),
        }
    }

    /// 认证：从 VRCX 读 cookie → 验证 → 缓存。返回用户。
    pub fn authenticate(&self) -> Result<User, String> {
        // 已有有效会话直接返回
        if let Some(u) = self.user.lock().unwrap().clone() {
            return Ok(u);
        }
        let cookie = vrchat::vrcx_auth_cookie()
            .ok_or_else(|| "没有可用 auth cookie：请确保 VRCX 正在运行（自动从 VRCX 数据库读取）".to_string())?;
        let api = Api::new(cookie);
        let user = api.current_user()?;
        *self.api.lock().unwrap() = Some(api);
        *self.user.lock().unwrap() = Some(user.clone());
        *self.auth_source.lock().unwrap() = Some("vrcx".to_string());
        Ok(user)
    }

    /// 取 API 客户端（未认证先认证）
    pub fn api(&self) -> Result<Api, String> {
        {
            let guard = self.api.lock().unwrap();
            if let Some(a) = guard.as_ref() {
                return Ok(a.fork());
            }
        }
        self.authenticate()?;
        let guard = self.api.lock().unwrap();
        guard.as_ref().map(|a| a.fork()).ok_or_else(|| "API 客户端未就绪".to_string())
    }

    pub fn user(&self) -> Option<User> {
        self.user.lock().unwrap().clone()
    }

    pub fn logout(&self) {
        *self.api.lock().unwrap() = None;
        *self.user.lock().unwrap() = None;
        *self.auth_source.lock().unwrap() = None;
    }
}
