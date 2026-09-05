<script setup>
// 方案A v2 预览：对齐 QQ20260905-175222 参考图（双栏控制台）
// 左 25% Session+OSC ｜ 右 75% 世界收藏大卡 + 最近记录 + 底部建房 ｜ 最右窄托盘
import { ref, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({ auth: Object })

const sessTab = ref('run')
const sessions = ref([
  { id: 1, name: '中文吧', sub: '今晚见 · #1', inst: 'wrld_957b8d6f…2317', st: 'ok' },
  { id: 2, name: '话剧社', sub: '彩排', inst: 'wrld_4a1c…9e02', st: 'ok' },
  { id: 3, name: '新手教程', sub: '带萌新', inst: 'wrld_61e3…dcd3', st: 'run' },
  { id: 4, name: '摸鱼图', sub: '私密房', inst: 'wrld_c0de…77aa', st: 'bad' },
])
const favTab = ref('全部')
const favGroups = ref(['全部', '我最爱', '中文吧'])
const worlds = ref([
  { name: '中文吧 Chinese Bar 8.1.3', by: 'By Kafei', img: 'https://picsum.photos/seed/w1/360/200' },
  { name: '中文教学教程 CN Tutorial', by: 'By CN组', img: 'https://picsum.photos/seed/w2/360/200' },
  { name: 'Fall Guys 糖豆人', by: 'By 糖豆', img: 'https://picsum.photos/seed/w3/360/200' },
  { name: 'Among Us 太空狼人杀', by: 'By Sussus', img: 'https://picsum.photos/seed/w4/360/200' },
  { name: '深夜图书馆 2.0', by: 'By 夜读', img: 'https://picsum.photos/seed/w5/360/200' },
  { name: 'KTV 点歌台 Live', by: 'By 麦霸', img: 'https://picsum.photos/seed/w6/360/200' },
  { name: '黄昏夜语营地', by: 'By Campfire', img: 'https://picsum.photos/seed/w7/360/200' },
  { name: 'VRChat Home 新手之家', by: 'By VRC', img: 'https://picsum.photos/seed/w8/360/200' },
])
const history = ref([
  { name: '中文吧 Chinese Bar 8.1.3', id: 'wrld_957b8d6f-2217-1245-group', region: '#jp', time: '09-05 16:52' },
  { name: '话剧社彩排 排练厅', id: 'wrld_4a1c…9e02-group', region: '#jp', time: '09-05 16:40' },
  { name: 'Among Us 太空狼人杀', id: 'wrld_9f3b…cc12-group', region: '#usw', time: '09-05 15:12' },
  { name: 'Fall Guys 糖豆人', id: 'wrld_2e77…8811-group', region: '#use', time: '09-05 14:03' },
])
const chatText = ref('')
const notif = ref(1)

// ===== 建房（紧凑版，逻辑同 HomeView）=====
const world = ref('')
const instanceType = ref('group')
const group = ref('')
const access = ref('public')
const region = ref('jp')
const queue = ref(true)
const groups = ref([])
const groupsErr = ref('')
const busy = ref(false)
const buildMsg = ref('')

async function loadGroups() {
  try {
    groups.value = await invoke('list_groups')
    groupsErr.value = ''
    if (groups.value.length && !group.value) group.value = groups.value[0].id
  } catch (e) { groupsErr.value = String(e) }
}

async function doBuild() {
  buildMsg.value = ''
  const wid = world.value.trim()
  if (!wid) { buildMsg.value = '请填写世界 ID'; return }
  if (instanceType.value === 'group' && !group.value) { buildMsg.value = '群组房需选群组'; return }
  busy.value = true
  try {
    const r = await invoke('create_instance', {
      args: {
        world: wid,
        group: instanceType.value === 'group' ? group.value : null,
        region: region.value,
        access: access.value,
        queue: queue.value,
        age_gate: false,
        instance_type: instanceType.value,
      },
    })
    buildMsg.value = '✅ 已建房：' + (r?.instanceId || r?.id || JSON.stringify(r).slice(0, 80))
  } catch (e) { buildMsg.value = '❌ ' + String(e) }
  finally { busy.value = false }
}

onMounted(() => { if (props.auth?.ok) loadGroups() })
watch(() => props.auth?.ok, (ok) => { if (ok) loadGroups() })
</script>

<template>
  <div class="lp" style="flex:1;display:flex;gap:14px;height:100%;overflow:hidden">
    <!-- 左 25% -->
    <div class="glass lp-card" style="width:25%;min-width:250px;display:flex;flex-direction:column;gap:12px;min-height:0">
      <div class="lp-h">
        <h3>Session 列表</h3>
        <div style="display:flex;gap:4px">
          <button class="g-pill" :class="{on:sessTab==='run'}" @click="sessTab='run'">进行中</button>
          <button class="g-pill" :class="{on:sessTab==='stop'}" @click="sessTab='stop'">已停止</button>
        </div>
      </div>
      <div class="lp-list">
        <div v-for="s in sessions" :key="s.id" class="lp-item">
          <span class="dot" :class="s.st==='ok'?'ok':(s.st==='run'?'warn':'bad')"></span>
          <div style="flex:1;min-width:0">
            <div style="font-weight:600;font-size:.88rem">{{ s.name }}</div>
            <div class="text-dim" style="font-size:.7rem;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">{{ s.sub }}</div>
          </div>
          <span class="mono" style="font-size:.62rem;color:var(--ink-dim);white-space:nowrap">{{ s.inst }}</span>
        </div>
      </div>
      <div style="display:flex;gap:8px;margin-top:auto">
        <input v-model="chatText" class="g-input" style="flex:1" placeholder="输入要发送的信息..." />
        <button class="g-btn primary" style="flex-shrink:0">发送</button>
      </div>
    </div>

    <!-- 右 75% -->
    <div style="flex:1;min-width:0;height:100%;min-height:0;display:flex;flex-direction:column;gap:14px">
      <!-- 世界收藏 -->
      <div class="glass lp-card" style="flex:2;display:flex;flex-direction:column;min-height:0">
        <div class="lp-h">
          <h3>世界收藏</h3>
          <div style="display:flex;gap:6px">
            <button v-for="g in favGroups" :key="g" class="g-pill" :class="{on:favTab===g}" @click="favTab=g">{{ g }}</button>
          </div>
        </div>
        <div class="lp-grid">
          <div v-for="w in worlds" :key="w.name" class="lp-wcard">
            <div class="lp-thumb"><img :src="w.img" :alt="w.name" loading="lazy" /></div>
            <div class="lp-wmeta">
              <div class="lp-wname">{{ w.name }}</div>
              <div class="text-dim" style="font-size:.7rem">{{ w.by }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- 底部：最近记录 + 建房 -->
      <div class="glass lp-card" style="flex:1;min-height:150px;display:flex;flex-direction:column">
        <div class="lp-h"><h3 style="margin:0">最近房间记录</h3><button class="g-btn ghost" style="padding:4px 12px;font-size:.75rem">↻ 刷新</button></div>
        <div style="overflow-y:auto;flex:1;min-height:0">
          <table style="width:100%;border-collapse:collapse;font-size:.8rem">
            <thead><tr class="text-dim" style="text-align:left;font-size:.7rem">
              <th style="padding:4px 8px">房间/世界</th><th style="padding:4px 8px">实例ID</th>
              <th style="padding:4px 8px">区域</th><th style="padding:4px 8px">时间</th>
            </tr></thead>
            <tbody>
              <tr v-for="h in history" :key="h.id">
                <td style="padding:5px 8px;font-weight:600">{{ h.name }}</td>
                <td style="padding:5px 8px" class="mono text-dim">{{ h.id }}</td>
                <td style="padding:5px 8px" class="mono">{{ h.region }}</td>
                <td style="padding:5px 8px" class="text-dim">{{ h.time }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div style="border-top:1px solid var(--stroke);margin-top:auto;padding-top:10px;display:flex;gap:8px;align-items:center;flex-wrap:wrap">
          <input v-model="world" class="g-input mono" style="width:230px" placeholder="世界 ID (wrld_…)" />
          <select v-model="instanceType" class="g-select" style="width:100px">
            <option value="group">群组房</option><option value="public">公开</option>
            <option value="friends">好友</option><option value="private">私密</option>
          </select>
          <select v-if="instanceType==='group'" v-model="group" class="g-select" style="width:140px">
            <option v-if="groupsErr" value="">{{ groupsErr }}</option>
            <option v-for="g in groups" :key="g.id" :value="g.id">{{ g.name }}</option>
          </select>
          <select v-model="region" class="g-select" style="width:86px">
            <option value="jp">🇯🇵 日</option><option value="usw">🇺🇸 西</option>
            <option value="use">🇺🇸 东</option><option value="eu">🇪🇺 欧</option>
          </select>
          <button class="g-btn primary" :disabled="busy" @click="doBuild" style="flex-shrink:0">🚀 {{ busy ? '建房中…' : '建房' }}</button>
          <span v-if="buildMsg" style="font-size:.75rem" :class="buildMsg.startsWith('✅') ? 'ok' : 'bad'">{{ buildMsg }}</span>
        </div>
      </div>
    </div>

    <!-- 最右窄托盘 -->
    <div class="glass lp-tray">
      <button class="lp-tray-btn" style="position:relative">🔔
        <span v-if="notif" class="lp-badge">{{ notif }}</span>
      </button>
      <button class="lp-tray-btn">⚙️</button>
      <div style="flex:1"></div>
      <button class="lp-tray-btn">👤</button>
    </div>
  </div>
</template>

<style scoped>
.lp { height: 100%; display: flex; animation: fadeUp .4s var(--ease-out); }
@keyframes fadeUp { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: none; } }
.lp-card { border-radius: var(--radius); padding: 15px 16px; }
.lp-h { display: flex; align-items: center; justify-content: space-between; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; }
.lp-h h3 { font-size: .98rem; margin: 0; }
.lp-list { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 4px; }
.lp-item {
  display: flex; align-items: center; gap: 8px; padding: 8px 9px;
  border-radius: 11px; transition: background .18s;
}
.lp-item:hover { background: rgba(255,255,255,.06); }
.lp-grid {
  flex: 1; overflow-y: auto; display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  grid-auto-rows: 1fr;               /* 行高自适应，卡片纵向铺满不留白 */
  gap: 14px; padding-bottom: 4px;
}
.lp-wcard { display: flex; flex-direction: column; border-radius: 14px; overflow: hidden; cursor: pointer;
  background: rgba(255,255,255,.045); border: 1px solid var(--stroke);
  box-shadow: inset 0 1px 0 rgba(255,255,255,.07);
  transition: transform .3s var(--ease-out), border-color .25s, box-shadow .3s; }
.lp-wcard:hover { transform: translateY(-3px); border-color: rgba(139,92,246,.5);
  box-shadow: 0 16px 40px rgba(0,0,0,.5), inset 0 1px 0 rgba(255,255,255,.1); }
.lp-wcard:hover .lp-thumb img { transform: scale(1.05); }
.lp-thumb { flex: 1; min-height: 0; overflow: hidden; background: rgba(0,0,0,.3); position: relative; }
.lp-thumb img { width: 100%; height: 100%; object-fit: cover; display: block;
  transition: transform .5s var(--ease-out); filter: saturate(.95); }
.lp-wcard:hover .lp-thumb img { transform: scale(1.05); }
.lp-wmeta { padding: 9px 11px; }
.lp-wname { font-size: .82rem; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.lp-tray {
  width: 46px; flex-shrink: 0; border-radius: 16px;
  display: flex; flex-direction: column; align-items: center;
  padding: 12px 0; gap: 12px;
}
.lp-tray-btn {
  background: transparent; border: none; color: var(--ink-dim);
  font-size: 1.1rem; cursor: pointer; width: 34px; height: 34px;
  border-radius: 10px; display: flex; align-items: center; justify-content: center;
  transition: background .2s, color .2s;
}
.lp-tray-btn:hover { background: rgba(139,92,246,.18); color: #fff; }
.lp-badge {
  position: absolute; top: -3px; right: -3px; min-width: 15px; height: 15px;
  background: #ef4444; color: #fff; font-size: .6rem; font-weight: 700;
  border-radius: 8px; display: flex; align-items: center; justify-content: center; padding: 0 3px;
}
</style>
