<!--
  MoLaunch - Minecraft Launcher
  Copyright (C) 2026 MoTeam

  This file is derived from Arco Design Vue (https://arco.design/).
  Original code licensed under the MIT License.

  MIT License full text will be added here
-->
<!--
  自定义颜色选择器组件（参考 Arco Design Vue ColorPicker 视觉风格）
  - 触发器：32px 高，色块 + HEX 文本，与项目自研 Select 风格一致
  - 下拉面板：预设色板（6 列 × 2 行）+ 自定义 HEX 输入 + 实时预览
  - 弹层定位逻辑复用 Select.vue 的实现，并补充右侧边界夹紧
-->
<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { PRESET_COLORS } from '@/utils/color'

interface Props {
  /** 当前选中的 HEX 颜色（如 "#165dff"） */
  modelValue: string
  /** 是否禁用 */
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
})

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

const open = ref(false)
const closing = ref(false)
const triggerRef = ref<HTMLElement | null>(null)
const dropdownStyle = ref<Record<string, string>>({})
const openUpward = ref(false)

// 自定义 HEX 输入值（与 modelValue 同步）
const customInput = ref(props.modelValue)
const inputError = ref(false)

function select(color: string) {
  emit('update:modelValue', color)
  customInput.value = color
  inputError.value = false
  open.value = false
}

function isValidHex(s: string): boolean {
  const v = s.trim()
  return /^#?([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/.test(v)
}

function normalizeHex(s: string): string {
  let v = s.trim()
  if (!v.startsWith('#')) v = '#' + v
  // 3 位扩展为 6 位
  if (v.length === 4) {
    v = '#' + v[1] + v[1] + v[2] + v[2] + v[3] + v[3]
  }
  return v.toLowerCase()
}

function onInputBlur() {
  if (!isValidHex(customInput.value)) {
    inputError.value = true
    customInput.value = props.modelValue
    return
  }
  inputError.value = false
  const normalized = normalizeHex(customInput.value)
  if (normalized !== props.modelValue) {
    emit('update:modelValue', normalized)
  }
}

function onInputEnter(e: KeyboardEvent) {
  ;(e.target as HTMLInputElement).blur()
}

function updateDropdownPosition() {
  if (!triggerRef.value) return
  const rect = triggerRef.value.getBoundingClientRect()
  const viewportH = window.innerHeight
  const viewportW = window.innerWidth
  // 下拉面板最小宽度 240px，高度约 220px
  const dropdownMinW = 240
  const dropdownMaxH = 220
  const gap = 4
  const margin = 8

  // 横向：先按触发器 left 对齐，但右侧/左侧溢出时夹紧到视口内
  const dropdownW = Math.max(rect.width, dropdownMinW)
  let left = rect.left
  if (left + dropdownW > viewportW - margin) {
    left = viewportW - margin - dropdownW
  }
  if (left < margin) {
    left = margin
  }

  const spaceBelow = viewportH - rect.bottom - gap
  const spaceAbove = rect.top - gap

  if (spaceBelow >= dropdownMaxH || spaceBelow >= spaceAbove) {
    openUpward.value = false
    dropdownStyle.value = {
      position: 'fixed',
      top: `${rect.bottom + gap}px`,
      left: `${left}px`,
      width: `${dropdownW}px`,
      zIndex: '9999',
      transformOrigin: 'top',
    }
  } else {
    openUpward.value = true
    dropdownStyle.value = {
      position: 'fixed',
      bottom: `${viewportH - rect.top + gap}px`,
      left: `${left}px`,
      width: `${dropdownW}px`,
      zIndex: '9999',
      transformOrigin: 'bottom',
    }
  }
}

function toggle() {
  if (props.disabled) return
  if (!open.value) {
    updateDropdownPosition()
    window.addEventListener('scroll', onScroll, true)
  } else {
    window.removeEventListener('scroll', onScroll, true)
  }
  open.value = !open.value
  if (open.value) {
    customInput.value = props.modelValue
    inputError.value = false
    nextTick(() => {
      const inp = document.querySelector('.color-picker-input') as HTMLInputElement | null
      if (inp) inp.focus()
    })
  }
}

function onScroll(e: Event) {
  if (!open.value || closing.value) return
  const target = e.target as Node
  const dropdown = document.querySelector('.color-picker-dropdown')
  if (dropdown && dropdown.contains(target)) return
  if (triggerRef.value && triggerRef.value.contains(target)) return

  closing.value = true
  window.removeEventListener('scroll', onScroll, true)
  requestAnimationFrame(() => {
    open.value = false
    setTimeout(() => {
      closing.value = false
    }, 150)
  })
}

