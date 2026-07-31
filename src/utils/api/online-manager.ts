/**
 * 联机功能统一 API（聚合入口）
 *
 * 后端 `online_manager` IPC 命令通过 `action` 字段分发到不同子模块
 * （参照 `community_manager` / `meta_manager` 模式）。
 *
 * 字段名约定：后端 Params 结构体使用 `#[serde(rename_all = "camelCase")]`，
 * 故前端 params 对象的字段名一律使用 camelCase。
 *
 * 按 action 类别拆分到 `./online-manager/` 子文件，本文件仅做 re-export 以保持
 * `@/utils/api/online-manager` 路径对调用方完全兼容：
 * - `core.ts`：onlineManager 调用入口 + ONLINE_ACTIONS 常量 + OnlineAction 类型
 * - `auth.ts`：设备认证（status/register/login/logout/clear/init/refresh）
 * - `room.ts`：房间信令（stun/create/get/close/join/submit/list/confirm/keepalive/leave/kick/ban/participants）
 * - `turn.ts`：TURN 中继（房主独占）
 * - `mesh.ts`：mesh 拓扑参与者级 SDP Offer
 * - `tun.ts`：TUN 桥接 + 管理员重启
 * - `whitelist.ts`：房主白名单管理
 * - `lobby.ts`：大厅浏览
 */
export * from './online-manager/core'
export * from './online-manager/auth'
export * from './online-manager/room'
export * from './online-manager/turn'
export * from './online-manager/mesh'
export * from './online-manager/tun'
export * from './online-manager/whitelist'
export * from './online-manager/lobby'
