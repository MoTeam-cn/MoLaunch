<script setup lang="ts">
/**
 * 系统内存占用可视化条
 *
 * 三段比例条（系统已用 / 游戏分配 / 剩余）+ 顶部总览 + 底部图例。
 * 数据由调用方从 useMemoryVisualizer 取出后经 props 传入，本组件不持有数据源。
 */
import { defineAsyncComponent } from 'vue'
import { formatBytes, formatMemoryMB } from '@/utils/format'
import type { SystemMemory } from '@/composables/useMemoryVisualizer'
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))

defineProps<{
  systemMemory: SystemMemory
  usedMb: number
  gameMb: number
  otherMb: number
  usedPercent: number
  gamePercent: number
}>()
</script>

<template>
  <div>
    <div class="flex items-center justify-between text-xs text-gray-500 mb-2">
      <span>{{ formatBytes(systemMemory.total) }} 总内存</span>
      <span>{{ systemMemory.usage_percent.toFixed(0) }}% 已占用</span>
    </div>
    <div class="h-6 rounded-full overflow-hidden flex bg-gray-100">
      <div class="h-full flex" :style="{ width: usedPercent + '%' }">
        <Tooltip :text="'系统已用: ' + formatMemoryMB(usedMb)" position="top" block>
          <div class="h-full w-full bg-orange-400 transition-all duration-500" />
        </Tooltip>
      </div>
      <div class="h-full flex" :style="{ width: gamePercent + '%' }">
        <Tooltip :text="'游戏分配: ' + formatMemoryMB(gameMb)" position="top" block>
          <div class="h-full w-full bg-primary-500 transition-all duration-500" />
        </Tooltip>
      </div>
      <div class="h-full flex flex-1">
        <Tooltip :text="'剩余: ' + formatMemoryMB(otherMb)" position="top" block>
          <div class="h-full w-full bg-green-400 transition-all duration-500" />
        </Tooltip>
      </div>
    </div>
    <div class="flex items-center gap-4 mt-2 text-xs">
      <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-orange-400" /> 系统占用 {{ formatMemoryMB(usedMb) }}</span>
      <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-primary-500" /> 游戏分配 {{ formatMemoryMB(gameMb) }}</span>
      <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-green-400" /> 剩余 {{ formatMemoryMB(otherMb) }}</span>
    </div>
  </div>
</template>
