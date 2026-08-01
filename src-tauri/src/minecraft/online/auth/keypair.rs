//! 复用密钥对容器（注册时一次性生成 Ed25519 + X25519）

use super::super::crypto::{Ed25519KeyPair, X25519StaticKeyPair};

/// 复用密钥对容器（注册时一次性生成 Ed25519 + X25519）
pub struct OnlineKeyPair {
    pub ed25519: Ed25519KeyPair,
    pub x25519: X25519StaticKeyPair,
}

impl OnlineKeyPair {
    /// 生成新的密钥对组合
    pub fn generate() -> Self {
        Self {
            ed25519: Ed25519KeyPair::generate(),
            x25519: X25519StaticKeyPair::generate(),
        }
    }
}
