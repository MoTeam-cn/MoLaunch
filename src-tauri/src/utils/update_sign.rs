//! 更新检查签名（MoSign-v1 轻量变体）
//!
//! 与 api-server 的 `utils/update_sign.rs` 保持同一协议：待签串为固定字段元组，
//! 签名参数走 query（`sign-lx` / `sign-ts` / `sign-v1`）。
//!
//! ## 密钥来源与轮换
//!
//! 1. **编译期内嵌**：构建时通过 `MOLAUNCH_UPDATE_SIGN_KEY` 注入初始密钥，
//!    保证首次请求即可签名。
//! 2. **响应下发**：服务端在签名有效的响应里回带比客户端更新的密钥
//!    （`sign_hint`），落盘到 appdata，下次请求优先使用。
//!
//! 因此轮换密钥不需要发版：老客户端靠服务端 soft 策略照常拿到更新，
//! 并在同一次响应里取回新密钥。
//!
//! 注意：内嵌密钥不是安全边界（二进制可被提取），它的作用是抬高接口滥用成本
//! 并提供稳定的灰度标识；真正的边界是 TLS + 安装包 Ed25519 签名 + 下载域名白名单。

use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

/// 签名方案版本：HMAC-SHA256 / 小写 hex
pub const SIGN_LX_HMAC_SHA256: &str = "1";

/// 编译期内嵌的初始签名密钥（hex）
///
/// 构建时通过环境变量 `MOLAUNCH_UPDATE_SIGN_KEY` 注入；未注入时为 None，
/// 此时客户端不带签名请求，服务端 soft 策略照常返回清单。
const EMBEDDED_SIGN_KEY: Option<&str> = option_env!("MOLAUNCH_UPDATE_SIGN_KEY");

/// 本地密钥提示文件名（存于 appdata 的 `updater` 子目录）
const HINT_FILE: &str = "update_sign_keys.json";

/// 用 hex 密钥计算 HMAC-SHA256，返回 64 位小写 hex
pub fn sign_hex(key_hex: &str, string_to_sign: &str) -> Option<String> {
    let key = decode_hex(key_hex)?;
    let mut mac = Hmac::<Sha256>::new_from_slice(&key).ok()?;
    mac.update(string_to_sign.as_bytes());
    Some(hex::encode(mac.finalize().into_bytes()))
}

/// 当前 Unix 秒时间戳
fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 加载可用密钥集合（内嵌 K0 + 本地落盘的 hint，同版本以 hint 为准）
pub fn load_keys() -> HashMap<String, String> {
    let mut keys = HashMap::new();
    if let Some(k) = EMBEDDED_SIGN_KEY.filter(|k| !k.is_empty()) {
        keys.insert(SIGN_LX_HMAC_SHA256.to_string(), k.to_string());
    }
    for (lx, key) in read_hint_file() {
        keys.insert(lx, key);
    }
    keys
}

/// 构造启动上报待签串
///
/// 形如 POST 换行 /v1/launch/ping 换行 device_pk 换行 device_id 换行 timestamp 换行 lx。
/// 必须与服务端 client_sign::build_ping_string_to_sign 逐字一致。
pub fn build_ping_string_to_sign(
    device_pk: &str,
    device_id: &str,
    timestamp: &str,
    lx: &str,
) -> String {
    format!("POST\n/v1/launch/ping\n{device_pk}\n{device_id}\n{timestamp}\n{lx}")
}

/// 构造启动上报的签名 query 片段
///
/// 返回 `("&sign-lx=1&sign-ts=...&sign-v1=...", Some("1"))`；无可用密钥时返回空片段。
pub fn build_ping_sign_query(device_pk: &str, device_id: &str) -> (String, Option<String>) {
    let keys = load_keys();
    // 取方案版本最高的密钥
    let Some((lx, key)) = keys
        .iter()
        .max_by_key(|(lx, _)| lx.parse::<u32>().unwrap_or(0))
        .map(|(lx, k)| (lx.clone(), k.clone()))
    else {
        return (String::new(), None);
    };

    let ts = now_unix().to_string();
    let string_to_sign = build_ping_string_to_sign(device_pk, device_id, &ts, &lx);
    let Some(sig) = sign_hex(&key, &string_to_sign) else {
        return (String::new(), None);
    };

    (
        format!("&sign-lx={lx}&sign-ts={ts}&sign-v1={sig}"),
        Some(lx),
    )
}

/// 保存服务端下发的密钥提示（与本地已有集合合并）
///
/// 由 `commands::online::manager::launch_ping` 在收到响应后调用：
/// 服务端在 `data.sign_hint` 里回带比本地更新的密钥时并入本地集合。
///
/// `hint` 形如 `{"keys": {"2": "<hex>"}, "preferred_lx": "2"}`；结构不符时静默忽略。
pub fn save_hint(hint: &serde_json::Value) {
    let Some(obj) = hint.get("keys").and_then(|v| v.as_object()) else {
        return;
    };
    let incoming: HashMap<String, String> = obj
        .iter()
        .filter_map(|(lx, v)| v.as_str().map(|s| (lx.clone(), s.to_string())))
        .filter(|(_, key)| !key.is_empty())
        .collect();
    if incoming.is_empty() {
        return;
    }

    let mut merged = read_hint_file();
    merged.extend(incoming);
    if let Err(e) = write_hint_file(&merged) {
        crate::log_warn!("[Updater] 保存签名密钥提示失败: {}", e);
    }
}

/// 读取本地 hint 文件（不存在或损坏时返回空集合）
fn read_hint_file() -> HashMap<String, String> {
    let Ok(dir) = crate::storage::appdata::ensure_appdata_subdir("updater") else {
        return HashMap::new();
    };
    let Ok(text) = std::fs::read_to_string(dir.join(HINT_FILE)) else {
        return HashMap::new();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

/// 写入本地 hint 文件
fn write_hint_file(keys: &HashMap<String, String>) -> Result<(), String> {
    let dir = crate::storage::appdata::ensure_appdata_subdir("updater")?;
    let text = serde_json::to_string_pretty(keys).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(HINT_FILE), text).map_err(|e| e.to_string())
}

/// hex 字符串解码（大小写均可），非法输入返回 None
fn decode_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for i in (0..bytes.len()).step_by(2) {
        out.push((nibble(bytes[i])? << 4) | nibble(bytes[i + 1])?);
    }
    Some(out)
}

/// 单个 hex 字符转半字节
fn nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
#[path = "update_sign_test.rs"]
mod update_sign_test;