function handleClickOutside(e: MouseEvent) {
  const dropdown = document.querySelector('.color-picker-dropdown')
  if (
    triggerRef.value &&
    !triggerRef.value.contains(e.target as Node) &&
    dropdown &&
    !dropdown.contains(e.target as Node)
  ) {
    open.value = false
  }
}

onMounted(() => document.addEventListener('click', handleClickOutside))
onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  window.removeEventListener('scroll', onScroll, true)
})
</script>

<template>
  <div ref="triggerRef" class="color-picker">
    <!-- 触发器：色块 + HEX 文本 -->
    <div
      class="color-picker-trigger"
      :class="{ active: open, disabled: disabled }"
      @click="toggle"
    >
      <span class="color-swatch" :style="{ backgroundColor: modelValue }" />
      <span class="color-value">{{ modelValue.toUpperCase() }}</span>
      <svg
        class="color-picker-arrow"
        :class="{ rotated: open }"
        viewBox="0 0 1024 1024"
        fill="currentColor"
      >
        <path d="M512 714.666667c-8.533333 0-17.066667-2.133333-23.466667-8.533334l-341.333333-341.333333c-12.8-12.8-12.8-32 0-44.8 12.8-12.8 32-12.8 44.8 0L512 637.866667l320-320c12.8-12.8 32-12.8 44.8 0 12.8 12.8 12.8 32 0 44.8l-341.333333 341.333333c-6.4 6.4-14.933333 10.666667-23.466667 10.666667z" />
      </svg>
    </div>

    <!-- 下拉面板：预设色板 + HEX 输入 -->
    <teleport to="body">
      <transition
        enter-active-class="cp-enter-active"
        :enter-from-class="openUpward ? 'cp-enter-from-up' : 'cp-enter-from'"
        enter-to-class="cp-enter-to"
        leave-active-class="cp-leave-active"
        :leave-from-class="openUpward ? 'cp-leave-from-up' : 'cp-leave-from'"
        :leave-to-class="openUpward ? 'cp-leave-to-up' : 'cp-leave-to'"
      >
        <div v-if="open" class="color-picker-dropdown" :style="dropdownStyle">
          <!-- 预设色板（4 列 × 3 行 = 12 色） -->
          <div class="cp-section-label">预设颜色</div>
          <div class="cp-preset-grid">
            <button
              v-for="color in PRESET_COLORS"
              :key="color"
              type="button"
              class="cp-preset-swatch"
              :class="{ selected: color.toLowerCase() === modelValue.toLowerCase() }"
              :style="{ backgroundColor: color }"
              :title="color"
              @click="select(color)"
            >
              <svg
                v-if="color.toLowerCase() === modelValue.toLowerCase()"
                class="cp-check"
                viewBox="0 0 1024 1024"
                fill="currentColor"
              >
                <path d="M912 192c-12.8 0-25.6 4.266667-34.133333 12.8L384 699.2 234.666667 548.266667c-17.066667-17.066667-46.933333-17.066667-64 0-17.066667 17.066667-17.066667 46.933333 0 64l179.2 179.2c8.533333 8.533333 21.333333 12.8 34.133333 12.8s25.6-4.266667 34.133333-12.8l520.533334-520.533334c17.066667-17.066667 17.066667-46.933333 0-64-8.533333-8.533333-21.333333-12.8-34.133334-12.8z" />
              </svg>
            </button>
          </div>

          <!-- 自定义 HEX 输入 -->
          <div class="cp-section-label">自定义颜色</div>
          <div class="cp-custom-row">
            <span class="cp-preview" :style="{ backgroundColor: modelValue }" />
            <input
              v-model="customInput"
              type="text"
              class="color-picker-input"
              :class="{ error: inputError }"
              placeholder="#165dff"
              maxlength="7"
              @blur="onInputBlur"
              @keydown.enter="onInputEnter"
            />
          </div>
          <div v-if="inputError" class="cp-error-msg">请输入有效的 HEX 颜色（如 #165dff）</div>
        </div>
      </transition>
    </teleport>
  </div>
</template>

<style scoped>
/* ============================================================
 * 触发器
 * 复用 Select 触发器尺寸与背景，与项目自研组件视觉一致
 * ============================================================ */
