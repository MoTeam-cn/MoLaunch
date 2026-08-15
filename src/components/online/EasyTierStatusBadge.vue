<script setup lang="ts">
/**
 * easytier 连接状态徽章（房主/房客面板共用）
 *
 * 消费 useEasyTier 的状态，展示「组网中 / 已组网 / 断开中 / 组网失败 / 未组网」。
 */
import { computed } from 'vue'
import { useEasyTier, type EasyTierStatus } from '@/composables/useEasyTier'

const easytier = useEasyTier()

const statusText = computed(() => {
  switch (easytier.status.value) {
    case 'joined': return '已组网'
    case 'joining': return '组网中…'
    case 'error': return '组网失败'
    case 'stopping': return '断开中…'
    default: return '未组网'
  }
})

const statusClass = computed(() => {
  switch (easytier.status.value as EasyTierStatus) {
    case 'joined': return 'bg-green-50 text-green-700'
    case 'joining': return 'bg-blue-50 text-blue-700'
    case 'error': return 'bg-red-50 text-red-700'
    case 'stopping': return 'bg-yellow-50 text-yellow-700'
    default: return 'bg-gray-50 text-gray-500'
  }
})
</script>

<template>
  <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium" :class="statusClass">
    {{ statusText }}
  </span>
</template>
