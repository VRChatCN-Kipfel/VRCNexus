<script setup>
import { ref, onMounted } from 'vue'
import { call } from '../lib/api'

const emit = defineEmits(['picked'])

const groups = ref([])
const worlds = ref([])
const total = ref(0)
const activeTag = ref('')
const q = ref('')
const loading = ref(false)
const err = ref('')

async function loadGroups() {
  loading.value = true; err.value = ''
  try {
    groups.value = await call('favorites_groups', { groupType: 'world' })
  } catch (e) { err.value = String(e) }
  loading.value = false
}

async function loadWorlds() {
  if (!activeTag.value) return
  loading.value = true; err.value = ''
  try {
    const r = await call('favorites_worlds', {
      group: activeTag.value,
      q: q.value || null,
      limit: 60,
      offset: 0,
    })
    worlds.value = r.items
    total.value = r.total
  } catch (e) { err.value = String(e) }
  loading.value = false
}

function pickGroup(tag) {
  activeTag.value = tag
  q.value = ''
  loadWorlds()
}

function pickWorld(w) {
  emit('picked', { id: w.id, name: w.name })
}

onMounted(loadGroups)
</script>

<template>
  <div class="stack">
    <div class="glass hero">
      <div>
        <h1>世界收藏</h1>
        <p class="text-dim" style="font-size:.85rem">点收藏夹查看世界，点卡片带去建房</p>
      </div>
      <div class="hero-right">
        <input v-model="q" class="g-input" style="width:190px" placeholder="搜索…"
          @keydown.enter="loadWorlds" :disabled="!activeTag" />
        <button class="g-btn ghost" @click="loadGroups">↻</button>
      </div>
    </div>

    <div class="glass panel">
      <!-- 收藏夹 pills -->
      <div class="pills">
        <span v-if="loading && !groups.length" class="text-dim" style="font-size:.85rem">加载中…</span>
        <span v-else-if="!groups.length && !err" class="text-dim" style="font-size:.85rem">（暂无收藏夹）</span>
        <span v-for="g in groups" :key="g.tag" class="g-pill" :class="{ on: activeTag === g.tag }"
          @click="pickGroup(g.tag)">
          {{ g.display_name }}
          <b style="font-weight:600">{{ g.count }}</b>
          <template v-if="g.visibility !== 'local'">/100</template>
        </span>
      </div>

      <div v-if="err" class="msg bad">{{ err }}</div>

      <!-- 世界网格 -->
      <div v-if="activeTag" class="meta text-dim">
        共 {{ total }} 个世界
      </div>
      <div class="grid" v-if="activeTag">
        <div v-for="w in worlds" :key="w.id + w.group_tag" class="wcard glass glass-hover" @click="pickWorld(w)">
          <img :src="w.thumbnail_image_url" loading="lazy" class="thumb"
            @error="$event.target.style.visibility = 'hidden'" />
          <div class="winfo">
            <div class="wname" :title="w.name">{{ w.name }}</div>
            <div class="wauthor text-dim">{{ w.author_name }}</div>
          </div>
        </div>
        <div v-if="!loading && !worlds.length" class="text-dim" style="grid-column:1/-1;text-align:center;padding:30px">
          （无匹配世界）
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.stack { display: flex; flex-direction: column; gap: 14px; }
.hero { display: flex; align-items: center; justify-content: space-between; padding: 18px 22px; }
.hero h1 { font-size: 1.25rem; font-weight: 700; }
.hero-right { display: flex; gap: 8px; align-items: center; }
.panel { padding: 18px 22px; }
.pills { display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 16px; }
.meta { margin-bottom: 10px; font-size: .82rem; }
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(158px, 1fr));
  gap: 12px;
}
.wcard { overflow: hidden; cursor: pointer; border-radius: 15px; }
.thumb { width: 100%; height: 92px; object-fit: cover; display: block; background: rgba(255,255,255,.05); }
.winfo { padding: 8px 10px 10px; }
.wname { font-size: .82rem; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.wauthor { font-size: .72rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-top: 2px; }
.msg { margin-top: 10px; padding: 8px 12px; border-radius: 10px; font-size: .85rem; background: rgba(248,113,113,.1); border: 1px solid rgba(248,113,113,.3); }
</style>
