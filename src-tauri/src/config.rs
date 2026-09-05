//! 三模式配置系统：mode(local/remote/service) + [remote]/[service] 段
//! 优先级：内置默认 < 配置文件(TOML) < 环境变量(VRCN_*)
//! 来源追踪(default/file/env) + merge 写回。纯 Rust，不依赖 tauri。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

pub const MODE_LOCAL: &str = "local";
pub const MODE_REMOTE: &str = "remote";
pub const MODE_SERVICE: &str = "service";

const DEFAULT_MODE: &str = MODE_LOCAL;
const DEFAULT_SERVER_URL: &str = "http://127.0.0.1:4455";
const DEFAULT_BIND: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 4455;
const DEFAULT_AUTH: &str = "challenge";

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

fn is_loopback(bind: &str) -> bool {
    bind == "127.0.0.1" || bind == "localhost" || bind == "::1"
}

fn gen_token() -> String {
    use rand::RngCore;
    let mut b = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut b);
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// 配置文件结构（None = 文件里未显式写该字段）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FileConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<RemoteCfg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<ServiceCfg>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RemoteCfg {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ServiceCfg {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<String>,
}

/// 环境变量覆盖视图（None = 该 env 未设置）
#[derive(Debug, Clone, Default)]
struct EnvView {
    mode: Option<String>,
    remote_server_url: Option<String>,
    remote_token: Option<String>,
    service_enabled: Option<String>,
    service_bind: Option<String>,
    service_port: Option<String>,
    service_token: Option<String>,
    service_auth: Option<String>,
}

fn env_view() -> EnvView {
    EnvView {
        mode: env("VRCN_MODE"),
        remote_server_url: env("VRCN_REMOTE_SERVER_URL"),
        remote_token: env("VRCN_REMOTE_TOKEN"),
        service_enabled: env("VRCN_SERVICE_ENABLED"),
        service_bind: env("VRCN_SERVICE_BIND"),
        service_port: env("VRCN_SERVICE_PORT"),
        service_token: env("VRCN_SERVICE_TOKEN"),
        service_auth: env("VRCN_SERVICE_AUTH"),
    }
}

/// 生效的服务配置（已应用 env + 运行时生成 token）
#[derive(Debug, Clone)]
pub struct ServiceEff {
    pub enabled: bool,
    pub bind: String,
    pub port: u16,
    pub token: String,
    pub auth: String,
}

struct StoreInner {
    file: FileConfig,
    path: PathBuf,
    generated_token: Option<String>,
}

pub struct ConfigStore {
    inner: Mutex<StoreInner>,
}

pub fn config_path() -> PathBuf {
    if let Some(p) = env("VRCN_CONFIG") {
        return PathBuf::from(p);
    }
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("cn.lyric.vrcnexus").join("config.toml")
}

fn load_file(path: &PathBuf) -> FileConfig {
    match std::fs::read_to_string(path) {
        Ok(s) => toml::from_str(&s).unwrap_or_default(),
        Err(_) => FileConfig::default(),
    }
}

fn put(
    values: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    file: Option<String>,
    envv: &Option<String>,
    default: String,
) {
    let (value, source) = if let Some(e) = envv {
        (e.clone(), "env")
    } else if let Some(f) = file {
        (f, "file")
    } else {
        (default, "default")
    };
    values.insert(
        key.into(),
        serde_json::json!({ "value": value, "source": source }),
    );
}

impl ConfigStore {
    pub fn new() -> Self {
        let path = config_path();
        let file = load_file(&path);
        Self { inner: Mutex::new(StoreInner { file, path, generated_token: None }) }
    }

    pub fn path(&self) -> PathBuf {
        self.inner.lock().unwrap().path.clone()
    }

    /// 生效 mode
    pub fn mode(&self) -> String {
        let g = self.inner.lock().unwrap();
        mode_effective(&g.file, &env_view())
    }

