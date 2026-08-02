//! 深度链接协议注册 / 卸载 / 状态查询工具（跨平台，供便携版运行时注册协议）

use serde::Serialize;

/// 协议名（不含 `://`）
///
/// 仅 Windows（注册表）/ Linux（desktop 文件）运行时注册使用；
/// macOS 协议由打包 Info.plist 的 CFBundleURLTypes 声明，无需运行时注册。
#[cfg(any(windows, target_os = "linux"))]
const PROTOCOL: &str = "molaunch";

/// deeplink 注册状态（返回给前端）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeeplinkStatus {
    /// 协议当前是否已注册
    pub registered: bool,
    /// 注册表中登记的 exe 路径（未注册为 None）
    pub registered_exe: Option<String>,
    /// 当前运行 exe 路径（None 表示获取失败）
    pub current_exe: Option<String>,
    /// 当前平台是否支持运行时注册/卸载（macOS 不支持）
    pub platform_supported: bool,
    /// 人类可读说明
    pub message: String,
}

impl DeeplinkStatus {
    fn unsupported() -> Self {
        Self {
            registered: false,
            registered_exe: None,
            current_exe: current_exe_path(),
            platform_supported: false,
            message: "当前平台不支持运行时注册 deeplink（macOS 协议由打包 Info.plist 声明）"
                .to_string(),
        }
    }
}

/// 获取当前运行 exe 的完整路径
fn current_exe_path() -> Option<String> {
    std::env::current_exe()
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

/// 当前平台是否支持运行时注册/卸载协议
///
/// Windows/Linux 支持（HKCU 注册表 / desktop 文件）；
/// macOS 不支持（协议由打包时 Info.plist 的 CFBundleURLTypes 声明）。
pub fn platform_supported() -> bool {
    cfg!(windows) || cfg!(target_os = "linux")
}

/// 查询 molaunch:// 协议当前注册状态
pub fn status() -> DeeplinkStatus {
    if !platform_supported() {
        return DeeplinkStatus::unsupported();
    }

    let registered_exe = registered_exe();
    let current = current_exe_path();

    let (registered, message) = match &registered_exe {
        Some(exe) if Some(exe) == current.as_ref() => {
            (true, "molaunch:// 已注册（指向当前程序）".to_string())
        }
        Some(_) => (
            true,
            "molaunch:// 已注册但指向其他路径（便携版可能被移动，可重新注册）".to_string(),
        ),
        None => (
            false,
            "molaunch:// 未注册（便携版需注册后才能点击协议链接）".to_string(),
        ),
    };

    DeeplinkStatus {
        registered,
        registered_exe,
        current_exe: current,
        platform_supported: true,
        message,
    }
}

/// 注册 molaunch:// 协议（幂等）
///
/// - 已注册且指向当前 exe → 直接 Ok（无操作）
/// - 已注册但指向其他路径 → 重注册到当前 exe（便携版移动场景）
/// - 未注册 → 注册
pub fn register() -> Result<(), String> {
    if !platform_supported() {
        return Err(
            "当前平台不支持运行时注册 deeplink（macOS 协议由打包 Info.plist 声明）".to_string(),
        );
    }

    #[cfg(windows)]
    {
        register_windows()
    }
    #[cfg(target_os = "linux")]
    {
        register_linux()
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        Err("当前平台不支持运行时注册 deeplink".to_string())
    }
}

/// 卸载 molaunch:// 协议（幂等）
///
/// 卸载后点击 `molaunch://` 链接系统将提示"无应用处理"。
/// 安装版卸载时由 NSIS 清理；便携版用户可在设置页手动卸载。
pub fn unregister() -> Result<(), String> {
    if !platform_supported() {
        return Err("当前平台不支持运行时卸载 deeplink".to_string());
    }

    #[cfg(windows)]
    {
        unregister_windows()
    }
    #[cfg(target_os = "linux")]
    {
        unregister_linux()
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        Err("当前平台不支持运行时卸载 deeplink".to_string())
    }
}

/// 便携版启动自动注册
///
/// - 已注册且指向当前 exe → 跳过（安装版场景，不重复写注册表）
/// - 已注册但指向其他路径 → 自动重注册（便携版被移动）
/// - 未注册 → 注册（便携版首次启动）
///
/// 返回是否执行了注册动作（仅日志用途）。
pub fn auto_register() -> Result<bool, String> {
    if !platform_supported() {
        return Ok(false);
    }
    let s = status();
    if s.registered && s.registered_exe.as_ref() == s.current_exe.as_ref() {
        return Ok(false); // 已就绪
    }
    register()?;
    Ok(true)
}

/// 查询注册表中登记的 exe 路径（Windows 读 command 键 / Linux 读 desktop Exec）
fn registered_exe() -> Option<String> {
    #[cfg(windows)]
    {
        registered_exe_windows()
    }
    #[cfg(target_os = "linux")]
    {
        registered_exe_linux()
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        None
    }
}

// ======================= Windows 实现 =======================

#[cfg(windows)]
fn registry_key() -> String {
    format!(r"Software\Classes\{}", PROTOCOL)
}

/// 读取注册表登记的 exe 路径
///
/// 从 `HKCU\Software\Classes\molaunch\shell\open\command` 的默认值
/// （形如 `"C:\path\MoLaunch.exe" "%1"`）中解析出引号内的 exe 路径。
#[cfg(windows)]
fn registered_exe_windows() -> Option<String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey(format!(r"{}\shell\open\command", registry_key()))
        .ok()?;
    let cmd: String = key.get_value("").ok()?;
    // 解析引号包裹的 exe 路径（`"C:\path\MoLaunch.exe" "%1"`）
    let trimmed = cmd.trim();
    trimmed
        .strip_prefix('"')
        .and_then(|rest| rest.split('"').next())
        .map(|s| s.to_string())
        .or_else(|| {
            // 无引号时取第一个空格前
            Some(trimmed.split_whitespace().next()?.to_string())
        })
}

