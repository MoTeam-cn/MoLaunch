//! 更新包签名校验（minisign 格式，与 tauri-plugin-updater 完全一致）
//!
//! 使用与 Tauri 官方 updater 插件同款的 `minisign-verify` crate：
//! - 公钥：与 `src-tauri/tauri.conf.json` 的 `plugins.updater.pubkey` 保持同一份
//!   （`dW` 开头完整 base64，解码后为两行 minisign.pub 文本）。更换签名密钥时，
//!   同步更新 tauri.conf.json 与本文件即可。
//! - 签名：标准 minisign `.sig` 文件内容（4 行：untrusted comment / 签名行 /
//!   trusted comment / 全局签名行），由 CI 的 `tauri signer` / `tauri-action` 生成。
//! - 校验：key_id 匹配 -> prehashed 用 BLAKE2b-512 摘要 -> Ed25519 验证签名
//!   与全局签名（签名 + trusted comment）。

use base64::engine::{general_purpose, Engine as _};
use minisign_verify::{PublicKey, Signature};
use std::fs;
use std::path::Path;

/// 与 tauri.conf.json `plugins.updater.pubkey` 完全一致（`dW` 开头完整 base64）
const PUBKEY_B64: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDEyNTgyQTQ3NDU5RjIwMTcKUldRWElKOUZSeXBZRXZJT2pkWUZtQkUvODdlYTdVZjgvRWFFN0FqYXhYTmhhVTdYU1lVLzY5NkUK";

pub fn verify_minisign(new_exe: &Path, signature: &str) -> Result<(), String> {
    // 1. 解码公钥：dW 开头 base64 -> minisign.pub 两行文本
    let pubkey_b64 = general_purpose::STANDARD
        .decode(PUBKEY_B64)
        .map_err(|e| format!("公钥 base64 解码失败: {e}"))?;
    let pubkey_text = std::str::from_utf8(&pubkey_b64)
        .map_err(|e| format!("公钥文本非 UTF-8: {e}"))?;
    let public_key = PublicKey::decode(pubkey_text)
        .map_err(|e| format!("公钥解析失败: {e}"))?;

    // 2. 解析签名（标准 minisign .sig 内容，4 行格式）
    let parsed_sig = Signature::decode(signature)
        .map_err(|e| format!("签名解析失败（需标准 minisign .sig 内容）: {e}"))?;

    // 3. 读取文件并校验（allow_legacy=true，与 Tauri 插件一致）
    let exe_bytes = fs::read(new_exe).map_err(|e| format!("读取新 exe 失败: {e}"))?;
    public_key
        .verify(&exe_bytes, &parsed_sig, true)
        .map_err(|e| format!("签名验证失败: {e}"))?;

    Ok(())
}
