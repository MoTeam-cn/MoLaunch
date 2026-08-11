/**
 * 虚拟网卡桥接 composable
 *
 * 封装 start（CIDR→tunStart）/ onTunPacket 回调 / forwardToTun / stop。
 * 事件监听走 onGlobalEvent 全局单例 listener（后台 TUN 持续推送，永不 unlisten），
 * handler 组件卸载时自动移除；forwardToTun 失败仅 warn 不抛错（避免回调链断掉）。
 */

import { onUnmounted, ref, shallowRef } from 'vue'
import { onGlobalEvent } from '@/composables/useGlobalTauriEvent'
import {
  tunStart,
  tunForwardTo,
  tunStop,
  restartAsAdmin,
} from '@/utils/api/online-manager'
import {
  EVENT_TUN_PACKET_OUT,
  type TunForwardResponse,
  type TunPacketPayload,
  type TunStartResponse,
} from '@/types/online'
import { showConfirmAsync, showInfo } from '@/utils/modal'
import { useOnlineStore } from '@/stores/online'
import { saveRelaunchSnapshot, clearRelaunchSnapshot } from '@/utils/relaunchSnapshot'

/** useVirtualLan 选项 */
export interface UseVirtualLanOptions {
  /**
   * TUN 读到 IP 包时的回调
   *
   * 后端将 IP 包编码为协议帧后 emit 事件，前端在此回调中将 ArrayBuffer 转发到 DataChannel。
   * - 房主：调 `hostMesh.broadcastPacket(raw)` 下发给所有参与者
   * - 加入方：调 `dataChannel.send(raw)` 发给房主
   *
   * @param raw 后端 emit 的协议帧字节（ArrayBuffer 形式）
   */
  onTunPacket: (raw: ArrayBuffer) => void
  /**
   * 组件卸载时自动 stop（默认 true）
   *
   * 全局联机会话传 `false`：桥接常驻应用生命周期，stop 由会话显式管理。
   */
  autoStop?: boolean
}

/**
 * 从 CIDR 字符串解析子网前缀长度
 *
 * 例如 `"10.244.1.0/24"` → `24`；解析失败回退到 `24`（C 类子网默认值）。
 *
 * @param subnet CIDR 字符串
 * @returns 子网前缀长度（0-32）
 */
export function parsePrefixLen(subnet: string): number {
  const parts = subnet.split('/')
  if (parts.length !== 2) return 24
  const n = parseInt(parts[1], 10)
  if (isNaN(n) || n < 0 || n > 32) return 24
  return n
}

/**
 * 虚拟网卡桥接 composable
 *
 * 调用方在进入房间后 `start`，在退出房间 / 关闭房间 / 组件卸载时 `stop`。
 * `onTunPacket` 回调由调用方注入，决定 TUN 读到的包如何分发到 DataChannel。
 */
