<script setup lang="ts">
/**
 * 穿透管理：隧道列表 + 创建/编辑/启停/删除/自检 + 从厂商同步。
 * 状态同步：监听 frp-tunnel-status 事件自动刷新列表。
 */
import { ref, computed, onMounted, inject } from 'vue'
import { useFrpStore } from '@/stores/frp'
import { showConfirm } from '@/utils/modal'
import { toastWarning, toastInfo } from '@/utils/toast'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import TunnelCreateForm from './TunnelCreateForm.vue'
import TunnelSelfCheck from './TunnelSelfCheck.vue'
import RemoteTunnelSync from './RemoteTunnelSync.vue'
import type { CreateTunnelParams, TunnelWithStatus, UpdateTunnelParams } from '@/types/frp'
import { ArrowPathIcon, PlusIcon, PlayIcon, StopIcon, TrashIcon, ChevronDownIcon, GlobeAltIcon, ServerIcon, DocumentTextIcon, PencilIcon, ShieldCheckIcon, CloudArrowDownIcon } from '@heroicons/vue/24/outline'

const store = useFrpStore()

/** 跳转到日志页查看指定隧道（由 Online.vue provide 的 emitter） */
const goToLogs = inject<(tunnelId: string) => void>('goToLogs', () => {})

const tunnels = computed(() => store.tunnels)
const loading = computed(() => store.tunnelsLoading)
const actionLoading = computed(() => store.tunnelActionLoading)
const providers = computed(() => store.providers)

/** 厂商名查找（隧道卡片展示用） */
function providerName(id: string): string {
  return providers.value.find(p => p.id === id)?.name ?? id
}

const showForm = ref(false)
const showSelfCheck = ref(false)
const showSync = ref(false)

onMounted(() => {
  void store.loadTunnels()
  void store.loadProviders()
  // 启动隧道状态事件监听器（store 内部防重复注册）
  store.startTunnelStatusListener()
})

async function handleCreate(params: CreateTunnelParams) {
  const ok = await store.createTunnel(params)
  if (ok) showForm.value = false
}

/** 编辑表单展开的隧道 ID */
const editingTunnelId = ref<string | null>(null)
const editingTunnel = computed(() =>
  tunnels.value.find(t => t.id === editingTunnelId.value),
)

function handleEdit(tunnel: TunnelWithStatus) {
  if (tunnel.status === 'running') {
    toastWarning('请先停止隧道再编辑')
    return
  }
  editingTunnelId.value = tunnel.id
}

async function handleUpdate(params: UpdateTunnelParams) {
  const ok = await store.updateTunnel(params)
  if (ok) editingTunnelId.value = null
}

async function handleStart(id: string) { await store.startTunnel(id) }
async function handleStop(id: string) { await store.stopTunnel(id) }

async function handleRefresh() { await store.loadTunnels(); toastInfo('隧道列表已刷新') }

function handleViewLogs(id: string) { goToLogs(id) }

function handleDelete(id: string, name: string) {
  showConfirm(
    '删除隧道',
    `确定要删除隧道「${name}」吗？此操作不可恢复。`,
    async () => {
      await store.deleteTunnel(id)
    },
  )
}
</script>

