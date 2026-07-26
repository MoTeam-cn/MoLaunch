//! 联机功能模块
//!
//! 对接 MoLaunch API Server 的 P2P 联机能力，包含：
//! - `crypto`：加密原语（Ed25519/X25519/HKDF/AES-GCM/RSA-OAEP/Base64Url）
//! - `ecies`：ECIES 加密信封（`{payload, key}`，业务接口端到端加密）
//! - `auth`：MoSign-v1 设备注册/登录/登出
//! - `storage`：设备密钥持久化（Ed25519/X25519 私钥、device_pk、device_token）
//! - `client`：api-server HTTP 客户端封装（统一加 ECIES 信封、JWT 携带）
//! - `signaling`：信令接口客户端（房间创建/加入/退出/踢人/保活等，阶段二实现）
//!
//! 协议参考：
//! - `api-server/docs/auth.md`：MoSign-v1 设备认证协议
//! - `api-server/docs/signaling.md`：P2P 联机信令接口
//!
//! 安全约束：
//! - 设备私钥持久化到本地安全存储（与 auth_storage 一致的位置）
//! - JWT 过期前主动登录续期，避免业务接口 401
//! - 所有 `/v1` 业务请求必须走 ECIES 加密信封

pub mod auth;
pub mod client;
pub mod crypto;
pub mod ecies;
pub mod signaling;
pub mod storage;
