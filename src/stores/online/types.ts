/**
 * 联机 store 类型定义（Scaffolding 收敛版）
 *
 * 从 stores/online.ts 抽取的房间相关类型与空状态工厂函数。
 * 主 store 文件 re-export 本文件以保持 `@/stores/online` 路径兼容。
 */

import type { ModpackMeta } from '@/types/online'

/** 房间角色 */
export type RoomRole = 'host' | 'guest' | null

/** 创建房间步骤（UI 进度反馈用） */
export type RoomCreateStep = 'create' | null

/** 房间状态（Scaffolding 方案） */
export interface RoomState {
  /** 角色：房主 / 加入方 / null（未在房间） */
  role: RoomRole
  /** 完整房间码（U/NNNN-NNNN-SSSS-SSSS，S 段仅本地持有） */
  roomCode: string
  /** N 段公开标识（6 位显示名） */
  publicIdentifier: string
  /** 房主备注 */
  remark: string
  /** 是否公开（公开房间进大厅聚类） */
  isPublic: boolean
  /** 是否设置密码 */
  hasPassword: boolean
  /** 最大人数（服务端下发，0 = 未获取） */
  maxPlayers: number
  /** 房间过期时间（Unix 秒） */
  expiresAt: number
  /** 房主 MC 版本 */
  hostMcVersion: string
  /** 房主 MC 端口 */
  hostMcPort: number
  /** 房主加载器类型 */
  hostLoader: string
  /** 房主加载器版本号 */
  hostLoaderVersion: string
  /**
   * 房主整合包元数据（undefined = 纯原版房间）
   *
   * 加入方通过 room_join 响应同步，据此判断是否需要一键安装。
   */
  hostModpack: ModpackMeta | undefined
}

/** 创建空房间状态 */
export function emptyRoom(): RoomState {
  return {
    role: null,
    roomCode: '',
    publicIdentifier: '',
    remark: '',
    isPublic: false,
    hasPassword: false,
    maxPlayers: 0,
    expiresAt: 0,
    hostMcVersion: '',
    hostMcPort: 0,
    hostLoader: '',
    hostLoaderVersion: '',
    hostModpack: undefined,
  }
}
