/**
 * 全局联机会话（App 级初始化，常驻整个应用生命周期）
 *
 * 新架构（easytier + Scaffolding）收敛版：会话持有全局单例的 easytier /
 * scaffolding composable 与房主编排实例，供各面板消费；运行状态统一写入
 * online store 的 easytier 切片。
 * - init()：空实现（easytier 为长驻子进程，无信令轮询）
 * - dispose()：停止 easytier（应用退出/登出时调用）
 */
import { useEasyTier } from '@/composables/useEasyTier'
import { useScaffolding } from '@/composables/useScaffolding'
import { useRoomHost } from '@/composables/useRoomHost'
import { useRoomReconnect } from '@/composables/useRoomReconnect'

/** 全局联机会话接口 */
export interface OnlineSession {
  /** 房主编排（hostStart / handleCloseRoom） */
  host: ReturnType<typeof useRoomHost>
  /** easytier 组网（join / stop / status） */
  easytier: ReturnType<typeof useEasyTier>
  /** Scaffolding 联机中心（hostStart / probe） */
  scaffolding: ReturnType<typeof useScaffolding>
  /** 房客重连（重新探测进服地址） */
  reconnect: ReturnType<typeof useRoomReconnect>
  init(): void
  dispose(): void
}

let session: OnlineSession | null = null

/** 初始化全局联机会话（幂等；App 挂载时调用一次） */
export function initOnlineSession(): OnlineSession {
  if (session) return session
  const host = useRoomHost()
  const reconnect = useRoomReconnect()
  session = {
    host,
    easytier: host.easytier,
    scaffolding: host.scaffolding,
    reconnect,
    init() {},
    dispose() {
      void host.easytier.stop()
    },
  }
  return session
}

/** 获取全局联机会话（未初始化时自动初始化） */
export function getOnlineSession(): OnlineSession {
  if (!session) return initOnlineSession()
  return session
}
