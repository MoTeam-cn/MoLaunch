//! 构建期检查 easytier-core 嵌入式资源
//!
//! easytier-core 通过 include_bytes! 在编译期嵌入二进制（见 resources.rs），
//! 若当前平台对应文件缺失，include_bytes! 只会报晦涩的编译错误。本模块在
//! `tauri_build::build()` 之前先检查并给出明确中文提示：缺失则终止构建并打印
//! 下载指引，存在则打印版本核对。跨平台按 CARGO_CFG_TARGET_OS/ARCH 定位目录。

use std::path::Path;
use std::process::Command;

/// 当前打包的 easytier-core 版本（仅供提示核对，不强制校验）
const EASYTIER_VERSION: &str = "2.6.4";

/// 官方下载地址（GitHub Releases）
const DOWNLOAD_URL: &str = "https://github.com/EasyTier/EasyTier/releases";

/// 当前平台对应的资源相对路径清单（与 resources.rs `embedded_bytes` 的 cfg 分支一致）
///
/// 返回 None 表示当前平台未内置（联机功能不可用，但允许构建）；
/// 返回 Some(files) 时列出核心与依赖 DLL 的相对路径。
fn platform_files() -> Option<Vec<String>> {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let (os_dir, arch_dir, core_name, deps): (&str, &str, &str, &[&str]) =
        match (os.as_str(), arch.as_str()) {
            ("windows", "x86_64") => (
                "windows",
                "x86_64",
                "easytier-core.exe",
                &["Packet.dll", "wintun.dll"],
            ),
            ("windows", "aarch64") => (
                "windows",
                "aarch64",
                "easytier-core.exe",
                &["Packet.dll", "wintun.dll"],
            ),
            ("linux", "x86_64") => ("linux", "x86_64", "easytier-core", &[]),
            ("linux", "aarch64") => ("linux", "aarch64", "easytier-core", &[]),
            ("macos", "x86_64") => ("macos", "x86_64", "easytier-core", &[]),
            ("macos", "aarch64") => ("macos", "aarch64", "easytier-core", &[]),
            _ => {
                println!(
                "cargo:warning=当前平台 {}/{} 未内置 easytier-core，联机功能不可用（不影响构建）",
                os, arch
            );
                return None;
            }
        };
    let mut files = vec![format!("easytier/{os_dir}/{arch_dir}/{core_name}")];
    files.extend(
        deps.iter()
            .map(|d| format!("easytier/{os_dir}/{arch_dir}/{d}")),
    );
    Some(files)
}

/// 检查当前平台 easytier-core 嵌入式资源是否存在；缺失时打印下载指引并终止构建
pub fn check_easytier() {
    let Some(files) = platform_files() else {
        return;
    };

    let missing: Vec<&String> = files
        .iter()
        .filter(|f| !Path::new("resources").join(f).exists())
        .collect();
    if !missing.is_empty() {
        for f in &missing {
            println!(
                "cargo:warning=缺少 easytier-core 嵌入式资源: src-tauri/resources/{}",
                f
            );
        }
        println!(
            "cargo:warning=联机功能依赖 easytier-core，请从 {} 下载 v{} 对应平台包，",
            DOWNLOAD_URL, EASYTIER_VERSION
        );
        println!(
            "cargo:warning=解压出可执行文件及相关 DLL 放入 src-tauri/resources/easytier/ 对应平台目录后重新构建。",
        );
        std::process::exit(1);
    }

    for f in &files {
        println!("cargo:rerun-if-changed=resources/{}", f);
    }

    if let Ok(output) = Command::new(Path::new("resources").join(&files[0]))
        .arg("--version")
        .output()
    {
        if output.status.success() {
            let ver = String::from_utf8_lossy(&output.stdout);
            println!(
                "cargo:warning=easytier-core 版本 {} (期望 v{})",
                ver.trim(),
                EASYTIER_VERSION
            );
        }
    }
}
