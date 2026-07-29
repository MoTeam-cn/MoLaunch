use base64::engine::{general_purpose, Engine as _};
use ed25519_dalek::{Signature, VerifyingKey};
use sha2::{Digest, Sha512};
use std::fs;
use std::path::Path;

// 从 tauri.conf.json plugins.updater.pubkey 提取的公钥第二行（base64）
// 解码后前 2 字节是算法标识（Ed25519），后 32 字节是真正的公钥
const PUBKEY_B64: &str = "RWQXIJ9FRypYEviIOjdYFmBE/87ea7Uf8/EaE7AjaxXNhaU7XSYU/696F";

pub fn verify_minisign(new_exe: &Path, signature: &str) -> Result<(), String> {
    let pubkey_full = general_purpose::STANDARD
        .decode(PUBKEY_B64)
        .map_err(|e| format!("公钥 base64 解码失败: {e}"))?;
    if pubkey_full.len() != 34 {
        return Err(format!("公钥长度错误: {}", pubkey_full.len()));
    }
    let pubkey_bytes: [u8; 32] = pubkey_full[2..]
        .try_into()
        .map_err(|_| "公钥切片失败".to_string())?;

    let sig_b64 = extract_signature_line(signature)?;
    let sig_bytes = general_purpose::STANDARD
        .decode(&sig_b64)
        .map_err(|e| format!("签名 base64 解码失败: {e}"))?;
    if sig_bytes.len() != 66 {
        return Err(format!("签名长度错误: {}", sig_bytes.len()));
    }
    let ed25519_sig: [u8; 64] = sig_bytes[2..]
        .try_into()
        .map_err(|_| "签名切片失败".to_string())?;

    let exe_bytes = fs::read(new_exe).map_err(|e| format!("读取新 exe 失败: {e}"))?;
    let mut hasher = Sha512::new();
    hasher.update(&exe_bytes);

    let pubkey = VerifyingKey::from_bytes(&pubkey_bytes)
        .map_err(|e| format!("公钥无效: {e}"))?;
    let sig = Signature::from_bytes(&ed25519_sig);
    pubkey
        .verify_prehashed(hasher, None, &sig)
        .map_err(|e| format!("签名验证失败: {e}"))?;

    Ok(())
}

fn extract_signature_line(signature: &str) -> Result<String, String> {
    for line in signature.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("untrusted comment:")
            || trimmed.starts_with("trusted comment:")
        {
            continue;
        }
        return Ok(trimmed.to_string());
    }
    Err("signature 中未找到签名行".into())
}
