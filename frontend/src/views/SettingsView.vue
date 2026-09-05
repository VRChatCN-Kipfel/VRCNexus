<script setup>
import { ref, reactive, onMounted, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { invalidateMode } from '../lib/api'
import GlassSelect from '../components/GlassSelect.vue'

const emit = defineEmits(['close'])

const section = ref('general') // general | remote | service | about
const loading = ref(true)
const saving = ref(false)
const error = ref('')
const notice = ref('')

// 原始视图（含 source）
const rawView = ref(null)

// 编辑态（仅 file 层可改；env 只读）
const form = reactive({
  mode: 'local',
  remoteServerUrl: '',
  remoteToken: '',
  serviceEnabled: true,
  serviceBind: '127.0.0.1',
  servicePort: 4455,
  serviceToken: '',
  serviceAuth: 'challenge',
})

const SOURCE_LABEL = { default: '默认', file: '配置文件', env: '环境变量', generated: '运行时生成' }
const SOURCE_BADGE = { default: 'dim', file: 'file', env: 'env', generated: 'gen' }

// 认证方式下拉选项
const authOpts = [
  { value: 'challenge', label: 'challenge（挑战应答，推荐）' },
  { value: 'off', label: 'off（关闭认证，仅限本机）' },
]

function srcOf(key) {
  return rawView.value?.values?.[key]?.source || 'default'
}
function isEnv(key) {
  return srcOf(key) === 'env'
}
function valOf(key) {
  return rawView.value?.values?.[key]?.value
}

function loadView(v) {
  rawView.value = v
  form.mode = v?.mode || 'local'
  form.remoteServerUrl = v?.values?.remote_server_url?.value || ''
  form.remoteToken = v?.values?.remote_token?.value || ''
  form.serviceEnabled = v?.values?.service_enabled?.value === 'true' || v?.values?.service_enabled?.value === true
  form.serviceBind = v?.values?.service_bind?.value || '127.0.0.1'
  form.servicePort = Number(v?.values?.service_port?.value || 4455)
  form.serviceToken = v?.values?.service_token?.value || ''
  form.serviceAuth = v?.values?.service_auth?.value || 'challenge'
}

async function refresh() {
  loading.value = true
  error.value = ''
  try {
    const v = await invoke('settings_get')
    loadView(v)
  } catch (e) {
    error.value = `读取设置失败：${e}`
  } finally {
    loading.value = false
  }
}

function genToken() {
  const arr = new Uint8Array(16)
  crypto.getRandomValues(arr)
  form.serviceToken = Array.from(arr).map((b) => b.toString(16).padStart(2, '0')).join('')
}

const isLoopbackBind = computed(() => {
  const b = form.serviceBind.trim()
  return b === '127.0.0.1' || b === 'localhost' || b === '::1'
})

const authOffInvalid = computed(() => form.serviceAuth === 'off' && !isLoopbackBind.value)

async function save() {
  error.value = ''
  notice.value = ''
  if (authOffInvalid.value) {
    error.value = '关闭认证(off)仅限本机地址(127.0.0.1/localhost)；外部地址必须启用 challenge 认证'
    return
  }
  if (!['local', 'remote', 'service'].includes(form.mode)) {
    error.value = '未知模式'
    return
  }
  saving.value = true
  try {
    const patch = {
      mode: form.mode,
      remote: {
        server_url: form.remoteServerUrl.trim() || null,
        token: form.remoteToken.trim() || null,
      },
      service: {
        enabled: !!form.serviceEnabled,
        bind: form.serviceBind.trim(),
        port: Number(form.servicePort) || 4455,
        token: form.serviceToken.trim() || null,
        auth: form.serviceAuth,
      },
    }
    const v = await invoke('settings_set', { patch })
    loadView(v)
    invalidateMode() // 清前端模式缓存，下次 call 重新读
    notice.value = '已保存。部分配置（模式切换/服务）重启后生效。'
  } catch (e) {
    error.value = `保存失败：${e}`
  } finally {
    saving.value = false
  }
}

async function resetDefault() {
  // 恢复默认：清空 file 层字段 → 写一个"空 patch 但显式置 null 的语义"暂不支持；
  // 退而求其次：把各字段写回默认值（env 仍优先覆盖）
  error.value = ''
  const def = {
    mode: 'local',
    remote: { server_url: 'http://127.0.0.1:4455', token: '' },
    service: { enabled: true, bind: '127.0.0.1', port: 4455, token: '', auth: 'challenge' },
  }
  saving.value = true
  try {
    const patch = {
      mode: def.mode,
      remote: { server_url: def.remote.server_url, token: null },
      service: { enabled: true, bind: def.service.bind, port: 4455, token: null, auth: 'challenge' },
    }
    const v = await invoke('settings_set', { patch })
    loadView(v)
    invalidateMode()
    notice.value = '已恢复默认。环境变量覆盖仍生效。'
  } catch (e) {
    error.value = `恢复默认失败：${e}`
  } finally {
    saving.value = false
  }
}

onMounted(refresh)
</script>

<template>
  <Transition name="modal">
    <div class="modal-mask" @click.self="emit('close')">
      <div class="modal glass">
      <header class="m-head">
        <h2>⚙ 设置</h2>
        <button class="m-close" @click="emit('close')">✕</button>
      </header>

      <div class="m-body">
        <!-- 左：小节 -->
        <nav class="m-nav">
          <button :class="['m-nav-item', { on: section === 'general' }]" @click="section = 'general'">通用</button>
          <button :class="['m-nav-item', { on: section === 'remote' }]" @click="section = 'remote'">远程连接</button>
          <button :class="['m-nav-item', { on: section === 'service' }]" @click="section = 'service'">服务</button>
          <button :class="['m-nav-item', { on: section === 'about' }]" @click="section = 'about'">关于</button>
        </nav>

        <!-- 右：表单 -->
        <div class="m-form">
          <div v-if="loading" class="text-dim" style="padding:20px">加载中…</div>
          <div v-else>
            <!-- 通用 -->
            <div v-if="section === 'general'" class="f-sec">
              <div class="f-title">运行模式</div>
              <p class="text-dim f-hint">模式 = 数据源/连接形态。本地=本机直驱；服务=本机 Core + 对外 RPC；远程=GUI 连远端服务。</p>
              <label v-for="(m, key) in { local: '本地模式', remote: '远程模式', service: '服务模式' }" :key="key" class="radio-row">
                <input type="radio" :value="key" v-model="form.mode" :disabled="isEnv('mode')" />
                <span>{{ m }}</span>
                <em v-if="isEnv('mode')" class="badge env">环境变量</em>
              </label>
              <div v-if="isEnv('mode')" class="text-dim f-note">当前模式由环境变量 VRCN_MODE 控制，此处只读。</div>
            </div>

            <!-- 远程连接 -->
            <div v-else-if="section === 'remote'" class="f-sec">
              <div class="f-title">远程连接（remote 模式用）</div>
              <div class="field">
                <label>服务地址 server_url</label>
                <input v-model="form.remoteServerUrl" class="g-input mono" placeholder="http://127.0.0.1:4455" :disabled="isEnv('remote_server_url')" />
                <em v-if="isEnv('remote_server_url')" class="badge env">环境变量</em>
              </div>
              <div class="field">
                <label>接入 token</label>
                <input v-model="form.remoteToken" class="g-input mono" placeholder="远端 service 的 token（建议走环境变量）" :disabled="isEnv('remote_token')" />
                <em v-if="isEnv('remote_token')" class="badge env">环境变量</em>
              </div>
              <p class="text-dim f-note">token 不会出现在网络请求中——通过挑战应答握手换取短时 session。</p>
            </div>

            <!-- 服务 -->
            <div v-else-if="section === 'service'" class="f-sec">
              <div class="f-title">RPC 服务（service 模式对外提供）</div>
              <label class="radio-row">
                <input type="checkbox" v-model="form.serviceEnabled" :disabled="isEnv('service_enabled')" />
                <span>启动时挂载 RPC 服务</span>
                <em v-if="isEnv('service_enabled')" class="badge env">环境变量</em>
              </label>
              <div class="grid2">
                <div class="field">
                  <label>监听地址 bind</label>
                  <input v-model="form.serviceBind" class="g-input mono" placeholder="127.0.0.1" :disabled="isEnv('service_bind')" />
                  <em v-if="isEnv('service_bind')" class="badge env">环境变量</em>
                </div>
                <div class="field">
                  <label>端口 port</label>
                  <input v-model.number="form.servicePort" type="number" class="g-input mono" :disabled="isEnv('service_port')" />
                  <em v-if="isEnv('service_port')" class="badge env">环境变量</em>
                </div>
              </div>
              <div class="field">
                <label>认证方式 auth</label>
                <GlassSelect v-model="form.serviceAuth" :options="authOpts" :disabled="isEnv('service_auth')" style="width:100%" />
                <em v-if="isEnv('service_auth')" class="badge env">环境变量</em>
                <div v-if="authOffInvalid" class="msg bad">关闭认证仅限本机地址——请把 bind 改回 127.0.0.1/localhost 或改用 challenge。</div>
              </div>
              <div class="field">
                <label>接入 token</label>
                <div class="row">
                  <input v-model="form.serviceToken" class="g-input mono grow" placeholder="留空则运行时自动生成" :disabled="isEnv('service_token')" />
                  <button v-if="!isEnv('service_token')" class="g-btn" @click="genToken">🎲 生成</button>
                </div>
                <em v-if="isEnv('service_token')" class="badge env">环境变量</em>
                <em v-else-if="srcOf('service_token') === 'generated'" class="badge gen">运行时生成</em>
                <p class="text-dim f-note">token 永不上线：仅以 SHA256(token+challenge) 参与握手。</p>
              </div>
            </div>

            <!-- 关于 -->
            <div v-else-if="section === 'about'" class="f-sec">
              <div class="f-title">关于</div>
              <div class="kv"><span>版本</span><code>v0.1.0</code></div>
              <div class="kv"><span>配置路径</span><code class="mono small">{{ rawView?.config_path || '—' }}</code></div>
              <div class="kv"><span>当前模式</span><code>{{ rawView?.mode || 'local' }}</code></div>
              <div v-if="rawView?.env_keys?.length" class="kv">
                <span>生效的环境变量</span>
                <code class="small">{{ rawView.env_keys.join(', ') }}</code>
              </div>
              <p class="text-dim f-note">优先级：默认 &lt; 配置文件 &lt; 环境变量(VRCN_*)</p>
            </div>

            <Transition name="fade">
              <div v-if="error" class="msg bad">{{ error }}</div>
            </Transition>
            <Transition name="fade">
              <div v-if="notice" class="msg ok">{{ notice }}</div>
            </Transition>
          </div>
        </div>
      </div>

      <footer class="m-foot">
        <button class="g-btn" :disabled="saving || loading" @click="resetDefault">恢复默认</button>
        <div class="spacer"></div>
        <button class="g-btn" :disabled="saving || loading" @click="emit('close')">取消</button>
        <button class="g-btn primary" :disabled="saving || loading || authOffInvalid" @click="save">
          {{ saving ? '保存中…' : '保存' }}
        </button>
      </footer>
    </div>
    </div>
    </Transition>
</template>

<style scoped>
.modal-mask {
  position: fixed; inset: 0; z-index: 100;
  background: rgba(10, 12, 20, .6);
  backdrop-filter: blur(6px);
  display: flex; align-items: center; justify-content: center;
}
@keyframes fadeIn { from { opacity: 0 } }
.modal {
  width: min(680px, 92vw); max-height: 86vh;
  display: flex; flex-direction: column;
  border-radius: 18px; padding: 0; overflow: hidden;
  box-shadow: 0 24px 70px rgba(0,0,0,.5);
}
.m-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: 16px 22px; border-bottom: 1px solid var(--stroke);
}
.m-head h2 { font-size: 1.1rem; font-weight: 700; margin: 0 }
.m-close {
  background: transparent; border: none; color: var(--ink-dim);
  font-size: 1rem; cursor: pointer; padding: 4px 8px; border-radius: 8px;
}
.m-close:hover { background: rgba(255,255,255,.06); color: var(--ink) }
.m-body { display: flex; min-height: 380px; }
.m-nav {
  width: 132px; flex-shrink: 0; display: flex; flex-direction: column;
  padding: 14px 10px; gap: 4px; border-right: 1px solid var(--stroke);
}
.m-nav-item {
  text-align: left; padding: 10px 13px; border-radius: 10px; cursor: pointer;
  background: transparent; border: 1px solid transparent; color: var(--ink-dim);
  font-size: .88rem; transition: all .2s;
}
.m-nav-item:hover { color: var(--ink); background: rgba(255,255,255,.04) }
.m-nav-item.on {
  background: linear-gradient(135deg, rgba(108,140,255,.22), rgba(139,92,246,.16));
  border-color: rgba(108,140,255,.35); color: #fff;
}
.m-form { flex: 1; padding: 18px 22px; overflow-y: auto; }
.f-sec { display: flex; flex-direction: column; gap: 13px; }
.f-title { font-weight: 700; font-size: .95rem }
.f-hint { font-size: .78rem; margin: -4px 0 2px }
.f-note { font-size: .75rem; opacity: .75 }
.field { display: flex; flex-direction: column; gap: 6px }
.field label { font-size: .76rem; color: var(--ink-dim); font-weight: 600; letter-spacing: .4px; text-transform: uppercase }
.grid2 { display: grid; grid-template-columns: 1fr 1fr; gap: 12px }
.row { display: flex; gap: 8px; align-items: center }
.grow { flex: 1 }
.radio-row { display: flex; align-items: center; gap: 9px; font-size: .9rem; cursor: pointer; padding: 6px 2px }
.radio-row input { accent-color: #6c8cff }
.kv { display: flex; justify-content: space-between; gap: 12px; font-size: .85rem; padding: 5px 0; border-bottom: 1px dashed var(--stroke) }
.kv code { font-family: var(--mono, monospace); font-size: .78rem; word-break: break-all; text-align: right }
.small { font-size: .7rem !important }
.badge {
  font-style: normal; font-size: .66rem; padding: 1px 7px; border-radius: 99px;
  background: rgba(255,255,255,.08); color: var(--ink-dim); margin-left: 2px;
}
.badge.env { background: rgba(250, 204, 21, .16); color: #facc15 }
.badge.gen { background: rgba(52, 211, 153, .16); color: #34d399 }
.msg { margin-top: 4px }
.spacer { flex: 1 }
.m-foot {
  display: flex; gap: 10px; align-items: center;
  padding: 14px 22px; border-top: 1px solid var(--stroke);
}
.fade-enter-active, .fade-leave-active { transition: opacity .2s }
.fade-enter-from, .fade-leave-to { opacity: 0 }

/* 弹窗打开/关闭：从中心放大 + 弹性下沉 */
.modal-enter-active { transition: opacity .3s var(--ease-out), transform .3s var(--ease-out); }
.modal-leave-active { transition: opacity .15s ease, transform .15s ease; }
.modal-enter-from { opacity: 0; transform: scale(.92) translateY(12px); }
.modal-leave-to { opacity: 0; transform: scale(.96); }

</style>
