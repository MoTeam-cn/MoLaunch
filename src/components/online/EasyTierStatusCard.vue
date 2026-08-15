<script setup lang="ts">
/**
 * easytier 虚拟组网状态卡片（设备面板展示）
 *
 * 展示组网状态 / core 版本 / 虚拟 IP / 虚拟网络名 / 进程 PID。
 * 状态来源双通道：后端 `easytier-status` 事件推送（加入/停止时 emit）+ 打开页面时
 * `easytier_status` 查询兜底，统一写入 online store 的 easytier 切片。
 */
import { computed, onMounted, defineAsyncComponent } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useTauriEvent } from '@/composables/useTauriEvent'
import { getEasyTierStatus } from '@/utils/api/online-manager/easytier'
import type { EasyTierStatusResult } from '@/types/online'
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const EasyTierStatusBadge = defineAsyncComponent(() => import('./EasyTierStatusBadge.vue'))
import {
  ServerStackIcon,
  GlobeAltIcon,
  TagIcon,
  CpuChipIcon,
} from '@heroicons/vue/24/outline'

const store = useOnlineStore()

const version = computed(() => store.easytierRuntime.version)
const ip = computed(() => store.easytierRuntime.virtualIp)
const networkName = computed(() => store.easytierRuntime.networkName)
const pid = computed(() => store.easytierRuntime.pid)

/** 后端 emit 推送：实时同步组网状态（joined/version/pid/rpcPortal） */
const { start } = useTauriEvent<EasyTierStatusResult>('easytier-status', (payload) => {
  store.setEasyTierRuntime({
    joined: payload.joined,
    version: payload.version ?? '',
    pid: payload.pid,
    rpcPortal: payload.rpcPortal ?? '',
  })
})

onMounted(async () => {
  // 打开页面时查询一次兜底（emit 仅在有动作时推送）
  start()
  try {
    const status = await getEasyTierStatus()
    store.setEasyTierRuntime({
      joined: status.joined,
      version: status.version ?? '',
      pid: status.pid,
      rpcPortal: status.rpcPortal ?? '',
    })
  } catch {
    // 查询失败保持现状，等待后续 emit 推送
  }
})
</script>

<template>
  <Card title="虚拟组网（easytier）">
    <div class="divide-y divide-gray-100">
      <div class="px-1 py-3 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <ServerStackIcon class="w-4 h-4 text-gray-400" />
          <span>组网状态</span>
        </div>
        <EasyTierStatusBadge />
      </div>
      <div class="px-1 py-3 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <TagIcon class="w-4 h-4 text-gray-400" />
          <span>core 版本</span>
        </div>
        <code class="text-xs text-gray-900 bg-gray-50 px-2 py-0.5 rounded">{{ version || '-' }}</code>
      </div>
      <div class="px-1 py-3 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <GlobeAltIcon class="w-4 h-4 text-gray-400" />
          <span>虚拟网络</span>
        </div>
        <code class="text-xs text-gray-900 bg-gray-50 px-2 py-0.5 rounded max-w-[220px] truncate">{{ networkName || '-' }}</code>
      </div>
      <div class="px-1 py-3 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <GlobeAltIcon class="w-4 h-4 text-gray-400" />
          <span>虚拟 IP</span>
        </div>
        <code class="text-xs text-gray-900 bg-gray-50 px-2 py-0.5 rounded font-mono">{{ ip || '-' }}</code>
      </div>
      <div class="px-1 py-3 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <CpuChipIcon class="w-4 h-4 text-gray-400" />
          <span>进程 PID</span>
        </div>
        <span class="text-xs text-gray-900 font-mono">{{ pid ?? '-' }}</span>
      </div>
    </div>
  </Card>
</template>