<template>
  <div class="space-y-4">
    <!-- 顶部操作栏 -->
    <div class="flex items-center justify-between">
      <p class="text-sm text-gray-500">
        共 {{ tunnels.length }} 条隧道
      </p>
      <div class="flex items-center gap-2">
        <Tooltip text="从厂商同步">
          <Button type="ghost" size="small" @click="showSync = !showSync">
            <template #icon><CloudArrowDownIcon class="w-4 h-4" /></template>
          </Button>
        </Tooltip>
        <Tooltip text="隧道自检">
          <Button type="ghost" size="small" @click="showSelfCheck = !showSelfCheck">
            <template #icon><ShieldCheckIcon class="w-4 h-4" /></template>
          </Button>
        </Tooltip>
        <Tooltip text="刷新列表">
          <Button
            type="ghost"
            size="small"
            :loading="loading"
            @click="handleRefresh"
          >
            <template #icon><ArrowPathIcon class="w-4 h-4" /></template>
          </Button>
        </Tooltip>
        <Button
          type="primary"
          size="small"
          @click="showForm = !showForm"
        >
          <template #icon>
            <PlusIcon v-if="!showForm" class="w-4 h-4" />
            <ChevronDownIcon v-else class="w-4 h-4 transition-transform duration-300" :class="showForm ? 'rotate-180' : ''" />
          </template>
          {{ showForm ? '收起' : '创建隧道' }}
        </Button>
      </div>
    </div>

    <!-- 创建表单（带展开/收起动画） -->
    <Transition
      enter-active-class="transition-all duration-300 ease-out origin-top"
      leave-active-class="transition-all duration-200 ease-in origin-top"
      enter-from-class="opacity-0 scale-y-95 -translate-y-2"
      leave-to-class="opacity-0 scale-y-95 -translate-y-2"
    >
      <TunnelCreateForm
        v-if="showForm"
        :providers="providers"
        :action-loading="actionLoading"
        @create="handleCreate"
        @cancel="showForm = false"
      />
    </Transition>

    <!-- 编辑表单 -->
    <Transition
      enter-active-class="transition-all duration-300 ease-out origin-top"
      leave-active-class="transition-all duration-200 ease-in origin-top"
      enter-from-class="opacity-0 scale-y-95 -translate-y-2"
      leave-to-class="opacity-0 scale-y-95 -translate-y-2"
    >
      <TunnelCreateForm
        v-if="editingTunnel"
        :providers="providers"
        :action-loading="actionLoading"
        :edit-tunnel="editingTunnel"
        @update="handleUpdate"
        @cancel="editingTunnelId = null"
      />
    </Transition>

    <!-- 自检面板 -->
    <Transition
      enter-active-class="transition-all duration-300 ease-out origin-top"
      leave-active-class="transition-all duration-200 ease-in origin-top"
      enter-from-class="opacity-0 scale-y-95 -translate-y-2"
      leave-to-class="opacity-0 scale-y-95 -translate-y-2"
    >
      <TunnelSelfCheck
        v-if="showSelfCheck"
        :tunnels="tunnels"
        :providers="providers"
        @close="showSelfCheck = false"
      />
    </Transition>

    <!-- 从厂商同步面板 -->
    <Transition
      enter-active-class="transition-all duration-300 ease-out origin-top"
      leave-active-class="transition-all duration-200 ease-in origin-top"
      enter-from-class="opacity-0 scale-y-95 -translate-y-2"
      leave-to-class="opacity-0 scale-y-95 -translate-y-2"
    >
      <RemoteTunnelSync
        v-if="showSync"
        :providers="providers"
        @close="showSync = false"
      />
    </Transition>

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
              @click="handleStart(tunnel.id)"
            >
              <template #icon><PlayIcon class="w-3.5 h-3.5" /></template>
              启动
            </Button>
            <Button
              v-else
              type="outline"
              size="mini"
              :loading="actionLoading"
              @click="handleStop(tunnel.id)"
            >
              <template #icon><StopIcon class="w-3.5 h-3.5" /></template>
              停止
            </Button>
            <Tooltip text="编辑配置">
              <Button
                type="ghost"
                size="mini"
                @click="handleEdit(tunnel)"
              >
                <template #icon><PencilIcon class="w-3.5 h-3.5" /></template>
              </Button>
            </Tooltip>
            <Tooltip text="查看日志">
              <Button
                type="ghost"
                size="mini"
                @click="handleViewLogs(tunnel.id)"
              >
                <template #icon><DocumentTextIcon class="w-3.5 h-3.5" /></template>
              </Button>
            </Tooltip>
            <Tooltip text="删除隧道">
              <Button
                type="ghost"
                size="mini"
                @click="handleDelete(tunnel.id, tunnel.name)"
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
  </div>
</template>
