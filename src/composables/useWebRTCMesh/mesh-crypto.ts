/**
 * WebRTC mesh - 加密帧广播切片
 *
 * 依赖 useMeshPeer 的 conns / channelOpen 状态与注入的 roomKey，
 * 负责向已联通通道发送加密/明文帧。
 */
import { type ShallowRef } from 'vue'
import { encryptFrame } from '@/utils/online/crypto'
import type { ParticipantConn } from './mesh-peer'

export interface MeshCryptoDeps {
  /** DataChannel 加密密钥（null 表示未启用加密） */
  roomKey: ShallowRef<CryptoKey | null>
  /** 参与者连接表（由 mesh-peer 切片持有） */
  conns: ShallowRef<Map<string, ParticipantConn>>
  /** 各参与者 DataChannel open 状态（reactive Map） */
  channelOpen: Map<string, boolean>
}

/** mesh 加密帧发送切片 */
export function useMeshCrypto(deps: MeshCryptoDeps) {
  const { roomKey, conns, channelOpen } = deps

  /**
   * 向所有已 open 的 DataChannel 广播二进制包
   *
   * 用于房主 TUN 读到的 IP 包下发到所有参与者。
   * 若 `roomKey` 已注入，先加密 `raw` 再发送；否则透传原始帧。
   * 加密只在发送前执行一次，随后广播给所有通道。
   *
   * @param raw 二进制数据（ArrayBuffer，原始协议帧）
   * @returns 实际发送到的参与者数量
   */
  async function broadcastPacket(raw: ArrayBuffer): Promise<number> {
    const key = roomKey.value
    const payload = key ? await encryptFrame(raw, key) : raw
    let sent = 0
    for (const [participantId, conn] of conns.value) {
      if (channelOpen.get(participantId) && conn.channel.readyState === 'open') {
        try {
          conn.channel.send(payload)
          sent++
        } catch {
          /* 单个通道发送失败不影响其他 */
        }
      }
    }
    return sent
  }

  /**
   * 向单个参与者发送二进制包
   *
   * 若 `roomKey` 已注入，先加密 `raw` 再发送。
   *
   * @param participantId 参与者 ID
   * @param raw 二进制数据（原始协议帧）
   */
  async function sendToParticipant(participantId: string, raw: ArrayBuffer): Promise<boolean> {
    const conn = conns.value.get(participantId)
    if (!conn || conn.channel.readyState !== 'open') return false
    const key = roomKey.value
    const payload = key ? await encryptFrame(raw, key) : raw
    try {
      conn.channel.send(payload)
      return true
    } catch {
      return false
    }
  }

  return {
    broadcastPacket,
    sendToParticipant,
  }
}