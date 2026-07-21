<!--
  MoLaunch - Minecraft Launcher
  Copyright (C) 2026 MoTeam

  This file is derived from Arco Design Vue (https://arco.design/).
  Original code licensed under the MIT License.

  MIT License full text will be added here
-->
<script setup lang="ts">
/**
 * 自定义下拉选择框组件
 *
 * - 触发器：32px 高，14px 字号，#f2f3f5 背景，2px 圆角，focus 时边框变蓝
 * - 下拉面板：#fff 背景，1px #e5e6eb 边框，4px 圆角，0 4px 10px 阴影
 * - 选项：36px 行高，hover #f2f3f5 背景，选中 font-weight 500
 * - 动画：scaleY 0.9→1 + opacity，0.2s cubic-bezier(0.34,0.69,0.1,1)
 */

import { ref, computed, onMounted, onUnmounted } from 'vue'

interface SelectOption {
  label: string
  value: string | number
  [key: string]: any
}

interface Props {
  modelValue: string | number
  options: SelectOption[]
  placeholder?: string
  /** 是否禁用 */
  disabled?: boolean
  /** 是否使用自定义选项渲染（启用后选项高度自适应，不再固定 36px） */
  customOption?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: '请选择',
  disabled: false,
  customOption: false,
})

const emit = defineEmits<{ 'update:modelValue': [value: string | number] }>()

const open = ref(false)
const closing = ref(false)
const triggerRef = ref<HTMLElement | null>(null)
const dropdownStyle = ref<Record<string, string>>({})
const openUpward = ref(false)

const selectedLabel = computed(
  () => props.options.find(o => o.value === props.modelValue)?.label || props.placeholder,
)

function select(value: string | number) {
  emit('update:modelValue', value)
  open.value = false
}

