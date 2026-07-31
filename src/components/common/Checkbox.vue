<!--
  MoLaunch - Minecraft Launcher
  Copyright (C) 2026 MoTeam

  This file is derived from Arco Design Vue (https://arco.design/).
  Original code licensed under the MIT License.

  MIT License full text will be added here
-->
<script setup lang="ts">
/**
 * 复选框组件
 *
 * 用法：
 * <Checkbox v-model="checked">启用 TLS</Checkbox>
 * <Checkbox :checked="sel" :disabled="!en" @change="onChange">选项</Checkbox>
 */
import { computed, ref } from 'vue'

interface Props {
  /** v-model 绑定值（与 checked 二选一，优先 modelValue） */
  modelValue?: boolean
  /** 受控选中状态（非 v-model 场景） */
  checked?: boolean
  /** 默认是否选中（非受控） */
  defaultChecked?: boolean
  /** 是否禁用 */
  disabled?: boolean
  /** 是否半选状态 */
  indeterminate?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: undefined,
  checked: undefined,
  defaultChecked: false,
  disabled: false,
  indeterminate: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'change': [value: boolean, e: Event]
  'focus': [e: FocusEvent]
  'blur': [e: FocusEvent]
}>()

const inputRef = ref<HTMLInputElement>()
const _checked = ref(props.defaultChecked)

/** 统一选中状态：modelValue > checked > 内部 _checked */
const isChecked = computed(() => {
  if (props.modelValue !== undefined) return props.modelValue
  if (props.checked !== undefined) return props.checked
  return _checked.value
})

function onChange(e: Event) {
  const target = e.target as HTMLInputElement
  const val = target.checked
  _checked.value = val
  emit('update:modelValue', val)
  emit('change', val, e)
  // 受控模式下（外部传 :checked），nextTick 修正 input 的视觉状态
  // 避免 :checked 未更新时 input 显示与外部数据不一致
  queueMicrotask(() => {
    if (inputRef.value && inputRef.value.checked !== isChecked.value) {
      inputRef.value.checked = isChecked.value
    }
  })
}
</script>

<template>
  <label
    class="checkbox"
    :class="{
      'checkbox-checked': isChecked,
      'checkbox-indeterminate': indeterminate,
      'checkbox-disabled': disabled,
    }"
    :aria-disabled="disabled"
  >
    <input
      ref="inputRef"
      type="checkbox"
      class="checkbox-target"
      :checked="isChecked"
      :disabled="disabled"
      @change="onChange"
      @focus="emit('focus', $event)"
      @blur="emit('blur', $event)"
    />
    <span class="checkbox-icon-hover">
      <span class="checkbox-icon">
        <svg
          v-if="isChecked && !indeterminate"
          class="checkbox-icon-check"
          viewBox="0 0 1024 1024"
          fill="currentColor"
          aria-hidden="true"
        >
          <path d="M877.44815445 206.10060629a64.72691371 64.72691371 0 0 0-95.14856334 4.01306852L380.73381888 685.46812814 235.22771741 533.48933518a64.72691371 64.72691371 0 0 0-92.43003222-1.03563036l-45.82665557 45.82665443a64.72691371 64.72691371 0 0 0-0.90617629 90.61767965l239.61903446 250.10479331a64.72691371 64.72691371 0 0 0 71.19960405 15.14609778 64.33855261 64.33855261 0 0 0 35.08198741-21.23042702l36.24707186-42.71976334 40.5190474-40.77795556-3.36579926-3.49525333 411.40426297-486.74638962a64.72691371 64.72691371 0 0 0-3.88361443-87.64024149l-45.3088404-45.43829334z" />
        </svg>
      </span>
    </span>
    <span v-if="$slots.default" class="checkbox-label">
      <slot />
    </span>
  </label>
</template>

<style scoped>
/* 容器：inline-flex，左侧 7px padding 让 icon-hover 背景居中 */
.checkbox {
  position: relative;
  display: inline-flex;
  align-items: center;
  box-sizing: border-box;
  padding-left: 7px;
  font-size: 14px;
  line-height: unset;
  cursor: pointer;
  user-select: none;
}

