/**
 * WebRTC mesh composable（房主专用，聚合入口）
 *
 * PC/DC 生命周期在 useWebRTCMesh/mesh-peer.ts，加密帧广播在 mesh-crypto.ts。
 * 本文件持有加密密钥 roomKey 并组合两个切片，对外 API 与原实现完全一致。
 */
import { onUnmounted, shallowRef } from 'vue'
import { useMeshPeer } from './useWebRTCMesh/mesh-peer'
import { useMeshCrypto } from './useWebRTCMesh/mesh-crypto'

export type { SdpResult } from './useWebRTCMesh/mesh-peer'

/**
 * 房主 mesh 多 PC 管理器
 *
 * 内部维护 `Map<participantId, ParticipantConn>`，所有操作按 participantId 索引。
 * 连接状态通过 `reactive(Map)` 暴露给 UI 层。
 */
export function useWebRTCMesh() {
  /**
   * DataChannel 加密密钥（阶段三子任务 8）
   *
   * null 表示未启用加密（兼容旧服务器）；非 null 时 `broadcastPacket` /
   * `sendToParticipant` 自动加密，`setDataChannelHandlers` 绑定的
   * `onMessage` 自动先解密再回调业务层。
   */
  const roomKey = shallowRef<CryptoKey | null>(null)

  /**
   * 注入 / 清除 DataChannel 加密密钥
   *
   * 房主创建房间后调用 `importRoomKey(store.roomState.roomKey)` 导入密钥，
   * 再调用此方法注入。房间关闭时调用 `setRoomKey(null)` 清除。
   *
   * @param key AES-GCM 密钥；null 表示禁用加密（透传原始帧）
   */
  function setRoomKey(key: CryptoKey | null): void {
    roomKey.value = key
  }

  // 切片组装：PC/DC 生命周期 + 加密帧发送
  const peer = useMeshPeer({ roomKey })
  const crypto = useMeshCrypto({ roomKey, conns: peer.conns, channelOpen: peer.channelOpen })

  // 组件卸载时自动关闭所有 PC，避免泄漏
  onUnmounted(() => peer.close())

  return {
    // 状态
    connectionStates: peer.connectionStates,
    channelOpen: peer.channelOpen,
    negotiating: peer.negotiating,
    // 方法
    createOfferFor: peer.createOfferFor,
    setRemoteAnswer: peer.setRemoteAnswer,
    setDataChannelHandlers: peer.setDataChannelHandlers,
    broadcastPacket: crypto.broadcastPacket,
    sendToParticipant: crypto.sendToParticipant,
    setRoomKey,
    closeParticipant: peer.closeParticipant,
    close: peer.close,
    getConnState: peer.getConnState,
    connectedCount: peer.connectedCount,
  }
}