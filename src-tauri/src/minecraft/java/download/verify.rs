//! Java 可执行文件验证

use std::path::Path;

use crate::log_info;
use crate::log_warn;

/// 阶段 5：验证下载的 Java 能跑起来
///
/// 返回 `Err` 而不是只打日志：`java.exe` 存在不等于这份 runtime 可用。
/// 缺模块、被杀毒删掉一半、下载中途失败都会留下一个「能执行但跑不起来」的文件，
/// 而调用方拿到路径就会把它写进配置，用户于是在启动游戏时吃一个看不懂的崩溃。
pub fn verify_downloaded_java(java_exe: &Path) -> Result<(), String> {
    match crate::minecraft::java::detect_java(java_exe) {
        Ok(runtime) => {
            log_info!(
                "[JavaDownload] Verified: Java {} ({})",
                runtime.version,
                java_exe.display()
            );
            Ok(())
        }
        Err(e) => {
            log_warn!(
                "[JavaDownload] Java verification failed: {} ({})",
                java_exe.display(),
                e
            );
            Err(format!(
                "下载的 Java 无法运行: {}（{}）",
                java_exe.display(),
                e
            ))
        }
    }
}

#[cfg(test)]
#[path = "verify_test.rs"]
mod verify_test;
