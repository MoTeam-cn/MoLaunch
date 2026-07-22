//! McSDK FFI 绑定层
//!
//! 提供跨平台 SDK 动态库加载和 FFI 函数绑定

mod ffi_types;
mod instance;
mod types;

use std::path::PathBuf;
use thiserror::Error;

// Re-export public types
pub use ffi_types::*;
pub use instance::{SdkFunctions, SdkInstance};
pub use types::*;

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
pub fn get_sdk_filename() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "run_sdk_lib-windows-x86_64.dll"
    }
    #[cfg(target_os = "macos")]
    {
        "run_sdk_lib-darwin-aarch64.dylib"
    }
    #[cfg(target_os = "linux")]
    {
        "run_sdk_lib-linux-x86_64.so"
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
