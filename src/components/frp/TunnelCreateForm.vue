<script setup lang="ts">
/**
 * 隧道创建/编辑表单：创建支持模式切换，编辑固定 self 模式
 * 自备模式 TCP 连通性检测；本机端口按钮拉取监听端口
 * 公共服务器逻辑抽至 usePublicServers composable
 */
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import Button from '@/components/common/Button.vue'
import Checkbox from '@/components/common/Checkbox.vue'
import Input from '@/components/common/Input.vue'
import InputGroup from '@/components/common/InputGroup.vue'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { tcpCheck } from '@/utils/api/tools'
import { usePublicServers } from '@/composables/usePublicServers'
import { toastError, toastInfo } from '@/utils/toast'
import { pickFile } from '@/utils/fileDialog'
import { importFrpcConfig } from '@/utils/api/frp-manager'
import { openPickerWindow } from '@/utils/picker-window'
import type { CreateTunnelParams, Tunnel, TunnelType, UpdateTunnelParams } from '@/types/frp'
import {
  AdjustmentsHorizontalIcon,
  ArrowPathIcon,
  CheckCircleIcon,
  ChevronDownIcon,
  ServerStackIcon,
  XCircleIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{
  actionLoading: boolean
  editTunnel?: Tunnel
}>()
const emit = defineEmits<{
  create: [params: CreateTunnelParams]
  update: [params: UpdateTunnelParams]
  cancel: []
}>()

type TunnelMode = 'self' | 'official'
const modeOptions = [
  { label: '用户自备服务器', value: 'self' },
  { label: '官方公共服务器', value: 'official' },
]
const typeOptions = [{ label: 'TCP', value: 'tcp' }, { label: 'UDP', value: 'udp' }]

const form = reactive({
  name: '',
  // 手动创建统一使用系统自带 frpc；厂商隧道只能通过同步导入。
  providerId: 'system-default',
  mode: 'self' as TunnelMode,
  tunnelType: 'tcp' as TunnelType,
  localIp: '127.0.0.1',
  localPort: 25565,
  serverAddr: '',
  serverPort: 7000,
  remotePort: 30000,
  token: '',
  useTls: false,
  bandwidthLimit: '',
  bandwidthLimitMode: 'server',
  proxyUseEncryption: false,
  proxyUseCompression: false,
  proxyProtocolVersion: 'v1',
  advancedOpen: false,
  publicServerId: '',
  allocationId: '',
})

/**
 * 厂商下拉：显示所有已启用厂商（不再按 frpcReady 过滤，未就绪厂商可选但
 * 表单下方有「frpc 未就绪」警告提示；否则从厂商同步导入的隧道无法在编辑时
 * 显示对应厂商）。编辑模式下额外确保当前隧道厂商在选项中（即使被禁用）。
 */
const isOfficial = computed(() => form.mode === 'official')
const isEdit = computed(() => !!props.editTunnel)

// 公共服务器（官方模式）—— 逻辑抽至 usePublicServers composable
const { publicServersLoading, allocating, publicServerOptions, loadPublicServers, handlePublicServerChange } = usePublicServers(form)

watch(() => form.mode, (m) => {
  if (m === 'official') void loadPublicServers()
  else { form.publicServerId = ''; form.allocationId = '' }
})

// 本机开放端口选择（子窗口模式）
const portSelecting = ref(false)

async function handleSelectPort() {
  if (portSelecting.value) return
  portSelecting.value = true
  try {
    const value = await openPickerWindow({
      title: '选择本机端口',
      template: 'port-picker',
      data: {},
      width: 400,
      height: 500,
    })
    form.localPort = Number(value)
  } catch (e) {
    // 用户取消时静默处理（取消不报 toast）
    if (!(e instanceof Error && e.message.includes('取消'))) {
      toastError('选择端口失败：' + e)
    }
  } finally {
    portSelecting.value = false
  }
}

// 连通性自动检测（自备模式，2 秒防抖）
type CheckState = 'idle' | 'checking' | 'ok' | 'fail'
const checkState = ref<CheckState>('idle')
const checkLatency = ref(0)
const checkError = ref('')
let debounceTimer: ReturnType<typeof setTimeout> | null = null
let checkSeq = 0

function scheduleCheck() {
  if (isOfficial.value) return
  if (debounceTimer) clearTimeout(debounceTimer)
  checkState.value = 'idle'
  if (!form.serverAddr.trim() || !form.serverPort) return
  debounceTimer = setTimeout(runCheck, 2000)
}

async function runCheck() {
  const host = form.serverAddr.trim()
  const port = Number(form.serverPort)
  if (!host || !Number.isInteger(port) || port <= 0) return
  const seq = ++checkSeq
  checkState.value = 'checking'
  try {
    const r = await tcpCheck(host, port)
    if (seq !== checkSeq) return
    if (r.reachable) { checkState.value = 'ok'; checkLatency.value = r.latency_ms; checkError.value = '' }
    else { checkState.value = 'fail'; checkError.value = r.error }
  } catch (e) {
    if (seq !== checkSeq) return
    checkState.value = 'fail'; checkError.value = String(e)
  }
}

watch(() => [form.serverAddr, form.serverPort], scheduleCheck)

const checkHint = computed(() => {
  switch (checkState.value) {
    case 'checking': return { text: '检测中...', type: 'default' as const }
    case 'ok': return { text: `可连接（${checkLatency.value}ms）`, type: 'success' as const }
    case 'fail': return { text: `不可连接：${checkError.value}`, type: 'error' as const }
    default: return null
  }
})

// 生命周期 + 提交
onMounted(() => {
  if (props.editTunnel) {
    const t = props.editTunnel
    form.name = t.name; form.providerId = t.providerId; form.mode = 'self'
    form.tunnelType = t.tunnelType; form.localIp = t.localIp; form.localPort = t.localPort
    form.serverAddr = t.serverAddr; form.serverPort = t.serverPort; form.remotePort = t.remotePort
    form.token = t.token ?? ''; form.useTls = t.useTls
    form.bandwidthLimit = t.bandwidthLimit ?? ''
    form.bandwidthLimitMode = t.bandwidthLimitMode ?? 'server'
    form.proxyUseEncryption = t.proxyUseEncryption ?? false
    form.proxyUseCompression = t.proxyUseCompression ?? false
    form.proxyProtocolVersion = t.proxyProtocolVersion ?? 'v1'
    form.advancedOpen = !!t.bandwidthLimit || t.useTls
  }
})

onUnmounted(() => {
  if (debounceTimer) clearTimeout(debounceTimer)
})

async function handleImportConfig() {
  const path = await pickFile({
    title: '导入 frpc 配置文件',
    filters: [{ name: 'frpc TOML', extensions: ['toml', 'conf'] }],
  })
  if (!path) return
  try {
    const imported = await importFrpcConfig(path)
    form.providerId = 'system-default'
    if (imported.name) form.name = imported.name
    if (imported.tunnelType) form.tunnelType = imported.tunnelType
    if (imported.localIp) form.localIp = imported.localIp
    if (imported.localPort) form.localPort = imported.localPort
    if (imported.serverAddr) form.serverAddr = imported.serverAddr
    if (imported.serverPort) form.serverPort = imported.serverPort
    if (imported.remotePort) form.remotePort = imported.remotePort
    if (imported.token) form.token = imported.token
    if (imported.bandwidthLimit) form.bandwidthLimit = imported.bandwidthLimit
    if (imported.bandwidthLimitMode) form.bandwidthLimitMode = imported.bandwidthLimitMode
    if (imported.proxyUseEncryption !== undefined) form.proxyUseEncryption = imported.proxyUseEncryption
    if (imported.proxyUseCompression !== undefined) form.proxyUseCompression = imported.proxyUseCompression
    if (imported.proxyProtocolVersion) form.proxyProtocolVersion = imported.proxyProtocolVersion
    form.useTls = imported.useTls
    form.advancedOpen = true
    toastInfo('配置已安全解析，请检查字段后再创建')
  } catch (e) {
    toastError('导入配置失败：' + (e instanceof Error ? e.message : String(e)))
  }
}

function handleSubmit() {
  if (!form.name.trim()) return
  const payload = {
    name: form.name.trim(),
    providerId: form.providerId,
    tunnelType: form.tunnelType,
    localIp: form.localIp || '127.0.0.1',
    localPort: form.localPort,
    serverAddr: form.serverAddr.trim(),
    serverPort: form.serverPort,
    remotePort: form.remotePort,
    token: form.token.trim() || undefined,
    bandwidthLimit: form.bandwidthLimit.trim() || undefined,
    bandwidthLimitMode: form.bandwidthLimit.trim() ? form.bandwidthLimitMode : undefined,
    proxyUseEncryption: form.proxyUseEncryption,
    proxyUseCompression: form.proxyUseCompression,
    proxyProtocolVersion: form.proxyProtocolVersion,
    useTls: form.useTls,
  }
  if (isEdit.value && props.editTunnel) emit('update', { id: props.editTunnel.id, ...payload })
  else emit('create', payload)
}
</script>

<template>
  <div class="rounded-lg border border-gray-200 bg-gray-50 p-4 space-y-3">
    <div v-if="!isEdit">
      <label class="block text-xs font-medium text-gray-700 mb-1">服务器模式</label>
      <Select v-model="form.mode" :options="modeOptions" />
    </div>

    <div v-if="isOfficial">
      <label class="block text-xs font-medium text-gray-700 mb-1">公共服务器</label>
      <Select
        v-model="form.publicServerId"
        :options="publicServerOptions"
        :disabled="publicServersLoading || allocating"
        :placeholder="publicServersLoading ? '加载中...' : '选择服务器'"
        @update:model-value="handlePublicServerChange"
      />
      <p v-if="allocating" class="mt-1 flex items-center gap-1 text-xs text-primary-600">
        <ArrowPathIcon class="w-3.5 h-3.5 animate-spin" />
        正在分配端口与 Token...
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
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label class="block text-xs font-medium text-gray-700 mb-1">本地 IP</label>
        <Input v-model="form.localIp" placeholder="127.0.0.1" />
      </div>
      <div>
        <label class="block text-xs font-medium text-gray-700 mb-1">本地端口</label>
        <div class="flex items-center gap-2">
          <Input v-model="form.localPort" type="number" placeholder="25565" class="flex-1" />
          <Tooltip text="选择本机端口">
            <Button type="outline" :loading="portSelecting" @click="handleSelectPort">
              <template #icon><ServerStackIcon class="w-4 h-4" /></template>
            </Button>
          </Tooltip>
        </div>
      </div>
    </div>

    <div>
      <label class="block text-xs font-medium text-gray-700 mb-1">服务器地址</label>
      <InputGroup :ratio="[3, 1]">
        <Input v-model="form.serverAddr" placeholder="frps.example.com" :readonly="isOfficial" />
        <Input v-model="form.serverPort" type="number" placeholder="7000" :readonly="isOfficial" />
      </InputGroup>
      <p
        v-if="checkHint"
        class="mt-1 flex items-center gap-1 text-xs"
        :class="checkHint.type === 'success' ? 'text-green-600' : checkHint.type === 'error' ? 'text-red-500' : 'text-gray-500'"
      >
        <CheckCircleIcon v-if="checkHint.type === 'success'" class="w-3.5 h-3.5" />
        <XCircleIcon v-else-if="checkHint.type === 'error'" class="w-3.5 h-3.5" />
        <ArrowPathIcon v-else class="w-3.5 h-3.5 animate-spin" />
        {{ checkHint.text }}
      </p>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label class="block text-xs font-medium text-gray-700 mb-1">远程端口</label>
        <Input v-model="form.remotePort" type="number" placeholder="30000" :readonly="isOfficial" />
      </div>
      <div>
        <label class="block text-xs font-medium text-gray-700 mb-1">Token（可选）</label>
        <Input v-model="form.token" placeholder="留空表示无鉴权" :readonly="isOfficial" />
      </div>
    </div>

    <div class="border-t border-dashed border-gray-200 pt-2">
      <button
        type="button"
        class="flex w-full items-center gap-3 py-2 text-left text-xs text-gray-500 transition-colors hover:text-gray-800"
        @click="form.advancedOpen = !form.advancedOpen"
      >
        <AdjustmentsHorizontalIcon class="h-4 w-4 text-gray-400" />
        <span class="flex-1">高级设置</span>
        <ChevronDownIcon class="h-4 w-4 transition-transform" :class="form.advancedOpen ? 'rotate-180' : ''" />
      </button>
      <Transition
        enter-active-class="transition-all duration-200 ease-out"
        leave-active-class="transition-all duration-150 ease-in"
        enter-from-class="max-h-0 opacity-0 -translate-y-1"
        enter-to-class="max-h-64 opacity-100 translate-y-0"
        leave-from-class="max-h-64 opacity-100 translate-y-0"
        leave-to-class="max-h-0 opacity-0 -translate-y-1"
      >
        <div v-if="form.advancedOpen" class="ml-7 max-h-64 space-y-3 overflow-hidden pb-2 pt-1">
          <div class="flex items-center gap-2">
            <span class="w-24 shrink-0 text-xs text-gray-500">带宽限制</span>
            <Input v-model="form.bandwidthLimit" placeholder="例如 4MB" />
            <Select
              v-model="form.bandwidthLimitMode"
              :options="[
                { label: '服务端', value: 'server' },
                { label: '客户端', value: 'client' },
              ]"
              class="w-28 shrink-0"
            />
          </div>
          <div class="flex items-center gap-2">
            <span class="w-24 shrink-0 text-xs text-gray-500">Proxy 传输</span>
            <Checkbox v-model="form.proxyUseEncryption">加密</Checkbox>
            <Checkbox v-model="form.proxyUseCompression">压缩</Checkbox>
            <Select
              v-model="form.proxyProtocolVersion"
              :options="[
                { label: 'v1', value: 'v1' },
                { label: 'v2', value: 'v2' },
              ]"
              class="w-24 shrink-0"
            />
          </div>
          <div class="flex items-center gap-3">
            <span class="w-24 shrink-0 text-xs text-gray-500">连接加密</span>
            <Checkbox v-model="form.useTls" :disabled="isOfficial">启用传输层 TLS</Checkbox>
          </div>
        </div>
      </Transition>
    </div>

    <div class="flex items-center justify-between border-t border-dashed border-gray-200 py-3">
      <span class="text-xs text-gray-400">从标准 frpc 配置快速填充</span>
      <Button type="outline" size="small" @click="handleImportConfig">导入配置</Button>
    </div>

    <div class="flex justify-end gap-2 pt-2">
      <Button type="ghost" size="small" @click="emit('cancel')">取消</Button>
      <Button
        type="primary"
        size="small"
        :loading="actionLoading"
        :disabled="!form.name.trim() || !form.serverAddr.trim() || (isOfficial && !form.allocationId)"
        @click="handleSubmit"
      >
        {{ isEdit ? '保存' : '创建' }}
      </Button>
    </div>
  </div>
</template>
