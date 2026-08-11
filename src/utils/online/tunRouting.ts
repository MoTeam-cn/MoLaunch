/**
 * TUN 数据包定向路由工具
 *
 * 房主侧将 TUN 读到的 IP 包解析目标虚拟 IP，映射到对应参与者的 DataChannel 单播，
 * 替代无差别广播以消除上行带宽随人数翻倍的冗余；无法识别目标时回退广播。
 */
import { decode } from '@/utils/online/protocol'
import type { ParticipantInfo } from '@/types/online'

/**
 * 从房主 TUN 读出的协议帧中解析目标参与者 ID
 *
 * 帧 payload 为完整 IPv4 包（IPv4 头最小 20 字节，目标地址固定位于 16-19 偏移）。
 * 非 IPv4 / 长度不足 / 未匹配到参与者时返回 null（调用方应回退广播）。
 *
 * @param raw 后端 emit 的协议帧（ArrayBuffer）
 * @param participants 房主侧参与者列表（含虚拟 IP 映射）
 * @returns 目标参与者的 participantId；无法定向时返回 null
 */
export function resolveTunParticipantId(
  raw: ArrayBuffer,
  participants: ParticipantInfo[],
): string | null {
  const msg = decode(raw)
  if (!msg || msg.kind !== 'data') return null
  const ip = msg.payload
  if (ip.length < 20 || (ip[0] >> 4) !== 4) return null
  const dst = `${ip[16]}.${ip[17]}.${ip[18]}.${ip[19]}`
  const hit = participants.find((p) => p.virtualIp === dst)
  return hit?.participantId ?? null
}
