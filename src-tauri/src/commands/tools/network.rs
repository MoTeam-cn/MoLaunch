//! 网络工具（延迟测试 + 服务器状态检测 SLP）

use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use futures_util::future::join_all;
use reqwest::Client;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::state::AppState;

use super::types::{
    LatencyItem, NetworkLatencyResult, NetworkLatencyTestParams, ServerPingParams, ServerPingResult,
};

/// 并发测试多个 URL 的 HTTP 延迟
///
/// 复用同一个 `reqwest::Client`（10 秒超时），用 `join_all` 并发请求所有 URL。
/// 失败时 `latency_ms=None`、`status_code=0`、`error` 填失败原因。
pub async fn latency_test(
    state: &AppState,
    params: NetworkLatencyTestParams,
) -> Result<serde_json::Value, String> {
    let _game_dir = {
        let config = state.config.lock().await;
        crate::state::resolve_game_dir(&config.game_dir)
    };

    log_info!("[NetworkLatency] 测试 {} 个 URL", params.urls.len());

    let client = Arc::new(
        Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(log_err("构建 HTTP client 失败"))?,
    );

    let futures: Vec<_> = params
        .urls
        .into_iter()
        .map(|url| {
            let client = client.clone();
            async move {
                let start = Instant::now();
                match client.get(&url).send().await {
                    Ok(resp) => {
                        let latency = start.elapsed().as_millis() as u64;
                        let status = resp.status().as_u16();
                        LatencyItem {
                            url,
                            latency_ms: Some(latency),
                            status_code: status,
                            error: String::new(),
                        }
                    }
                    Err(e) => LatencyItem {
                        url,
                        latency_ms: None,
                        status_code: 0,
                        error: e.to_string(),
                    },
                }
            }
        })
        .collect();

    let results = join_all(futures).await;

    log_info!("[NetworkLatency] 完成 {} 个 URL 测试", results.len());

    let result = NetworkLatencyResult { results };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// Minecraft 服务器状态检测（SLP 1.7+）
///
/// 失败时（连接超时 / 被拒 / 协议异常）所有字段填默认值，`error` 填失败原因，`latency_ms=0`。
pub async fn server_ping(
    state: &AppState,
    params: ServerPingParams,
) -> Result<serde_json::Value, String> {
    let _game_dir = {
        let config = state.config.lock().await;
        crate::state::resolve_game_dir(&config.game_dir)
    };

    log_info!("[ServerPing] ping {}:{}", params.host, params.port);

    let result = match ping_server(&params.host, params.port).await {
        Ok(r) => {
            log_info!(
                "[ServerPing] success: {}:{} latency={}ms online={}/{}",
                params.host,
                params.port,
                r.latency_ms,
                r.online,
                r.max
            );
            r
        }
        Err(e) => {
            log_warn!(
                "[ServerPing] failed: {}:{} err={}",
                params.host,
                params.port,
                e
            );
            ServerPingResult {
                motd: String::new(),
                online: 0,
                max: 0,
                version: String::new(),
                latency_ms: 0,
                favicon: None,
                error: e,
            }
        }
    };

    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 执行 SLP 协议交换
///
/// TCP 连接超时 5 秒；连接成功后整个握手 + 状态请求 + Ping 交换超时 5 秒。
async fn ping_server(host: &str, port: u16) -> Result<ServerPingResult, String> {
    // 1. TCP 连接（5 秒超时）
    let connect_fut = TcpStream::connect((host, port));
    let mut stream = tokio::time::timeout(Duration::from_secs(5), connect_fut)
        .await
        .map_err(|_| format!("连接超时 (5s): {}:{}", host, port))?
        .map_err(|e| format!("连接失败: {}", e))?;
    stream.set_nodelay(true).ok();

    // 2-5. SLP 交换（整体 5 秒超时）
    let slp_fut = async {
        // 发送 Handshake 包 (Packet ID 0x00)
        let mut handshake = Vec::new();
        write_varint(&mut handshake, 0x00); // Packet ID
        write_varint(&mut handshake, -1); // Protocol version = -1 (兼容)
        write_string(&mut handshake, host); // 服务器地址（UTF-8 + VarInt 长度前缀）
        handshake.extend_from_slice(&port.to_be_bytes()); // 服务器端口 (u16 Big Endian)
        write_varint(&mut handshake, 1); // Next state = Status

        let mut packet = Vec::new();
        write_varint(&mut packet, handshake.len() as i32);
        packet.extend_from_slice(&handshake);
        stream
            .write_all(&packet)
            .await
            .map_err(|e| format!("发送 Handshake 失败: {}", e))?;

        // 发送 Status Request 包 (Packet ID 0x00，空 payload)
        let mut status_req = Vec::new();
        write_varint(&mut status_req, 1); // Packet length = 1 (仅 Packet ID)
        write_varint(&mut status_req, 0x00); // Packet ID
        stream
            .write_all(&status_req)
            .await
            .map_err(|e| format!("发送 Status Request 失败: {}", e))?;

        // 读取 Status Response：包长度 + 包 ID + JSON 长度 + JSON
        let _packet_length = read_varint(&mut stream).await?;
        let _packet_id = read_varint(&mut stream).await?;
        let json_length = read_varint(&mut stream).await? as usize;
        let mut json_buf = vec![0u8; json_length];
        stream
            .read_exact(&mut json_buf)
            .await
            .map_err(|e| format!("读取 Status JSON 失败: {}", e))?;
        let json_str =
            String::from_utf8(json_buf).map_err(|e| format!("JSON UTF-8 解码失败: {}", e))?;
        let status: serde_json::Value =
            serde_json::from_str(&json_str).map_err(|e| format!("JSON 解析失败: {}", e))?;

        // 发送 Ping 包 (Packet ID 0x01, 8 字节时间戳)
        let t1 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let mut ping_payload = Vec::new();
        write_varint(&mut ping_payload, 0x01); // Packet ID
        ping_payload.extend_from_slice(&t1.to_be_bytes());
        let mut ping_packet = Vec::new();
        write_varint(&mut ping_packet, ping_payload.len() as i32);
        ping_packet.extend_from_slice(&ping_payload);
        stream
            .write_all(&ping_packet)
            .await
            .map_err(|e| format!("发送 Ping 失败: {}", e))?;

        // 读取 Pong 包 (0x01, 8 字节时间戳)
        let _pong_len = read_varint(&mut stream).await?;
        let _pong_id = read_varint(&mut stream).await?;
        let mut ts_buf = [0u8; 8];
        stream
            .read_exact(&mut ts_buf)
            .await
            .map_err(|e| format!("读取 Pong 失败: {}", e))?;
        let ts = u64::from_be_bytes(ts_buf);
        let t2 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let latency = t2.saturating_sub(ts);

        // 提取字段
        let motd = status
            .get("description")
            .map(extract_motd)
            .unwrap_or_default();
        let players = status.get("players").cloned().unwrap_or_default();
        let online = players
            .get("online")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let max = players
            .get("max")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let version = status
            .get("version")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let favicon = status
            .get("favicon")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(ServerPingResult {
            motd,
            online,
            max,
            version,
            latency_ms: latency,
            favicon,
            error: String::new(),
        })
    };

    tokio::time::timeout(Duration::from_secs(5), slp_fut)
        .await
        .map_err(|_| format!("SLP 交换超时 (5s): {}:{}", host, port))?
}

// ===== VarInt 编解码 + 字符串写入 =====

/// 写 VarInt 到 buf
///
/// 内部转 u32 进行位移以正确处理负数（如协议版本 -1）。
/// 负数会被编码为 5 字节（与 Java VarInt 一致：-1 → FF FF FF FF 0F）。
fn write_varint(buf: &mut Vec<u8>, value: i32) {
    let mut value = value as u32;
    while value & !0x7Fu32 != 0 {
        buf.push(((value & 0x7F) as u8) | 0x80);
        value >>= 7;
    }
    buf.push((value & 0x7F) as u8);
}

/// 写 Minecraft 协议字符串（VarInt 长度前缀 + UTF-8 字节）
fn write_string(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_varint(buf, bytes.len() as i32);
    buf.extend_from_slice(bytes);
}

/// 从 AsyncRead 读 VarInt
async fn read_varint<R: AsyncRead + Unpin>(reader: &mut R) -> Result<i32, String> {
    let mut result = 0i32;
    let mut shift = 0u32;
    loop {
        let mut byte = [0u8; 1];
        reader
            .read_exact(&mut byte)
            .await
            .map_err(|e| e.to_string())?;
        result |= ((byte[0] & 0x7F) as i32) << shift;
        if byte[0] & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 32 {
            return Err("VarInt 太大".to_string());
        }
    }
    Ok(result)
}

// ===== MOTD 提取 =====

/// 从 description 字段提取纯文本 MOTD
///
/// description 可能是：
/// - 字符串：直接返回
/// - 对象 `{ "text": "...", "extra": [...] }`：拼接 text 和所有 extra[].text
///
/// 最后剥离 §格式化代码（§后跟一个字符）。
fn extract_motd(description: &serde_json::Value) -> String {
    let mut text = String::new();
    match description {
        serde_json::Value::String(s) => text.push_str(s),
        serde_json::Value::Object(obj) => {
            if let Some(serde_json::Value::String(t)) = obj.get("text") {
                text.push_str(t);
            }
            if let Some(serde_json::Value::Array(arr)) = obj.get("extra") {
                for item in arr {
                    collect_extra_text(item, &mut text);
                }
            }
        }
        _ => {}
    }
    strip_section_codes(&text)
}

/// 递归收集 extra 数组中每个元素的 text 字段
fn collect_extra_text(value: &serde_json::Value, out: &mut String) {
    match value {
        serde_json::Value::String(s) => out.push_str(s),
        serde_json::Value::Object(o) => {
            if let Some(serde_json::Value::String(t)) = o.get("text") {
                out.push_str(t);
            }
            if let Some(serde_json::Value::Array(arr)) = o.get("extra") {
                for item in arr {
                    collect_extra_text(item, out);
                }
            }
        }
        _ => {}
    }
}

/// 剥离 Minecraft 格式化代码（§后跟一个字符）
fn strip_section_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '§' {
            // 跳过 § 和紧跟的一个字符（格式化代码 0-9, a-f, k-o, r）
            chars.next();
        } else {
            result.push(c);
        }
    }
    result
}
