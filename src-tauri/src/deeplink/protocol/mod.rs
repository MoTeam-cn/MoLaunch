//! 深度链接协议注册 / 卸载 / 状态查询工具（跨平台，供便携版运行时注册协议）
//! 平台实现按 cfg 拆分为 windows.rs（HKCU 注册表）与 linux.rs（desktop 文件）。

#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

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
        windows::register_windows()
    }
    #[cfg(target_os = "linux")]
    {
        linux::register_linux()
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
        windows::unregister_windows()
    }
    #[cfg(target_os = "linux")]
    {
        linux::unregister_linux()
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
        windows::registered_exe_windows()
    }
    #[cfg(target_os = "linux")]
    {
        linux::registered_exe_linux()
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        None
    }
}