<script setup lang="ts">
/**
 * 提示框组件 V2 —— 灰底简洁风格
 *
 * 浅灰背景 + Element Plus Icons + 文字，视觉轻量柔和。
 * 与 Alert.vue（Arco 白底左色条风格）互补，适合弹窗内提示。
 *
 * 图标来源：Element Plus Icons (MIT License)
 *
 * 用法：<AlertV2 type="info" message="提示文字" />
 * 支持 5 种类型：info / warning / error / success / debug
 * 传入默认插槽时渲染插槽内容（可含多行/列表），否则渲染 message。
 */

import { elementIcons } from '@/utils/element-icons'

interface Props {
  type?: 'info' | 'warning' | 'error' | 'success' | 'debug'
  message?: string
}

const props = withDefaults(defineProps<Props>(), {
  type: 'info',
  message: '',
})

const typeConfig = {
  info: { icon: elementIcons.info, iconColor: 'text-gray-400' },
  warning: { icon: elementIcons.warning, iconColor: 'text-amber-400' },
  error: { icon: elementIcons.error, iconColor: 'text-red-400' },
  success: { icon: elementIcons.success, iconColor: 'text-green-400' },
  debug: { icon: elementIcons.debug, iconColor: 'text-cyan-400' },
}

const config = typeConfig[props.type]
</script>

<template>
  <div class="flex items-start gap-2 rounded-md bg-gray-50 p-2.5 text-xs text-gray-500">
    <svg
      class="mt-0.5 h-3.5 w-3.5 shrink-0"
      :class="config.iconColor"
      :viewBox="config.icon.viewBox"
      fill="currentColor"
    >
      <path :d="config.icon.path" />
    </svg>
    <div class="min-w-0 flex-1 leading-relaxed">
      <slot>{{ message }}</slot>
    </div>
  </div>
</template>
