<script setup lang="ts">
/**
 * 提示框组件
 * 用于显示各种类型的提示信息（info / warning / error / debug）
 *
 * 用法：<Alert type="warning" message="提示文字" />
 * 截断模式（默认）：单行显示，超出截断
 * 换行模式：传 :truncate="false" 允许换行完整显示
 */

import { ExclamationTriangleIcon, InformationCircleIcon, CheckCircleIcon, ExclamationCircleIcon, BugAntIcon } from '@heroicons/vue/24/outline'

interface Props {
  type?: 'info' | 'warning' | 'error' | 'success' | 'debug'
  message: string
  /** 是否单行截断（默认 true）；传 false 允许换行完整显示 */
  truncate?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  type: 'info',
  truncate: true,
})

const typeConfig = {
  info: {
    color: 'bg-blue-500',
    icon: InformationCircleIcon,
    iconColor: 'text-blue-500',
  },
  warning: {
    color: 'bg-amber-500',
    icon: ExclamationTriangleIcon,
    iconColor: 'text-amber-500',
  },
  error: {
    color: 'bg-red-500',
    icon: ExclamationCircleIcon,
    iconColor: 'text-red-500',
  },
  success: {
    color: 'bg-green-500',
    icon: CheckCircleIcon,
    iconColor: 'text-green-500',
  },
  debug: {
    color: 'bg-cyan-500',
    icon: BugAntIcon,
    iconColor: 'text-cyan-500',
  },
}

const config = typeConfig[props.type]
</script>

<template>
  <div
    class="flex rounded-r-lg bg-white shadow-sm overflow-hidden"
    :class="truncate ? 'items-center h-8' : 'items-start min-h-8'"
  >
    <div class="w-[3px] self-stretch shrink-0" :class="config.color"></div>
    <component
      :is="config.icon"
      class="w-4 h-4 ml-2.5 shrink-0"
      :class="[config.iconColor, truncate ? '' : 'mt-2']"
    />
    <span
      class="text-[13px] font-medium text-gray-800 ml-2 mr-3"
      :class="truncate ? 'truncate' : 'py-2'"
    >{{ message }}</span>
  </div>
</template>
