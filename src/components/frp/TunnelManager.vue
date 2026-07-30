<script setup lang="ts">
/**
 * 穿透管理
 *
 * 隧道列表 + 创建表单 + 启停/删除操作。
 * 阶段二支持厂商选择：内置 system-default + 外部厂商（仅 enabled && frpcReady）。
 * 厂商选择联动：未就绪的外部厂商显示 frpc 下载提示。
 */
import { ref, reactive, onMounted, computed } from 'vue'
import { useFrpStore } from '@/stores/frp'
import { showConfirm } from '@/utils/modal'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import type { CreateTunnelParams, TunnelType } from '@/types/frp'
import {
  ArrowPathIcon,
  PlusIcon,
  PlayIcon,
  StopIcon,
  TrashIcon,
  ChevronDownIcon,
  GlobeAltIcon,
  ServerIcon,
  ExclamationCircleIcon,
} from '@heroicons/vue/24/outline'

const store = useFrpStore()

const tunnels = computed(() => store.tunnels)
const loading = computed(() => store.tunnelsLoading)
const actionLoading = computed(() => store.tunnelActionLoading)
const providers = computed(() => store.providers)

/** 可选厂商：仅启用且 frpc 就绪（system-default 始终可选） */
const providerOptions = computed(() =>
  providers.value
    .filter(p => p.enabled && (p.frpcReady || p.builtin))
    .map(p => ({ label: p.name, value: p.id })),
)

/** 当前选中厂商对象（用于联动提示） */
const selectedProvider = computed(() =>
  providers.value.find(p => p.id === form.providerId),
)

/** 厂商名查找（隧道卡片展示用） */
function providerName(id: string): string {
  return providers.value.find(p => p.id === id)?.name ?? id
}

/** 创建表单展开 */
const showForm = ref(false)

/** 隧道类型选项 */
const typeOptions = [
  { label: 'TCP', value: 'tcp' },
  { label: 'UDP', value: 'udp' },
]

/** 创建表单 */
const form = reactive({
  name: '',
  providerId: 'system-default',
  tunnelType: 'tcp' as TunnelType,
  localIp: '127.0.0.1',
  localPort: 25565,
  serverAddr: '',
  serverPort: 7000,
  remotePort: 30000,
  token: '',
  useTls: false,
})

onMounted(() => {
  void store.loadTunnels()
  void store.loadProviders()
})

function resetForm() {
  Object.assign(form, {
    name: '', providerId: 'system-default', tunnelType: 'tcp' as TunnelType,
    localIp: '127.0.0.1', localPort: 25565, serverAddr: '',
    serverPort: 7000, remotePort: 30000, token: '', useTls: false,
  })
}

async function handleCreate() {
  if (!form.name.trim()) return
  const params: CreateTunnelParams = {
    name: form.name.trim(),
    providerId: form.providerId,
    tunnelType: form.tunnelType,
    localIp: form.localIp || '127.0.0.1',
    localPort: form.localPort,
    serverAddr: form.serverAddr.trim(),
    serverPort: form.serverPort,
    remotePort: form.remotePort,
    token: form.token.trim() || undefined,
    useTls: form.useTls,
  }
  const ok = await store.createTunnel(params)
  if (ok) {
    resetForm()
    showForm.value = false
  }
}

async function handleStart(id: string) { await store.startTunnel(id) }
async function handleStop(id: string) { await store.stopTunnel(id) }

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
      <Button
        type="primary"
        size="small"
        @click="showForm = !showForm"
      >
        <template #icon>
          <PlusIcon v-if="!showForm" class="w-4 h-4" />
          <ChevronDownIcon v-else class="w-4 h-4" />
        </template>
        {{ showForm ? '收起' : '创建隧道' }}
      </Button>
    </div>

    <!-- 创建表单 -->
    <div v-if="showForm" class="rounded-lg border border-gray-200 bg-gray-50 p-4 space-y-3">
      <div>
        <label class="block text-xs font-medium text-gray-700 mb-1">厂商</label>
        <Select v-model="form.providerId" :options="providerOptions" />
        <p
          v-if="selectedProvider && !selectedProvider.frpcReady && !selectedProvider.builtin"
          class="mt-1 flex items-center gap-1 text-xs text-amber-600"
        >
          <ExclamationCircleIcon class="w-3.5 h-3.5" />
          该厂商 frpc 未就绪，启动隧道前请先在「厂商列表」页下载 frpc
        </p>
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">隧道名称</label>
          <Input v-model="form.name" placeholder="我的隧道" />
        </div>
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">隧道类型</label>
          <Select v-model="form.tunnelType" :options="typeOptions" />
        </div>
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">本地 IP</label>
          <Input v-model="form.localIp" placeholder="127.0.0.1" />
        </div>
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">本地端口</label>
          <Input v-model="form.localPort" type="number" placeholder="25565" />
        </div>
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">服务器地址</label>
          <Input v-model="form.serverAddr" placeholder="frps.example.com" />
        </div>
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">服务器端口</label>
          <Input v-model="form.serverPort" type="number" placeholder="7000" />
        </div>
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">远程端口</label>
          <Input v-model="form.remotePort" type="number" placeholder="30000" />
        </div>
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">Token（可选）</label>
          <Input v-model="form.token" placeholder="留空表示无鉴权" />
        </div>
      </div>
      <div class="flex items-center gap-2">
        <input
          v-model="form.useTls"
          type="checkbox"
          class="w-4 h-4 rounded border-gray-300 text-primary-600"
        />
        <span class="text-xs text-gray-700">启用 TLS 加密</span>
      </div>
      <div class="flex justify-end gap-2 pt-1">
        <Button type="outline" size="small" @click="showForm = false">取消</Button>
        <Button
          type="primary"
          size="small"
          :loading="actionLoading"
          :disabled="!form.name.trim() || !form.serverAddr.trim()"
          @click="handleCreate"
        >
          创建
        </Button>
      </div>
    </div>

    <!-- 隧道列表 -->
    <div v-if="tunnels.length > 0" class="space-y-3">
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
                class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium"
                :class="tunnel.status === 'running'
                  ? 'bg-green-50 text-green-700'
                  : 'bg-gray-100 text-gray-500'"
              >
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
    </div>

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
