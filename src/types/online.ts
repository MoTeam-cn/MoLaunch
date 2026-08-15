/**
 * 联机功能类型定义（聚合入口）
 *
 * 各类型按域拆分到 ./online/ 子文件，本文件仅 re-export 以保持 @/types/online 路径兼容；
 * 字段命名 camelCase，与后端 minecraft::online / utils::online_manager action 对应。
 */
// 为保持中间状态可编译，暂时保留旧类型 re-export（清理阶段再移除）。
export * from './online/auth'
export * from './online/signaling'
export * from './online/modpack'
export * from './online/room'
export * from './online/nat'
export * from './online/whitelist'
export * from './online/lobby'
export * from './online/easytier'
export * from './online/lan'
export type { TunStartParams, TunStartResponse, TunForwardResponse, TunPacketPayload } from './online/tun'
export { EVENT_TUN_PACKET_OUT } from './online/tun'
