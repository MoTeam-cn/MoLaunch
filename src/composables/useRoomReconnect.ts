/**
 * 房客重连 composable（easytier + Scaffolding 收敛版）
 *
 * easytier 为长驻虚拟网络，短时网络抖动由 easytier 自愈；
 * 此处仅提供「手动重新探测进服地址」能力（scaffolding_client_probe），
 * 替代旧 WebRTC 的全量重连逻辑。probe 结果写入 store（mcIp/mcPort）。
 */
import { useOnlineStore } from '@/stores/online'
import { useScaffolding } from './useScaffolding'

/** 房客重连 composable */
export function useRoomReconnect() {
  const store = useOnlineStore()
  const scaffolding = useScaffolding()

  /** 重新探测房主 MC 服务地址（需已在加入方房间） */
  async function reconnect(): Promise<{ ok: boolean; mcIp?: string; mcPort?: number; error?: string }> {
    const state = store.roomState
    if (state.role !== 'guest' || !state.roomCode) {
      return { ok: false, error: '未在加入方房间' }
    }
    return scaffolding.probe(state.roomCode)
  }

  return { reconnect }
}
