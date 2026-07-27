/**
 * DataChannel 消息协议（与后端 `src-tauri/src/minecraft/online/protocol.rs` 一一对应）
 *
 * 二进制帧格式（大端序）：
 * ```
 * +--------+--------+--------+--------+--------+--------+--------+-----------+
 * | type   |    seq (u32 BE)    |    length (u16 BE)  | payload             |
 * | 1 byte |       4 bytes      |       2 bytes       | N bytes             |
 * +--------+--------+--------+--------+--------+--------+--------+-----------+
 * ```
 *
 * - type=0x01 Data：IP 包
 * - type=0x02 Control：控制消息（payload 首字节为 subtype）
 *   - subtype=0x01 Heartbeat
 *   - subtype=0x02 StatusQuery
 *   - subtype=0x03 StatusResponse
 *   - subtype=0x04 HostMcPort（payload 为 2 字节大端序 u16 端口）
 *   - subtype=0x05 TurnServers（payload 为 JSON UTF-8 字节，结构 IceServerEntry[]）
 * - type=0x03 Error：UTF-8 错误描述
 *
 * 用途：
 * - 前端 DataChannel.onMessage 收到 ArrayBuffer 后，用 `decode` 解析消息类型
 * - 房主侧检测到 MC 局域网端口后，用 `encodeHostMcPort` 编码并广播给所有参与者
 * - 加入方侧解码后，若为 HostMcPort 控制消息，更新本地 store.roomState.hostMcPort
 * - 房主拉取系统 TURN 服务器后，用 `encodeTurnServers` 编码并广播给所有参与者
 * - 加入方侧解码后，若为 TurnServers 控制消息，重建 PeerConnection 以应用新 ICE 配置
 */

import type { IceServerEntry } from '@/types/online'

/** 消息类型枚举值（与后端 MessageType 一致） */
export const MESSAGE_TYPE = {
  DATA: 0x01,
  CONTROL: 0x02,
  ERROR: 0x03,
} as const

/** 控制消息子类型枚举值（与后端 ControlSubtype 一致） */
export const CONTROL_SUBTYPE = {
  HEARTBEAT: 0x01,
  STATUS_QUERY: 0x02,
  STATUS_RESPONSE: 0x03,
  HOST_MC_PORT: 0x04,
  TURN_SERVERS: 0x05,
} as const

/** 帧头部长度（type + seq + length = 1 + 4 + 2 = 7 字节） */
export const FRAME_HEADER_LEN = 7

/** 解析后的消息抽象表示 */
export type ProtocolMessage =
  | { kind: 'data'; seq: number; payload: Uint8Array }
  | { kind: 'control'; seq: number; subtype: number; payload: Uint8Array }
  | { kind: 'error'; seq: number; message: string }

/**
 * 解码二进制帧为消息对象
 *
 * 期望 `bytes` 包含完整的一帧（头部 + payload）。
 * 解析失败时返回 null（调用方应静默丢弃，避免单条坏帧影响整体流程）。
 */
export function decode(bytes: ArrayBuffer): ProtocolMessage | null {
  if (bytes.byteLength < FRAME_HEADER_LEN) return null
  const view = new DataView(bytes)
  const type = view.getUint8(0)
  const seq = view.getUint32(1, false) // big-endian
  const length = view.getUint16(5, false)
  if (bytes.byteLength < FRAME_HEADER_LEN + length) return null

  const payloadBytes = new Uint8Array(bytes, FRAME_HEADER_LEN, length)

  switch (type) {
    case MESSAGE_TYPE.DATA:
      return { kind: 'data', seq, payload: payloadBytes }
    case MESSAGE_TYPE.CONTROL:
      if (payloadBytes.length < 1) return null
      return {
        kind: 'control',
        seq,
        subtype: payloadBytes[0],
        payload: payloadBytes.slice(1),
      }
    case MESSAGE_TYPE.ERROR:
      try {
        return {
          kind: 'error',
          seq,
          message: new TextDecoder().decode(payloadBytes),
        }
      } catch {
        return null
      }
    default:
      return null
  }
}

