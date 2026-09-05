<!-- SessionsView：Session 会话页(占位)
  SessionManager(进程管理/启动队列) Rust 后端实现中，当前为空态占位
-->

<script setup>
import { ref, onMounted } from 'vue'
import { call } from '../lib/api'

const sessions = ref([])
const err = ref('')

async function load() {
  try {
    // SessionManager 后端实现后接入；当前展示占位
    sessions.value = []
  } catch (e) { err.value = String(e) }
}

onMounted(load)
</script>

<template>
  <div class="stack">
    <div class="glass hero">
      <div>
        <h1>Session 会话</h1>
        <p class="text-dim" style="font-size:.85rem">多开实例的启动队列与状态管理</p>
      </div>
      <button class="g-btn ghost" @click="load">↻ 刷新</button>
    </div>

    <div class="glass panel">
      <div v-if="err" class="msg bad">{{ err }}</div>
      <div v-if="!sessions.length && !err" class="empty">
        <div class="empty-ic">▤</div>
        <p>启动队列与多开会话管理正在接入中</p>
        <p class="text-dim" style="font-size:.82rem">当前已可：建房 / 收藏浏览 / OSC 聊天</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.stack { display: flex; flex-direction: column; gap: 14px; }
.hero { display: flex; align-items: center; justify-content: space-between; padding: 18px 22px; }
.hero h1 { font-size: 1.25rem; font-weight: 700; }
.panel { padding: 20px 22px; }
.empty { text-align: center; padding: 48px 0; color: var(--ink-dim); }
.empty-ic { font-size: 2.4rem; margin-bottom: 10px; opacity: .5; }
.msg { margin-top: 10px; padding: 8px 12px; border-radius: 10px; font-size: .85rem; background: rgba(248,113,113,.1); border: 1px solid rgba(248,113,113,.3); }
</style>
