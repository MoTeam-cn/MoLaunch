<!--
  MoLaunch - Minecraft Launcher
  Copyright (C) 2026 MoTeam

  This file is derived from Arco Design Vue (https://arco.design/).
  Original code licensed under the MIT License.

  MIT License full text will be added here
-->
<script setup lang="ts">
/**
 * 输入框组件
 *
 * 特性：
 * - 灰底无边框（default #f2f3f5），focus 时白底蓝边框
 * - 支持 prefix / suffix 插槽（图标等）
 * - 支持前后置标签（prepend / append）
 * - 支持 clearable 清除按钮
 *
 * 用法：
 * <Input v-model="text" placeholder="请输入" />
 * <Input v-model="keyword" placeholder="搜索...">
 *   <template #prefix><SearchIcon /></template>
 * </Input>
 * <Input v-model="value" clearable @clear="onClear" />
 */
import { computed } from 'vue'

interface Props {
  modelValue: string | number
  placeholder?: string
  type?: string
  disabled?: boolean
  readonly?: boolean
  clearable?: boolean
  maxlength?: number
  size?: 'mini' | 'small' | 'default' | 'large'
  /** 是否渲染为 textarea（文本域） */
  textarea?: boolean
  /** textarea 的行数（仅 textarea 模式生效） */
  rows?: number
  /** textarea 是否允许用户调整大小（仅 textarea 模式生效） */
  resize?: 'none' | 'vertical' | 'horizontal' | 'both'
  /** datalist 的 id（用于输入框自动补全，透传到内部 input 的 list 属性） */
  list?: string
  /** 自定义宽度 CSS 值（如 '200px'、'50%'），传入后会覆盖默认 100% 宽度 */
  width?: string
  /** 输入框下方提示文字（不传则不渲染） */
  hint?: string
  /** 提示类型：default=灰色、error=红色、success=绿色 */
  hintType?: 'default' | 'error' | 'success'
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: '',
  type: 'text',
  disabled: false,
  readonly: false,
  clearable: false,
  size: 'default',
  textarea: false,
  rows: 3,
  resize: 'vertical',
  list: undefined,
  width: undefined,
  hint: undefined,
  hintType: 'default',
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  input: [value: string]
  change: [value: string]
  focus: [e: FocusEvent]
  blur: [e: FocusEvent]
  clear: []
  keydown: [e: KeyboardEvent]
}>()

const inputValue = computed({
  get: () => props.modelValue,
  set: (val) => {
    const str = String(val ?? '')
    emit('update:modelValue', str)
    emit('input', str)
  },
})

function onInput(e: Event) {
  const target = e.target as HTMLInputElement
  inputValue.value = target.value
}

function onChange(e: Event) {
  const target = e.target as HTMLInputElement
  emit('change', target.value)
}

function onClear() {
  inputValue.value = ''
  emit('clear')
}

const sizeClass = computed(() => `input-size-${props.size}`)
</script>

<template>
  <!-- 外层包裹：参考 Arco FormItem 的 wrapper-col 结构，承载输入框 + 下方提示文字 -->
  <span class="input-root">
    <!-- textarea 模式 -->
    <div
      v-if="textarea"
      class="input-wrapper textarea-wrapper"
      :class="{ 'input-disabled': disabled, 'input-readonly': readonly }"
    >
      <textarea
        v-model="inputValue"
        :rows="rows"
        :placeholder="placeholder"
        :disabled="disabled"
        :readonly="readonly"
        :maxlength="maxlength"
        class="textarea-inner"
        :style="{ resize }"
        @input="onInput"
        @change="onChange"
        @focus="$emit('focus', $event)"
        @blur="$emit('blur', $event)"
        @keydown="$emit('keydown', $event)"
      />
    </div>

    <!-- input 模式 -->
    <div
      v-else
      class="input-wrapper"
      :class="[sizeClass, { 'input-disabled': disabled, 'input-readonly': readonly }]"
      :style="width ? { width } : undefined"
    >
      <!-- 前置标签 -->
      <div v-if="$slots.prepend" class="input-prepend">
        <slot name="prepend" />
      </div>

      <!-- prefix 图标 -->
      <div v-if="$slots.prefix" class="input-prefix">
        <slot name="prefix" />
      </div>

      <!-- 输入框 -->
      <input
        v-model="inputValue"
        :type="type"
        :placeholder="placeholder"
        :disabled="disabled"
        :readonly="readonly"
        :maxlength="maxlength"
        :list="list"
        class="input-inner"
        @input="onInput"
        @change="onChange"
        @focus="$emit('focus', $event)"
        @blur="$emit('blur', $event)"
        @keydown="$emit('keydown', $event)"
      >

      <!-- clear 按钮 -->
      <div
        v-if="clearable && inputValue && !disabled && !readonly"
        class="input-clear"
        @click="onClear"
      >
        <svg viewBox="0 0 1024 1024" fill="currentColor">
          <path d="M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64zm165.4 549.4l-45.2 45.2L512 557.2l-120.2 101.4-45.2-45.2L466.8 512 365.6 391.8l45.2-45.2L512 466.8l120.2-101.4 45.2 45.2L577.2 512l101.2 101.4z" />
        </svg>
      </div>

      <!-- suffix 图标 -->
      <div v-if="$slots.suffix" class="input-suffix">
        <slot name="suffix" />
      </div>

      <!-- 后置标签 -->
      <div v-if="$slots.append" class="input-append">
        <slot name="append" />
      </div>
    </div>

    <!-- 提示文字（参考 Arco FormItemMessage：min-height 防抖动 + transition 动画）-->
    <transition name="input-hint">
      <div
        v-if="hint"
        class="input-hint"
        :class="`input-hint-${hintType}`"
        role="alert"
      >
        {{ hint }}
      </div>
    </transition>
  </span>
</template>

<style scoped>
/* 外层根元素：参考 Arco FormItem 的 wrapper-col 包裹输入框 + 提示文字 */
.input-root {
  display: inline-block;
  width: 100%;
  vertical-align: top;
}

/* 输入框 wrapper */
.input-wrapper {
  display: inline-flex;
  align-items: center;
  box-sizing: border-box;
  width: 100%;
  height: 32px;
  padding: 0 12px;
  font-size: 14px;
  line-height: 1.5715;
  color: #1d2129;
  background-color: #f2f3f5;
  border: 1px solid transparent;
  border-radius: 2px;
  transition: color 0.1s cubic-bezier(0, 0, 1, 1),
    border-color 0.1s cubic-bezier(0, 0, 1, 1),
    background-color 0.1s cubic-bezier(0, 0, 1, 1);
}

.input-wrapper:hover:not(.input-disabled) {
  background-color: #e5e6eb;
}

/* focus-within：内部 input 聚焦时整个 wrapper 变白底主色边 */
.input-wrapper:focus-within {
  z-index: 1;
  background-color: #ffffff;
  border-color: var(--color-primary-500);
}

.input-disabled {
  color: #c9cdd4;
  background-color: #f2f3f5;
  cursor: not-allowed;
}

/* 内部 input */
.input-inner {
  flex: 1;
  min-width: 0;
  width: 100%;
  height: 100%;
  padding: 0;
  color: inherit;
  font-size: inherit;
  line-height: inherit;
  background: none;
  border: none;
  border-radius: 0;
  outline: none;
  -webkit-appearance: none;
}
.input-inner::placeholder {
  color: #86909c;
}
.input-inner:disabled {
  -webkit-text-fill-color: #c9cdd4;
  cursor: not-allowed;
}

/* prefix / suffix */
.input-prefix,
.input-suffix {
  display: inline-flex;
  align-items: center;
  flex-shrink: 0;
  white-space: nowrap;
  user-select: none;
  color: #4e5969;
}
.input-prefix {
  padding-right: 12px;
}
.input-suffix {
  padding-left: 12px;
}
.input-prefix :deep(svg),
.input-suffix :deep(svg) {
  width: 14px;
  height: 14px;
}

/* clear 按钮 */
.input-clear {
  display: inline-flex;
  align-items: center;
  flex-shrink: 0;
  padding-left: 8px;
  color: #4e5969;
  cursor: pointer;
}
.input-clear :deep(svg) {
  width: 12px;
  height: 12px;
}
.input-clear:hover {
  color: #c9cdd4;
}

/* prepend / append */
.input-prepend,
.input-append {
  display: inline-flex;
  align-items: center;
  flex-shrink: 0;
  padding: 0 12px;
  color: #1d2129;
  white-space: nowrap;
  background-color: #f2f3f5;
  border: 1px solid transparent;
}
.input-prepend {
  border-right: 1px solid #e5e6eb;
}
.input-append {
  border-left: 1px solid #e5e6eb;
}

/* 尺寸 */
.input-size-mini {
  height: 24px;
  font-size: 12px;
}
.input-size-small {
  height: 28px;
}
.input-size-default {
  height: 32px;
}
.input-size-large {
  height: 36px;
}

/* ============================================================
 * Textarea 模式
 * wrapper padding 0，textarea 自身 padding 4px 12px
 * min-height 32px, font-size 14px, line-height 1.5715
 * ============================================================ */
.textarea-wrapper {
  display: block;
  padding: 0;
  overflow: hidden;
  height: auto;
  min-height: 32px;
}

.textarea-inner {
  display: block;
  box-sizing: border-box;
  width: 100%;
  height: 100%;
  min-height: 32px;
  padding: 4px 12px;
  font-size: 14px;
  line-height: 1.5715;
  color: inherit;
  background: none;
  border: none;
  border-radius: 0;
  outline: none;
  -webkit-appearance: none;
  vertical-align: top;
}
.textarea-inner::placeholder {
  color: #86909c;
}
.textarea-inner:disabled {
  -webkit-text-fill-color: #c9cdd4;
  cursor: not-allowed;
}

/* 输入框下方提示文字（参考 Arco FormItemMessage：min-height 防抖动 + 透明度动画）*/
.input-hint {
  margin-top: 4px;
  min-height: 20px;
  font-size: 12px;
  line-height: 20px;
  color: #86909c;
  word-break: break-all;
}
.input-hint-error { color: #f53f3f; }
.input-hint-success { color: #00b42a; }
.input-hint-default { color: #86909c; }
.input-hint-enter-from, .input-hint-leave-to { opacity: 0; }
.input-hint-enter-active, .input-hint-leave-active { transition: opacity 0.2s ease; }
</style>
