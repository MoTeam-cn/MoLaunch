/**
 * Frp 内网穿透相关类型定义
 *
 * 与后端 commands/frp/mod.rs 中的 Rust 类型一一对应。
 * 按域拆分：隧道/日志见 tunnel，厂商/公共服务器见 provider，认证见 auth。
 */
export * from './frp/tunnel'
export * from './frp/provider'
export * from './frp/auth'