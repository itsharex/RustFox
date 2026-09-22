<script setup lang="ts">
/**
 * CustomSelect：自绘下拉选择器。
 * - 触发按钮 + Teleport 到 body 的浮层（避免被 overflow 裁剪），滚动/尺寸变化自动收起；
 * - 键盘：↑/↓ 移动高亮（循环）、Enter 选中、Esc 关闭；外部点击关闭；
 * - 五态：default / hover / focus / active(open) / disabled × 深/浅主题；
 * - 作用域插槽 #display 定制触发区文案（如方法着色）、#option 定制选项行、
 *   #search 顶部搜索区（列表上方，带淡分隔线；过滤由父组件改写 options 完成）、
 *   #actions 定制选项右侧操作区（作为 .cs-opt 直接子项参与 flex 布局；
 *   点击需自行 .stop 防误选中）、#footer 底部固定操作栏（分割线下方，
 *   不参与列表滚动与键盘高亮循环）。
 */
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useLocaleStore } from '../../stores/locale'
import Icon from './Icon.vue'

export interface SelectOption {
  value: string | number
  label: string
}

const props = withDefaults(
  defineProps<{
    modelValue?: string | number | null
    options: SelectOption[]
    placeholder?: string
    disabled?: boolean
    size?: 'sm' | 'md'
    popClass?: string
    /** 弹窗最小宽度（px）：触发器太窄时（如请求栏模块选择）保证选项可读，默认跟触发器等宽。 */
    popMinWidth?: number
  }>(),
  { modelValue: null, placeholder: '', disabled: false, size: 'md', popClass: '', popMinWidth: 0 },
)

const locale = useLocaleStore()
const t = locale.t

/** 未传 placeholder 时按当前语言兜底。 */
const effectivePlaceholder = computed(() => props.placeholder || t('select.ph'))

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
  change: [value: string | number]
  open: []
  close: []
}>()

const open = ref(false)
const highlight = ref(-1)
const triggerEl = ref<HTMLButtonElement | null>(null)
const popupEl = ref<HTMLDivElement | null>(null)
const pos = ref<{ left: number; top: number; width: number; up: boolean }>({
  left: 0,
  top: 0,
  width: 0,
  up: false,
})

const selectedIndex = computed(() => {
  const idx = props.options.findIndex((o) => String(o.value) === String(props.modelValue))
  return idx === -1 ? -1 : idx
})

const displayLabel = computed(() => {
  const o = props.options[selectedIndex.value]
  return o ? o.label : ''
})

function measure(): void {
  const el = triggerEl.value
  if (!el) return
  const rect = el.getBoundingClientRect()
  // 挂载后优先用浮层真实高度（两行选项 / 底部栏会让按行数的估算偏小）；未挂载回退估算。
  const estimated = Math.min(props.options.length * 30 + 8, 280)
  const height = popupEl.value?.offsetHeight || estimated
  const spaceBelow = window.innerHeight - rect.bottom - 8
  const up = spaceBelow < height && rect.top > height
  pos.value = {
    left: rect.left,
    top: up ? rect.top - height - 4 : rect.bottom + 4,
    width: Math.max(rect.width, props.popMinWidth ?? 0),
    up,
  }
}

function openPopup(): void {
  if (props.disabled || open.value) return
  measure()
  open.value = true
  highlight.value = selectedIndex.value
  emit('open')
  // 浮层挂载后按真实高度复测上下翻转（首测发生在 v-if 挂载前，只能用估算）。
  void nextTick(() => {
    if (open.value) measure()
  })
}

function close(): void {
  if (!open.value) return
  open.value = false
  emit('close')
}

function pick(option: SelectOption): void {
  emit('update:modelValue', option.value)
  emit('change', option.value)
  close()
}

function onKeydown(event: KeyboardEvent): void {
  if (props.disabled) return
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    if (!props.options.length) return
    if (!open.value) {
      openPopup()
      return
    }
    highlight.value = (highlight.value + 1) % props.options.length
    scrollToHighlight()
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    if (!props.options.length) return
    if (!open.value) {
      openPopup()
      return
    }
    highlight.value = (highlight.value - 1 + props.options.length) % props.options.length
    scrollToHighlight()
  } else if (event.key === 'Enter') {
    event.preventDefault()
    const target = open.value ? props.options[highlight.value] : undefined
    if (target) {
      pick(target)
    } else {
      openPopup()
    }
  } else if (event.key === 'Escape') {
    event.preventDefault()
    close()
  } else if (event.key === 'Tab') {
    close()
  }
}

function scrollToHighlight(): void {
  requestAnimationFrame(() => {
    const el = popupEl.value?.querySelector<HTMLElement>('.cs-opt.hl')
    el?.scrollIntoView({ block: 'nearest' })
  })
}

function onDocMouseDown(event: MouseEvent): void {
  const target = event.target as Node
  if (triggerEl.value?.contains(target) || popupEl.value?.contains(target)) return
  close()
}

function onReposition(): void {
  if (open.value) measure()
}

watch(open, (isOpen) => {
  if (isOpen) {
    document.addEventListener('mousedown', onDocMouseDown, true)
    window.addEventListener('scroll', onReposition, true)
    window.addEventListener('resize', onReposition)
  } else {
    document.removeEventListener('mousedown', onDocMouseDown, true)
    window.removeEventListener('scroll', onReposition, true)
    window.removeEventListener('resize', onReposition)
  }
})

