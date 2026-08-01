//! TLS 证书管理模块
//!
//! 提供自定义/系统根证书加载与 certs 目录管理，信任源模式由 `AppConfig.tls.trust_mode` 控制。

use crate::log_info;
use crate::storage::Storage;
use serde::Serialize;
use std::path::PathBuf;

/// 自定义证书信息（list_custom_certs 返回项）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomCertInfo {
    /// 文件名（certs 目录下的相对名称，如 `my-root.pem`）
    pub filename: String,
    /// 证书 Subject CN（解析失败时回退为文件名）
    pub subject: String,
    /// 证书过期时间（PEM 解析失败时为空字符串）
    pub not_after: String,
}

/// 校验证书文件名：仅允许字母数字下划线连字符和点，防止路径遍历
fn validate_filename(filename: &str) -> Result<(), String> {
    if filename.is_empty() {
        return Err("证书文件名不能为空".to_string());
    }
    if !filename
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(format!(
            "证书文件名包含非法字符: {}（仅允许字母数字、下划线、连字符和点）",
            filename
        ));
    }
    Ok(())
}

/// 证书目录：`%APPDATA%/.Molaunch/certs/`（不存在则创建）
pub fn cert_dir() -> PathBuf {
    let dir = Storage::instance().base_dir().join("certs");
    if !dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&dir) {
            crate::log_error!("Failed to create certs directory: {}", e);
        } else {
            log_info!("Created certs directory: {}", dir.display());
        }
    }
    dir
}

/// 列出 certs 目录下所有 `.pem` 文件
///
/// 每项返回 `CustomCertInfo`，包含文件名、Subject CN 和过期时间。
/// PEM 解析失败时 `subject` 回退为文件名，`not_after` 为空字符串（不阻塞列表展示）。
pub fn list_custom_certs() -> Vec<CustomCertInfo> {
    let dir = cert_dir();
    let mut result = Vec::new();

    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return result,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("pem") {
            continue;
        }
        let filename = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let pem_bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(_) => continue,
        };

        let (subject, not_after) = parse_pem_meta(&pem_bytes, &filename);
        result.push(CustomCertInfo {
            filename,
            subject,
            not_after,
        });
    }

    result
}

/// 添加自定义证书：从源路径读取 PEM 文件，复制到 certs 目录
///
/// - 源文件名作为目标文件名（保留 `.pem` 后缀）
/// - 同名文件已存在时返回错误（避免覆盖既有信任锚）
pub fn add_custom_cert(src_path: &str) -> Result<(), String> {
    let src = std::path::Path::new(src_path);
    if !src.exists() {
        return Err(format!("源证书文件不存在: {}", src_path));
    }

    let filename = src
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "无法解析源文件名".to_string())?
        .to_string();

    validate_filename(&filename)?;

    if !filename.ends_with(".pem") {
        return Err("证书文件必须为 .pem 格式".to_string());
    }

    let dest = cert_dir().join(&filename);
    if dest.exists() {
        return Err(format!("证书文件已存在: {}", filename));
    }

    // 读取并校验 PEM 格式（reqwest::Certificate::from_pem 验证）
    let pem_bytes = std::fs::read(src).map_err(|e| format!("读取源文件失败: {}", e))?;
    reqwest::Certificate::from_pem(&pem_bytes).map_err(|e| format!("PEM 格式无效: {}", e))?;

    std::fs::write(&dest, &pem_bytes).map_err(|e| format!("写入证书文件失败: {}", e))?;

    log_info!("[Certs] Added custom cert: {}", filename);
    Ok(())
}

/// 删除自定义证书：按文件名删除 certs 目录下对应文件
pub fn remove_custom_cert(filename: &str) -> Result<(), String> {
    validate_filename(filename)?;
    let path = cert_dir().join(filename);
    if !path.exists() {
        return Err(format!("证书文件不存在: {}", filename));
    }
    std::fs::remove_file(&path).map_err(|e| format!("删除证书文件失败: {}", e))?;
    log_info!("[Certs] Removed custom cert: {}", filename);
    Ok(())
}

/// 加载自定义根证书：读取 certs 目录所有 PEM 文件，返回 `Vec<reqwest::Certificate>`
///
/// 单个证书解析失败时跳过并记录日志，不阻塞整体加载。
pub fn load_custom_root_certificates() -> Vec<reqwest::Certificate> {
    let dir = cert_dir();
    let mut certs = Vec::new();

    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return certs,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("pem") {
            continue;
        }
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
        match std::fs::read(&path) {
            Ok(pem_bytes) => match reqwest::Certificate::from_pem(&pem_bytes) {
                Ok(cert) => certs.push(cert),
                Err(e) => crate::log_warn!("[Certs] Failed to parse PEM file {}: {}", filename, e),
            },
            Err(e) => crate::log_warn!("[Certs] Failed to read PEM file {}: {}", filename, e),
        }
    }

    certs
}