function updateDropdownPosition() {
  if (!triggerRef.value) return
  const rect = triggerRef.value.getBoundingClientRect()
  const viewportH = window.innerHeight
  const viewportW = window.innerWidth
  // 下拉面板 max-height 是 200px
  const dropdownMaxH = 200
  const gap = 4

  const spaceBelow = viewportH - rect.bottom - gap
  const spaceAbove = rect.top - gap

  if (spaceBelow >= dropdownMaxH || spaceBelow >= spaceAbove) {
    openUpward.value = false
    dropdownStyle.value = {
      position: 'fixed',
      top: `${rect.bottom + gap}px`,
      left: `${rect.left}px`,
      minWidth: `${rect.width}px`,
      width: 'max-content',
      maxWidth: `${viewportW - rect.left - 8}px`,
      zIndex: '9999',
      transformOrigin: 'top',
    }
  } else {
    openUpward.value = true
    dropdownStyle.value = {
      position: 'fixed',
      bottom: `${viewportH - rect.top + gap}px`,
      left: `${rect.left}px`,
      minWidth: `${rect.width}px`,
      width: 'max-content',
      maxWidth: `${viewportW - rect.left - 8}px`,
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
}

function onScroll(e: Event) {
  if (!open.value || closing.value) return
  const target = e.target as Node
  const dropdown = document.querySelector('.select-dropdown')
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
  if (triggerRef.value && !triggerRef.value.contains(e.target as Node)) {
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
  <div ref="triggerRef" class="custom-select">
    <!-- 触发器：可通过 #trigger 完全自定义，或通过 #selected 自定义触发器内容 -->
    <slot name="trigger" :label="selectedLabel" :open="open" :toggle="toggle">
      <div
        class="select-trigger"
        :class="{ active: open, disabled: disabled }"
        @click="toggle"
      >
        <span v-if="$slots.selected" class="select-value">
          <slot name="selected" :label="selectedLabel" />
        </span>
        <span v-else class="select-value" :class="{ placeholder: !options.find(o => o.value === modelValue) }">
          {{ selectedLabel }}
        </span>
        <svg
          class="select-arrow"
          :class="{ rotated: open }"
          viewBox="0 0 1024 1024"
          fill="currentColor"
        >
          <path d="M512 714.666667c-8.533333 0-17.066667-2.133333-23.466667-8.533334l-341.333333-341.333333c-12.8-12.8-12.8-32 0-44.8 12.8-12.8 32-12.8 44.8 0L512 637.866667l320-320c12.8-12.8 32-12.8 44.8 0 12.8 12.8 12.8 32 0 44.8l-341.333333 341.333333c-6.4 6.4-14.933333 10.666667-23.466667 10.666667z" />
        </svg>
      </div>
    </slot>

    <!-- 下拉面板 -->
    <teleport to="body">
      <transition
        enter-active-class="select-enter-active"
        :enter-from-class="openUpward ? 'select-enter-from-up' : 'select-enter-from'"
        enter-to-class="select-enter-to"
        leave-active-class="select-leave-active"
        :leave-from-class="openUpward ? 'select-leave-from-up' : 'select-leave-from'"
        :leave-to-class="openUpward ? 'select-leave-to-up' : 'select-leave-to'"
      >
        <div v-if="open" class="select-dropdown" :style="dropdownStyle">
          <!-- 选项列表（可滚动） -->
          <div class="select-options-wrapper">
            <div
              v-for="opt in options"
              :key="opt.value"
              class="select-option"
              :class="{ selected: opt.value === modelValue, 'select-option-custom': customOption }"
              @click="select(opt.value)"
            >
              <slot name="option" :option="opt" :selected="opt.value === modelValue">
                <span class="select-option-content">{{ opt.label }}</span>
                <svg
                  v-if="opt.value === modelValue"
                  class="select-check-icon"
                  viewBox="0 0 1024 1024"
                  fill="currentColor"
                >
                  <path d="M912 192c-12.8 0-25.6 4.266667-34.133333 12.8L384 699.2 234.666667 548.266667c-17.066667-17.066667-46.933333-17.066667-64 0-17.066667 17.066667-17.066667 46.933333 0 64l179.2 179.2c8.533333 8.533333 21.333333 12.8 34.133333 12.8s25.6-4.266667 34.133333-12.8l520.533334-520.533334c17.066667-17.066667 17.066667-46.933333 0-64-8.533333-8.533333-21.333333-12.8-34.133334-12.8z" />
                </svg>
              </slot>
            </div>
            <div v-if="options.length === 0" class="select-empty">
              <slot name="empty">无选项</slot>
            </div>
          </div>
          <!-- 底部额外内容（如"下载新版本"按钮） -->
          <slot name="footer" />
        </div>
      </transition>
    </teleport>
  </div>
</template>

<style scoped>
/* ============================================================
 * 触发器
 * height: 32px, font-size: 14px, bg: #f2f3f5, border-radius: 2px
 * ============================================================ */
.custom-select {
  position: relative;
  min-width: 100px;
  max-width: 100%;
}

.select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  /* padding 0 12px, height 32px → 垂直 padding ≈ 4px (32 - 2*border - line-height*font-size) / 2 */
  height: 32px;
  padding: 0 12px;
  box-sizing: border-box;
  /* 文字色 #1d2129, 字号 14px */
  color: #1d2129;
  font-size: 14px;
  /* 背景 #f2f3f5, 边框透明 */
  background-color: #f2f3f5;
  border: 1px solid transparent;
  /* 圆角 2px */
  border-radius: 2px;
  cursor: pointer;
  user-select: none;
  max-width: 100%;
  overflow: hidden;
  /* 过渡 0.1s */
  transition: color 0.1s cubic-bezier(0, 0, 1, 1),
    border-color 0.1s cubic-bezier(0, 0, 1, 1),
    background-color 0.1s cubic-bezier(0, 0, 1, 1);
}

/* hover：背景 #e5e6eb */
.select-trigger:hover {
  background-color: #e5e6eb;
  border-color: transparent;
}

/* 展开时：背景 #fff, 边框主色 */
.select-trigger.active {
  background-color: #fff;
  border-color: var(--color-primary-500);
  /* 无外扩阴影，仅边框变色 */
  box-shadow: none;
}

/* 禁用态：文字 #c9cdd4, 背景 #f2f3f5 */
.select-trigger.disabled {
  color: #c9cdd4;
  background-color: #f2f3f5;
  border-color: transparent;
  cursor: not-allowed;
}

.select-value {
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.5715;
}

.select-value.placeholder {
  color: #86909c;
}

.select-arrow {
  width: 12px;
  height: 12px;
  color: #86909c;
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.select-arrow.rotated {
  transform: rotate(180deg);
}

/* ============================================================
 * 下拉面板
 * bg: #fff, border 1px #e5e6eb, radius 4px, shadow 0 4px 10px
 * padding: 4px 0, max-height 200px
 * ============================================================ */
.select-dropdown {
  background-color: #fff;
  border: 1px solid #e5e6eb;
  border-radius: 4px;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.1);
  padding: 4px 0;
  box-sizing: border-box;
}

.select-options-wrapper {
  max-height: 200px;
  overflow-y: auto;
}

/* ============================================================
 * 选项
 * line-height: 36px, padding 0 12px, font-size 14px
 * hover bg: #f2f3f5, selected font-weight 500
 * ============================================================ */
.select-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  box-sizing: border-box;
  padding: 0 12px;
  /* line-height 36px */
  line-height: 36px;
  height: 36px;
  font-size: 14px;
  color: #1d2129;
  text-align: left;
  background-color: transparent;
  cursor: pointer;
  transition: all 0.1s cubic-bezier(0, 0, 1, 1);
}

.select-option:hover {
  color: #1d2129;
  background-color: #f2f3f5;
}

/* 选中态：font-weight 500, 背景透明, 颜色不变 */
.select-option.selected {
  color: #1d2129;
  font-weight: 500;
  background-color: transparent;
}

/* 自定义选项模式：高度自适应，支持多行内容（主标题+描述等） */
.select-option-custom {
  height: auto;
  min-height: 36px;
  line-height: 1.5715;
  padding: 6px 12px; /* 垂直 padding 让多行内容有呼吸空间 */
  align-items: flex-start; /* 多行内容顶部对齐 */
}

.select-option-content {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  min-width: 0;
}

.select-check-icon {
  width: 12px;
  height: 12px;
  color: var(--color-primary-500);
  flex-shrink: 0;
}

.select-empty {
  padding: 8px 12px;
  text-align: center;
  font-size: 14px;
  color: #86909c;
}

/* ============================================================
 * 动画
 * transform: scaleY 0.9→1, opacity 0→1
 * 0.2s cubic-bezier(0.34, 0.69, 0.1, 1)
 * ============================================================ */
.select-enter-active {
  transition: transform 0.2s cubic-bezier(0.34, 0.69, 0.1, 1),
    opacity 0.2s cubic-bezier(0.34, 0.69, 0.1, 1);
}

.select-enter-from {
  transform: scaleY(0.9);
  opacity: 0;
}

.select-enter-from-up {
  transform: scaleY(0.9);
  opacity: 0;
}

.select-enter-to {
  transform: scaleY(1);
  opacity: 1;
}

.select-leave-active {
  transition: transform 0.2s cubic-bezier(0.34, 0.69, 0.1, 1),
    opacity 0.2s cubic-bezier(0.34, 0.69, 0.1, 1);
}

.select-leave-from {
  transform: scaleY(1);
  opacity: 1;
}

.select-leave-from-up {
  transform: scaleY(1);
  opacity: 1;
}

.select-leave-to {
  transform: scaleY(0.9);
  opacity: 0;
}

.select-leave-to-up {
  transform: scaleY(0.9);
  opacity: 0;
}
</style>
