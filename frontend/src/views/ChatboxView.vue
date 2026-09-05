<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const text = ref('')
const port = ref('9000')
const notify = ref(false)
const hist = ref(loadHist())
const busy = ref(false)
const okMsg = ref('')
const err = ref('')
const sessions = ref([])

function loadHist() {
  try { return JSON.parse(localStorage.getItem('vrcnexus_chat_hist') || '[]') } catch { return [] }
}
function saveHist(t) {
  const arr = [t, ...hist.value.filter(x => x !== t)].slice(0, 20)
  hist.value = arr
  localStorage.setItem('vrcnexus_chat_hist', JSON.stringify(arr))
}

async function loadSessions() {
  // 暂用占位：后续接 SessionManager 后列出各实例 osc 端口
  // sessions.value = await invoke('list_sessions')
}

async function send() {
  const t = text.value.trim()
  if (!t) { err.value = '请输入消息内容'; return }
  busy.value = true; okMsg.value = ''; err.value = ''
  try {
    const r = await invoke('send_chatbox', { args: { text: t, port: port.value ? Number(port.value) : null, notify: notify.value } })
    saveHist(t)
    okMsg.value = `✓ 已发送到 ${r.to}`
    text.value = ''
  } catch (e) {
    err.value = String(e)
  } finally { busy.value = false }
}

function pickHist(t) { text.value = t }

onMounted(() => { loadSessions() })
</script>

<template>
  <div class="stack">
    <div class="glass hero">
      <div>
        <h1>OSC 聊天推送</h1>
        <p class="text-dim" style="font-size:.85rem">往 VRChat 聊天框发消息（/chatbox/input）</p>
      </div>
    </div>

    <div class="glass panel">
      <div class="row">
        <div class="field" style="width:150px">
          <label>目标端口</label>
          <input v-model="port" class="g-input mono" placeholder="9000" />
        </div>
        <div class="field" style="flex:1">
          <label>消息内容</label>
          <input v-model="text" class="g-input" placeholder="输入需要推送的消息..." @keydown.enter="send" />
        </div>
        <div class="field" style="width:110px">
          <label>&nbsp;</label>
          <button class="g-btn primary" style="width:100%;justify-content:center" :disabled="busy" @click="send">
            {{ busy ? '发送中…' : '发送' }}
          </button>
        </div>
      </div>
      <div class="row" style="margin-top:8px">
        <span class="g-pill" :class="{ on: notify }" @click="notify = !notify">通知气泡</span>
      </div>

      <!-- 历史 -->
      <div class="hist-title text-dim">最近消息（点选复用）</div>
      <div class="hist" v-if="hist.length">
        <span v-for="h in hist" :key="h" class="hist-chip glass-hover" @click="pickHist(h)">{{ h }}</span>
      </div>
      <div v-else class="text-dim" style="font-size:.82rem">（暂无历史，发送后自动保存）</div>

      <div v-if="okMsg" class="msg ok">{{ okMsg }}</div>
      <div v-if="err" class="msg bad">{{ err }}</div>
    </div>
  </div>
</template>

<style scoped>
.stack { display: flex; flex-direction: column; gap: 14px; }
.hero { padding: 18px 22px; }
.hero h1 { font-size: 1.25rem; font-weight: 700; }
.panel { padding: 20px 22px; }
.row { display: flex; gap: 12px; align-items: flex-end; }
.field { display: flex; flex-direction: column; gap: 6px; }
.field label { font-size: .78rem; color: var(--ink-dim); font-weight: 600; }
.hist-title { margin: 18px 0 8px; font-size: .78rem; }
.hist { display: flex; flex-wrap: wrap; gap: 7px; }
.hist-chip {
  padding: 6px 12px; border-radius: 999px; font-size: .8rem;
  background: var(--glass); border: 1px solid var(--stroke);
  cursor: pointer; color: var(--ink); max-width: 260px;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.msg { margin-top: 12px; padding: 9px 13px; border-radius: 10px; font-size: .85rem; }
.msg.ok { background: rgba(52,211,153,.1); border: 1px solid rgba(52,211,153,.3); }
.msg.bad { background: rgba(248,113,113,.1); border: 1px solid rgba(248,113,113,.3); }
</style>
