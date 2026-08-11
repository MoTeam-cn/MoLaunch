/**
 * 联机 store 类型定义
 *
 * 从 stores/online.ts 抽取的房间相关类型与空状态工厂函数。
 * 主 store 文件 re-export 本文件以保持 `@/stores/online` 路径兼容。
 */

import type {
  IceServerEntry,
  ModpackMeta,
  ParticipantInfo,
} from '@/types/online'

/** 房间角色 */
export type RoomRole = 'host' | 'guest' | null

/**
 * 创建房间步骤（UI 进度反馈用）
 *
 * mesh 拓扑下房主创建房间不再生成本地 Offer（改为 per-participant 按需生成）：
 * - `stun`：获取 STUN 服务器列表
 * - `create`：调用后端创建房间
 * - `null`：未在创建中 / 已完成 / 失败
 */
export type RoomCreateStep = 'stun' | 'create' | null

/** 房间状态（阶段二） */
export interface RoomState {
  /** 角色：房主 / 加入方 / null（未在房间） */
  role: RoomRole
  /** 房间码 */
  roomCode: string
  /** 房主虚拟 IP */
  hostVirtualIp: string
  /** 自己的虚拟 IP */
  selfVirtualIp: string
  /** 子网 CIDR */
  subnet: string
  /** 最大人数 */
  maxPlayers: number
  /** 房间过期时间（Unix 秒） */
  expiresAt: number
  /**
   * STUN 服务器列表（兼容字段，旧房间使用）
   *
   * 阶段三子任务 7 后优先使用 `iceServers`；此字段仅当 `iceServers` 为空时回退使用。
   */
  stunServers: string[]
  /**
   * ICE 服务器列表（统一 STUN + TURN，含凭据）
   *
   * 阶段三子任务 7 新增。WebRTC PeerConnection 配置优先使用此字段。
   * 房主创建房间时由 STUN + 用户自定义 TURN 组合；加入方从后端响应获取。
   */
  iceServers: IceServerEntry[]
  /** 房主 MC 版本（加入方需匹配） */
  hostMcVersion: string
  /** 房主 MC 端口 */
  hostMcPort: number
  /**
   * 房主是否手动指定 MC 端口（最高可信度）
   *
   * `true` 时以手动值为准，忽略自动捕获（日志/监听端口）更新；
   * 手动值经 HOST_MC_PORT 控制消息广播给参与者。
   */
  hostMcPortManual: boolean
  /** 当前参与者列表（房主维护） */
  participants: ParticipantInfo[]
  /** 加入方的 participant_id */
  participantId: string | null
  /**
   * 是否启用白名单（阶段三子任务 8 安全加强）
   *
   * `true` 时仅白名单内设备可加入；`false` 时允许任何已注册设备加入。
   * 房主创建房间时由表单开关决定，运行期可通过 `setWhitelistEnabled` 动态修改。
   */
  whitelistEnabled: boolean
  /**
   * DataChannel 加密密钥（Base64Url 编码的 32 字节 AES-256 密钥）
   *
   * 阶段三子任务 8 新增。空字符串表示未启用加密（兼容旧服务器）。
   * 房主创建房间 / 加入方加入房间时由后端下发，房主与参与者共享同一密钥。
   * 前端 protocol.ts 用此密钥做 AES-GCM 加解密。
   */
  roomKey: string
  /**
   * 房主整合包元数据（联机大厅阶段 4 新增）
   *
   * `undefined` 表示纯原版房间（房主未关联整合包）；
   * 加入方通过 `refreshRoomInfo` 从 `RoomInfoResponse.modpack` 同步，
   * 据此判断是否需要一键安装。
   */
  hostModpack: ModpackMeta | undefined
}

/** 创建空房间状态 */
export function emptyRoom(): RoomState {
  return {
    role: null,
    roomCode: '',
    hostVirtualIp: '',
    selfVirtualIp: '',
    subnet: '',
    maxPlayers: 0,
    expiresAt: 0,
    stunServers: [],
    iceServers: [],
    hostMcVersion: '',
    hostMcPort: 0,
    hostMcPortManual: false,
    participants: [],
    participantId: null,
    whitelistEnabled: false,
    roomKey: '',
    hostModpack: undefined,
  }
}
