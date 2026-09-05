//! VRChat API 客户端（Rust 原生实现，替代原 Python vrchat_api/domain）
//! - auth：从 VRCX sqlite cookies 表读活 cookie（与 Python 版同源）
//! - 建房：POST /instances（group 房 + region + queue + ageGate）
//! - 收藏：/favorite/groups + /worlds/favorites（世界收藏夹与列表）

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const API_BASE: &str = "https://api.vrchat.cloud/api/1";
// 官方客户端 clientApiKey（与 Python 版一致）
pub const API_KEY: &str = "JlE5Jldo5Jibnk5O5hTx6XVqsJu4WJ26";
const UA: &str = "VRCNexus/0.1.0 (tauri)";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FavoriteGroup {
    pub name: String, // worlds1 / vrcPlusWorlds1 ...
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "type")]
    pub group_type: String, // world / vrcPlusWorld / avatar / friend
    pub visibility: String,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FavoriteWorld {
    pub id: String,
    pub name: String,
    #[serde(rename = "authorName")]
    pub author_name: String,
    #[serde(rename = "thumbnailImageUrl")]
    pub thumbnail_image_url: String,
    #[serde(rename = "favoriteGroup")]
    pub favorite_group: String,
    #[serde(rename = "releaseStatus")]
    pub release_status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstanceResult {
    pub id: String,
    pub world: Option<InstanceWorld>,
    #[serde(rename = "worldId")]
    pub world_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstanceWorld {
    pub id: String,
}

/// 从 VRCX sqlite 的 cookies 表读活 auth cookie（base64 存 JSON 数组）
pub fn vrcx_auth_cookie() -> Option<String> {
    let db = vrcx_db_path()?;
    if !db.exists() {
        return None;
    }
    let conn = rusqlite::Connection::open_with_flags(
        &db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .ok()?;
    let row: Option<String> = conn
        .query_row("SELECT value FROM cookies WHERE key='default'", [], |r| r.get(0))
        .ok()?;
    drop(conn);
    let row = row?;
    // base64 → JSON [{Name:"auth",Value:"authcookie_xxx",Expired:false}, ...]
    let bytes = base64_decode(&row)?;
    let arr: Vec<serde_json::Value> = serde_json::from_slice(&bytes).ok()?;
    for c in arr {
        if c.get("Name").and_then(|v| v.as_str()) == Some("auth")
            && c.get("Expired").and_then(|v| v.as_bool()).unwrap_or(true) == false
        {
            if let Some(v) = c.get("Value").and_then(|v| v.as_str()) {
                if v.starts_with("authcookie_") {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

fn vrcx_db_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("VRCX_DB_PATH") {
        if !p.is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    dirs::data_dir().map(|d| d.join("VRCX").join("VRCX.sqlite3"))
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s).ok()
}

/// 阻塞式 API 客户端（命令在 tauri 异步线程调用）
#[derive(Clone)]
pub struct Api {
    client: reqwest::blocking::Client,
    cookie: String,
}

impl Api {
    pub fn new(cookie: String) -> Self {
        let client = reqwest::blocking::Client::builder()
            .user_agent(UA)
            .no_proxy() // 禁用系统代理（9/4 教训：代理会致 401）
            .build()
            .expect("reqwest client");
        Self { client, cookie }
    }

    /// 供 state 复制出独立客户端（共享底层连接池）
    pub fn fork(&self) -> Api {
        Api {
            client: self.client.clone(),
            cookie: self.cookie.clone(),
        }
    }

    pub fn get(&self, path: &str) -> Result<serde_json::Value, String> {
        let url = format!("{API_BASE}{path}");
        let resp = self
            .client
            .get(&url)
            .header("x-api-key", API_KEY)
            .header("Cookie", format!("auth={}", self.cookie))
            .send()
            .map_err(|e| format!("网络错误: {e}"))?;
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status.as_u16(), text.chars().take(200).collect::<String>()));
        }
        serde_json::from_str(&text).map_err(|e| format!("JSON 解析失败: {e}"))
    }

    pub fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{API_BASE}{path}");
        let resp = self
            .client
            .post(&url)
            .header("x-api-key", API_KEY)
            .header("Cookie", format!("auth={}", self.cookie))
            .json(body)
            .send()
            .map_err(|e| format!("网络错误: {e}"))?;
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status.as_u16(), text.chars().take(200).collect::<String>()));
        }
        serde_json::from_str(&text).map_err(|e| format!("JSON 解析失败: {e}"))
    }

    /// GET /auth/user — 验证 cookie
    pub fn current_user(&self) -> Result<User, String> {
        let v = self.get("/auth/user")?;
        serde_json::from_value(v).map_err(|e| format!("解析用户失败: {e}"))
    }

    /// 当前用户 owned 群组（id 规范化为 grp_xxx）
    pub fn owned_groups(&self, uid: &str) -> Result<Vec<serde_json::Value>, String> {
        let v = self.get(&format!("/users/{uid}/groups"))?;
        let arr = v.as_array().cloned().unwrap_or_default();
        let mut out = Vec::new();
        for mut g in arr {
            let gid = g.get("groupId").and_then(|x| x.as_str())
                .or_else(|| g.get("id").and_then(|x| x.as_str()))
                .unwrap_or("").to_string();
            let owner = g.get("ownerId").and_then(|x| x.as_str()).unwrap_or("");
            if owner == uid && !gid.is_empty() {
                g["id"] = serde_json::Value::String(gid);
                out.push(g);
            }
        }
        Ok(out)
    }

    /// 收藏夹组目录（world 类 + vrcPlusWorld）
    pub fn favorite_groups(&self, group_type: &str) -> Result<Vec<FavoriteGroup>, String> {
        let mut out = Vec::new();
        let v = self.get("/favorite/groups?n=100&offset=0")?;
        if let Some(arr) = v.as_array() {
            for g in arr {
                let t = g.get("type").and_then(|x| x.as_str()).unwrap_or("");
                let wanted = match group_type {
                    "all" => true,
                    "world" => t == "world" || t == "vrcPlusWorld",
                    _ => t == group_type,
                };
                if !wanted {
                    continue;
                }
                if let Ok(fg) = serde_json::from_value::<FavoriteGroup>(g.clone()) {
                    out.push(fg);
                }
            }
        }
        Ok(out)
    }

    /// 全部收藏世界详情（翻页 offset 按 100 块 + (id,group) 去重）
    pub fn favorite_worlds(&self) -> Result<Vec<FavoriteWorld>, String> {
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut offset = 0u32;
        loop {
            let v = self.get(&format!("/worlds/favorites?n=100&offset={offset}"))?;
            let arr = match v.as_array() {
                Some(a) if !a.is_empty() => a.clone(),
                _ => break,
            };
            let mut new = 0;
            for w in &arr {
                let id = w.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
                let grp = w.get("favoriteGroup").and_then(|x| x.as_str()).unwrap_or("").to_string();
                if id.is_empty() || !seen.insert((id.clone(), grp.clone())) {
                    continue;
                }
                new += 1;
                if let Ok(fw) = serde_json::from_value::<FavoriteWorld>(w.clone()) {
                    out.push(fw);
                }
            }
            offset += 100;
            if new == 0 || offset > 5000 {
                break;
            }
        }
        Ok(out)
    }

    /// 建房：POST /instances
    pub fn create_instance(
        &self,
        world_id: &str,
        group_id: Option<&str>,
        region: &str,
        access: &str,      // public / plus / members（group 房）
        queue: bool,
        age_gate: bool,
        instance_type: &str, // group / public / friends / private
    ) -> Result<InstanceResult, String> {
        let mut body = serde_json::json!({
            "worldId": world_id,
            "type": instance_type,
            "region": region,
        });
        if instance_type == "group" {
            if let Some(g) = group_id {
                body["ownerId"] = serde_json::Value::String(g.to_string());
                body["groupAccessType"] = serde_json::Value::String(access.to_string());
            }
        }
        if queue {
            body["queueEnabled"] = serde_json::Value::Bool(true);
        }
        if age_gate {
            body["ageGate"] = serde_json::Value::Bool(true);
        }
        let v = self.post("/instances", &body)?;
        serde_json::from_value(v).map_err(|e| format!("解析建房结果失败: {e}"))
    }
}
