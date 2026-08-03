//! SDK 动态库加载入口：文件名 / 释放路径 / 加载校验 + SdkError

use std::path::PathBuf;
use thiserror::Error;

/// SDK 错误类型
#[derive(Error, Debug)]
pub enum SdkError {
    #[error("SDK not initialized")]
    NotInitialized,
    #[error("Failed to load SDK library: {0}")]
    LoadFailed(String),
    #[error("FFI call failed with code: {0}")]
    FfiFailed(i32),
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("Null pointer returned")]
    NullPointer,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// 获取当前平台的 SDK 文件名
///
/// 平台覆盖矩阵（与编译产物 `src-tauri/resources/sdk/` 一一对应）：
/// - Windows x86_64 → `run_sdk_lib-windows-x86_64.dll`
/// - macOS aarch64 (Apple Silicon) → `run_sdk_lib-darwin-aarch64.dylib`
/// - Linux x86_64 → `run_sdk_lib-linux-x86_64.so`
///
/// 未覆盖平台（Intel Mac / Linux aarch64 / FreeBSD 等）返回
/// `"unsupported-platform"`，`check_sdk_library()` 在 `extract_sdk()` 时
/// 因嵌入资源不存在返回明确错误，避免编译失败但运行时无法加载。
/// 新增平台支持时，需同步：1) 编译 SDK 产物；2) 加入 resources/sdk/；
/// 3) 在下方添加对应 `#[cfg]` 分支。
pub fn get_sdk_filename() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "run_sdk_lib-windows-x86_64.dll"
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "run_sdk_lib-darwin-aarch64.dylib"
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "run_sdk_lib-linux-x86_64.so"
    }

    // 未覆盖平台 fallback：返回占位文件名，extract_sdk() 会因嵌入资源不存在
    // 返回 Err，给出明确的"平台不支持"错误，而非编译失败。
    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "linux", target_arch = "x86_64"),
    )))]
    {
        "unsupported-platform"
    }
}

/// 获取 SDK 动态库的释放路径（临时目录）
///
/// SDK 在编译时嵌入二进制，运行时由 `resources::extract_sdk()` 释放到
/// `<temp>/MoLaunch/sdk/<filename>`。此函数仅返回路径，不触发释放。
pub fn get_sdk_library_path() -> PathBuf {
    let sdk_filename = get_sdk_filename();
    crate::utils::cache_temp::sdk_library_path(sdk_filename)
}

/// 确保 SDK 已释放到临时目录，返回动态库路径
///
/// 调用 `resources::extract_sdk()` 释放嵌入的 SDK 到临时目录。
/// sha256 校验机制保证：只在版本不匹配时重新释放，避免每次启动重复写盘。
pub fn check_sdk_library() -> Result<PathBuf, SdkError> {
    match crate::resources::extract_sdk() {
        Ok(path) => Ok(path),
        Err(e) => Err(SdkError::LoadFailed(format!(
            "Failed to extract SDK to temp dir: {}",
            e
        ))),
    }
}