<script setup lang="ts">
/**
 * 隧道列表：卡片列表（状态/地址/操作按钮）+ 空状态/加载态。
 * 从 TunnelManager.vue 拆出，避免 Vue 组件超 300 行。
 */
import { computed } from 'vue'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { copyToClipboard } from '@/utils/clipboard'
import { buildTunnelLink } from '@/utils/frp-tunnel-link'
import type { ProviderInfo, TunnelWithStatus } from '@/types/frp'
import {
  ArrowPathIcon, PlayIcon, StopIcon, TrashIcon,
  GlobeAltIcon, ServerIcon, DocumentTextIcon, PencilIcon,
  LinkIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{
  tunnels: TunnelWithStatus[]
  loading: boolean
  actionLoading: boolean
  providers: ProviderInfo[]
}>()

const emit = defineEmits<{
  start: [id: string]
  stop: [id: string]
  edit: [tunnel: TunnelWithStatus]
  viewLogs: [id: string]
  delete: [id: string, name: string]
}>()

/** 厂商名查找（隧道卡片展示用） */
const providerName = computed(() => {
  const map = new Map(props.providers.map(p => [p.id, p.name]))
  return (id: string): string => map.get(id) ?? id
})

/** 组装隧道访问链接并复制（serverAddr:remotePort） */
async function handleCopyLink(tunnel: TunnelWithStatus) {
  const link = buildTunnelLink(tunnel.serverAddr, tunnel.remotePort, tunnel.tunnelType)
  await copyToClipboard(link, { toast: true })
}
</script>

<template>
  <!-- 隧道列表 -->
  <TransitionGroup
    v-if="tunnels.length > 0"
    tag="div"
    class="space-y-3"
    enter-active-class="transition-all duration-300 ease-out"
    leave-active-class="transition-all duration-200 ease-in absolute"
    enter-from-class="opacity-0 translate-y-2"
    leave-to-class="opacity-0 -translate-y-2"
    move-class="transition-transform duration-300"
  >
    <div
      v-for="tunnel in tunnels"
      :key="tunnel.id"
      class="rounded-lg border border-gray-200 bg-white p-4 hover:border-primary-300 hover:shadow-sm transition-all"
    >
      <div class="flex items-start justify-between gap-3">
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2 flex-wrap">
            <span class="text-sm font-semibold text-gray-900">{{ tunnel.name }}</span>
            <span
              class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-xs font-medium"
              :class="tunnel.status === 'running'
                ? 'bg-green-50 text-green-700'
                : 'bg-gray-100 text-gray-500'"
            >
              <span
                class="w-1.5 h-1.5 rounded-full"
                :class="tunnel.status === 'running' ? 'bg-green-500 animate-pulse' : 'bg-gray-400'"
              />
              {{ tunnel.status === 'running' ? '运行中' : '已停止' }}
            </span>
            <span class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-primary-50 text-primary-700 uppercase">
              {{ tunnel.tunnelType }}
            </span>
            <span class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-gray-50 text-gray-500">
              {{ providerName(tunnel.providerId) }}
            </span>
          </div>
          <div class="mt-1.5 flex flex-wrap gap-x-4 gap-y-0.5 text-xs text-gray-500">
            <span class="flex items-center gap-1">
              <ServerIcon class="w-3.5 h-3.5" />
              {{ tunnel.localIp }}:{{ tunnel.localPort }}
            </span>
            <span class="flex items-center gap-1">
              <GlobeAltIcon class="w-3.5 h-3.5" />
              {{ tunnel.serverAddr }}:{{ tunnel.serverPort }}
            </span>
            <span>远程端口: {{ tunnel.remotePort }}</span>
            <span v-if="tunnel.useTls" class="text-green-600">TLS</span>
          </div>
        </div>
        <div class="flex items-center gap-1.5 shrink-0">
          <Button
            v-if="tunnel.status === 'stopped'"
            type="primary"
            size="mini"
            :loading="actionLoading"
            @click="emit('start', tunnel.id)"
          >
            <template #icon><PlayIcon class="w-3.5 h-3.5" /></template>
            启动
          </Button>
          <Button
            v-else
            type="outline"
            size="mini"
            :loading="actionLoading"
            @click="emit('stop', tunnel.id)"
          >
            <template #icon><StopIcon class="w-3.5 h-3.5" /></template>
            停止
          </Button>
          <Tooltip text="编辑配置">
            <Button
              type="ghost"
              size="mini"
              @click="emit('edit', tunnel)"
            >
              <template #icon><PencilIcon class="w-3.5 h-3.5" /></template>
            </Button>
          </Tooltip>
          <Tooltip text="复制访问链接">
            <Button
              type="ghost"
              size="mini"
              @click="handleCopyLink(tunnel)"
            >
              <template #icon><LinkIcon class="w-3.5 h-3.5" /></template>
            </Button>
          </Tooltip>
          <Tooltip text="查看日志">
            <Button
              type="ghost"
              size="mini"
              @click="emit('viewLogs', tunnel.id)"
            >
              <template #icon><DocumentTextIcon class="w-3.5 h-3.5" /></template>
            </Button>
          </Tooltip>
          <Tooltip text="删除隧道">
            <Button
              type="ghost"
              size="mini"
              @click="emit('delete', tunnel.id, tunnel.name)"
            >
              <template #icon><TrashIcon class="w-3.5 h-3.5" /></template>
            </Button>
          </Tooltip>
        </div>
      </div>
    </div>
  </TransitionGroup>

  <!-- 空状态 -->
  <div v-else-if="!loading" class="flex flex-col items-center justify-center py-16">
    <ArrowPathIcon class="w-12 h-12 text-gray-300 mb-3" />
    <p class="text-sm font-medium text-gray-500">暂无隧道</p>
    <p class="text-xs text-gray-400 mt-1">点击「创建隧道」开始使用 Frp 内网穿透</p>
  </div>
  <div v-else class="flex items-center justify-center py-16">
    <ArrowPathIcon class="w-6 h-6 text-gray-400 animate-spin" />
  </div>
</template>
