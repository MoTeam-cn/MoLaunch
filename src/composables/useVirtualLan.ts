/**
 * 虚拟网卡桥接 composable（阶段三子任务 5：数据分发打通）
 *
 * 房主与加入方共用此 composable，封装：
 * - `start(selfVirtualIp, subnet)`：解析 CIDR → 调用 `tunStart` → 订阅 `online://tun-packet-out` 事件
 * - `onTunPacket` 回调：TUN 读到 IP 包时触发（房主调 `hostMesh.broadcastPacket`，加入方调 `dataChannel.send`）
 * - `forwardToTun(raw)`：将 DataChannel 收到的二进制消息转发到后端 TUN（base64 编码后 invoke `tun_forward_to`）
 * - `stop()`：停止桥接，销毁 TUN 接口
 *
 * # 数据流
 *
 * ```text
 * 后端 TUN 读包                     前端 DataChannel
 *   │                                  │
 *   │ 1. TUN.recv → IP 包              │
 *   │ 2. protocol::encode → 帧         │
 *   │ 3. emit(EVENT_TUN_PACKET_OUT)    │
 *   │ ─────────────────────────────>   │ 4. listen → onTunPacket 回调
 *   │                                  │ 5. hostMesh.broadcastPacket(raw) 或 dataChannel.send(raw)
 *   │                                  │
 *   │                                  │ 6. DataChannel.onmessage → ArrayBuffer
 *   │ <─────────────────────────────   │ 7. forwardToTun(raw) → base64 → invoke `tun_forward_to`
 *   │ 8. base64 decode → 帧            │
 *   │ 9. protocol::decode → IP 包      │
 *   │ 10. TUN.send → 写入接口          │
 * ```
 *
 * # 设计约束
 *
 * - 监听器在 `start` 时注册，`stop` / `onUnmounted` 时清理（避免泄漏）
 * - 竞态保护：`listen` 是异步的，await 期间组件卸载时立即 unlisten 新拿到的句柄
 * - `forwardToTun` 失败不抛错（避免 DataChannel.onmessage 回调链断掉），仅打 warn
 *
 * @example 房主侧使用
 * const lan = useVirtualLan({
 *   onTunPacket: (raw) => hostMesh.broadcastPacket(raw),
 * })
 * await lan.start(room.selfVirtualIp, room.subnet)
 */

import { onUnmounted, ref, shallowRef } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import {
  tunStart,
  tunForwardTo,
  tunStop,
} from '@/utils/api/online-manager'
import {
  EVENT_TUN_PACKET_OUT,
  type TunForwardResponse,
  type TunPacketPayload,
  type TunStartResponse,
} from '@/types/online'

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
  /** 桥接是否运行中 */
  const running = ref(false)
  /** TUN 接口信息（start 成功后填充） */
  const interfaceInfo = shallowRef<TunStartResponse | null>(null)
  /** 最近一次错误（启动失败时填充，null 表示无错误） */
  const lastError = ref<string | null>(null)

  let unlisten: UnlistenFn | null = null
  let isMounted = true

  /**
   * 启动 TUN 桥接
   *
   * 1. 若已在运行，先停止（防止泄漏）
   * 2. 注册 `online://tun-packet-out` 事件监听器
   * 3. 调用后端 `tun_start` 创建 TUN 接口 + 启动读写循环
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

    // 注册事件监听器（如未注册）
    if (!unlisten) {
      const unlistenFn = await listen<TunPacketPayload>(
        EVENT_TUN_PACKET_OUT,
        (event) => {
          if (!isMounted || !running.value) return
          // number[] → ArrayBuffer
          const bytes = new Uint8Array(event.payload)
          options.onTunPacket(bytes.buffer)
        },
      )
      // await 期间组件已卸载：立即 unlisten 刚拿到的句柄，避免泄漏
      if (!isMounted) {
        unlistenFn()
        throw new Error('组件已卸载，TUN 桥接启动取消')
      }
      unlisten = unlistenFn
    }

    // 解析子网前缀长度
    const prefixLen = parsePrefixLen(subnet)

    // 调用后端启动 TUN
    const info = await tunStart({ ipv4: selfVirtualIp, prefixLen })
    interfaceInfo.value = info
    running.value = true
    lastError.value = null
    return info
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
   * 2. 取消事件监听器
   */
  async function stop(): Promise<void> {
    if (!running.value) {
      // 即使 running 为 false，也清理可能残留的监听器
      if (unlisten) {
        unlisten()
        unlisten = null
      }
      return
    }

    try {
      await tunStop()
    } catch (e) {
      console.warn('[Online] tunStop 失败:', e)
    } finally {
      running.value = false
      interfaceInfo.value = null
      if (unlisten) {
        unlisten()
        unlisten = null
      }
    }
  }

  onUnmounted(() => {
    isMounted = false
    if (unlisten) {
      unlisten()
      unlisten = null
    }
    // 后端 stop 不阻塞卸载（异步触发即可）
    void stop()
  })

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
