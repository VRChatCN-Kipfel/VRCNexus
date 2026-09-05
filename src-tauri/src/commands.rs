//! Tauri commands：前端 invoke 的入口
//! 命名与 Python 版 /api/* 对应，前端迁移平滑

use crate::state::AppState;
use crate::vrchat;
use serde::Serialize;
use std::sync::Arc;
use tauri::State;

// ---------- 认证 ----------

#[tauri::command]
pub fn auth_status(state: State<Arc<AppState>>) -> Result<serde_json::Value, String> {
    match state.authenticate() {
        Ok(u) => Ok(serde_json::json!({
            "ok": true,
            "user": {"id": u.id, "displayName": u.display_name},
            "auth_source": "vrcx",
        })),
        Err(e) => Ok(serde_json::json!({"ok": false, "error": e})),
    }
}

#[tauri::command]
pub fn auth_logout(state: State<Arc<AppState>>) {
    state.logout();
}

// ---------- 群组 ----------

#[tauri::command]
pub fn list_groups(state: State<Arc<AppState>>) -> Result<Vec<serde_json::Value>, String> {
    let api = state.api()?;
    let uid = state
        .user()
        .ok_or_else(|| "未认证".to_string())?
        .id;
    api.owned_groups(&uid)
}

// ---------- 收藏 ----------

#[derive(Serialize)]
pub struct FavGroup {
    pub tag: String,
    pub display_name: String,
    pub group_type: String,
    pub visibility: String,
    pub count: usize,
}

#[derive(Serialize)]
pub struct FavWorld {
    pub id: String,
    pub name: String,
    pub author_name: String,
    pub thumbnail_image_url: String,
    pub group_tag: String,
}

/// 收藏夹目录（轻量）。group_type: world(默认)/all/avatar/friend
#[tauri::command]
pub fn favorites_groups(
    state: State<Arc<AppState>>,
    group_type: Option<String>,
) -> Result<Vec<FavGroup>, String> {
    let api = state.api()?;
    let gt = group_type.unwrap_or_else(|| "world".into());
    let groups = api.favorite_groups(&gt)?;
    let worlds = api.favorite_worlds()?;
    let mut out = Vec::new();
    for g in groups {
        let count = worlds.iter().filter(|w| w.favorite_group == g.name).count();
        out.push(FavGroup {
            tag: g.name.clone(),
            display_name: if g.display_name.is_empty() { g.name.clone() } else { g.display_name },
            group_type: g.group_type.clone(),
            visibility: g.visibility.clone(),
            count,
        });
    }
    Ok(out)
}

/// 某收藏夹的世界列表（内存过滤 + 分页）。group 传 tag；q 关键词；limit/offset 分页
#[tauri::command]
pub fn favorites_worlds(
    state: State<Arc<AppState>>,
    group: Option<String>,
    q: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<serde_json::Value, String> {
    let api = state.api()?;
    let worlds = api.favorite_worlds()?;
    let tag = group.unwrap_or_default();
    let keyword = q.unwrap_or_default().to_lowercase();
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let mut items: Vec<serde_json::Value> = Vec::new();
    for w in worlds {
        if !tag.is_empty() && w.favorite_group != tag {
            continue;
        }
        if !keyword.is_empty()
            && !w.name.to_lowercase().contains(&keyword)
            && !w.author_name.to_lowercase().contains(&keyword)
        {
            continue;
        }
        items.push(serde_json::json!({
            "id": w.id,
            "name": w.name,
            "authorName": w.author_name,
            "thumbnailImageUrl": w.thumbnail_image_url,
            "favoriteGroup": w.favorite_group,
        }));
    }
    let total = items.len();
    let paged = items.into_iter().skip(offset).take(limit).collect::<Vec<_>>();
    Ok(serde_json::json!({"total": total, "items": paged}))
}

// ---------- 建房 ----------

#[derive(serde::Deserialize)]
pub struct CreateInstanceArgs {
    pub world: String,        // wrld_xxx
    pub group: Option<String>, // grp_xxx（group 房必填）
    pub region: Option<String>, // jp/usw/use/eu
    pub access: Option<String>, // public/plus/members
    pub queue: Option<bool>,
    pub age_gate: Option<bool>,
    pub instance_type: Option<String>, // group/public/friends/private
}

/// 建房（同步快速失败）。返回 instance 信息
#[tauri::command]
pub fn create_instance(
    state: State<Arc<AppState>>,
    args: CreateInstanceArgs,
) -> Result<serde_json::Value, String> {
    let api = state.api()?;
    let region = args.region.unwrap_or_else(|| "jp".into());
    let access = args.access.unwrap_or_else(|| "public".into());
    let itype = args.instance_type.unwrap_or_else(|| "group".into());
    let queue = args.queue.unwrap_or(true);
    let age_gate = args.age_gate.unwrap_or(false);
    let inst = api.create_instance(
        &args.world,
        args.group.as_deref(),
        &region,
        &access,
        queue,
        age_gate,
        &itype,
    )?;
    Ok(serde_json::json!({
        "instanceId": inst.id,
        "worldId": inst.world.as_ref().map(|w| w.id.clone()).or(inst.world_id),
    }))
}

// ---------- 世界解析（模糊找 ID）----------

#[tauri::command]
pub fn resolve_world(state: State<Arc<AppState>>, token: String) -> Result<serde_json::Value, String> {
    if token.starts_with("wrld_") {
        let api = state.api()?;
        let v = api.get(&format!("/worlds/{token}"))?;
        return Ok(v);
    }
    // 搜索兜底
    let api = state.api()?;
    let v = api.get(&format!("/worlds?search={}&n=5", urlencoding::encode(&token)))?;
    if let Some(arr) = v.as_array() {
        if let Some(first) = arr.first() {
            return Ok(first.clone());
        }
    }
    Err(format!("找不到世界 {token}"))
}

// ---------- 设置（三模式配置）----------

/// 读取配置生效值视图（含来源 default/file/env）
#[tauri::command]
pub fn settings_get(state: State<Arc<AppState>>) -> serde_json::Value {
    state.config.view()
}

/// 写配置（merge patch），返回新视图
#[tauri::command]
pub fn settings_set(
    state: State<Arc<AppState>>,
    patch: serde_json::Value,
) -> Result<serde_json::Value, String> {
    state.config.apply_patch(patch)
}

/// 当前模式（供前端 api.js 分派；local/service 都直调本机，仅 remote 走 RPC）
#[tauri::command]
pub fn app_mode(state: State<Arc<AppState>>) -> String {
    state.config.mode()
}

// ---------- OSC 聊天 ----------

#[derive(serde::Deserialize)]
pub struct ChatboxArgs {
    pub text: String,
    pub port: Option<u16>, // 目标 OSC in 端口，默认 9000
    pub notify: Option<bool>,
}

/// OSC /chatbox/input 推送
#[tauri::command]
pub fn send_chatbox(args: ChatboxArgs) -> Result<serde_json::Value, String> {
    let port = args.port.unwrap_or(9000);
    let notify = args.notify.unwrap_or(false);
    crate::osc::send_chatbox(&args.text, port, notify)?;
    Ok(serde_json::json!({"ok": true, "to": format!("osc:{port}")}))
}
