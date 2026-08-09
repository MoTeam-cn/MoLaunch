<script setup lang="ts">
/**
 * 开关行组件 - 统一 label + description + Tooltip + Toggle 按钮布局
 *
 * 复用场景：
 * - SettingsLaunch.vue 高级选项（3 个开关，无 tooltip）
 * - SetupTab.vue 进阶开关（5 个开关，带 tooltip）
 *
 * 通过 v-model 双向绑定开关状态；tooltipText 可选，未提供时不渲染 Tooltip。
 */
import Tooltip from '@/components/common/Tooltip.vue'

interface Props {
  /** 开关状态（v-model） */
  modelValue: boolean
  /** 主标题 */
  label: string
  /** 描述文本（可选） */
  description?: string
  /** Tooltip 提示文本（可选，不传则不渲染 Tooltip） */
  tooltipText?: string
  /** Tooltip 位置，默认 top */
  position?: 'top' | 'bottom' | 'left' | 'right'
  /** Tooltip 显示延迟（ms），默认 0 */
  delay?: number
  /** 是否启用 hover 高亮（与父级 divide 配合），默认 true */
  hover?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  description: '',
  tooltipText: '',
  position: 'top',
  delay: 0,
  hover: true,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

function toggle() {
  emit('update:modelValue', !props.modelValue)
}
</script>

<template>
  <!-- 有 tooltipText 时用 Tooltip 包裹（block 模式撑满父容器），否则直接渲染行 -->
  <component
    :is="tooltipText ? Tooltip : 'div'"
    v-bind="tooltipText ? { text: tooltipText, position, delay, block: true } : {}"
  >
    <div
      class="px-5 py-4 flex items-center justify-between transition-colors"
      :class="hover ? (tooltipText ? 'w-full hover:bg-gray-50' : 'hover:bg-gray-50') : ''"
    >
      <div class="min-w-0 mr-4">
        <p class="text-sm font-medium text-gray-900">{{ label }}</p>
        <p v-if="description" class="text-xs text-gray-500 mt-0.5">{{ description }}</p>
      </div>
      <!-- 保留原生 button：Toggle 开关（h-6 w-11 rounded-full + 滑块动画），
           Button.vue 的 scoped size 类与布局不适合开关组件 -->
      <button
        type="button"
        class="relative inline-flex h-6 w-11 flex-none items-center rounded-full transition-colors"
        :class="modelValue ? 'bg-primary-500' : 'bg-gray-300'"
        @click="toggle"
      >
        <span
          class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
          :class="modelValue ? 'translate-x-6' : 'translate-x-1'"
        />
      </button>
    </div>
  </component>
</template>
