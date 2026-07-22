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
import Tooltip from '@/components/common/Tooltip.vue'

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
            <Tooltip
              v-for="color in PRESET_COLORS"
              :key="color"
              :text="color"
              position="top"
            >
              <div
                role="button"
                tabindex="0"
                class="cp-preset-swatch"
                :class="{ selected: color.toLowerCase() === modelValue.toLowerCase() }"
                :style="{ backgroundColor: color }"
                @click="select(color)"
                @keydown.enter="select(color)"
              >
                <svg
                  v-if="color.toLowerCase() === modelValue.toLowerCase()"
                  class="cp-check"
                  viewBox="0 0 1024 1024"
                  fill="currentColor"
                >
                  <path d="M912 192c-12.8 0-25.6 4.266667-34.133333 12.8L384 699.2 234.666667 548.266667c-17.066667-17.066667-46.933333-17.066667-64 0-17.066667 17.066667-17.066667 46.933333 0 64l179.2 179.2c8.533333 8.533333 21.333333 12.8 34.133333 12.8s25.6-4.266667 34.133333-12.8l520.533334-520.533334c17.066667-17.066667 17.066667-46.933333 0-64-8.533333-8.533333-21.333333-12.8-34.133334-12.8z" />
                </svg>
              </div>
            </Tooltip>
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

<style scoped src="./ColorPicker.css"></style>
