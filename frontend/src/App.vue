<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import HomeView from './views/HomeView.vue'
import SessionsView from './views/SessionsView.vue'
import FavoritesView from './views/FavoritesView.vue'
import ChatboxView from './views/ChatboxView.vue'
import LayoutPreviewView from './views/LayoutPreviewView.vue' // 临时：方案A布局预览
import SettingsView from './views/SettingsView.vue' // 设置弹窗

const view = ref('layout') // 临时：默认进布局预览(方案A展示用)；看毕改回 'home'
const showSettings = ref(false)
const auth = ref(null)
const authError = ref('')
const pendingWorld = ref(null)
const spotEl = ref(null)

const NAV = [
  { key: 'home', label: '建房', icon: '⌂' },
  { key: 'sessions', label: '会话', icon: '▤' },
  { key: 'favorites', label: '收藏', icon: '♡' },
  { key: 'chatbox', label: '聊天', icon: '✎' },
  { key: 'layout', label: '布局', icon: '🧭' }, // 临时：方案A预览入口
]

async function refreshAuth() {
  try {
    const r = await invoke('auth_status')
    auth.value = r
    authError.value = r?.ok ? '' : (r?.error || '认证失败')
  } catch (e) {
    authError.value = String(e)
  }
}

function onPicked(world) {
  pendingWorld.value = world
  view.value = 'home'
}

// 全局 spotlight：鼠标移动时给 .spot 元素设置光晕坐标
function onMove(e) {
  const els = document.querySelectorAll('.spot')
  for (const el of els) {
    const r = el.getBoundingClientRect()
    el.style.setProperty('--mx', (e.clientX - r.left) + 'px')
    el.style.setProperty('--my', (e.clientY - r.top) + 'px')
  }
}

onMounted(() => {
  refreshAuth()
  window.addEventListener('mousemove', onMove, { passive: true })
})
onUnmounted(() => window.removeEventListener('mousemove', onMove))
</script>

<template>
  <div class="bg-scene">
    <div class="bg-blob blob-1"></div>
    <div class="bg-blob blob-2"></div>
    <div class="bg-blob blob-3"></div>
    <div class="bg-blob blob-4"></div>
  </div>

  <div class="shell">
    <!-- 左侧导航 -->
    <aside class="rail glass spot">
      <div class="brand">
        <div class="logo">
          <span class="logo-in">V</span>
        </div>
        <div class="brand-text">
          <div class="name">VRCNexus</div>
          <div class="sub">VRChat 自动化终端</div>
        </div>
      </div>

      <nav class="nav">
        <button v-for="(n, i) in NAV" :key="n.key"
          class="nav-item" :class="{ on: view === n.key }"
          :style="{ '--i': i }"
          @click="view = n.key">
          <span class="ni-icon">{{ n.icon }}</span>
          <span class="ni-label">{{ n.label }}</span>
          <span v-if="view === n.key" class="ni-ind"></span>
        </button>
      </nav>

      <div class="rail-foot">
        <div class="auth-chip glass" :class="auth?.ok ? 'good' : 'bad'">
          <span class="dot" :class="auth?.ok ? 'ok pulse' : 'bad'"></span>
          <span class="auth-name" :title="authError">{{ auth?.ok ? auth.user.displayName : (authError || '未连接') }}</span>
        </div>
        <div class="rail-tools">
          <div class="ver text-dim">v0.1.0 · Tauri</div>
          <button class="gear-btn" title="设置" @click="showSettings = true">⚙</button>
        </div>
      </div>
    </aside>

    <!-- 主内容 -->
    <main class="main">
      <Transition name="view" mode="out-in">
        <div :key="view" class="view-wrap">
          <HomeView v-if="view === 'home'" :auth="auth" :preset-world="pendingWorld"
            @changed="refreshAuth" @clear-preset="pendingWorld = null" />
          <SessionsView v-else-if="view === 'sessions'" @changed="refreshAuth" />
          <FavoritesView v-else-if="view === 'favorites'" @picked="onPicked" />
          <ChatboxView v-else-if="view === 'chatbox'" @changed="refreshAuth" />
          <LayoutPreviewView v-else-if="view === 'layout'" :auth="auth" />
        </div>
      </Transition>
    </main>
  </div>

  <!-- 设置弹窗 -->
  <SettingsView v-if="showSettings" @close="showSettings = false" />
</template>

<style scoped>
.shell { display: flex; height: 100vh; padding: 14px; gap: 14px; }

/* 左栏 */
.rail { width: 210px; display: flex; flex-direction: column; padding: 18px 13px; flex-shrink: 0;
  overflow: hidden; }