.color-picker {
  position: relative;
  min-width: 120px;
  max-width: 100%;
}

.color-picker-trigger {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 32px;
  padding: 0 12px;
  box-sizing: border-box;
  color: #1d2129;
  font-size: 14px;
  background-color: #f2f3f5;
  border: 1px solid transparent;
  border-radius: 2px;
  cursor: pointer;
  user-select: none;
  max-width: 100%;
  transition: color 0.1s cubic-bezier(0, 0, 1, 1),
    border-color 0.1s cubic-bezier(0, 0, 1, 1),
    background-color 0.1s cubic-bezier(0, 0, 1, 1);
}

.color-picker-trigger:hover {
  background-color: #e5e6eb;
  border-color: transparent;
}

.color-picker-trigger.active {
  background-color: #fff;
  border-color: var(--color-primary-500);
  box-shadow: none;
}

.color-picker-trigger.disabled {
  color: #c9cdd4;
  background-color: #f2f3f5;
  border-color: transparent;
  cursor: not-allowed;
}

.color-swatch {
  width: 16px;
  height: 16px;
  border-radius: 2px;
  border: 1px solid rgba(0, 0, 0, 0.1);
  flex-shrink: 0;
}

.color-value {
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  letter-spacing: 0.3px;
}

.color-picker-arrow {
  width: 12px;
  height: 12px;
  color: #86909c;
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.color-picker-arrow.rotated {
  transform: rotate(180deg);
}

/* ============================================================
 * 下拉面板
 * ============================================================ */
.color-picker-dropdown {
  background-color: #fff;
  border: 1px solid #e5e6eb;
  border-radius: 4px;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.1);
  padding: 12px;
  box-sizing: border-box;
}

.cp-section-label {
  font-size: 12px;
  color: #86909c;
  margin-bottom: 8px;
}

/* 非首个 section-label（即"自定义颜色"标题）需要与上方色板拉开距离 */
.cp-section-label:not(:first-child) {
  margin-top: 16px;
}

/* 预设色板：4 列 × 3 行 */
.cp-preset-grid {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 6px;
}

.cp-preset-swatch {
  position: relative;
  width: 100%;
  aspect-ratio: 1;
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 2px;
  cursor: pointer;
  padding: 0;
  transition: transform 0.1s cubic-bezier(0, 0, 1, 1);
}

.cp-preset-swatch:hover {
  transform: scale(1.1);
}

.cp-preset-swatch.selected {
  border: 2px solid #1d2129;
}

.cp-check {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 12px;
  height: 12px;
  color: #fff;
  filter: drop-shadow(0 0 1px rgba(0, 0, 0, 0.4));
}

/* 自定义 HEX 输入 */
.cp-custom-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.cp-preview {
  width: 28px;
  height: 28px;
  border-radius: 2px;
  border: 1px solid rgba(0, 0, 0, 0.1);
  flex-shrink: 0;
}

.color-picker-input {
  flex: 1;
  height: 32px;
  padding: 0 10px;
  box-sizing: border-box;
  font-size: 13px;
  font-family: 'Consolas', 'Monaco', monospace;
  color: #1d2129;
  background-color: #f2f3f5;
  border: 1px solid transparent;
  border-radius: 2px;
  outline: none;
  transition: all 0.1s cubic-bezier(0, 0, 1, 1);
}

.color-picker-input:hover {
  background-color: #e5e6eb;
}

.color-picker-input:focus {
  background-color: #fff;
  border-color: var(--color-primary-500);
}

.color-picker-input.error {
  border-color: #f53f3f;
}

.cp-error-msg {
  margin-top: 6px;
  font-size: 12px;
  color: #f53f3f;
}

/* ============================================================
 * 弹出动画（复用 Select 的 scaleY + opacity）
 * ============================================================ */
.cp-enter-active,
.cp-leave-active {
  transition: opacity 0.2s cubic-bezier(0.34, 0.69, 0.1, 1),
    transform 0.2s cubic-bezier(0.34, 0.69, 0.1, 1);
}
.cp-enter-from,
.cp-leave-to {
  opacity: 0;
  transform: scaleY(0.9);
}
.cp-enter-from-up,
.cp-leave-to-up {
  opacity: 0;
  transform: scaleY(0.9);
}
.cp-enter-to,
.cp-leave-from,
.cp-enter-from-up,
.cp-leave-from-up {
  opacity: 1;
  transform: scaleY(1);
}
</style>
