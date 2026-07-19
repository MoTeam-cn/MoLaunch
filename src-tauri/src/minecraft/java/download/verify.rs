//! SHA1 校验与 Java 验证

use std::path::Path;

use crate::{log_info, log_warn};
use crate::minecraft::utils::file_checker::compute_sha1_hex;

/// 校验字节的 SHA1，`expected_sha1` 为空则跳过（返回 Ok）
///
/// 用于下载前校验镜像源返回内容、以及断点续传时校验已存在文件，
/// 防止攻击者预先放置任意内容绕过尺寸检查。
pub fn verify_bytes_sha1(
    bytes: &[u8],
    expected_sha1: &str,
    path_str: &str,
) -> Result<(), String> {
    if expected_sha1.is_empty() {
        return Ok(());
    }
    let computed = compute_sha1_hex(bytes);
    if computed.to_lowercase() != expected_sha1.to_lowercase() {
        log_warn!(
            "[JavaDownload] SHA1 mismatch for {}: expected {}, got {}",
            path_str,
            expected_sha1,
            computed
        );
        return Err(format!("SHA1 verification failed for {}", path_str));
    }
    log_info!("[JavaDownload] SHA1 verified for {}", path_str);
    Ok(())
}

/// 阶段 5：验证下载的 Java（调用 detect_java，不阻断流程）
///
/// 验证失败仅记录日志，不返回 Err，仍然让调用方拿到 java.exe 路径。
pub fn verify_downloaded_java(java_exe: &Path) {
    match crate::minecraft::java::detect_java(java_exe) {
        Ok(runtime) => {
            log_info!(
                "[JavaDownload] Verified: Java {} ({})",
                runtime.version,
                java_exe.display()
            );
        }
        Err(e) => {
            log_info!("[JavaDownload] Java verification failed: {}", e);
        }
    }
}
