//! SDK AES-256-CBC 加密封装 + 旧格式（v2 文件级 / SDK DES）兼容回退。
//! 新写入统一走 SDK `mc_encrypt_token`（AES-256-CBC）；解密 SDK 优先
//! （内部自动 AES→DES 回退），失败回退 v2 文件级解密（更早期版本数据）。

use crate::sdk::SdkInstance;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

/// v2 文件级加密格式前缀：`v2:` + base64(nonce(12B) ++ ct++tag)
const V2_PREFIX: &str = "v2:";
/// 主密钥文件名（AppData 根目录）
const KEY_FILE: &str = "master.key";
/// AES-256 主密钥长度（32 字节）
const KEY_LEN: usize = 32;
/// GCM nonce 长度（12 字节）
const NONCE_LEN: usize = 12;

/// 进程内缓存的 32 字节主密钥（首次从文件加载）
static MASTER_KEY: OnceCell<[u8; KEY_LEN]> = OnceCell::new();

/// master.key 文件路径（AppData 根目录）
fn master_key_path() -> Result<PathBuf, String> {
    Ok(crate::storage::appdata::appdata_root()?.join(KEY_FILE))
}

/// DPAPI 加密（Windows 用户级保护，仅当前用户可解密）
#[cfg(all(test, windows))]
fn dpapi_protect(plain: &[u8]) -> Result<Vec<u8>, String> {
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::Win32::Security::Cryptography::{CryptProtectData, CRYPT_INTEGER_BLOB};

    let in_blob = CRYPT_INTEGER_BLOB {
        cbData: plain.len() as u32,
        pbData: plain.as_ptr() as *mut u8,
    };
    let mut out_blob = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptProtectData(
            &in_blob,
            windows::core::PCWSTR::null(),
            None,
            None,
            None,
            0,
            &mut out_blob,
        )
    }
    .map_err(|e| format!("DPAPI 加密失败: {}", e))?;
    let out =
        unsafe { std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec() };
    unsafe {
        let _ = LocalFree(HLOCAL(out_blob.pbData.cast()));
    }
    Ok(out)
}

/// DPAPI 解密
#[cfg(windows)]
fn dpapi_unprotect(cipher: &[u8]) -> Result<Vec<u8>, String> {
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};

    let in_blob = CRYPT_INTEGER_BLOB {
        cbData: cipher.len() as u32,
        pbData: cipher.as_ptr() as *mut u8,
    };
    let mut out_blob = CRYPT_INTEGER_BLOB::default();
    unsafe { CryptUnprotectData(&in_blob, None, None, None, None, 0, &mut out_blob) }
        .map_err(|e| format!("DPAPI 解密失败: {}", e))?;
    let out =
        unsafe { std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec() };
    unsafe {
        let _ = LocalFree(HLOCAL(out_blob.pbData.cast()));
    }
    Ok(out)
}

/// 主密钥编码：Windows 走 DPAPI；其余平台 raw。
///
/// 生产已停用自实现加密（不再创建 master.key），仅测试生成临时密钥时使用。
#[cfg(test)]
fn encode_master_key(key: &[u8; KEY_LEN]) -> Result<String, String> {
    #[cfg(windows)]
    {
        let protected = dpapi_protect(key)?;
        Ok(format!("dpapi:{}", STANDARD.encode(protected)))
    }
    #[cfg(not(windows))]
    {
        Ok(format!("raw:{}", STANDARD.encode(key)))
    }
}

fn decode_master_key(content: &str) -> Result<[u8; KEY_LEN], String> {
    let content = content.trim();
    let bytes: Vec<u8> = if let Some(rest) = content.strip_prefix("dpapi:") {
        #[cfg(windows)]
        {
            let blob = STANDARD
                .decode(rest)
                .map_err(|e| format!("master.key base64 解码失败: {}", e))?;
            dpapi_unprotect(&blob)?
        }
        #[cfg(not(windows))]
        {
            return Err("master.key 为 DPAPI 格式，非 Windows 平台无法解密".to_string());
        }
    } else if let Some(rest) = content.strip_prefix("raw:") {
        STANDARD
            .decode(rest)
            .map_err(|e| format!("master.key base64 解码失败: {}", e))?
    } else {
        return Err("master.key 缺少 dpapi:/raw: 前缀".to_string());
    };
    if bytes.len() != KEY_LEN {
        return Err(format!(
            "master.key 密钥长度异常: {}（期望 {}）",
            bytes.len(),
            KEY_LEN
        ));
    }
    let mut key = [0u8; KEY_LEN];
    key.copy_from_slice(&bytes);
    Ok(key)
}

/// 加载已有主密钥（v2 旧数据回退解密用；不再自动创建新密钥）
fn load_master_key() -> Result<[u8; KEY_LEN], String> {
    let path = master_key_path()?;
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("读取 master.key 失败: {}", e))?;
    decode_master_key(&content)
}

fn master_key() -> Result<&'static [u8; KEY_LEN], String> {
    MASTER_KEY.get_or_try_init(load_master_key)
}

