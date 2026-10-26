//! 加密原语（转发到私有协议库）
//!
//! MoSign-v1 所需的 Base64Url / Ed25519 / X25519 / HKDF / AES-GCM / RSA-OAEP
//! 实现位于 `molaunch-protocol`，本模块只做转发，保持既有调用路径不变。

pub use molaunch_protocol::crypto::*;
