/**
 * easytier 虚拟组网 composable
 *
 * 封装 easytier_join / easytier_stop IPC，运行状态同步到 online store 的
 * easytier 切片（供全局访问与重启恢复）。
 */
import { computed, ref } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { joinEasyTier, stopEasyTier } from '@/utils/api/online-manager/easytier'
import { EASYTIER_HOST_VIRTUAL_IP } from '@/types/online'
import type { EasyTierJoinParams } from '@/types/online'

/** easytier 连接状态 */
export type EasyTierStatus = 'idle' | 'joining' | 'joined' | 'stopping' | 'error'

/** easytier 虚拟组网 composable */
export function useEasyTier() {
  const store = useOnlineStore()
  const joining = ref(false)
  const stopping = ref(false)
  const error = ref('')
  /** 虚拟网络内节点列表（经 rpc-portal 查询，暂未接入） */
  const peers = ref<string[]>([])

  const status = computed<EasyTierStatus>(() => {
    if (store.easytierRuntime.joined) return 'joined'
    if (joining.value) return 'joining'
    if (stopping.value) return 'stopping'
    if (error.value) return 'error'
    return 'idle'
  })

  /** 本机虚拟 IP（房主固定 10.244.0.1；房客 DHCP 未回显） */
  const ip = computed(() => store.easytierRuntime.virtualIp)

  /** 虚拟网络名 */
  const networkName = computed(() => store.easytierRuntime.networkName)

  /** easytier-core 版本号 */
  const version = computed(() => store.easytierRuntime.version)

  /** 加入虚拟网络（房主 isHost=true 固定 IP） */
  async function join(params: EasyTierJoinParams): Promise<{ ok: boolean; error?: string }> {
    joining.value = true
    error.value = ''
    try {
      const res = await joinEasyTier(params)
      store.setEasyTierRuntime({
        joined: res.success,
        networkName: params.networkName,
        networkSecret: params.networkSecret,
        virtualIp: params.isHost ? EASYTIER_HOST_VIRTUAL_IP : '',
        rpcPortal: res.rpcPortal,
        pid: res.pid,
        version: res.version ?? '',
      })
      return { ok: res.success }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return { ok: false, error: error.value }
    } finally {
      joining.value = false
    }
  }

  /** 停止 easytier 子进程 */
  async function stop(): Promise<void> {
    stopping.value = true
    try {
      await stopEasyTier()
    } finally {
      store.resetEasyTierRuntime()
      stopping.value = false
    }
  }

  return { status, ip, peers, networkName, version, joining, stopping, error, join, stop }
}
