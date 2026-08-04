<script setup lang="ts">
/**
 * 穿透管理：隧道列表 + 创建/编辑/启停/删除/自检 + 从厂商同步。
 * 状态同步：监听 frp-tunnel-status 事件自动刷新列表。
 * 列表卡片已拆至 TunnelList.vue，本文件保留操作栏与面板组装。
 */
import { ref, computed, onMounted, onUnmounted, inject } from 'vue'
import { useFrpStore } from '@/stores/frp'
import { showConfirm } from '@/utils/modal'
import { toastWarning, toastInfo } from '@/utils/toast'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import TunnelCreateForm from './TunnelCreateForm.vue'
import TunnelSelfCheck from './TunnelSelfCheck.vue'
import RemoteTunnelSync from './RemoteTunnelSync.vue'
import TunnelList from './TunnelList.vue'
import type { CreateTunnelParams, TunnelWithStatus, UpdateTunnelParams } from '@/types/frp'
import { ArrowPathIcon, PlusIcon, ChevronDownIcon, ShieldCheckIcon, CloudArrowDownIcon } from '@heroicons/vue/24/outline'

const store = useFrpStore()

/** 跳转到日志页查看指定隧道（由 Online.vue provide 的 emitter） */
const goToLogs = inject<(tunnelId: string) => void>('goToLogs', () => {})

const tunnels = computed(() => store.tunnels)
const loading = computed(() => store.tunnelsLoading)
const actionLoading = computed(() => store.tunnelActionLoading)
const providers = computed(() => store.providers)

const showForm = ref(false)
const showSelfCheck = ref(false)
const showSync = ref(false)

onMounted(() => {
  void store.loadTunnels()
  void store.loadProviders()
  // 启动隧道状态事件监听器（store 内部防重复注册）
  store.startTunnelStatusListener()
  // 拖拽安装/更新 frp 厂商包后刷新厂商列表
  window.addEventListener('frp:providers-changed', handleProvidersChanged)
})

onUnmounted(() => {
  window.removeEventListener('frp:providers-changed', handleProvidersChanged)
})

function handleProvidersChanged(): void {
  void store.loadProviders()
  void store.loadTunnels()
}

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

    <!-- 隧道列表（含空状态/加载态） -->
    <TunnelList
      :tunnels="tunnels"
      :loading="loading"
      :action-loading="actionLoading"
      :providers="providers"
      @start="handleStart"
      @stop="handleStop"
      @edit="handleEdit"
      @view-logs="handleViewLogs"
      @delete="handleDelete"
    />
  </div>
</template>
