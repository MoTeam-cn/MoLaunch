//! WS 鉴权：token 生成与消息校验

use rand::RngCore;

/// 鉴权超时（秒）
pub const AUTH_TIMEOUT_SECS: u64 = 3;

/// 生成 32 字节随机 token（64 位十六进制）
pub fn generate_ws_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// 校验客户端首条消息是否为合法鉴权帧
///
/// 期望格式：`{"type":"auth","token":"<token>"}`
pub fn verify_auth_message(text: &str, expected_token: &str) -> bool {
    let v: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let msg_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let token = v.get("token").and_then(|t| t.as_str()).unwrap_or("");
    msg_type == "auth" && token == expected_token
}

/// 构造鉴权成功 ack
pub fn auth_ok_message() -> serde_json::Value {
    serde_json::json!({ "type": "auth_ok", "msg": "authenticated" })
}
