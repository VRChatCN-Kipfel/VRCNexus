// VRCNexus 前端统一 API 调用层
// call(method, params):
//   - mode==='remote' → HTTP RPC（带 session 的挑战应答握手）
//   - 否则 → invoke(method, params) 原样（兼容现状）
//
// method 映射：tauri command 名(snake_case) ⇄ RPC 点分名
//   命令侧直接透传 params 对象给 invoke；RPC 侧平铺参数。
// 本文件不含 UI 状态，供各 view 调用。

import { invoke } from '@tauri-apps/api/core'

// tauri command 名 → RPC 点分 method 名（未列出的保持原名走 invoke）
const RPC_METHOD_MAP = {
  auth_status: 'auth.status',
  auth_logout: 'auth.logout',
  list_groups: 'groups.list',
  favorites_groups: 'favorites.groups',
  favorites_worlds: 'favorites.worlds',
  create_instance: 'instance.create',
  resolve_world: 'world.resolve',
  send_chatbox: 'chatbox.send',
}

// params 适配：invoke 参数名 snake_case 嵌套 args → RPC 平铺
// create_instance 在 invoke 侧是 { args: {...} }，RPC 侧平铺
function adaptParams(method, params) {
  if (method === 'create_instance' && params && params.args) {
    return params.args
  }
  return params || {}
}

let modeCache = null // 'local' | 'remote' | 'service' | null(未知)
let rpcConf = null   // { server_url, token }
let session = null   // { token, expires_at } | null

export async function getMode(force) {
  if (modeCache && !force) return modeCache
  try {
    const s = await invoke('settings_get')
    // settings_get 返回 { mode, values: { mode:{value,source}, remote_server_url:{value}, ... } }
    const mode = s?.mode || s?.values?.mode?.value || 'local'
    const conf = {
      server_url: s?.values?.remote_server_url?.value || 'http://127.0.0.1:4455',
      token: s?.values?.remote_token?.value || '',
    }
    modeCache = mode
    rpcConf = conf
    return mode
  } catch (e) {
    // settings_get 未就绪（后端还没实现）→ 回落 local，UI 不白屏
    console.warn('[api] settings_get 失败，回落 local:', e)
    modeCache = 'local'
    rpcConf = { server_url: 'http://127.0.0.1:4455', token: '' }
    return 'local'
  }
}

export function getRpcConf() {
  return rpcConf
}

export function getSession() {
  return session
}

// ---- remote 握手 ----

async function sha256Hex(str) {
  const buf = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(str))
  return Array.from(new Uint8Array(buf)).map((b) => b.toString(16).padStart(2, '0')).join('')
}

async function rpcFetch(path, body, extraHeaders) {
  const conf = rpcConf || { server_url: 'http://127.0.0.1:4455', token: '' }
  const url = conf.server_url.replace(/\/$/, '') + path
  const headers = { 'Content-Type': 'application/json', ...(extraHeaders || {}) }
  const res = await fetch(url, { method: 'POST', headers, body: JSON.stringify(body) })
  let data = null
  try { data = await res.json() } catch { /* 非 JSON */ }
  if (!res.ok) {
    const err = new Error(data?.error || `HTTP ${res.status}`)
    err.status = res.status
    err.data = data
    throw err
  }
  return data
}

// 挑战应答登录，返回 session_token
async function handshake() {
  const conf = rpcConf || { server_url: 'http://127.0.0.1:4455', token: '' }
  if (!conf.token) throw new Error('remote 模式缺少 token（请在设置中配置，或设 VRCN_REMOTE_TOKEN）')

  const { challenge } = await rpcFetch('/rpc/auth/challenge', {})
  const response = await sha256Hex(conf.token + challenge)
  const { session_token, expires_at } = await rpcFetch('/rpc/auth/verify', { challenge, response })
  session = { token: session_token, expires_at: expires_at * 1000 }
  return session
}

async function ensureSession() {
  if (session && session.expires_at > Date.now() + 60000) return session
  return handshake()
}

async function rpcCall(method, params) {
  await ensureSession()
  try {
    const data = await rpcFetch('/rpc', { method, params }, { 'X-VRCN-Session': session.token })
    if (data && data.ok === false) {
      const e = new Error(data.error || 'RPC 调用失败')
      e.data = data
      throw e
    }
    return data?.data
  } catch (e) {
    // session 过期 → 重握手一次再试
    if (e.status === 401 || (e.data && e.data.error && e.data.error.includes('session'))) {
      session = null
      await handshake()
      const data = await rpcFetch('/rpc', { method, params }, { 'X-VRCN-Session': session.token })
      if (data && data.ok === false) throw new Error(data.error || 'RPC 调用失败')
      return data?.data
    }
    throw e
  }
}

// ---- 统一入口 ----

/**
 * 统一调用：本地走 invoke，远程走 RPC
 * @param {string} method  tauri command 名（snake_case），内部自动映射 RPC 点分名
 * @param {object} params  invoke 侧参数对象（保持现状）；RPC 侧自动适配
 */
export async function call(method, params) {
  const mode = await getMode()
  if (mode === 'remote') {
    const rpcMethod = RPC_METHOD_MAP[method] || method.replace(/_/g, '.')
    return rpcCall(rpcMethod, adaptParams(method, params))
  }
  // local / service：直调本机 command（service 模式下 GUI 也用 invoke）
  return invoke(method, params || {})
}

/** 强制刷新模式缓存（设置保存后调用） */
export function invalidateMode() {
  modeCache = null
  rpcConf = null
  session = null
}

export default { call, getMode, invalidateMode, getSession }
