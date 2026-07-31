<script setup lang="ts">
/**
 * 隧道创建/编辑表单：创建支持模式切换，编辑固定 self 模式
 * 自备模式 TCP 连通性检测；本机端口按钮拉取监听端口
 * 公共服务器逻辑抽至 usePublicServers composable
 */
import { ref, reactive, computed, watch, onMounted, onUnmounted } from 'vue'
import Button from '@/components/common/Button.vue'
import Checkbox from '@/components/common/Checkbox.vue'
import Input from '@/components/common/Input.vue'
import InputGroup from '@/components/common/InputGroup.vue'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { tcpCheck } from '@/utils/api/tools'
import { usePublicServers } from '@/composables/usePublicServers'
import { toastError } from '@/utils/toast'
import { openPickerWindow } from '@/utils/picker-window'
import type { CreateTunnelParams, ProviderInfo, Tunnel, TunnelType, UpdateTunnelParams } from '@/types/frp'
import { ExclamationCircleIcon, CheckCircleIcon, XCircleIcon, ArrowPathIcon, ServerStackIcon } from '@heroicons/vue/24/outline'

const props = defineProps<{
  providers: ProviderInfo[]
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
  publicServerId: '',
  allocationId: '',
})

const providerOptions = computed(() =>
  props.providers.filter(p => p.enabled && (p.frpcReady || p.builtin))
    .map(p => ({ label: p.name, value: p.id })),
)
const selectedProvider = computed(() => props.providers.find(p => p.id === form.providerId))
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
  const port = form.serverPort
  if (!host || !port) return
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
  }
})

onUnmounted(() => {
  if (debounceTimer) clearTimeout(debounceTimer)
})

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
    useTls: form.useTls,
  }
  if (isEdit.value && props.editTunnel) emit('update', { id: props.editTunnel.id, ...payload })
  else emit('create', payload)
}
</script>

<template>
  <div class="rounded-lg border border-gray-200 bg-gray-50 p-4 space-y-3 shadow-sm">
    <div>
      <label class="block text-xs font-medium text-gray-700 mb-1">厂商</label>
      <Select v-model="form.providerId" :options="providerOptions" />
      <p
v-if="selectedProvider && !selectedProvider.frpcReady && !selectedProvider.builtin"
         class="mt-1 flex items-center gap-1 text-xs text-amber-600">
        <ExclamationCircleIcon class="w-3.5 h-3.5" />
        该厂商 frpc 未就绪，启动隧道前请先在「厂商列表」页下载 frpc
      </p>
    </div>

    <div v-if="!isEdit">
      <label class="block text-xs font-medium text-gray-700 mb-1">服务器模式</label>
      <Select v-model="form.mode" :options="modeOptions" />
    </div>

    <div v-if="isOfficial">
      <label class="block text-xs font-medium text-gray-700 mb-1">公共服务器</label>
      <Select
v-model="form.publicServerId" :options="publicServerOptions"
              :disabled="publicServersLoading || allocating"
              :placeholder="publicServersLoading ? '加载中...' : '选择服务器'"
              @update:model-value="handlePublicServerChange" />
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
          <Tooltip text="本机开放端口">
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
v-if="checkHint" class="mt-1 flex items-center gap-1 text-xs"
         :class="checkHint.type === 'success' ? 'text-green-600'
           : checkHint.type === 'error' ? 'text-red-500' : 'text-gray-500'">
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

    <div class="flex items-center gap-2">
      <Checkbox v-model="form.useTls" :disabled="isOfficial">启用 TLS 加密</Checkbox>
    </div>

    <div class="flex justify-end gap-2 pt-1">
      <Button type="outline" size="small" @click="emit('cancel')">取消</Button>
      <Button
type="primary" size="small" :loading="actionLoading"
              :disabled="!form.name.trim() || !form.serverAddr.trim() || (isOfficial && !form.allocationId)"
              @click="handleSubmit">
        {{ isEdit ? '保存' : '创建' }}
      </Button>
    </div>
  </div>
</template>
