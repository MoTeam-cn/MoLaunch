/**
 * 公共服务器（官方模式）逻辑
 *
 * 从 TunnelCreateForm.vue 抽离，避免组件超 300 行约束。
 * 包含公共服务器列表加载、分配端口与 token、错误回滚。
 * form 由调用方传入（reactive 对象），composable 直接写入分配结果。
 */
import { ref, computed } from 'vue'
import { listPublicServers, allocatePublicServer } from '@/utils/api/frp-manager'
import { toastError } from '@/utils/toast'
import type { PublicFrpServer, TunnelType } from '@/types/frp'

/** composable 需要读写的表单字段子集 */
interface PublicServerFormTarget {
  tunnelType: TunnelType
  serverAddr: string
  serverPort: number
  remotePort: number
  token: string
  useTls: boolean
  allocationId: string
  publicServerId: string
}

export function usePublicServers(form: PublicServerFormTarget) {
  const publicServers = ref<PublicFrpServer[]>([])
  const publicServersLoading = ref(false)
  const allocating = ref(false)
  const publicServerOptions = computed(() =>
    publicServers.value
      .filter(s => s.allocatable)
      .map(s => ({
        label: `${s.name}（${s.region}）· 负载 ${s.loadPercent}% · ${s.onlineUsers}/${s.maxUsers} 人`,
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

  async function handlePublicServerChange(id: string | number) {
    if (!id) return
    allocating.value = true
    try {
      const resp = await allocatePublicServer({ serverId: String(id), tunnelType: form.tunnelType })
      form.serverAddr = resp.server.serverAddr
      form.serverPort = resp.server.serverPort
      form.remotePort = resp.remotePort
      form.token = resp.frpToken
      form.useTls = resp.server.tlsEnabled
      form.allocationId = resp.allocationId
    } catch (e) {
      toastError('分配公共服务器失败：' + e)
      form.publicServerId = ''
    } finally {
      allocating.value = false
    }
  }

  return {
    publicServers,
    publicServersLoading,
    allocating,
    publicServerOptions,
    loadPublicServers,
    handlePublicServerChange,
  }
}
