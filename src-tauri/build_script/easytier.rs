//! 构建期检查 easytier-core 侧车二进制
//!
//! tauri-build v2 要求 externalBin 文件命名为 `easytier-core-{target_triple}.exe`
//! （见 tauri-utils external_binaries 实现：强制追加 `-{target_triple}{.exe}`），
//! 缺失时 tauri-build 只会报晦涩的英文错误。本模块在 `tauri_build::build()` 之前
//! 先检查并给出明确中文提示：缺失则终止构建并打印下载指引，存在则打印版本核对。
//!
//! ## 非 Windows 平台
//!
//! 当前仅集成 Windows x86_64 二进制，故本模块整体 `#[cfg(target_os = "windows")]`；
//! 其他平台如需构建，需自行提供对应 target triple 的 easytier-core 并放开条件。

#![cfg(target_os = "windows")]

use std::path::Path;
use std::process::Command;

/// 当前打包的 easytier-core 版本（仅供提示核对，不强制校验）
const EASYTIER_VERSION: &str = "2.6.4";

/// 官方下载地址（GitHub Releases，zip 内解压出 easytier-core.exe）
const DOWNLOAD_URL: &str = "https://github.com/EasyTier/EasyTier/releases/download/v2.6.4/easytier-windows-x86_64-v2.6.4.zip";

/// 检查 easytier-core 侧车是否存在；缺失时打印下载指引并终止构建
pub fn check_easytier() {
    let target_triple = std::env::var("TARGET").unwrap_or_default();
    let bin_name = format!("easytier-core-{target_triple}.exe");
    let bin_path = Path::new("binaries").join(&bin_name);

    if !bin_path.exists() {
        println!(
            "cargo:warning=缺少 easytier-core 侧车二进制: {} (target triple: {})",
            bin_path.display(),
            target_triple
        );
        println!(
            "cargo:warning=联机功能依赖 easytier-core，请下载 v{} 并解压出 easytier-core.exe：",
            EASYTIER_VERSION
        );
        println!("cargo:warning=  {}", DOWNLOAD_URL);
        println!(
            "cargo:warning=将 easytier-core.exe 重命名为 {} 放入 src-tauri/binaries/ 后重新构建。",
            bin_name
        );
        std::process::exit(1);
    }

    if let Ok(output) = Command::new(&bin_path).arg("--version").output() {
        if output.status.success() {
            let ver = String::from_utf8_lossy(&output.stdout);
            println!(
                "cargo:warning=easytier-core {} (期望 v{})",
                ver.trim(),
                EASYTIER_VERSION
            );
        }
    }

    // 二进制文件变化时重新运行本构建脚本
    println!("cargo:rerun-if-changed={}", bin_path.display());
}
