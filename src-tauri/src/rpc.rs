//! RPC 服务层（axum）—— service 模式对外提供
//! 方法中心 + HTTP/JSON。认证：带 session 的挑战应答（可开关 off）。
//!
//! 端点：
//!   GET  /health              → 存活探针（无鉴权）
//!   GET  /rpc/methods         → 可用方法清单
//!   POST /rpc/auth/challenge  → {challenge}（60s 过期）
//!   POST /rpc/auth/verify     → {challenge,response} → {session_token, expires_at}（24h）
//!   POST /rpc                 → 头 X-VRCN-Session + {method,params} → {ok,data}|{ok,error}

use crate::commands::{CreateInstanceArgs, FavGroup, FavWorld};
use crate::state::AppState;
use crate::vrchat;
use axum::extract::State as AxState;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

const CHALLENGE_TTL: Duration = Duration::from_secs(60);
const SESSION_TTL: Duration = Duration::from_secs(24 * 3600);

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn gen_hex(len: usize) -> String {
    use rand::RngCore;
    let mut b = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut b);
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    let out = h.finalize();
    out.iter().map(|x| format!("{x:02x}")).collect()
}

#[derive(Clone)]
struct RpcState {
    auth: AuthState,
    app: Arc<AppState>,
}

#[derive(Clone)]
struct AuthState {
    challenges: Arc<RwLock<HashMap<String, u64>>>, // challenge -> expires_at
    sessions: Arc<RwLock<HashMap<String, u64>>>,   // session_token -> expires_at
    token: String,
    enabled: bool,
    counter: Arc<AtomicU64>,
}

impl AuthState {
    fn new(token: String, enabled: bool) -> Self {
        Self {
            challenges: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            token,
            enabled,
            counter: Arc::new(AtomicU64::new(0)),
        }
    }

    async fn issue_challenge(&self) -> String {
        let c = format!("{}{}", gen_hex(16), self.counter.fetch_add(1, Ordering::Relaxed));
        self.challenges.write().await.insert(c.clone(), now() + CHALLENGE_TTL.as_secs());
        c
    }

