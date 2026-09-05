<script setup>
import { ref, onMounted, watch } from 'vue'
import { call } from '../lib/api'

const props = defineProps({
  auth: Object,
  presetWorld: [String, null],
})
const emit = defineEmits(['changed', 'clear-preset'])

const world = ref('')
const group = ref('')
const access = ref('public')
const region = ref('jp')
const queue = ref(true)
const ageGate = ref(false)
const instanceType = ref('group')
const nameHist = ref(loadHist())

const groups = ref([])
const groupsErr = ref('')
const groupsLoading = ref(false)

const busy = ref(false)
const result = ref(null)
const error = ref('')

watch(() => props.presetWorld, (v) => {
  if (v) {
    world.value = v.id || v
    emit('clear-preset')
    flashSuccess(`已从收藏带入：${v.name || v.id}`)
  }
})

function loadHist() {
  try { return JSON.parse(localStorage.getItem('vrcnexus_names') || '[]') } catch { return [] }
}
function saveName() {
  const n = world.value.trim()
  if (!n) return
  const arr = [n, ...nameHist.value.filter(x => x !== n)].slice(0, 10)
  nameHist.value = arr
  localStorage.setItem('vrcnexus_names', JSON.stringify(arr))
}

async function loadGroups() {
  groupsLoading.value = true
  try {
    groups.value = await call('list_groups')
    groupsErr.value = ''
    if (groups.value.length && !group.value) group.value = groups.value[0].id
  } catch (e) {
    groupsErr.value = String(e)
  } finally { groupsLoading.value = false }
}

async function doCreate() {
  error.value = ''; result.value = null
  const wid = world.value.trim()
  if (!wid) { error.value = '请填写世界 ID（wrld_xxx）'; return }
  if (instanceType.value === 'group' && !group.value) { error.value = '群组房需要选择群组'; return }
  busy.value = true
  try {
    let worldId = wid
    if (!wid.startsWith('wrld_')) {
      const w = await call('resolve_world', { token: wid })
      worldId = w?.id || wid
    }
    const r = await call('create_instance', {
      args: {
        world: worldId,
        group: instanceType.value === 'group' ? group.value : null,
        region: region.value,
        access: access.value,
        queue: queue.value,
        age_gate: ageGate.value,
        instance_type: instanceType.value,
      },
    })
    result.value = r
    saveName()
    emit('changed')
  } catch (e) {
    error.value = String(e)
  } finally { busy.value = false }
}

function flashSuccess(msg) {
  error.value = ''
  result.value = null
  setTimeout(() => { result.value = { notice: msg } }, 30)
}

onMounted(() => { if (props.auth?.ok) loadGroups() })
watch(() => props.auth?.ok, (ok) => { if (ok) loadGroups() })
</script>

<template>
  <div class="stack stagger">
    <!-- 状态横幅 -->
    <div class="glass hero spot">
      <div>
        <h1>新建 Session</h1>
        <p class="text-dim" style="font-size:.85rem">建房 + 自动拉起，一步到位</p>
      </div>
      <div class="hero-right">
        <span v-if="auth?.ok" class="g-pill on"><span class="dot ok pulse"></span>已连接</span>
        <span v-else class="g-pill" style="border-color:rgba(248,113,113,.4)"><span class="dot bad"></span>未连接</span>
      </div>
    </div>

    <!-- 建房表单 -->
    <div class="glass panel flow-border spot">
      <div class="grid">
        <div class="field span2">
          <label>世界 ID</label>
          <input v-model="world" class="g-input mono" list="world-hist"
            placeholder="wrld_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
            @focus="$event.target.select()" />
          <datalist id="world-hist">
            <option v-for="n in nameHist" :key="n" :value="n" />
          </datalist>
        </div>

        <div class="field">
          <label>实例类型</label>
          <select v-model="instanceType" class="g-select">
            <option value="group">群组房</option>
            <option value="public">公开</option>
            <option value="friends">好友</option>
            <option value="private">私密</option>
          </select>
        </div>

        <template v-if="instanceType === 'group'">
          <div class="field">
            <label>群组</label>
            <select v-model="group" class="g-select">
              <option v-if="groupsLoading" value="">加载群组…</option>
              <option v-else-if="groupsErr" value="">{{ groupsErr }}</option>
              <option v-for="g in groups" :key="g.id" :value="g.id">{{ g.name }}</option>
            </select>
          </div>
          <div class="field">
            <label>可见性</label>
            <select v-model="access" class="g-select">
              <option value="public">群组公开</option>
              <option value="plus">群组+</option>
              <option value="members">仅限群组</option>
            </select>
          </div>
        </template>

        <div class="field">
          <label>区域</label>
          <select v-model="region" class="g-select">
            <option value="jp">🇯🇵 日本</option>
            <option value="usw">🇺🇸 美西</option>
            <option value="use">🇺🇸 美东</option>
            <option value="eu">🇪🇺 欧洲</option>
          </select>
        </div>

        <div class="field">
          <label>选项</label>
          <div class="row">
            <span class="g-pill" :class="{ on: queue }" @click="queue = !queue">
              <span class="dot" :class="queue ? 'ok' : 'idle'"></span>允许排队
            </span>
            <span class="g-pill" :class="{ on: ageGate }" @click="ageGate = !ageGate">
              <span class="dot" :class="ageGate ? 'warn' : 'idle'"></span>年龄验证
            </span>
          </div>
        </div>

        <div class="field span2" style="display:flex;align-items:flex-end;gap:10px">
          <button class="g-btn primary big" :disabled="busy" @click="doCreate">
            <span class="btn-ic" :class="{ spin: busy }">🚀</span>
            {{ busy ? '建房中…' : '建房并启动' }}
          </button>
        </div>
      </div>

      <Transition name="fade">
        <div v-if="error" class="msg bad"><b>失败：</b>{{ error }}</div>
      </Transition>
      <Transition name="fade">
        <div v-if="result" class="msg ok">
          <template v-if="result.notice">{{ result.notice }}</template>
          <template v-else>
            <b>✓ 建房成功</b>
            <div class="mono" style="margin-top:4px;word-break:break-all">{{ result.instanceId }}</div>
          </template>
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.stack { display: flex; flex-direction: column; gap: 14px; }
.hero { display: flex; align-items: center; justify-content: space-between; padding: 22px 24px; }
.hero h1 { font-size: 1.4rem; font-weight: 700; letter-spacing: .3px;
  background: linear-gradient(90deg, #fff, #b9c6ff);
  -webkit-background-clip: text; background-clip: text; -webkit-text-fill-color: transparent; }
.hero-right { display: flex; gap: 8px; }
.panel { padding: 22px 24px; }
.grid { display: grid; grid-template-columns: 1fr 1fr; gap: 15px 18px; }
.field { display: flex; flex-direction: column; gap: 6px; }
.field label { font-size: .76rem; color: var(--ink-dim); font-weight: 600; letter-spacing: .4px;
  text-transform: uppercase; }
.span2 { grid-column: span 2; }
.row { display: flex; gap: 8px; flex-wrap: wrap; }
.g-btn.big { padding: 12px 26px; font-size: 1rem; border-radius: 15px; min-width: 180px; }
.btn-ic { display: inline-block; transition: transform .3s; }
.btn-ic.spin { animation: icSpin 1s linear infinite; }
@keyframes icSpin { to { transform: rotate(360deg); } }
.msg { animation: none; }
</style>
