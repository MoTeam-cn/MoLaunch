<script setup lang="ts">
/**
 * 开发者 - 系统信息子页签
 *
 * 展示应用版本、操作系统、架构、内存等系统信息。
 * 数据由父组件 SettingsDeveloper.vue 统一加载后通过 props 下发。
 */
import { computed } from 'vue'
import type { SystemInfo } from '@/utils/api/developer'
import { formatBytes } from '@/utils/format'
import { osDisplay, archDisplay } from '@/utils/system-display'

const props = defineProps<{
  systemInfo: SystemInfo | null
}>()

/** 系统信息卡片条目（key 用于 v-for 稳定 key） */
const systemEntries = computed<{ key: string; label: string; value: string }[]>(() => {
  if (!props.systemInfo) return []
  const s = props.systemInfo
  return [
    { key: 'appVersion', label: '应用版本', value: 'v' + s.appVersion },
    { key: 'os', label: '操作系统', value: osDisplay(s.os) },
    { key: 'arch', label: '架构', value: archDisplay(s.arch) },
    { key: 'bit', label: '位数', value: s.is64bit ? '64 位' : '32 位' },
    { key: 'total', label: '总内存', value: formatBytes(s.totalMemory) },
    { key: 'used', label: '已用内存', value: formatBytes(s.usedMemory) },
    { key: 'avail', label: '可用内存', value: formatBytes(s.availableMemory) },
    { key: 'usage', label: '内存使用率', value: s.memoryUsagePercent.toFixed(1) + '%' },
  ]
})
</script>

<template>
  <div v-if="systemInfo" class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">系统信息</h3>
    <div class="divide-y divide-gray-200">
      <div
        v-for="entry in systemEntries"
        :key="entry.key"
        class="px-5 py-3 flex items-center justify-between"
      >
        <span class="text-sm text-gray-500">{{ entry.label }}</span>
        <span class="text-sm text-gray-900 font-mono">{{ entry.value }}</span>
      </div>
    </div>
  </div>
</template>
