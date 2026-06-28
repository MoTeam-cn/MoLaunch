//! McSDK FFI 绑定层
//!
//! 提供跨平台 SDK 动态库加载和 FFI 函数绑定

mod ffi_types;
mod instance;
mod types;
mod helpers;

use std::path::PathBuf;
use thiserror::Error;

// Re-export public types
pub use ffi_types::*;
pub use instance::{SdkInstance, SdkFunctions};
pub use types::*;
pub use helpers::*;

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
        "mc_sdk.dll"
    }
    #[cfg(target_os = "macos")]
    {
        "mc_sdk.dylib"
    }
    #[cfg(target_os = "linux")]
    {
        "mc_sdk.so"
    }
}

/// 获取 SDK 资源目录路径
pub fn get_sdk_resource_dir() -> PathBuf {
    // 在开发模式下，sdk_data 在项目根目录
    // 在发布模式下，sdk_data 被打包到 resources 目录
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sdk_data");

    if dev_path.exists() {
        return dev_path;
    }

    // 发布模式下，尝试从 Tauri 资源目录获取
    if let Ok(resource_dir) = std::env::current_exe() {
        if let Some(exe_dir) = resource_dir.parent() {
            let resource_path = exe_dir.join("resources").join("sdk_data");
            if resource_path.exists() {
                return resource_path;
            }
        }
    }

    // 兜底返回开发路径
    dev_path
}

/// 获取 SDK 动态库的完整路径
pub fn get_sdk_library_path() -> PathBuf {
    let resource_dir = get_sdk_resource_dir();
    let filename = get_sdk_filename();
    resource_dir.join(filename)
}

/// 检查 SDK 库是否存在
pub fn check_sdk_library() -> Result<PathBuf, SdkError> {
    let path = get_sdk_library_path();
    if path.exists() {
        Ok(path)
    } else {
        Err(SdkError::LoadFailed(format!(
            "SDK library not found at: {}",
            path.display()
        )))
    }
}
