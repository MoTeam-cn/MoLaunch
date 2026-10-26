//! 事件签名与文本脱敏
//!
//! 签名用于后台聚合去重：把可变部分（数字、十六进制、UUID、路径、时间）归一化后
//! 取哈希，同一类错误在不同用户机器上得到同一个签名。

use once_cell::sync::Lazy;
use regex::Regex;

static DIGITS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\d+").expect("digits regex"));
static HEX_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"0x[0-9a-fA-F]+").expect("hex regex"));
static UUID_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
        .expect("uuid regex")
});
static WIN_USER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)([A-Za-z]:\\Users\\)[^\\\s"]+"#).expect("win user regex"));
static NIX_USER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(/home/)[^/\s]+").expect("nix user regex"));

/// 归一化信息：抹掉数字 / 十六进制 / UUID / 用户名，只留稳定的错误骨架
pub fn normalize_message(message: &str) -> String {
    let step1 = UUID_RE.replace_all(message, "<uuid>");
    let step2 = HEX_RE.replace_all(&step1, "<hex>");
    let step3 = DIGITS_RE.replace_all(&step2, "<n>");
    step3.trim().to_string()
}

/// 计算事件签名（取 sha256 前 16 位十六进制）
pub fn signature(module: &str, message: &str) -> String {
    let normalized = normalize_message(message);
    let raw = format!("{}|{}", module, normalized);
    let full = crate::utils::hash::sha256_hex(raw.as_bytes());
    full.chars().take(16).collect()
}

/// 上报前的文本脱敏：敏感字段（token/密码）+ 路径中的用户名
pub fn sanitize_text(text: &str) -> String {
    let sensitive = crate::logger::sanitize_sensitive_info(text);
    let win = WIN_USER_RE.replace_all(&sensitive, "$1<user>");
    NIX_USER_RE.replace_all(&win, "$1<user>").to_string()
}