    async fn verify(&self, challenge: &str, response: &str) -> Option<String> {
        let mut ch = self.challenges.write().await;
        match ch.remove(challenge) {
            Some(exp) if exp >= now() => {
                let expected = sha256_hex(&format!("{}{}", self.token, challenge));
                if response == expected {
                    let sess = gen_hex(32);
                    self.sessions.write().await.insert(sess.clone(), now() + SESSION_TTL.as_secs());
                    Some(sess)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    async fn check_session(&self, headers: &HeaderMap) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }
        let token: String = headers
            .get("x-vrcn-session")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string())
            .ok_or_else(|| "401 auth failed: missing X-VRCN-Session".to_string())?;
        let mut sess = self.sessions.write().await;
        match sess.get(&token).copied() {
            Some(exp) if exp >= now() => Ok(()),
            Some(_) => {
                sess.remove(&token);
                Err("401 session expired".into())
            }
            None => Err("401 auth failed: invalid session".into()),
        }
    }

    async fn revoke_session(&self, headers: &HeaderMap) {
        if let Some(t) = headers.get("x-vrcn-session").and_then(|v| v.to_str().ok()) {
            self.sessions.write().await.remove(t);
        }
    }
}

/// 方法名 → 处理器。处理器接收 AppState + 平铺 params → serde_json::Value
type Handler = fn(&AppState, serde_json::Value) -> Result<serde_json::Value, String>;

fn method_table() -> Vec<(&'static str, Handler)> {
    vec![
        ("auth.status", |st, _p| {
            match st.authenticate() {
                Ok(u) => Ok(serde_json::json!({
                    "ok": true,
                    "user": {"id": u.id, "displayName": u.display_name},
                    "auth_source": "vrcx",
                })),
                Err(e) => Ok(serde_json::json!({"ok": false, "error": e})),
            }
        }),
        ("auth.logout", |st, _p| {
            st.logout();
            Ok(serde_json::json!({"ok": true}))
        }),
        ("groups.list", |st, _p| {
            let api = st.api()?;
            let uid = st.user().ok_or_else(|| "未认证".to_string())?.id;
            Ok(serde_json::Value::Array(api.owned_groups(&uid)?))
        }),
        ("favorites.groups", |st, p| {
            let gt = p.get("group_type").and_then(|v| v.as_str()).unwrap_or("world").to_string();
            let api = st.api()?;
            let groups = api.favorite_groups(&gt)?;
            let worlds = api.favorite_worlds()?;
            let mut out: Vec<serde_json::Value> = Vec::new();
            for g in groups {
                let count = worlds.iter().filter(|w| w.favorite_group == g.name).count();
                out.push(serde_json::json!({
                    "tag": g.name,
                    "display_name": if g.display_name.is_empty() { g.name } else { g.display_name },
                    "group_type": g.group_type,
                    "visibility": g.visibility,
                    "count": count,
                }));
            }
            Ok(serde_json::Value::Array(out))
        }),
        ("favorites.worlds", |st, p| {
            let tag = p.get("group").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
            let keyword = p.get("q").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
            let limit = p.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
            let offset = p.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let api = st.api()?;
            let worlds = api.favorite_worlds()?;
            let mut items: Vec<serde_json::Value> = Vec::new();
            for w in worlds {
                if !tag.is_empty() && w.favorite_group.to_lowercase() != tag {
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
            Ok(serde_json::json!({ "total": total, "items": paged }))
        }),
        ("instance.create", |st, p| {
            let args = CreateInstanceArgs {
                world: p.get("world").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                group: p.get("group").and_then(|v| v.as_str()).map(|s| s.to_string()),
                region: p.get("region").and_then(|v| v.as_str()).map(|s| s.to_string()),
                access: p.get("access").and_then(|v| v.as_str()).map(|s| s.to_string()),
                queue: p.get("queue").and_then(|v| v.as_bool()),
                age_gate: p.get("age_gate").and_then(|v| v.as_bool()),
                instance_type: p.get("instance_type").and_then(|v| v.as_str()).map(|s| s.to_string()),
            };
            if args.world.is_empty() {
                return Err("instance.create 缺少 world".into());
            }
            let api = st.api()?;
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
        }),
        ("world.resolve", |st, p| {
            let token = p.get("token").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if token.starts_with("wrld_") {
                let api = st.api()?;
                return api.get(&format!("/worlds/{token}"));
            }
            let api = st.api()?;
            let v = api.get(&format!("/worlds?search={}&n=5", urlencoding::encode(&token)))?;
            if let Some(arr) = v.as_array() {
                if let Some(first) = arr.first() {
                    return Ok(first.clone());
                }
            }
            Err(format!("找不到世界 {token}"))
        }),
        ("chatbox.send", |_st, p| {
            let text = p.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let port = p.get("port").and_then(|v| v.as_u64()).map(|x| x as u16).unwrap_or(9000);
            let notify = p.get("notify").and_then(|v| v.as_bool()).unwrap_or(false);
            crate::osc::send_chatbox(&text, port, notify)?;
            Ok(serde_json::json!({"ok": true, "to": format!("osc:{port}")}))
        }),
    ]
}

fn json_err(status: StatusCode, msg: &str) -> Response {
    (status, Json(serde_json::json!({ "ok": false, "error": msg }))).into_response()
}

async fn handle_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn handle_methods() -> Json<serde_json::Value> {
    let names: Vec<String> = method_table().into_iter().map(|(n, _)| n.to_string()).collect();
    Json(serde_json::json!(names))
}

async fn handle_challenge(AxState(st): AxState<RpcState>) -> Response {
    let auth = &st.auth;
    if !auth.enabled {
        return json_err(StatusCode::BAD_REQUEST, "认证已关闭(off)，无需握手");
    }
    let c = auth.issue_challenge().await;
    (StatusCode::OK, Json(serde_json::json!({ "challenge": c }))).into_response()
}

#[derive(serde::Deserialize)]
struct VerifyBody {
    challenge: String,
    response: String,
}

async fn handle_verify(
    AxState(st): AxState<RpcState>,
    Json(body): Json<VerifyBody>,
) -> Response {
    let auth = &st.auth;
    if !auth.enabled {
        return json_err(StatusCode::BAD_REQUEST, "认证已关闭(off)，无需握手");
    }
    match auth.verify(&body.challenge, &body.response).await {
        Some(sess) => {
            let exp = now() + SESSION_TTL.as_secs();
            (StatusCode::OK, Json(serde_json::json!({ "session_token": sess, "expires_at": exp }))).into_response()
        }
        None => json_err(StatusCode::UNAUTHORIZED, "401 auth failed"),
    }
}

#[derive(serde::Deserialize)]
struct RpcBody {
    method: String,
    #[serde(default)]
    params: serde_json::Value,
}

async fn handle_rpc(
    AxState(st): AxState<RpcState>,
    headers: HeaderMap,
    Json(body): Json<RpcBody>,
) -> Response {
    let auth = &st.auth;
    let state = &st.app;
    if let Err(e) = auth.check_session(&headers).await {
        return json_err(StatusCode::UNAUTHORIZED, &e);
    }
    // logout 特判：先吊销 session
    if body.method == "auth.logout" {
        auth.revoke_session(&headers).await;
    }
    let params = if body.params.is_null() { serde_json::json!({}) } else { body.params };
    let table = method_table();
    for (name, handler) in table {
        if name == body.method {
            let resp = match handler(&state, params) {
                Ok(data) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "data": data }))).into_response(),
                Err(e) => {
                    if e.starts_with("401") {
                        json_err(StatusCode::UNAUTHORIZED, &e)
                    } else {
                        json_err(StatusCode::INTERNAL_SERVER_ERROR, &e)
                    }
                }
            };
            return resp;
        }
    }
    json_err(StatusCode::BAD_REQUEST, &format!("400 unknown method: {}", body.method))
}