    /// 生效服务配置；token 为空时生成随机 32hex 并缓存（同进程内稳定）
    pub fn service(&self) -> ServiceEff {
        let mut g = self.inner.lock().unwrap();
        let ev = env_view();
        let file = &g.file;
        let f = file.service.clone().unwrap_or_default();
        let enabled = ev
            .service_enabled
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| f.enabled.unwrap_or(true));
        let bind = ev.service_bind.clone().unwrap_or_else(|| f.bind.clone().unwrap_or_else(|| DEFAULT_BIND.into()));
        let port = ev
            .service_port
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| f.port.unwrap_or(DEFAULT_PORT));
        let auth = ev
            .service_auth
            .clone()
            .unwrap_or_else(|| f.auth.clone().unwrap_or_else(|| DEFAULT_AUTH.into()));
        let token = ev
            .service_token
            .clone()
            .or_else(|| f.token.clone())
            .unwrap_or_else(|| {
                if g.generated_token.is_none() {
                    g.generated_token = Some(gen_token());
                }
                g.generated_token.clone().unwrap()
            });
        ServiceEff { enabled, bind, port, token, auth }
    }

    /// settings_get 视图
    pub fn view(&self) -> serde_json::Value {
        let g = self.inner.lock().unwrap();
        let file = &g.file;
        let ev = env_view();
        let mut values = serde_json::Map::new();
        put(&mut values, "mode", file.mode.clone(), &ev.mode, DEFAULT_MODE.into());
        put(
            &mut values,
            "remote_server_url",
            file.remote.as_ref().and_then(|r| r.server_url.clone()),
            &ev.remote_server_url,
            DEFAULT_SERVER_URL.into(),
        );
        put(&mut values, "remote_token", file.remote.as_ref().and_then(|r| r.token.clone()), &ev.remote_token, String::new());
        put(
            &mut values,
            "service_enabled",
            file.service.as_ref().and_then(|s| s.enabled).map(|b| b.to_string()),
            &ev.service_enabled,
            "true".into(),
        );
        put(&mut values, "service_bind", file.service.as_ref().and_then(|s| s.bind.clone()), &ev.service_bind, DEFAULT_BIND.into());
        put(
            &mut values,
            "service_port",
            file.service.as_ref().and_then(|s| s.port).map(|p| p.to_string()),
            &ev.service_port,
            DEFAULT_PORT.to_string(),
        );
        // token：显示 file/env；运行时已生成则显示 generated
        let tok_file = file.service.as_ref().and_then(|s| s.token.clone());
        if let Some(t) = ev.service_token.clone() {
            values.insert("service_token".into(), serde_json::json!({ "value": t, "source": "env" }));
        } else if let Some(t) = tok_file.clone() {
            values.insert("service_token".into(), serde_json::json!({ "value": t, "source": "file" }));
        } else if let Some(t) = g.generated_token.clone() {
            values.insert("service_token".into(), serde_json::json!({ "value": t, "source": "generated" }));
        } else {
            values.insert("service_token".into(), serde_json::json!({ "value": "", "source": "default" }));
        }
        put(&mut values, "service_auth", file.service.as_ref().and_then(|s| s.auth.clone()), &ev.service_auth, DEFAULT_AUTH.into());

        let env_keys: Vec<String> = [
            ev.mode.is_some().then(|| "VRCN_MODE".to_string()),
            ev.remote_server_url.is_some().then(|| "VRCN_REMOTE_SERVER_URL".to_string()),
            ev.remote_token.is_some().then(|| "VRCN_REMOTE_TOKEN".to_string()),
            ev.service_enabled.is_some().then(|| "VRCN_SERVICE_ENABLED".to_string()),
            ev.service_bind.is_some().then(|| "VRCN_SERVICE_BIND".to_string()),
            ev.service_port.is_some().then(|| "VRCN_SERVICE_PORT".to_string()),
            ev.service_token.is_some().then(|| "VRCN_SERVICE_TOKEN".to_string()),
            ev.service_auth.is_some().then(|| "VRCN_SERVICE_AUTH".to_string()),
        ]
        .into_iter()
        .flatten()
        .collect();

        serde_json::json!({
            "mode": mode_effective(file, &ev),
            "config_path": g.path.display().to_string(),
            "values": values,
            "env_keys": env_keys,
        })
    }

    /// settings_set：merge 写回，返回新视图。patch: { mode?, remote:{server_url?,token?}, service:{enabled?,bind?,port?,token?,auth?} }
    pub fn apply_patch(&self, patch: serde_json::Value) -> Result<serde_json::Value, String> {
        let mut g = self.inner.lock().unwrap();
        let ev = env_view();

        // mode 校验
        let new_mode = patch
            .get("mode")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| mode_effective(&g.file, &ev));
        if !matches!(new_mode.as_str(), MODE_LOCAL | MODE_REMOTE | MODE_SERVICE) {
            return Err(format!("未知模式: {new_mode}"));
        }

        // 组装新的 service 段（patch > file）
        let mut svc = g.file.service.clone().unwrap_or_default();
        if let Some(s) = patch.get("service") {
            if let Some(v) = s.get("enabled").and_then(|x| x.as_bool()) {
                svc.enabled = Some(v);
            }
            if let Some(v) = s.get("bind").and_then(|x| x.as_str()) {
                svc.bind = Some(v.to_string());
            }
            if let Some(v) = s.get("port").and_then(|x| x.as_u64()).map(|x| x as u16) {
                svc.port = Some(v);
            }
            if let Some(v) = s.get("token").and_then(|x| x.as_str()) {
                svc.token = if v.is_empty() { None } else { Some(v.to_string()) };
            }
            if let Some(v) = s.get("auth").and_then(|x| x.as_str()) {
                svc.auth = Some(v.to_string());
            }
        }
        // 认证 off 校验（env 优先，其次 patch/file 结果）
        let eff_bind = ev.service_bind.clone().or_else(|| svc.bind.clone()).unwrap_or_else(|| DEFAULT_BIND.into());
        let eff_auth = ev.service_auth.clone().or_else(|| svc.auth.clone()).unwrap_or_else(|| DEFAULT_AUTH.into());
        if eff_auth == "off" && !is_loopback(&eff_bind) {
            return Err("关闭认证(off)仅限本机地址(127.0.0.1/localhost)；外部 bind 必须启用 challenge 认证".into());
        }
        if eff_auth != "off" && eff_auth != "challenge" {
            return Err(format!("未知认证方式: {eff_auth}（challenge / off）"));
        }

        let mut remote = g.file.remote.clone().unwrap_or_default();
        if let Some(r) = patch.get("remote") {
            if let Some(v) = r.get("server_url").and_then(|x| x.as_str()) {
                remote.server_url = if v.is_empty() { None } else { Some(v.to_string()) };
            }
            if let Some(v) = r.get("token").and_then(|x| x.as_str()) {
                remote.token = if v.is_empty() { None } else { Some(v.to_string()) };
            }
        }

        // 应用
        let mut file = g.file.clone();
        if let Some(m) = patch.get("mode").and_then(|x| x.as_str()) {
            file.mode = Some(m.to_string());
        }
        file.service = Some(svc);
        file.remote = Some(remote);

        // 写盘（原子：先写临时文件再 rename）
        if let Some(parent) = g.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {e}"))?;
        }
        let s = toml::to_string_pretty(&file).map_err(|e| format!("序列化配置失败: {e}"))?;
        let tmp = g.path.with_extension("toml.tmp");
        std::fs::write(&tmp, &s).map_err(|e| format!("写入配置失败: {e}"))?;
        std::fs::rename(&tmp, &g.path).map_err(|e| format!("保存配置失败: {e}"))?;
        g.file = file;
        g.generated_token = None; // 配置变更后重新生成

        Ok(self.view())
    }
}

fn mode_effective(file: &FileConfig, ev: &EnvView) -> String {
    ev.mode
        .clone()
        .or_else(|| file.mode.clone())
        .unwrap_or_else(|| DEFAULT_MODE.into())
}
