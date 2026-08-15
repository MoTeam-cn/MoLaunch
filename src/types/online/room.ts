/**
 * 联机功能类型定义 - 房间域（Scaffolding 收敛版）
 *
 * 房间码为 `U/NNNN-NNNN-SSSS-SSSS`，N 段为公开标识，S 段为组网密钥。
 * 房主本地生成完整码后经 `room_create` 登记；加入方经 `room_join` 闸门拿完整码。
 */
import type { ModpackMeta } from './modpack'

/** 创建房间请求（房主本地生成完整 Scaffolding 码后登记） */
export interface CreateRoomParams {
  /** 完整 Scaffolding 房间码 `U/NNNN-NNNN-SSSS-SSSS` */
  roomCode: string
  /** 房主备注（大厅展示） */
  remark?: string
  /** 是否公开（公开房间进大厅，按整合包聚类） */
  isPublic?: boolean
  /** 房间密码（空串 = 无密码） */
  password?: string
  /** 房主 MC 版本 */
  hostMcVersion: string
  /** 房主 MC 端口（联机中心 hostname 用） */
  hostMcPort: number
  /** 房主加载器类型 */
  hostLoader?: string
  /** 房主加载器版本号 */
  hostLoaderVersion?: string
  /** 整合包元数据（undefined = 纯原版房间） */
  modpack?: ModpackMeta
}

/** 创建房间响应 */
export interface CreateRoomResponse {
  roomCode: string
  createdAt: number
  expiresAt: number
}

/** 房间公开信息（room_get） */
export interface RoomInfoResponse {
  /** 完整房间码（无权限的公开查询可能省略） */
  roomCode: string
  /** N 段公开标识（大厅展示/去重） */
  publicIdentifier: string
  hostDevicePk: string
  hasPassword: boolean
  remark: string
  status: string
  createdAt: number
  expiresAt: number
  hostMcVersion: string
  hostMcPort: number
  hostLoader?: string
  hostLoaderVersion?: string
  modpack?: ModpackMeta
}

/** 加入房间响应（join 闸门通过后返回完整码，供房客解析组网） */
export interface JoinRoomResponse {
  roomCode: string
  publicIdentifier: string
  remark: string
  hasPassword: boolean
  hostMcVersion: string
  hostMcPort: number
  hostLoader?: string
  hostLoaderVersion?: string
  modpack?: ModpackMeta
}