export function useVirtualLan(options: UseVirtualLanOptions) {
  const autoStop = options.autoStop ?? true
  const store = useOnlineStore()
  /** 桥接是否运行中 */
  const running = ref(false)
  /** TUN 接口信息（start 成功后填充） */
  const interfaceInfo = shallowRef<TunStartResponse | null>(null)
  /** 最近一次错误（启动失败时填充，null 表示无错误） */
  const lastError = ref<string | null>(null)

  // 全局单例 Tauri listener：后台 TUN 数据流持续推送，永不 unlisten（避免 Tauri 2.x unlisten 竞态）
  // handler 在组件卸载时由 onGlobalEvent 自动移除（全局会话传 autoRemove:false 永久驻留），
  // running 守卫保证仅桥接运行中才分发数据
  onGlobalEvent<TunPacketPayload>(
    EVENT_TUN_PACKET_OUT,
    (payload) => {
      if (!running.value) return
      // number[] → ArrayBuffer
      const bytes = new Uint8Array(payload)
      options.onTunPacket(bytes.buffer)
    },
    { autoRemove: !autoStop },
  )

  /**
   * 启动 TUN 桥接
   *
   * 1. 若已在运行，先停止（防止泄漏）
   * 2. 调用后端 `tun_start` 创建 TUN 接口 + 启动读写循环
   *
   * @param selfVirtualIp 自己的虚拟 IP（如 `10.244.1.1`）
   * @param subnet 子网 CIDR（如 `10.244.1.0/24`，仅用于解析前缀长度）
   * @returns TUN 接口信息
   */
  async function start(
    selfVirtualIp: string,
    subnet: string,
  ): Promise<TunStartResponse> {
    // 若已运行，先停止
    if (running.value) {
      await stop()
    }

    // 解析子网前缀长度
    const prefixLen = parsePrefixLen(subnet)

    // 调用后端启动 TUN
    try {
      const info = await tunStart({ ipv4: selfVirtualIp, prefixLen })
      interfaceInfo.value = info
      running.value = true
      lastError.value = null
      return info
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      // 后端检测到权限不足（os error 5）且非管理员时返回此前缀
      if (msg.startsWith('TUN_PERMISSION_DENIED:')) {
        const prompt = msg.split(':').slice(1).join(':')
        const confirmed = await showConfirmAsync('需要管理员权限', prompt)
        if (confirmed) {
          // 提权重启前加密保存当前页面与房间快照，新实例启动后恢复会话
          await saveRelaunchSnapshot({
            path: window.location.pathname + window.location.search,
            room: store.roomState.role ? { ...store.roomState } : null,
          })
          // 触发 UAC 提权重启，后端延迟 500ms 退出当前进程
          // dev 模式下后端不重启，返回 dev_mode 标记，此处展示提示
          const result = await restartAsAdmin().catch(() => {
            // UAC 被用户拒绝等：清除恢复标记，避免下次正常启动误恢复
            clearRelaunchSnapshot()
            return null
          })
          if (result && result.dev_mode) {
            showInfo('开发模式提示', result.message ?? '请用管理员权限终端运行 npm run tauri dev')
          } else if (result) {
            // 已确认提权重启：应用即将以管理员重启，避免向调用方抛原始权限错误
            // 误导用户（后面 500ms 内会退出进程，重启后快照恢复自动重建 TUN）
            throw new Error('正在以管理员权限重启，重启后自动恢复虚拟网卡')
          }
        }
        lastError.value = msg
        throw e
      }
      throw e
    }
  }

  /**
   * 将 DataChannel 收到的二进制消息转发到后端 TUN
   *
   * 调用方在 `DataChannel.onmessage` 回调中调用此函数。后端 base64 解码 →
   * 协议帧 decode → 写入 TUN 接口。
   *
   * 失败时仅 console.warn，不抛错（避免 DataChannel.onmessage 回调链断掉）。
   *
   * @param raw DataChannel 收到的二进制消息
   * @returns 后端返回的转发结果；失败时返回 null
   */
  async function forwardToTun(
    raw: ArrayBuffer | Uint8Array,
  ): Promise<TunForwardResponse | null> {
    if (!running.value) return null
    try {
      return await tunForwardTo(raw)
    } catch (e) {
      console.warn('[Online] forwardToTun 失败:', e)
      return null
    }
  }

  /**
   * 停止 TUN 桥接（幂等）
   *
   * 1. 调用后端 `tun_stop` 销毁 TUN 接口
   * （事件 handler 由 onGlobalEvent 管理，无需手动移除）
   */
  async function stop(): Promise<void> {
    if (!running.value) return

    try {
      await tunStop()
    } catch (e) {
      console.warn('[Online] tunStop 失败:', e)
    } finally {
      running.value = false
      interfaceInfo.value = null
    }
  }

  if (autoStop) {
    onUnmounted(() => {
      // 事件 handler 由 onGlobalEvent 的 onUnmounted 自动移除；此处仅停止后端 TUN 接口
      // 后端 stop 不阻塞卸载（异步触发即可）
      void stop()
    })
  }

  return {
    // 状态
    running,
    interfaceInfo,
    lastError,
    // 方法
    start,
    forwardToTun,
    stop,
  }
}
