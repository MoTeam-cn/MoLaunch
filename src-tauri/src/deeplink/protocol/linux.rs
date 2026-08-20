//! Linux deeplink 协议实现（desktop 文件 + xdg-mime，免 root）

use super::PROTOCOL;

/// desktop 文件名（形如 `molaunch-handler.desktop`）
fn desktop_file_name() -> String {
    let bin = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "molaunch".to_string());
    format!("{}-handler.desktop", bin)
}

/// desktop 文件目录（user 级，免 root）
fn desktop_file_dir() -> Option<std::path::PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        return Some(std::path::Path::new(&home).join(".local/share/applications"));
    }
    None
}

fn desktop_file_path() -> Option<std::path::PathBuf> {
    Some(desktop_file_dir()?.join(desktop_file_name()))
}

/// Linux 注册 molaunch:// 协议（写 desktop 文件 + xdg-mime）
pub(super) fn register_linux() -> Result<(), String> {
    use std::io::Write;

    let exe = super::current_exe_path().ok_or("无法获取当前 exe 路径")?;
    let dir = desktop_file_dir().ok_or("无法确定 desktop 目录（缺少 HOME）")?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建 desktop 目录失败: {}", e))?;
    let file = desktop_file_path().unwrap();

    let content = format!(
        "[Desktop Entry]\nType=Application\nName={}\nComment={} protocol handler\nExec=\"{}\" %u\nTerminal=false\nCategories=Utility;\nMimeType=x-scheme-handler/{};\nNoDisplay=true\n",
        PROTOCOL,
        PROTOCOL,
        exe,
        PROTOCOL
    );
    let mut f =
        std::fs::File::create(&file).map_err(|e| format!("写入 desktop 文件失败: {}", e))?;
    f.write_all(content.as_bytes())
        .map_err(|e| format!("写入 desktop 文件失败: {}", e))?;

    // 注册为默认 handler
    let _ = crate::minecraft::system::shell::run_command_status(
        "xdg-mime",
        &[
            "default".to_string(),
            desktop_file_name(),
            format!("x-scheme-handler/{}", PROTOCOL),
        ],
    );
    let _ = crate::minecraft::system::shell::run_command_status(
        "update-desktop-database",
        &[dir.to_string_lossy().into_owned()],
    );

    Ok(())
}

/// Linux 卸载 molaunch:// 协议（删除 desktop 文件 + 清理 mime）
pub(super) fn unregister_linux() -> Result<(), String> {
    if let Some(file) = desktop_file_path() {
        if file.exists() {
            std::fs::remove_file(&file).map_err(|e| format!("删除 desktop 文件失败: {}", e))?;
        }
    }
    let _ = crate::minecraft::system::shell::run_command_status(
        "xdg-mime",
        &[
            "uninstall".to_string(),
            "mimeinfo".to_string(),
            "/dev/null".to_string(), // 无实际卸载语义，占位避免 xdg 报错；真正清理靠删除 desktop 文件
        ],
    );
    Ok(())
}

/// Linux 读取 desktop 文件中登记的 exe 路径
pub(super) fn registered_exe_linux() -> Option<String> {
    let content = std::fs::read_to_string(desktop_file_path()?).ok()?;
    content.lines().find_map(|line| {
        let line = line.trim();
        if let Some(exec) = line.strip_prefix("Exec=") {
            let trimmed = exec.trim().trim_matches('"');
            // 提取 `"exe" %u` 中的 exe 路径
            trimmed
                .split_whitespace()
                .next()
                .map(|s| s.trim_matches('"').to_string())
        } else {
            None
        }
    })
}
