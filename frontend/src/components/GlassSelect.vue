<script setup>
// 玻璃拟态下拉（替代原生 select —— 原生弹出列表是 Windows 系统绘制，CSS 无法玻璃化）
// 用法: <GlassSelect v-model="x" :options="[{value,label}]" placeholder="选择…" :disabled="false" />
import { ref, computed, onMounted, onUnmounted } from 'vue'

const props = defineProps({
  modelValue: [String, Number],
  options: { type: Array, default: () => [] }, // [{value,label}] 或 [{value,label,disabled}]
  placeholder: { type: String, default: '选择…' },
  disabled: { type: Boolean, default: false },
  width: { type: String, default: '' },
})
const emit = defineEmits(['update:modelValue', 'change'])

const open = ref(false)
const rootEl = ref(null)

const label = computed(() => {
  const hit = props.options.find(o => String(o.value) === String(props.modelValue))
  return hit ? hit.label : props.placeholder
})

function toggle() { if (!props.disabled) open.value = !open.value }
function pick(o) {
  if (o.disabled) return
  emit('update:modelValue', o.value)
  emit('change', o.value)
  open.value = false
}
function onDocClick(e) {
  if (rootEl.value && !rootEl.value.contains(e.target)) open.value = false
}
onMounted(() => document.addEventListener('click', onDocClick))
onUnmounted(() => document.removeEventListener('click', onDocClick))
</script>

<template>
  <div ref="rootEl" class="gsel" :class="{ on: open, disabled }" :style="width ? { width } : {}" @click.stop="toggle">
    <span class="gsel-label" :class="{ ph: !options.some(o => String(o.value) === String(modelValue)) }">{{ label }}</span>
    <span class="gsel-arrow">▾</span>
    <Transition name="gs-pop">
      <div v-if="open" class="gsel-drop glass">
        <div v-for="o in options" :key="String(o.value)"
          class="gsel-item" :class="{ on: String(o.value) === String(modelValue), dis: o.disabled }"
          @click.stop="pick(o)">
          <span>{{ o.label }}</span>
          <span v-if="String(o.value) === String(modelValue)" class="gsel-check">✓</span>
        </div>
        <div v-if="!options.length" class="gsel-empty">（无选项）</div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.gsel {
  position: relative; display: inline-flex; align-items: center; justify-content: space-between; gap: 8px;
  background: var(--glass); border: 1px solid var(--stroke);
  border-radius: 13px; padding: 8px 13px; font-size: .88rem;
  cursor: pointer; user-select: none; min-width: 0;
  transition: border-color .2s, background .2s; white-space: nowrap;
}
.gsel:hover { border-color: var(--stroke-hi); }
.gsel.on { border-color: rgba(139,92,246,.55); }
.gsel.disabled { opacity: .5; cursor: not-allowed; }
.gsel-label { overflow: hidden; text-overflow: ellipsis; }
.gsel-label.ph { color: var(--ink-dim); }
.gsel-arrow { color: var(--ink-dim); font-size: .75rem; flex-shrink: 0; transition: transform .2s; }
.gsel.on .gsel-arrow { transform: rotate(180deg); }
.gsel-drop {
  position: absolute; top: calc(100% + 6px); left: 0; min-width: 100%;
  background: rgba(18, 22, 36, .97);
  backdrop-filter: blur(20px); border: 1px solid var(--stroke);
  border-radius: 13px; padding: 5px; z-index: 60;
  box-shadow: 0 14px 34px rgba(0,0,0,.6); max-height: 260px; overflow-y: auto;
}
.gsel-item {
  display: flex; align-items: center; justify-content: space-between; gap: 10px;
  padding: 8px 11px; border-radius: 9px; cursor: pointer; font-size: .87rem;
  color: var(--ink-dim); transition: all .16s; white-space: nowrap;
}
.gsel-item:hover { background: rgba(255,255,255,.08); color: #fff; }
.gsel-item.on {
  background: linear-gradient(135deg, rgba(108,140,255,.3), rgba(139,92,246,.22));
  color: #fff;
}
.gsel-item.dis { opacity: .45; cursor: not-allowed; }
.gsel-check { color: #c4b5fd; font-size: .8rem; }
.gsel-empty { padding: 8px 11px; color: var(--ink-dim); font-size: .82rem; }
.gs-pop-enter-active, .gs-pop-leave-active { transition: opacity .16s ease, transform .16s var(--ease-out); }
.gs-pop-enter-from, .gs-pop-leave-to { opacity: 0; transform: translateY(-4px) scale(.98); }
</style>
