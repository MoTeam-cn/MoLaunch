/**
 * 联机功能类型定义（聚合入口）
 *
 * 与后端 `minecraft::online` 模块及 `utils::online_manager` 中注册的 action 对应。
 * 字段命名采用 camelCase（后端 `#[serde(rename_all = "camelCase")]` 或显式 `rename`）。
 *
 * 类型按域拆分到 `./online/` 子文件，本文件仅做 re-export 以保持
 * `@/types/online` 路径对调用方完全兼容：
 * - `auth.ts`：设备认证状态 + 启动静默认证结果 + 服务器时间
 * - `signaling.ts`：统一业务响应 + STUN/ICE/TURN 服务器类型
 * - `modpack.ts`：整合包元数据 + 本地元数据文件 + 校验结果
 * - `room.ts`：房间创建/加入/查询/参与者/封禁/mesh Offer 全部房间相关类型
 * - `tun.ts`：TUN 桥接参数/响应 + 数据包事件
 * - `nat.ts`：NAT 类型枚举 + 检测结果
 * - `whitelist.ts`：房主白名单条目 + 列表响应
 * - `lobby.ts`：大厅列表/分类/整合包摘要
 */
export * from './online/auth'
export * from './online/signaling'
export * from './online/modpack'
export * from './online/room'
export * from './online/tun'
export * from './online/nat'
export * from './online/whitelist'
export * from './online/lobby'
