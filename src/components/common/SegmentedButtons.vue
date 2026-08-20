<script setup lang="ts">
/**
 * 分段按钮组
 *
 * 复用项目 Button 组件渲染分段切换；选中态通过 Tailwind utilities 层类名覆盖
 * （utilities 优先级高于 components 层 .btn-*，故 hover/active 表现确定）。
 *
 * 支持两种使用方式：
 * 1. v-model 绑定（直接赋值）
 * 2. @select 事件回调（处理自定义逻辑，如保存）
 *
 * @example
 * <SegmentedButtons v-model="proxyMode" :options="[
 *   { label: '不使用代理', value: 'none' },
 *   { label: '系统代理', value: 'system' },
 *   { label: '自定义', value: 'custom' },
 * ]" />
 */
import { computed, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))

export interface SegmentedOption<T = string | number | boolean> {
  label: string
  value: T
  /** 可选：禁用该项 */
  disabled?: boolean
}

const props = withDefaults(defineProps<{
  /** 当前选中值 */
  modelValue?: string | number | boolean | null
  /** 选项列表 */
  options: SegmentedOption[]
  /** 每个按钮的额外 class（如 'flex-1' 让按钮等宽填充） */
  buttonClass?: string
}>(), {
  modelValue: null,
  buttonClass: '',
})

const emit = defineEmits<{
  'update:modelValue': [value: string | number | boolean]
  select: [value: string | number | boolean]
}>()

const currentValue = computed(() => props.modelValue)

function handleClick(option: SegmentedOption) {
  if (option.disabled) return
  emit('update:modelValue', option.value)
  emit('select', option.value)
}
</script>

<template>
  <div class="flex gap-2">
    <Button
      v-for="opt in options"
      :key="String(opt.value)"
      type="ghost"
      size="mini"
      :disabled="opt.disabled"
      class="rounded-md disabled:opacity-50"
      :class="[
        currentValue === opt.value
          ? 'border-primary-500 bg-primary-50 text-primary-700'
          : 'border-gray-200 text-gray-600 hover:border-gray-300',
        buttonClass,
      ]"
      @click="handleClick(opt)"
    >{{ opt.label }}</Button>
  </div>
</template>