/**
 * 编码 HostMcPort 控制消息为二进制帧
 *
 * 房主检测到 MC 局域网端口后，调用此函数生成 ArrayBuffer，
 * 通过 hostMesh.broadcastPacket 下发给所有已联通的参与者。
 *
 * 帧结构（与后端 protocol.rs encode 一致）：
 * - type(1) = 0x02 Control
 * - seq(4) = 大端序 u32
 * - length(2) = 3（subtype 1 字节 + port 2 字节）
 * - subtype(1) = 0x04 HostMcPort
 * - port(2) = 大端序 u16
 */
export function encodeHostMcPort(seq: number, port: number): ArrayBuffer {
  const buf = new ArrayBuffer(10)
  const view = new DataView(buf)
  view.setUint8(0, MESSAGE_TYPE.CONTROL)
  view.setUint32(1, seq, false) // big-endian
  view.setUint16(5, 3, false) // length = subtype(1) + port(2) = 3
  view.setUint8(7, CONTROL_SUBTYPE.HOST_MC_PORT)
  view.setUint16(8, port, false) // big-endian
  return buf
}

/**
 * 从 Control + HostMcPort 消息的 payload 解析端口号
 *
 * 期望 payload 长度为 2 字节（大端序 u16）。其他长度返回 null。
 */
export function parseHostMcPortPayload(payload: Uint8Array): number | null {
  if (payload.length !== 2) return null
  return (payload[0] << 8) | payload[1]
}

/**
 * 编码 TurnServers 控制消息为二进制帧
 *
 * 房主拉取系统 TURN 服务器后，调用此函数生成 ArrayBuffer，
 * 通过 hostMesh.broadcastPacket 下发给所有已联通的参与者。
 *
 * payload 为 `IceServerEntry[]` 的 JSON UTF-8 字节，结构示例：
 * ```json
 * [{"urls":["turn:turn.example.com:3478"],"username":"foo","credential":"bar"}]
 * ```
 *
 * 帧结构（与后端 protocol.rs encode 一致）：
 * - type(1) = 0x02 Control
 * - seq(4) = 大端序 u32
 * - length(2) = 1 + jsonBytes.length（subtype 1 字节 + JSON N 字节）
 * - subtype(1) = 0x05 TurnServers
 * - json(N) = UTF-8 编码的 JSON 字符串
 *
 * 单条 TURN 列表通常 < 1KB，远低于 DataChannel 16KB 上限。
 * 即使 `iceServers` 为空数组也会构造合法消息（参与者收到后应跳过空列表，不重建 PC）。
 */
export function encodeTurnServers(seq: number, iceServers: IceServerEntry[]): ArrayBuffer {
  const json = JSON.stringify(iceServers)
  const jsonBytes = new TextEncoder().encode(json)
  // type(1) + seq(4) + length(2) + subtype(1) + json(N) = 8 + N
  const buf = new ArrayBuffer(8 + jsonBytes.length)
  const view = new DataView(buf)
  view.setUint8(0, MESSAGE_TYPE.CONTROL)
  view.setUint32(1, seq, false) // big-endian
  view.setUint16(5, 1 + jsonBytes.length, false) // length = subtype(1) + json(N)
  view.setUint8(7, CONTROL_SUBTYPE.TURN_SERVERS)
  new Uint8Array(buf, 8, jsonBytes.length).set(jsonBytes)
  return buf
}

/**
 * 从 Control + TurnServers 消息的 payload 解析 ICE 服务器列表
 *
 * 期望 payload 为 `IceServerEntry[]` 的 JSON UTF-8 字节。
 * 解析失败时返回 null（调用方应静默丢弃，保持现有 PC 不变）。
 *
 * @returns 解析后的 ICE 服务器列表；空数组表示房主明确下发空列表
 *          （如系统 TURN 全部不可用），调用方可据此决定是否重建 PC
 */
export function decodeTurnServersPayload(payload: Uint8Array): IceServerEntry[] | null {
  try {
    const json = new TextDecoder().decode(payload)
    const parsed = JSON.parse(json)
    if (!Array.isArray(parsed)) return null
    // 校验每项结构：必须有 urls 数组
    const result: IceServerEntry[] = []
    for (const item of parsed) {
      if (!item || typeof item !== 'object' || !Array.isArray(item.urls)) {
        return null
      }
      const entry: IceServerEntry = { urls: item.urls as string[] }
      if (typeof item.username === 'string') entry.username = item.username
      if (typeof item.credential === 'string') entry.credential = item.credential
      result.push(entry)
    }
    return result
  } catch {
    return null
  }
}
