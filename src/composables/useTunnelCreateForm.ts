import { computed, onMounted, onUnmounted, reactive, ref, watch, type Ref } from 'vue'
import { tcpCheck } from '@/utils/api/tools'
import { usePublicServers } from '@/composables/usePublicServers'
import { toastError, toastInfo } from '@/utils/toast'
import { pickFile } from '@/utils/fileDialog'
import { importFrpcConfig } from '@/utils/api/frp-manager'
import { openPickerWindow } from '@/utils/picker-window'
import type { CreateTunnelParams, Tunnel, TunnelType, UpdateTunnelParams } from '@/types/frp'

export type TunnelMode = 'self' | 'official'
export type CheckState = 'idle' | 'checking' | 'ok' | 'fail'

export const modeOptions = [
  { label: '用户自备服务器', value: 'self' },
  { label: '官方公共服务器', value: 'official' },
]
export const typeOptions = [{ label: 'TCP', value: 'tcp' }, { label: 'UDP', value: 'udp' }]
export const bandwidthLimitModeOptions = [
  { label: '服务端', value: 'server' },
  { label: '客户端', value: 'client' },
]
export const proxyProtocolVersionOptions = [
  { label: 'v1', value: 'v1' },
  { label: 'v2', value: 'v2' },
]

function createForm() {
  return reactive({
    name: '', providerId: 'system-default', mode: 'self' as TunnelMode,
    tunnelType: 'tcp' as TunnelType, localIp: '127.0.0.1', localPort: 25565,
    serverAddr: '', serverPort: 7000, remotePort: 30000, token: '', useTls: false,
    bandwidthLimit: '', bandwidthLimitMode: 'server', proxyUseEncryption: false,
    proxyUseCompression: false, proxyProtocolVersion: 'v1', advancedOpen: false,
    publicServerId: '', allocationId: '',
  })
}

export function useTunnelCreateForm(
  editTunnel: Ref<Tunnel | undefined>,
  emitCreate: (params: CreateTunnelParams) => void,
  emitUpdate: (params: UpdateTunnelParams) => void,
) {
  const form = createForm()
  const isEdit = computed(() => !!editTunnel.value)
  const isOfficial = computed(() => form.mode === 'official')
  const { publicServersLoading, allocating, publicServerOptions, loadPublicServers, handlePublicServerChange } = usePublicServers(form)

  watch(() => form.mode, (mode) => {
    if (mode === 'official') void loadPublicServers()
    else { form.publicServerId = ''; form.allocationId = '' }
  })

  const portSelecting = ref(false)
  async function handleSelectPort() {
    if (portSelecting.value) return
    portSelecting.value = true
    try {
      const value = await openPickerWindow({ title: '选择本机端口', template: 'port-picker', data: {}, width: 400, height: 500 })
      form.localPort = Number(value)
    } catch (e) {
      if (!(e instanceof Error && e.message.includes('取消'))) toastError('选择端口失败：' + e)
    } finally { portSelecting.value = false }
  }

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
      const result = await tcpCheck(host, port)
      if (seq !== checkSeq) return
      if (result.reachable) {
        checkState.value = 'ok'; checkLatency.value = result.latency_ms; checkError.value = ''
      } else { checkState.value = 'fail'; checkError.value = result.error }
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

  function applyTunnel(tunnel: Tunnel) {
    form.name = tunnel.name; form.providerId = tunnel.providerId; form.mode = 'self'
    form.tunnelType = tunnel.tunnelType; form.localIp = tunnel.localIp; form.localPort = tunnel.localPort
    form.serverAddr = tunnel.serverAddr; form.serverPort = tunnel.serverPort; form.remotePort = tunnel.remotePort
    form.token = tunnel.token ?? ''; form.useTls = tunnel.useTls
    form.bandwidthLimit = tunnel.bandwidthLimit ?? ''; form.bandwidthLimitMode = tunnel.bandwidthLimitMode ?? 'server'
    form.proxyUseEncryption = tunnel.proxyUseEncryption ?? false; form.proxyUseCompression = tunnel.proxyUseCompression ?? false
    form.proxyProtocolVersion = tunnel.proxyProtocolVersion ?? 'v1'
    form.advancedOpen = !!tunnel.bandwidthLimit || tunnel.useTls
  }

  onMounted(() => { if (editTunnel.value) applyTunnel(editTunnel.value) })
  onUnmounted(() => { if (debounceTimer) clearTimeout(debounceTimer) })

  async function handleImportConfig() {
    const path = await pickFile({ title: '导入 frpc 配置文件', filters: [{ name: 'frpc TOML', extensions: ['toml', 'conf'] }] })
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
      form.useTls = imported.useTls; form.advancedOpen = true
      toastInfo('配置已安全解析，请检查字段后再创建')
    } catch (e) { toastError('导入配置失败：' + (e instanceof Error ? e.message : String(e))) }
  }

  function handleSubmit() {
    if (!form.name.trim()) return
    const payload = {
      name: form.name.trim(), providerId: form.providerId, tunnelType: form.tunnelType,
      localIp: form.localIp || '127.0.0.1', localPort: form.localPort, serverAddr: form.serverAddr.trim(),
      serverPort: form.serverPort, remotePort: form.remotePort, token: form.token.trim() || undefined,
      bandwidthLimit: form.bandwidthLimit.trim() || undefined,
      bandwidthLimitMode: form.bandwidthLimit.trim() ? form.bandwidthLimitMode : undefined,
      proxyUseEncryption: form.proxyUseEncryption, proxyUseCompression: form.proxyUseCompression,
      proxyProtocolVersion: form.proxyProtocolVersion, useTls: form.useTls,
    }
    if (isEdit.value && editTunnel.value) emitUpdate({ id: editTunnel.value.id, ...payload })
    else emitCreate(payload)
  }

  return {
    form, isEdit, isOfficial, publicServersLoading, allocating, publicServerOptions,
    handlePublicServerChange, portSelecting, handleSelectPort, checkHint, handleImportConfig, handleSubmit,
  }
}
