/**
 * 公共服务器（官方模式）逻辑
 *
 * 从 TunnelCreateForm.vue 抽离，避免组件超 300 行约束。
 * 列表接口直接返回完整连接信息（token/地址/端口），选择服务器后回填表单。
 * form 由调用方传入（reactive 对象），composable 直接写入填充结果。
 */
import { computed, ref } from 'vue'
import { listPublicServers } from '@/utils/api/frp-manager'
import { toastError } from '@/utils/toast'
import type { PublicFrpServer } from '@/types/frp'

/** 官方公共服务器不分配远程端口，客户端为 frpc 随机生成一个。 */
function pickRemotePort(): number {
  const min = 10_000
  const max = 60_000
  const range = max - min + 1
  const cryptoApi = globalThis.crypto
  if (cryptoApi?.getRandomValues) {
    const values = new Uint32Array(1)
    cryptoApi.getRandomValues(values)
    return min + (values[0] % range)
  }
  return min + Math.floor(Math.random() * range)
}

/** composable 需要读写的表单字段子集 */
interface PublicServerFormTarget {
  serverAddr: string
  serverPort: number
  remotePort: number
  token: string
  useTls: boolean
  publicServerId: string
}

export function usePublicServers(form: PublicServerFormTarget) {
  const publicServers = ref<PublicFrpServer[]>([])
  const publicServersLoading = ref(false)
  const publicServerOptions = computed(() =>
    publicServers.value.map(s => ({
      label: `${s.name}（${s.region}）`,
      value: s.id,
    })),
  )

  async function loadPublicServers() {
    if (publicServers.value.length > 0 || publicServersLoading.value) return
    publicServersLoading.value = true
    try {
      publicServers.value = await listPublicServers()
    } catch (e) {
      toastError('加载公共服务器列表失败：' + e)
    } finally {
      publicServersLoading.value = false
    }
  }

  function handlePublicServerChange(id: string | number) {
    if (!id) return
    const server = publicServers.value.find(s => s.id === String(id))
    if (!server) return
    form.serverAddr = server.serverAddr
    form.serverPort = server.serverPort
    form.remotePort = pickRemotePort()
    form.token = server.publicToken
    form.useTls = server.tlsEnabled
  }

  return {
    publicServers,
    publicServersLoading,
    publicServerOptions,
    loadPublicServers,
    handlePublicServerChange,
  }
}