/// 文件级解密：仅处理 `v2:` 前缀，其他输入返回 Err（供 SDK 解密失败时回退旧数据）
pub fn decrypt_file_securely(data: &str) -> Result<String, String> {
    let payload_b64 = data
        .strip_prefix(V2_PREFIX)
        .ok_or_else(|| "数据缺少 v2: 前缀，可能为明文或 SDK 加密格式".to_string())?;
    let payload = STANDARD
        .decode(payload_b64)
        .map_err(|e| format!("v2 密文 base64 解码失败: {}", e))?;
    if payload.len() < NONCE_LEN {
        return Err("v2 密文长度异常".to_string());
    }
    let (nonce_bytes, ciphertext) = payload.split_at(NONCE_LEN);
    let key = master_key()?;
    let cipher = Aes256Gcm::new(key.into());
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|_| "AES-256-GCM 解密失败".to_string())?;
    String::from_utf8(plaintext).map_err(|e| format!("解密结果非 UTF-8: {}", e))
}

/// 加密字符串（SDK AES-256-CBC，协议见 docs/token-encryption.md）
pub async fn encrypt_with_sdk(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    data: &str,
    ctx: &str,
) -> Result<String, String> {
    let sdk = sdk_arc.lock().await;
    match sdk.as_ref() {
        Some(sdk) => sdk
            .encrypt_token(data)
            .map_err(|e| format!("{}加密失败: {}", ctx, e)),
        None => Err(format!("SDK 未加载，无法加密{}", ctx)),
    }
}

/// 解密字符串（SDK 解密，内部自动 AES→DES 回退），失败返回 Err
pub async fn decrypt_with_sdk(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    data: &str,
    ctx: &str,
) -> Result<String, String> {
    let sdk = sdk_arc.lock().await;
    match sdk.as_ref() {
        Some(sdk) => sdk
            .decrypt_token(data)
            .map_err(|e| format!("{}解密失败: {}", ctx, e)),
        None => Err(format!("SDK 未加载，无法解密{}", ctx)),
    }
}

/// 加密字符串：统一走 SDK AES-256-CBC（协议见 docs/token-encryption.md）
pub async fn encrypt_with_secure_sdk(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    data: &str,
    ctx: &str,
) -> Result<String, String> {
    encrypt_with_sdk(sdk_arc, data, ctx).await
}

/// 解密字符串：SDK 优先（内部自动 AES→DES 回退），失败回退 v2 文件级（更早期旧数据）
pub async fn decrypt_with_secure_sdk(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    data: &str,
    ctx: &str,
) -> Result<String, String> {
    match decrypt_with_sdk(sdk_arc, data, ctx).await {
        Ok(s) => Ok(s),
        Err(e) => {
            crate::log_warn!("[SDK Crypto] {}SDK 解密失败，回退 v2 文件级: {}", ctx, e);
            decrypt_file_securely(data)
        }
    }
}

/// 解密字符串，失败记 warn 并返回 None（供 frp/community 使用）
pub async fn decrypt_with_sdk_optional(
    sdk_arc: &Arc<TokioMutex<Option<SdkInstance>>>,
    data: &str,
    ctx: &str,
) -> Option<String> {
    match decrypt_with_secure_sdk(sdk_arc, data, ctx).await {
        Ok(s) => Some(s),
        Err(e) => {
            crate::log_warn!("[SDK Crypto] {}: {}", ctx, e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};
    use rand::rngs::OsRng;
    use rand::RngCore;

    /// 测试专用 v2 加密辅助（生产已停用自实现加密，仅用于生成回退解密样本）
    fn encrypt_v2_for_test(data: &str) -> Result<String, String> {
        let key = master_key()?;
        let cipher = Aes256Gcm::new(key.into());
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, data.as_bytes())
            .map_err(|_| "AES-256-GCM 加密失败".to_string())?;
        let mut payload = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&ciphertext);
        Ok(format!("{}{}", V2_PREFIX, STANDARD.encode(payload)))
    }

    /// 测试专用主密钥初始化（生成并写入临时 master.key）
    fn setup_test_master_key() -> Result<(), String> {
        let path = master_key_path()?;
        let mut key = [0u8; KEY_LEN];
        OsRng.fill_bytes(&mut key);
        let encoded = encode_master_key(&key)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建主密钥目录失败: {}", e))?;
        }
        std::fs::write(&path, encoded).map_err(|e| format!("写入 master.key 失败: {}", e))?;
        Ok(())
    }

    #[test]
    fn v2_decrypt_roundtrip() {
        let old_appdata = std::env::var("APPDATA").ok();
        let dir = std::env::temp_dir().join(format!("molaunch_crypto_{}", std::process::id()));
        std::env::set_var("APPDATA", &dir);

        setup_test_master_key().expect("主密钥初始化应成功");
        let plain = "联机设备凭证/API Key 测试负载";
        let enc = encrypt_v2_for_test(plain).expect("encrypt 应成功");
        assert!(enc.starts_with("v2:"), "输出必须带 v2: 前缀");
        let dec = decrypt_file_securely(&enc).expect("decrypt 应成功");
        assert_eq!(dec, plain);
        assert!(
            decrypt_file_securely("legacy-des-data").is_err(),
            "非 v2: 输入必须返回 Err"
        );

        let _ = std::fs::remove_dir_all(&dir);
        match old_appdata {
            Some(v) => std::env::set_var("APPDATA", v),
            None => std::env::remove_var("APPDATA"),
        }
    }
}