// 搜索过滤等导致选项收缩时钳住高亮，避免 Enter 命中越界空项。
watch(
  () => props.options,
  (list) => {
    if (highlight.value >= list.length) highlight.value = list.length - 1
  },
)

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', onDocMouseDown, true)
  window.removeEventListener('scroll', onReposition, true)
  window.removeEventListener('resize', onReposition)
})

defineExpose({ close })
</script>

<template>
  <div class="cs" :class="[`size-${size}`, { open, disabled }]">
    <button
      ref="triggerEl"
      type="button"
      class="cs-trigger"
      :disabled="disabled"
      aria-haspopup="listbox"
      :aria-expanded="open"
      @click="open ? close() : openPopup()"
      @keydown="onKeydown"
    >
      <span class="cs-value" :class="{ 'is-empty': !displayLabel }">
        <slot name="display" :label="displayLabel" :selected="options[selectedIndex] ?? null">
          {{ displayLabel || effectivePlaceholder }}
        </slot>
      </span>
      <Icon class="cs-caret" :name="open ? 'chevron-up' : 'chevron-down'" :size="12" />
    </button>

    <Teleport to="body">
      <div
        v-if="open"
        ref="popupEl"
        class="cs-pop"
        :class="[popClass, { up: pos.up }]"
        :style="{ left: `${pos.left}px`, top: `${pos.top}px`, width: `${pos.width}px` }"
        role="listbox"
      >
        <div v-if="$slots.search" class="cs-pop-search" role="search">
          <slot name="search" />
        </div>
        <div class="cs-pop-list">
          <div
            v-for="(o, i) in options"
            :key="String(o.value)"
            class="cs-opt"
            :class="{ hl: highlight === i, sel: String(o.value) === String(modelValue) }"
            role="option"
            :aria-selected="String(o.value) === String(modelValue)"
            @click="pick(o)"
            @mouseenter="highlight = i"
          >
            <span class="cs-opt-check">
              <Icon v-if="String(o.value) === String(modelValue)" name="check" :size="12" />
            </span>
            <span class="cs-opt-label" :title="o.label">
              <slot name="option" :option="o" :selected="String(o.value) === String(modelValue)">
                {{ o.label }}
              </slot>
            </span>
            <slot
              v-if="$slots.actions"
              name="actions"
              :option="o"
              :selected="String(o.value) === String(modelValue)"
            />
          </div>
        </div>
        <div v-if="$slots.footer" class="cs-pop-footer">
          <slot name="footer" />
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.cs {
  position: relative;
  display: inline-flex;
  flex: 0 0 auto;
}

.cs-trigger {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  border: 1px solid var(--border);
  background: var(--bg-card);
  color: var(--text-1);
  border-radius: var(--radius);
  font-size: 13px;
  font-family: var(--font-mono);
  padding: 0 8px 0 10px;
  cursor: pointer;
  user-select: none;
  transition:
    background var(--dur) var(--ease),
    border-color var(--dur) var(--ease),
    box-shadow var(--dur) var(--ease);
}
.cs.size-md .cs-trigger {
  height: var(--h-md);
}
.cs.size-sm .cs-trigger {
  height: var(--h-sm);
  font-size: 12px;
}
.cs-trigger:hover:not(:disabled) {
  background: var(--bg-elevated);
  border-color: var(--border-strong);
}
.cs-trigger:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 1px;
}
.cs.open .cs-trigger {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-tint);
}
.cs.disabled .cs-trigger {
  opacity: 0.45;
  cursor: default;
}

.cs-value {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: left;
  display: inline-flex;
  align-items: center;
}
.cs-value.is-empty {
  color: var(--text-3);
}

.cs-caret {
  flex-shrink: 0;
  color: var(--text-3);
  transition: transform var(--dur) var(--ease);
}

.cs-pop {
  position: fixed;
  z-index: 1000;
  background: var(--bg-elevated);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
  padding: 4px;
  transform-origin: top;
  animation: cs-in 120ms var(--ease);
}
.cs-pop.up {
  transform-origin: bottom;
}
/* 选项列表独立滚动：footer 固定在其下方，不随列表滚走 */
.cs-pop-list {
  max-height: 280px;
  overflow-y: auto;
}
/* 顶部搜索区：mb-1 px-1.5 pt-1 外边距 + 与列表间极淡分隔线（主题变量调透明，浅色下不隐形） */
.cs-pop-search {
  padding: 4px 6px;
  margin-bottom: 4px;
  border-bottom: 1px solid color-mix(in srgb, var(--border) 55%, transparent);
}
.cs-pop-footer {
  margin-top: 4px;
  padding-top: 6px;
  border-top: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
}

.cs-opt {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 30px;
  padding: 0 10px 0 8px;
  cursor: pointer;
  font-size: 12.5px;
  font-family: var(--font-mono);
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  user-select: none;
  transition: background var(--dur) var(--ease);
}
.cs-opt.hl {
  background: var(--bg-hover);
}
.cs-opt.sel {
  color: var(--accent);
}
.cs-opt:active {
  background: var(--accent-tint);
}

.cs-opt-check {
  width: 14px;
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--accent);
}

.cs-opt-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@keyframes cs-in {
  from {
    opacity: 0;
    transform: translateY(-4px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}
</style>