/// 加载系统根证书：使用 `rustls_native_certs::load_native_certs()` 加载
///
/// 返回 `Vec<reqwest::Certificate>`（DER → reqwest::Certificate::from_der）。
/// 单个证书转换失败时跳过并记录日志。
pub fn load_system_root_certificates() -> Vec<reqwest::Certificate> {
    let mut certs = Vec::new();

    match rustls_native_certs::load_native_certs() {
        Ok(raw_certs) => {
            for cert in raw_certs {
                // rustls-native-certs 0.6 返回 rustls::Certificate（内含 Vec<u8> DER 字节）
                let der_bytes: &[u8] = cert.as_ref();
                match reqwest::Certificate::from_der(der_bytes) {
                    Ok(c) => certs.push(c),
                    Err(e) => {
                        crate::log_debug!("[Certs] Skipped system cert (DER→reqwest fail): {}", e)
                    }
                }
            }
            log_info!("[Certs] Loaded {} system root certificates", certs.len());
        }
        Err(e) => {
            crate::log_warn!("[Certs] Failed to load system root certs: {}", e);
        }
    }

    certs
}

// PEM 元信息解析（简易实现，避免引入 x509-parser 依赖）

/// 从 PEM 字节中解析 Subject CN 和 NOT AFTER 时间
///
/// 简易实现：base64 解码后查找 `CN=` 和 `NOT AFTER` 子串。
/// 解析失败时 `subject` 回退为 `fallback_filename`，`not_after` 为空字符串。
fn parse_pem_meta(pem_bytes: &[u8], fallback_filename: &str) -> (String, String) {
    // PEM → base64 解码（提取 BEGIN/END 之间的内容）
    let pem_str = match std::str::from_utf8(pem_bytes) {
        Ok(s) => s,
        Err(_) => return (fallback_filename.to_string(), String::new()),
    };

    let mut b64_lines = Vec::new();
    let mut in_body = false;
    for line in pem_str.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("-----BEGIN") {
            in_body = true;
            continue;
        }
        if trimmed.starts_with("-----END") {
            break;
        }
        if in_body && !trimmed.is_empty() {
            b64_lines.push(trimmed);
        }
    }

    if b64_lines.is_empty() {
        return (fallback_filename.to_string(), String::new());
    }

    let b64_content: String = b64_lines.concat();
    use base64::Engine;
    let der_bytes = match base64::engine::general_purpose::STANDARD.decode(&b64_content) {
        Ok(b) => b,
        Err(_) => return (fallback_filename.to_string(), String::new()),
    };

    // 在 DER 字节中查找 ASCII 子串（CN= / NOT AFTER）
    // 这种方式对常见 X.509 证书有效，但不保证所有证书都能匹配
    let der_str: Vec<u8> = der_bytes
        .iter()
        .map(|&b| {
            if b.is_ascii_graphic() || b == b' ' {
                b
            } else {
                b' '
            }
        })
        .collect();
    let der_text = String::from_utf8_lossy(&der_str);

    let subject = extract_cn(&der_text, fallback_filename);
    let not_after = extract_not_after(&der_text);

    (subject, not_after)
}

/// 从 DER 文本中提取 CN= 后的内容
fn extract_cn(text: &str, fallback: &str) -> String {
    if let Some(pos) = text.find("CN=") {
        let start = pos + 3;
        let end = text[start..]
            .find([',', '/', '\n'])
            .map(|i| start + i)
            .unwrap_or(text.len());
        let cn = text[start..end].trim();
        if !cn.is_empty() {
            return cn.to_string();
        }
    }
    fallback.to_string()
}

/// 从 DER 文本中提取 NOT AFTER 后的时间字符串
fn extract_not_after(text: &str) -> String {
    // X.509 v3 证书中常见 "Not After" 标签（UTCTime 或 GeneralizedTime）
    let markers = ["Not After : ", "Not After ", "NOT AFTER:"];
    for marker in markers {
        if let Some(pos) = text.find(marker) {
            let start = pos + marker.len();
            let end = text[start..]
                .find(['\n', ','])
                .map(|i| start + i)
                .unwrap_or((start + 24).min(text.len()));
            let value = text[start..end].trim();
            if !value.is_empty() {
                return value.to_string();
            }
        }
    }
    String::new()
}

#[cfg(test)]
#[path = "certs_tests.rs"]
mod tests;
