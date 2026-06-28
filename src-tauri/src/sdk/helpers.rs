//! 独立工具函数

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

/// 校验加载器兼容性（独立函数，无需 SDK 句柄）
pub fn validate_loaders(
    mc_version: &str,
    forge_version: Option<&str>,
    neoforge_version: Option<&str>,
    fabric_version: Option<&str>,
    optifine_version: Option<&str>,
) -> Result<(), SdkError> {
    let lib_path = check_sdk_library()?;
    let lib = unsafe {
        libloading::Library::new(&lib_path)
            .map_err(|e| SdkError::LoadFailed(format!("Failed to load library: {}", e)))?
    };
    let func: McValidateLoaders = unsafe {
        *lib.get(b"mc_validate_loaders").map_err(|e| {
            SdkError::LoadFailed(format!("Failed to get mc_validate_loaders: {}", e))
        })?
    };

    let mc_c = std::ffi::CString::new(mc_version).map_err(|e| SdkError::InvalidParameter(e.to_string()))?;
    let forge_c = forge_version.map(|v| std::ffi::CString::new(v).map_err(|e| SdkError::InvalidParameter(e.to_string()))).transpose()?;
    let neoforge_c = neoforge_version.map(|v| std::ffi::CString::new(v).map_err(|e| SdkError::InvalidParameter(e.to_string()))).transpose()?;
    let fabric_c = fabric_version.map(|v| std::ffi::CString::new(v).map_err(|e| SdkError::InvalidParameter(e.to_string()))).transpose()?;
    let optifine_c = optifine_version.map(|v| std::ffi::CString::new(v).map_err(|e| SdkError::InvalidParameter(e.to_string()))).transpose()?;

    let code = unsafe {
        func(
            mc_c.as_ptr(),
            forge_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            neoforge_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            fabric_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            optifine_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
        )
    };

    if code != 0 {
        return Err(SdkError::FfiFailed(code));
    }
    Ok(())
}