/// 启动 RPC 服务（阻塞当前线程直到 server 退出）。bind 非法 / off+外网 时返回 Err。
pub fn serve(state: Arc<AppState>) -> Result<(), String> {
    let svc = state.config.service();
    if !svc.enabled {
        return Ok(()); // 未启用，静默返回
    }
    let auth_on = svc.auth != "off";
    if !auth_on && !(svc.bind == "127.0.0.1" || svc.bind == "localhost" || svc.bind == "::1") {
        return Err(format!("认证已关闭(off)但 bind 为 {}，拒绝启动：off 仅限本机", svc.bind));
    }
    if svc.token.is_empty() {
        return Err("service token 为空，无法启动 RPC（请配置 VRCN_SERVICE_TOKEN 或配置文件 token）".into());
    }

    let shared = RpcState { auth: AuthState::new(svc.token.clone(), auth_on), app: Arc::clone(&state) };
    let app = Router::new()
        .route("/health", get(handle_health))
        .route("/rpc/methods", get(handle_methods))
        .route("/rpc/auth/challenge", post(handle_challenge))
        .route("/rpc/auth/verify", post(handle_verify))
        .route("/rpc", post(handle_rpc))
        .with_state(shared);

    let addr: SocketAddr = format!("{}:{}", svc.bind, svc.port)
        .parse()
        .map_err(|e| format!("RPC 监听地址非法 {}:{}: {e}", svc.bind, svc.port))?;
    log::info!("VRCNexus RPC 服务启动: http://{addr} (auth={})", svc.auth);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("创建 tokio runtime 失败: {e}"))?;
    rt.block_on(async {
        axum::serve(
            tokio::net::TcpListener::bind(addr).await.map_err(|e| format!("bind {addr} 失败: {e}"))?,
            app,
        )
        .await
        .map_err(|e| format!("RPC server 运行失败: {e}"))
    })
}

// 引用 FavGroup/FavWorld 避免未使用告警（结构体由命令层序列化使用；此处仅为编译期可见性占位）
#[allow(dead_code)]
fn _type_anchor(_: FavGroup, _: FavWorld, _: &vrchat::User) {}
