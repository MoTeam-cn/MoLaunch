//! 文件权限限制
//!
//! `restrict_file_permissions` 尽力限制文件权限为当前用户，防止敏感信息被其他用户读取。

use crate::{log_error, log_info};

/// 尽力限制文件权限为当前用户（防止敏感信息被其他用户读取）
///
/// 仅尽力执行，失败只记日志不返回错误（调用方不关心失败）：
/// - Windows: `icacls <path> /inheritance:r /grant:r "<user>:F"`
///   移除继承权限并仅保留当前用户完全控制
/// - Unix: `chmod 600`（仅当前用户可读写）
pub fn restrict_file_permissions(path: &std::path::Path) {
    log_info!("[Shell] restrict_file_permissions: {}", path.display());

    #[cfg(target_os = "windows")]
    {
        let username = std::env::var("USERNAME").unwrap_or_default();
        if username.is_empty() {
            log_error!("[Shell] icacls skipped: USERNAME env empty");
            return;
        }
        let grant = format!("{}:F", username);
        match std::process::Command::new("icacls")
            .arg(path)
            .arg("/inheritance:r")
            .arg("/grant:r")
            .arg(&grant)
            .output()
        {
            Ok(out) if !out.status.success() => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                log_error!("[Shell] icacls failed: {}", stderr.trim());
            }
            Err(e) => log_error!("[Shell] icacls failed: {}", e),
            _ => {}
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)) {
            log_error!("[Shell] chmod 600 failed: {}", e);
        }
    }

    #[cfg(not(any(target_os = "windows", unix)))]
    {
        let _ = path;
    }
}

/// 尽力限制目录权限为当前用户（防止其他进程写入信任锚等敏感目录）
///
/// - Windows: icacls 对目录同样生效，复用 `restrict_file_permissions`
/// - Unix: `chmod 700`（目录需执行位才能列出，不能用文件的 600）
pub fn restrict_dir_permissions(path: &std::path::Path) {
    log_info!("[Shell] restrict_dir_permissions: {}", path.display());

    #[cfg(target_os = "windows")]
    {
        restrict_file_permissions(path);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)) {
            log_error!("[Shell] chmod 700 failed: {}", e);
        }
    }

    #[cfg(not(any(target_os = "windows", unix)))]
    {
        let _ = path;
    }
}