/// Windows 注册 molaunch:// 协议（写 HKCU，免管理员）
#[cfg(windows)]
fn register_windows() -> Result<(), String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let exe = current_exe_path().ok_or("无法获取当前 exe 路径")?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // HKCU\Software\Classes\molaunch
    let base = hkcu
        .create_subkey(registry_key())
        .map_err(|e| format!("创建协议键失败: {}", e))?
        .0;
    base.set_value("", &format!("URL:{} protocol", PROTOCOL))
        .map_err(|e| format!("写入协议描述失败: {}", e))?;
    base.set_value("URL Protocol", &"")
        .map_err(|e| format!("写入 URL Protocol 失败: {}", e))?;

    // DefaultIcon
    let icon_key = hkcu
        .create_subkey(format!(r"{}\DefaultIcon", registry_key()))
        .map_err(|e| format!("创建图标键失败: {}", e))?
        .0;
    icon_key
        .set_value("", &format!("{},0", exe))
        .map_err(|e| format!("写入图标路径失败: {}", e))?;

    // shell\open\command
    let cmd_key = hkcu
        .create_subkey(format!(r"{}\shell\open\command", registry_key()))
        .map_err(|e| format!("创建 command 键失败: {}", e))?
        .0;
    cmd_key
        .set_value("", &format!("\"{}\" \"%1\"", exe))
        .map_err(|e| format!("写入打开命令失败: {}", e))?;

    Ok(())
}

/// Windows 卸载 molaunch:// 协议（删除整个 HKCU 键）
#[cfg(windows)]
fn unregister_windows() -> Result<(), String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.delete_subkey_all(registry_key()) {
        Ok(()) => Ok(()),
        Err(e) => {
            // 键不存在也算卸载成功（幂等）
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(format!("删除协议键失败: {}", e))
            }
        }
    }
}

// ======================= Linux 实现 =======================

#[cfg(target_os = "linux")]
fn desktop_file_name() -> String {
    let bin = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "molaunch".to_string());
    format!("{}-handler.desktop", bin)
}

#[cfg(target_os = "linux")]
fn desktop_file_dir() -> Option<std::path::PathBuf> {
    // 优先 ~/.local/share/applications（user 级，免 root）
    if let Ok(home) = std::env::var("HOME") {
        return Some(std::path::Path::new(&home).join(".local/share/applications"));
    }
    None
}

#[cfg(target_os = "linux")]
fn desktop_file_path() -> Option<std::path::PathBuf> {
    Some(desktop_file_dir()?.join(desktop_file_name()))
}

/// Linux 注册 molaunch:// 协议（写 desktop 文件 + xdg-mime）
#[cfg(target_os = "linux")]
fn register_linux() -> Result<(), String> {
    use std::io::Write;

    let exe = current_exe_path().ok_or("无法获取当前 exe 路径")?;
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
    let _ = std::process::Command::new("xdg-mime")
        .args([
            "default",
            &desktop_file_name(),
            &format!("x-scheme-handler/{}", PROTOCOL),
        ])
        .status();
    let _ = std::process::Command::new("update-desktop-database")
        .arg(&dir)
        .status();

    Ok(())
}

/// Linux 卸载 molaunch:// 协议（删除 desktop 文件 + 清理 mime）
#[cfg(target_os = "linux")]
fn unregister_linux() -> Result<(), String> {
    if let Some(file) = desktop_file_path() {
        if file.exists() {
            std::fs::remove_file(&file).map_err(|e| format!("删除 desktop 文件失败: {}", e))?;
        }
    }
    let _ = std::process::Command::new("xdg-mime")
        .args([
            "uninstall",
            "mimeinfo",
            "/dev/null", // 无实际卸载语义，占位避免 xdg 报错；真正清理靠删除 desktop 文件
        ])
        .status();
    Ok(())
}

/// Linux 读取 desktop 文件中登记的 exe 路径
#[cfg(target_os = "linux")]
fn registered_exe_linux() -> Option<String> {
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
