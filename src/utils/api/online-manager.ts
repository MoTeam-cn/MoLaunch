/**
 * 联机功能统一 API（聚合入口）
 *
 * 按 action 类别拆分到 ./online-manager/ 子文件，本文件仅 re-export 保持路径兼容；
 * 后端 online_manager IPC 按 action 分发，params 字段一律 camelCase。
 */
export * from './online-manager/core'
export * from './online-manager/auth'
export * from './online-manager/room'
export * from './online-manager/lobby'
export * from './online-manager/easytier'
export * from './online-manager/lan'
