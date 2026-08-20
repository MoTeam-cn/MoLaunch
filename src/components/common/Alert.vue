<script setup lang="ts">
/**
 * 提示框组件（统一入口，原 Alert / AlertV2 合并）
 *
 * 两种风格：
 * - variant="bar"（默认）：白底左色条 + Heroicons 图标，支持单行截断/换行
 * - variant="soft"：灰底简洁 + Element Plus Icons，支持插槽与外部链接
 *
 * 用法：<Alert type="warning" message="提示文字" />
 *       <Alert variant="soft" type="info" message="提示文字" />
 */

import { ExclamationTriangleIcon, InformationCircleIcon, CheckCircleIcon, ExclamationCircleIcon, BugAntIcon } from '@heroicons/vue/24/outline'
import { elementIcons } from '@/utils/element-icons'
import { open as openUrl } from '@tauri-apps/plugin-shell'
import { toastInfo } from '@/utils/toast'

interface Props {
  type?: 'info' | 'warning' | 'error' | 'success' | 'debug'
  message?: string
  /** 是否单行截断（默认 true，仅 bar 风格生效）；传 false 允许换行完整显示 */
  truncate?: boolean
  /** 外部链接配置（soft 风格）：slot 内通过 openLink 触发，点击后 toast 提示并延迟 0.5s 跳转 */
  link?: { url: string; toast?: string }
  /** 风格：bar = 白底左色条（原 Alert）；soft = 灰底简洁（原 AlertV2） */
  variant?: 'bar' | 'soft'
}

const props = withDefaults(defineProps<Props>(), {
  type: 'info',
  message: '',
  truncate: true,
  link: undefined,
  variant: 'bar',
})

const barConfig = {
  info: { color: 'bg-blue-500', icon: InformationCircleIcon, iconColor: 'text-blue-500' },
  warning: { color: 'bg-amber-500', icon: ExclamationTriangleIcon, iconColor: 'text-amber-500' },
  error: { color: 'bg-red-500', icon: ExclamationCircleIcon, iconColor: 'text-red-500' },
  success: { color: 'bg-green-500', icon: CheckCircleIcon, iconColor: 'text-green-500' },
  debug: { color: 'bg-cyan-500', icon: BugAntIcon, iconColor: 'text-cyan-500' },
}[props.type]

const softConfig = {
  info: { icon: elementIcons.info, iconColor: 'text-gray-400' },
  warning: { icon: elementIcons.warning, iconColor: 'text-amber-400' },
  error: { icon: elementIcons.error, iconColor: 'text-red-400' },
  success: { icon: elementIcons.success, iconColor: 'text-green-400' },
  debug: { icon: elementIcons.debug, iconColor: 'text-cyan-400' },
}[props.type]

function handleLinkClick() {
  const link = props.link
  if (!link) return
  toastInfo(link.toast ?? '正在打开外部链接…')
  setTimeout(() => {
    openUrl(link.url)
  }, 500)
}
</script>

<template>
  <!-- bar 风格：白底左色条 -->
  <div
    v-if="variant === 'bar'"
    class="flex rounded-r-lg bg-white shadow-sm overflow-hidden"
    :class="truncate ? 'items-center h-8' : 'items-start min-h-8'"
  >
    <div class="w-[3px] self-stretch shrink-0" :class="barConfig.color"></div>
    <component
      :is="barConfig.icon"
      class="w-4 h-4 ml-2.5 shrink-0"
      :class="[barConfig.iconColor, truncate ? '' : 'mt-2']"
    />
    <span
      class="text-[13px] font-medium text-gray-800 ml-2 mr-3"
      :class="truncate ? 'truncate' : 'py-2'"
    >{{ message }}</span>
  </div>

  <!-- soft 风格：灰底简洁 -->
  <div
    v-else
    class="flex items-start gap-2 rounded-md bg-gray-50 p-2.5 text-xs text-gray-500"
  >
    <svg
      class="mt-0.5 h-3.5 w-3.5 shrink-0"
      :class="softConfig.iconColor"
      :viewBox="softConfig.icon.viewBox"
      fill="currentColor"
    >
      <path :d="softConfig.icon.path" />
    </svg>
    <div class="min-w-0 flex-1 leading-relaxed">
      <slot :open-link="handleLinkClick">{{ message }}</slot>
    </div>
  </div>
</template>