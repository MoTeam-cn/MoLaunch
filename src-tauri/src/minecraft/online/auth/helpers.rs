//! MoSign-v1 协议辅助函数

use rand::RngCore;

use super::super::crypto::b64u_encode;

/// 生成 16 字节随机 nonce（Base64Url）
pub(super) fn generate_nonce_b64u() -> String {
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    b64u_encode(&bytes)
}

/// 当前 Unix 时间戳（秒）
pub(super) fn now_timestamp() -> u64 {
    chrono::Utc::now().timestamp() as u64
}

/// 生成设备友好标识 `mcsdk-xxxx-xxxx-xxxx-xxxx`（小写十六进制）
pub fn generate_device_id() -> String {
    let mut bytes = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let hex = hex::encode(bytes);
    format!(
        "mcsdk-{}-{}-{}-{}",
        &hex[0..4],
        &hex[4..8],
        &hex[8..12],
        &hex[12..16]
    )
}
