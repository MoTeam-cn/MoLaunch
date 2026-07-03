//! 独立工具函数（lite 版本）

use super::ffi_types::*;
use super::types::*;
use super::{check_sdk_library, SdkError};

/// 获取系统内存信息（独立函数，无需 SDK 句柄）
pub fn get_system_memory_static() -> Result<SystemMemory, SdkError> {
    let lib_path = check_sdk_library()?;
    let lib = unsafe {
        libloading::Library::new(&lib_path)
            .map_err(|e| SdkError::LoadFailed(format!("Failed to load library: {}", e)))?
    };

    let func: McGetSystemMemory = unsafe {
        *lib.get(b"mc_get_system_memory").map_err(|e| {
            SdkError::LoadFailed(format!("Failed to get mc_get_system_memory: {}", e))
        })?
    };

    let mut memory = FFISystemMemory {
        total: 0,
        used: 0,
        available: 0,
        usage_percent: 0.0,
    };

    let code = unsafe { func(&mut memory) };
    if code != 0 {
        return Err(SdkError::FfiFailed(code));
    }

    Ok(SystemMemory::from_ffi(&memory))
}
