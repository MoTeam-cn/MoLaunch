//! 证书管理：certs 目录操作 + 自定义/系统根证书加载

use crate::log_info;
use crate::storage::appdata;
use serde::Serialize;
use std::path::PathBuf;

use super::pem;

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
///
/// 白名单变体：只放行 ASCII 字母数字 `_-.`，比 `utils::path::sanitize_file_name`
/// 的「黑名单拒绝 `/` `\\` `..` `\0`」更严格（连空格、`&`、`()` 等也拒绝）。
/// 二者目标一致（防路径遍历），此处按证书域需求采用更严的白名单。
pub(super) fn validate_filename(filename: &str) -> Result<(), String> {
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

/// 证书目录：`%APPDATA%/.Molaunch/certs/`（全局共享，不存在则创建）
///
/// 跨启动器实例共享：一台设备只信任一次自定义根证书，所有 MoLaunch 实例复用同一份。
/// 旧路径 `<exe_dir>/.Molaunch/certs/` 由 `Storage::init` 启动时自动迁移。
pub fn cert_dir() -> PathBuf {
    let dir = appdata::ensure_appdata_subdir("certs").unwrap_or_else(|e| {
        crate::log_error!("Failed to create certs directory in AppData: {}", e);
        // 降级回便携式目录（极少发生：APPDATA 环境变量缺失）
        crate::storage::Storage::instance().base_dir().join("certs")
    });
    // 收紧信任锚目录权限，防止同机其他进程写入
    crate::minecraft::system::shell::restrict_dir_permissions(&dir);
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

        let (subject, not_after) = pem::parse_pem_meta(&pem_bytes, &filename);
        result.push(CustomCertInfo {
            filename,
            subject,
            not_after,
        });
    }

    result
}

/// 校验 PEM 为有效期内 CA 根证书（BasicConstraints CA:TRUE），失败含 Subject CN
fn validate_ca_cert(pem_bytes: &[u8]) -> Result<(), String> {
    let (_, pem) =
        x509_parser::pem::parse_x509_pem(pem_bytes).map_err(|e| format!("PEM 格式无效: {}", e))?;
    let cert = pem.parse_x509().map_err(|e| format!("X.509 解析失败: {}", e))?;

    let cn = cert
        .subject()
        .iter_common_name()
        .next()
        .and_then(|c| c.as_str().ok())
        .unwrap_or("<unknown>");

    let is_ca = cert
        .basic_constraints()
        .map_err(|e| format!("解析 BasicConstraints 扩展失败: {}", e))?
        .map(|bc| bc.value.ca)
        .unwrap_or(false);
    if !is_ca {
        return Err(format!(
            "证书不是 CA 根证书（BasicConstraints CA 必须为 TRUE），Subject CN: {}",
            cn
        ));
    }

    let now = x509_parser::time::ASN1Time::now();
    let validity = cert.validity();
    if validity.not_before > now {
        return Err(format!(
            "证书尚未生效（Subject CN: {}，not_before: {}）",
            cn, validity.not_before
        ));
    }
    if validity.not_after < now {
        return Err(format!(
            "证书已过期（Subject CN: {}，not_after: {}）",
            cn, validity.not_after
        ));
    }

    Ok(())
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

    // 读取并校验 PEM 格式与 CA 属性（有效期、BasicConstraints）
    let pem_bytes = std::fs::read(src).map_err(|e| format!("读取源文件失败: {}", e))?;
    validate_ca_cert(&pem_bytes)?;
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