/* 隐藏原生 input，保留焦点可达性 */
.checkbox-target {
  position: absolute;
  top: 0;
  left: 0;
  width: 0;
  height: 0;
  opacity: 0;
}
.checkbox-target:focus-visible + .checkbox-icon-hover::before {
  background-color: rgb(var(--color-primary-rgb-100) / 0.6);
}

/* icon-hover 背景：28x28 圆形浅色背景，hover/focus 时显示 */
.checkbox-icon-hover {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  flex-shrink: 0;
}
.checkbox-icon-hover::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 50%;
  background-color: transparent;
  transition: background-color 0.1s cubic-bezier(0, 0, 1, 1);
}
.checkbox:hover .checkbox-icon-hover::before {
  background-color: rgb(var(--color-primary-rgb-100) / 0.6);
}
.checkbox.checkbox-checked:hover .checkbox-icon-hover::before,
.checkbox.checkbox-indeterminate:hover .checkbox-icon-hover::before,
.checkbox.checkbox-disabled:hover .checkbox-icon-hover::before {
  background-color: transparent;
}

/* icon 方框：14x14，2px 边框，2px 圆角 */
.checkbox-icon {
  position: relative;
  box-sizing: border-box;
  width: 14px;
  height: 14px;
  background-color: #ffffff;
  border: 2px solid #c9cdd4;
  border-radius: 2px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: border-color 0.1s cubic-bezier(0, 0, 1, 1),
    transform 0.3s cubic-bezier(0.3, 1.1, 0.3, 1.1);
}
/* 半选横条 ::after */
.checkbox-icon::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  display: block;
  width: 6px;
  height: 2px;
  background: #ffffff;
  border-radius: 0.5px;
  transform: translateX(-50%) translateY(-50%) scale(0);
}

/* 勾选图标 */
.checkbox-icon-check {
  position: relative;
  display: block;
  width: 10px;
  height: 100%;
  color: #ffffff;
  transform: scale(0);
  transform-origin: center 75%;
}

/* hover 时边框变深 */
.checkbox:hover .checkbox-icon {
  border-color: #86909c;
}

/* 选中状态：主色底 + 透明边框 + 勾选图标 scale 1 */
.checkbox.checkbox-checked .checkbox-icon,
.checkbox.checkbox-indeterminate .checkbox-icon {
  background-color: var(--color-primary-500);
  border-color: transparent;
}
.checkbox.checkbox-checked .checkbox-icon-check {
  transform: scale(1);
  transition: transform 0.3s cubic-bezier(0.3, 1.1, 0.3, 1.1);
}
/* 半选状态：隐藏勾选图标，显示横条 */
.checkbox.checkbox-indeterminate .checkbox-icon-check {
  transform: scale(0);
}
.checkbox.checkbox-indeterminate .checkbox-icon::after {
  transform: translateX(-50%) translateY(-50%) scale(1);
  transition: transform 0.3s cubic-bezier(0.3, 1.1, 0.3, 1.1);
}
/* 选中/半选 hover 不变边框 */
.checkbox.checkbox-checked:hover .checkbox-icon,
.checkbox.checkbox-indeterminate:hover .checkbox-icon {
  border-color: transparent;
}

/* 文字标签 */
.checkbox-label {
  margin-left: 8px;
  color: #1d2129;
}

/* 禁用状态 */
.checkbox.checkbox-disabled {
  cursor: not-allowed;
}
.checkbox.checkbox-disabled .checkbox-icon-hover {
  cursor: not-allowed;
}
.checkbox.checkbox-disabled .checkbox-icon {
  background-color: #f2f3f5;
  border-color: #c9cdd4;
}
.checkbox.checkbox-disabled.checkbox-checked .checkbox-icon,
.checkbox.checkbox-disabled.checkbox-indeterminate .checkbox-icon {
  background-color: var(--color-primary-300);
  border-color: transparent;
}
.checkbox.checkbox-disabled:hover .checkbox-icon {
  border-color: #c9cdd4;
}
.checkbox.checkbox-disabled .checkbox-label {
  color: #c9cdd4;
}
.checkbox.checkbox-disabled .checkbox-icon-check {
  color: #c9cdd4;
}
.checkbox.checkbox-disabled:hover .checkbox-icon-hover::before {
  background-color: transparent;
}
</style>
