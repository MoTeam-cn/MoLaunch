<script setup lang="ts">
/**
 * easytier 虚拟网络在线设备列表（Scaffolding 收敛版）
 *
 * 每 5 秒经 `easytier_peers` IPC 查询虚拟网络节点（过滤中继，含本机），展示组网
 * 人数与各节点 hostname / 虚拟 IP / 延迟，房主与房客均可据此判断对方是否已组网。
 * 同时监听后端 `easytier-status` 事件（新成员加入触发），收到后立即刷新。
 */
import { onMounted, onUnmounted, ref, defineAsyncComponent } from 'vue'
import { getEasyTierPeers } from '@/utils/api/online-manager'
import { useTauriEvent } from '@/utils/tauriEvent'
import type { EasyTierPeer } from '@/types/online'
import { UsersIcon } from '@heroicons/vue/24/outline'
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))

/** 组网设备查询周期（5s，与房间内端口轮询一致） */
const REFRESH_INTERVAL_MS = 5000

const peers = ref<EasyTierPeer[]>([])
let timer: ReturnType<typeof setInterval> | null = null

async function refresh(): Promise<void> {
  try {
    peers.value = await getEasyTierPeers()
  } catch {
    // 查询失败静默等待下一轮（easytier 未运行 / CLI 偶发错误）
  }
}

// 后端检测到新成员加入时推送 easytier-status 事件，立即刷新组网列表
const { start: startPeersEvent } = useTauriEvent('easytier-status', () => void refresh())

onMounted(() => {
  startPeersEvent()
  void refresh()
  timer = setInterval(() => void refresh(), REFRESH_INTERVAL_MS)
})

onUnmounted(() => {
  if (timer) {
    clearInterval(timer)
    timer = null
  }
})
</script>

<template>
  <div class="pt-3 mt-3 border-t border-gray-100">
    <div class="flex items-center justify-between text-xs text-gray-500">
      <span>组网设备（{{ peers.length }}）</span>
      <span class="text-gray-400">每 5 秒自动刷新</span>
    </div>
    <div v-if="peers.length === 0" class="py-4 flex flex-col items-center justify-center gap-1.5 text-gray-400">
      <UsersIcon class="w-6 h-6" />
      <span class="text-xs">虚拟网络中暂无其他设备</span>
    </div>
    <ul v-else class="mt-2 space-y-1.5">
      <li v-for="p in peers" :key="p.virtualIp" class="flex items-center justify-between gap-2 text-xs">
        <div class="flex items-center gap-1.5 min-w-0">
          <span class="text-gray-800 truncate">{{ p.hostname }}</span>
          <Tag v-if="p.isSelf" size="small" color="arcoblue" class="shrink-0">本机</Tag>
        </div>
        <div class="flex items-center gap-2 shrink-0">
          <code class="font-mono text-gray-600">{{ p.virtualIp }}</code>
          <span class="text-gray-400 w-12 text-right">{{ p.latencyMs === '-' ? '--' : `${p.latencyMs}ms` }}</span>
        </div>
      </li>
    </ul>
  </div>
</template>
