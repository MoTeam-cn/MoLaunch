<script setup lang="ts">
/**
 * 提示框组件
 * 用于显示各种类型的提示信息
 */

import { ExclamationTriangleIcon, InformationCircleIcon, CheckCircleIcon } from '@heroicons/vue/24/outline'

interface Props {
  type?: 'warning' | 'info' | 'success'
  message: string
}

const props = withDefaults(defineProps<Props>(), {
  type: 'info',
})

const typeConfig = {
  warning: {
    color: 'bg-amber-500',
    icon: ExclamationTriangleIcon,
    iconColor: 'text-amber-500',
  },
  info: {
    color: 'bg-blue-500',
    icon: InformationCircleIcon,
    iconColor: 'text-blue-500',
  },
  success: {
    color: 'bg-green-500',
    icon: CheckCircleIcon,
    iconColor: 'text-green-500',
  },
}

const config = typeConfig[props.type]
</script>

<template>
  <div class="flex items-center h-8 rounded-r-lg bg-white shadow-sm overflow-hidden">
    <div class="w-[3px] h-full shrink-0" :class="config.color"></div>
    <component :is="config.icon" class="w-4 h-4 ml-2.5 shrink-0" :class="config.iconColor" />
    <span class="text-[13px] font-medium text-gray-800 ml-2 mr-3 truncate">{{ message }}</span>
  </div>
</template>