.brand { display: flex; align-items: center; gap: 11px; padding: 2px 6px 18px; }
.logo {
  width: 40px; height: 40px; border-radius: 13px; flex-shrink: 0; position: relative;
  background: linear-gradient(135deg, #64748b, #475569);
  display: flex; align-items: center; justify-content: center;
  box-shadow: 0 8px 24px rgba(100,116,139,.38), inset 0 1px 0 rgba(255,255,255,.3);
  overflow: hidden;
}
.logo::before {
  content: ""; position: absolute; inset: 0;
  background: linear-gradient(120deg, transparent 30%, rgba(255,255,255,.4), transparent 70%);
  animation: logoShine 3.5s ease-in-out infinite;
}
@keyframes logoShine { 0%, 55% { transform: translateX(-100%); } 85%, 100% { transform: translateX(100%); } }
.logo-in { font-weight: 800; font-size: 21px; color: #fff; position: relative; }
.brand-text .name { font-weight: 700; font-size: 1.03rem; letter-spacing: .2px; }
.brand-text .sub { font-size: .7rem; color: var(--ink-dim); margin-top: 2px; }

.nav { display: flex; flex-direction: column; gap: 5px; flex: 1; }
.nav-item {
  position: relative; display: flex; align-items: center; gap: 11px;
  background: transparent; border: 1px solid transparent;
  color: var(--ink-dim); font-size: .92rem;
  padding: 11px 14px; border-radius: 13px; cursor: pointer;
  transition: all .22s var(--ease-out); text-align: left; width: 100%;
  overflow: hidden;
}
.nav-item::before {  /* 悬停微光 */
  content: ""; position: absolute; inset: 0; border-radius: inherit;
  background: linear-gradient(90deg, rgba(148,163,184,.12), transparent 80%);
  opacity: 0; transition: opacity .25s;
}
.nav-item:hover { color: var(--ink); }
.nav-item:hover::before { opacity: 1; }
.nav-item.on {
  background: linear-gradient(135deg, rgba(148,163,184,.24), rgba(100,116,139,.18));
  border-color: rgba(148,163,184,.4);
  color: #fff;
  box-shadow: 0 6px 20px rgba(100,116,139,.18), inset 0 1px 0 rgba(255,255,255,.15);
}
.ni-icon { width: 22px; text-align: center; font-size: 1.02rem; position: relative;
  transition: transform .25s var(--ease-out); }
.nav-item:hover .ni-icon { transform: scale(1.18); }
.nav-item.on .ni-icon { filter: drop-shadow(0 0 6px rgba(148,163,184,.7)); }
.ni-label { position: relative; }
.ni-ind {
  position: absolute; right: 10px; top: 50%; transform: translateY(-50%);
  width: 4px; height: 4px; border-radius: 50%; background: #fff;
  box-shadow: 0 0 8px rgba(255,255,255,.9);
  animation: indPulse 2s ease-in-out infinite;
}
@keyframes indPulse { 0%,100% { opacity: .5; } 50% { opacity: 1; } }

.rail-foot { padding-top: 13px; border-top: 1px solid var(--stroke); display: flex; flex-direction: column; gap: 7px; }
.auth-chip {
  display: flex; align-items: center; gap: 8px;
  padding: 9px 11px; border-radius: 12px; font-size: .8rem;
  background: var(--glass);
}
.auth-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.auth-chip.bad { border-color: rgba(248,113,113,.35); }
.auth-chip.bad .auth-name { color: var(--bad); }
.ver { text-align: center; font-size: .68rem; opacity: .7; }
.rail-tools { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.gear-btn {
  background: transparent; border: 1px solid var(--stroke); color: var(--ink-dim);
  font-size: .95rem; cursor: pointer; padding: 3px 9px; border-radius: 9px;
  transition: all .2s; line-height: 1.4;
}
.gear-btn:hover { color: #fff; border-color: rgba(148,163,184,.5); background: rgba(148,163,184,.1); }

/* 主区 */
.main { flex: 1; overflow: hidden; /* 改为 hidden，由内层组件自行滚动，避免外层滚动条挤压高度 */ border-radius: var(--radius); display: flex; flex-direction: column; min-height: 0; }
.view-wrap { padding: 4px 8px 0; /* 去掉底部的 24px */ flex: 1; display: flex; flex-direction: column; min-height: 0; }
.view-wrap > * { flex: 1; min-height: 0; }

/* 视图切换动效：微缩放 + 极小位移 + 优雅淡入 */
.view-enter-active { transition: opacity .35s var(--ease-out), transform .35s var(--ease-out); }
.view-leave-active { transition: opacity .15s ease, transform .15s ease; }
.view-enter-from { opacity: 0; transform: translateY(6px) scale(.99); }
.view-leave-to { opacity: 0; transform: translateY(-4px) scale(.995); }
</style>
