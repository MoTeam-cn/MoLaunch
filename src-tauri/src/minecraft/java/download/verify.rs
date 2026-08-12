//! Java 可执行文件验证

use std::path::Path;

use crate::log_info;

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
