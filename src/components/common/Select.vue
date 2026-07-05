<script setup lang="ts">
/**
 * 自定义下拉选择框组件
 */

import { ref, computed, onMounted, onUnmounted } from 'vue'

interface Props {
  modelValue: string | number
  options: { label: string; value: string | number }[]
  placeholder?: string
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: '请选择',
})

const emit = defineEmits<{ 'update:modelValue': [value: string | number] }>()

const open = ref(false)
const closing = ref(false)
const triggerRef = ref<HTMLElement | null>(null)
const dropdownStyle = ref({})
const openUpward = ref(false)

const selectedLabel = computed(() => props.options.find(o => o.value === props.modelValue)?.label || props.placeholder)

function select(value: string | number) {
  emit('update:modelValue', value)
  open.value = false
}

function updateDropdownPosition() {
  if (!triggerRef.value) return
  const rect = triggerRef.value.getBoundingClientRect()
  const viewportH = window.innerHeight
  const dropdownMaxH = 240
  const gap = 4

  const spaceBelow = viewportH - rect.bottom - gap
  const spaceAbove = rect.top - gap

  if (spaceBelow >= dropdownMaxH || spaceBelow >= spaceAbove) {
    openUpward.value = false
    dropdownStyle.value = {
      position: 'fixed',
      top: `${rect.bottom + gap}px`,
      left: `${rect.left}px`,
      width: `${rect.width}px`,
      zIndex: 9999,
    }
  } else {
    openUpward.value = true
    dropdownStyle.value = {
      position: 'fixed',
      bottom: `${viewportH - rect.top + gap}px`,
      left: `${rect.left}px`,
      width: `${rect.width}px`,
      zIndex: 9999,
    }
  }
}

function toggle() {
  if (!open.value) {
    updateDropdownPosition()
    window.addEventListener('scroll', onScroll, true)
  } else {
    window.removeEventListener('scroll', onScroll, true)
  }
  open.value = !open.value
}

function onScroll() {
  if (open.value && !closing.value) {
    closing.value = true
    window.removeEventListener('scroll', onScroll, true)
    requestAnimationFrame(() => {
      open.value = false
      // 等过渡动画结束再重置标记（动画时长 100ms）
      setTimeout(() => { closing.value = false }, 150)
    })
  }
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
    <!-- 触发器 -->
    <div
      class="select-trigger"
      :class="{ active: open }"
      @click="toggle"
    >
      <span class="select-value">{{ selectedLabel }}</span>
      <svg
        class="select-arrow"
        :class="{ rotated: open }"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
      </svg>
    </div>

    <!-- 下拉面板 -->
    <teleport to="body">
      <transition
        :enter-active-class="openUpward
          ? 'transition ease-out duration-150'
          : 'transition ease-out duration-150'"
        :enter-from-class="openUpward
          ? 'opacity-0 translate-y-2'
          : 'opacity-0 -translate-y-2'"
        :enter-to-class="openUpward
          ? 'opacity-100 translate-y-0'
          : 'opacity-100 translate-y-0'"
        :leave-active-class="'transition ease-in duration-100'"
        :leave-from-class="'opacity-100 translate-y-0'"
        :leave-to-class="openUpward
          ? 'opacity-0 translate-y-2'
          : 'opacity-0 -translate-y-2'"
      >
        <div v-if="open" class="select-dropdown" :style="dropdownStyle">
          <div
            v-for="opt in options"
            :key="opt.value"
            class="select-option"
            :class="{ selected: opt.value === modelValue }"
            @click="select(opt.value)"
          >
            <span>{{ opt.label }}</span>
            <svg v-if="opt.value === modelValue" class="w-4 h-4 text-primary-500" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M16.704 4.153a.75.75 0 01.143 1.052l-8 10.5a.75.75 0 01-1.127.075l-4.5-4.5a.75.75 0 011.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 011.05-.143z" clip-rule="evenodd" />
            </svg>
          </div>
        </div>
      </transition>
    </teleport>
  </div>
</template>

<style scoped>
.custom-select {
  position: relative;
  min-width: 120px;
}

.select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 7px 12px;
  background: #f9fafb;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
  user-select: none;
}

.select-trigger:hover {
  border-color: #9ca3af;
}

.select-trigger.active {
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.select-value {
  font-size: 13px;
  color: #1f2937;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.select-arrow {
  width: 16px;
  height: 16px;
  color: #9ca3af;
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.select-arrow.rotated {
  transform: rotate(180deg);
}

.select-dropdown {
  background: #fff;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  max-height: 240px;
  overflow-y: auto;
  transform-origin: top;
}

.select-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  font-size: 13px;
  color: #374151;
  cursor: pointer;
  transition: background 0.1s ease;
}

.select-option:first-child {
  border-radius: 8px 8px 0 0;
}

.select-option:last-child {
  border-radius: 0 0 8px 8px;
}

.select-option:hover {
  background: #f3f4f6;
}

.select-option.selected {
  background: #eff6ff;
  color: #1d4ed8;
}
</style>
