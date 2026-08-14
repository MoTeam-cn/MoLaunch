<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
/**
 * 虚拟 IP 显示卡片行（阶段三走查重构）
 *
 * 从 RoomGuestPanel.vue / RoomHostPanel.vue 抽出的公共行组件：
 * 左侧图标 + 标签，右侧 IP 代码块 + 复制按钮。
 * 复制逻辑自包含，调用方无需传入 copyText。
 */
import { WifiIcon, ClipboardDocumentIcon } from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import { copyToClipboard } from '@/utils/clipboard'

defineProps<{
  ip: string
  label: string
}>()

async function copyIp(text: string) {
  if (!text) return
  await copyToClipboard(text, { toast: true })
}
</script>

<template>
  <div class="px-1 py-3 flex items-center justify-between">
    <div class="flex items-center gap-2 text-sm text-gray-600">
      <WifiIcon class="w-4 h-4 text-gray-400" />
      <span>{{ label }}</span>
    </div>
    <div class="flex items-center gap-1">
      <code class="text-xs text-gray-900 bg-gray-50 px-2 py-0.5 rounded">{{ ip }}</code>
      <Tooltip :text="`复制${label}`">
        <Button type="ghost" size="mini" @click="copyIp(ip)">
          <template #icon><ClipboardDocumentIcon class="w-3.5 h-3.5" /></template>
        </Button>
      </Tooltip>
    </div>
  </div>
</template>
